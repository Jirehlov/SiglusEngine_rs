use std::{collections::BTreeMap, sync::Arc, time::Instant};

use anyhow::{Context, Result, bail};

use crate::{dat::SceneDat, lexer::SceneLexer, stack::IfcStack};

mod api;
mod command_head;
mod command_misc;
mod command_syscom;
mod command_syscom_endio;
mod command_syscom_return;
mod command_effect;
mod command_sound;
mod command_tail;
mod command_world;
mod command_try;
mod core;
mod end_save_runtime;
mod end_save_state;
mod opcode;
mod persistent;
mod props;
mod stack_ops;

pub use api::*;
pub use end_save_state::*;
pub use persistent::*;

pub trait SceneProvider {
    fn get_scene(&mut self, scene: &str) -> Result<Arc<SceneDat>>;

    fn inc_cmd_count(&self) -> i32 {
        0
    }

    fn get_inc_cmd_target(&mut self, _user_cmd_id: i32) -> Result<Option<(String, i32)>> {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct VmOptions {
    /// Trace property evaluations.
    pub trace_prop: bool,
    /// Trace command calls.
    pub trace_cmd: bool,
    /// Trace stack state (expensive / noisy).
    pub trace_stack: bool,
    /// Optional scene/z to jump to when script calls returnmenu.
    pub return_menu_scene: Option<(String, i32)>,
    /// Whether to honor script wait commands in real time.
    pub realtime_wait: bool,
    /// Current key-skip policy for wipe wait when script passes key_wait_mode=-1.
    /// Mirrors C++ runtime system.skip_wipe_anime_flag behavior.
    pub skip_wipe_anime: bool,
    /// Initial/default skip-wipe-anime policy (INI-equivalent fallback target).
    pub skip_wipe_anime_default: bool,
    /// Initial/default no-wipe-anime policy (INI-equivalent fallback target).
    pub no_wipe_anime_default: bool,
    /// Current no-wipe-anime policy mirroring C++ runtime system.no_wipe_anime_flag.
    pub no_wipe_anime: bool,
    /// C++ tnm_ini.cpp: LOAD_WIPE first parameter (default 0).
    pub load_wipe_type: i32,
    /// C++ tnm_ini.cpp: LOAD_WIPE second parameter (default 1000).
    pub load_wipe_time_ms: u64,
    /// C++ tnm_ini.cpp: SYSTEM.EXTRA_INT_VALUE indexed array.
    pub system_extra_int_values: Vec<i32>,
    /// C++ tnm_ini.cpp: SYSTEM.EXTRA_STR_VALUE indexed array.
    pub system_extra_str_values: Vec<String>,
    /// C++ tnm_ini.h: LOAD_AFTER_CALL scene name (empty / None = disabled).
    pub load_after_call_scene: Option<String>,
    /// C++ tnm_ini.h: LOAD_AFTER_CALL z-label index (default 0).
    pub load_after_call_z_no: i32,
}

impl VmOptions {
    fn wait_enabled(&self) -> bool {
        self.realtime_wait
    }
}

impl Default for VmOptions {
    fn default() -> Self {
        Self {
            trace_prop: false,
            trace_cmd: false,
            trace_stack: false,
            return_menu_scene: Some(("__sys_menu".to_string(), 0)),
            realtime_wait: true,
            // C++ tnm_ini.cpp defaults skip_wipe_anime.onoff to true.
            skip_wipe_anime: true,
            skip_wipe_anime_default: true,
            no_wipe_anime_default: false,
            no_wipe_anime: false,
            load_wipe_type: 0,
            load_wipe_time_ms: 1000,
            system_extra_int_values: Vec::new(),
            system_extra_str_values: Vec::new(),
            load_after_call_scene: None,
            load_after_call_z_no: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VmStats {
    pub opcode_hits: [u64; 256],
}

impl Default for VmStats {
    fn default() -> Self {
        Self {
            opcode_hits: [0u64; 256],
        }
    }
}

#[derive(Debug, Clone)]
struct CallProp {
    prop_id: i32,
    form: i32,
    value: PropValue,
}

#[derive(Debug, Clone)]
struct CallContext {
    /// call.L (integer flags)
    l: Vec<i32>,
    /// call.K (string flags)
    k: Vec<String>,
    /// user_call_prop_list (declared via CD_DEC_PROP)
    user_props: Vec<CallProp>,
}

impl CallContext {
    fn new(call_flag_cnt: usize) -> Self {
        Self {
            l: vec![0; call_flag_cnt],
            k: vec![String::new(); call_flag_cnt],
            user_props: Vec::new(),
        }
    }
}

#[inline]
fn elm_code(v: i32) -> u16 {
    (v as u32 & 0xFFFF) as u16
}

const DEFAULT_CALL_FLAG_CNT: usize = 32;

#[derive(Debug, Clone)]
struct FrameAction {
    end_time: i32,
    real_flag: i32,
    scn_name: String,
    cmd_name: String,
    args: Vec<Prop>,
    end_action_flag: bool,
}

impl Default for FrameAction {
    fn default() -> Self {
        Self {
            end_time: 0,
            real_flag: 0,
            scn_name: String::new(),
            cmd_name: String::new(),
            args: Vec::new(),
            end_action_flag: false,
        }
    }
}

impl FrameAction {
    fn reinit(&mut self) {
        self.end_time = 0;
        self.real_flag = 0;
        self.scn_name.clear();
        self.cmd_name.clear();
        self.args.clear();
        self.end_action_flag = false;
    }

    fn set_param(
        &mut self,
        end_time: i32,
        real_flag: i32,
        scn_name: String,
        cmd_name: String,
        args: Vec<Prop>,
    ) {
        self.end_time = end_time;
        self.real_flag = real_flag;
        self.scn_name = scn_name;
        self.cmd_name = cmd_name;
        self.args = args;
        self.end_action_flag = false;
    }
}
#[derive(Debug, Clone)]
struct Frame {
    /// PC to return to in the caller scene.
    return_pc: usize,
    /// Scene name to restore on return.
    return_scene: String,
    /// Scene data to restore on return.
    return_dat: Arc<SceneDat>,
    /// Line number to restore on return.
    return_line_no: i32,

    /// Return form expected when the callee returns to this frame.
    expect_ret_form: i32,
    /// Called as a frame action (tnm_scene_proc_call_user_cmd(..., frame_action_flag=true)).
    frame_action_flag: bool,
    /// Argument count at call time (used only when frame_action_flag is true).
    arg_cnt: usize,
    call: CallContext,
}

#[derive(Debug, Clone)]
struct LocalSaveStamp {
    year: i32,
    month: i32,
    day: i32,
    weekday: i32,
    hour: i32,
    minute: i32,
    second: i32,
    millisecond: i32,
}

impl Default for LocalSaveStamp {
    fn default() -> Self {
        Self {
            year: 1970,
            month: 1,
            day: 1,
            weekday: 4,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct LocalSaveSlot {
    stamp: LocalSaveStamp,
    scene_title: String,
    message: String,
    state: VmLocalState,
}

#[derive(Debug, Clone)]
struct VmLocalState {
    scene: String,
    lexer: SceneLexer,
    stack: IfcStack,
    frames: Vec<Frame>,
    scene_title: String,
    user_prop_forms: Vec<i32>,
    user_prop_values: Vec<PropValue>,
    frame_action: FrameAction,
    frame_action_ch: Vec<FrameAction>,
    flags_a: Vec<i32>,
    flags_b: Vec<i32>,
    flags_c: Vec<i32>,
    flags_d: Vec<i32>,
    flags_e: Vec<i32>,
    flags_f: Vec<i32>,
    flags_x: Vec<i32>,
    flags_g: Vec<i32>,
    flags_z: Vec<i32>,
    flags_s: Vec<String>,
    flags_m: Vec<String>,
    global_namae: Vec<String>,
    local_namae: Vec<String>,
    save_point_set: bool,
    sel_point_set: bool,
    save_point_snapshot: Option<VmPersistentState>,
    sel_point_snapshot: Option<VmPersistentState>,
    sel_point_stock: Option<VmPersistentState>,
    cur_mwnd_element: Vec<i32>,
    cur_sel_mwnd_element: Vec<i32>,
    last_sel_msg: String,
    hide_mwnd_onoff_flag: i32,
    hide_mwnd_enable_flag: i32,
    hide_mwnd_exist_flag: i32,
    read_skip_onoff_flag: i32,
    read_skip_enable_flag: i32,
    read_skip_exist_flag: i32,
    auto_mode_onoff_flag: i32,
    auto_mode_enable_flag: i32,
    auto_mode_exist_flag: i32,
    msg_back_enable_flag: i32,
    msg_back_exist_flag: i32,
    msg_back_open_flag: i32,
    msg_back_has_message: i32,
    msg_back_disable_flag: i32,
    msg_back_off_flag: i32,
    msg_back_disp_off_flag: i32,
    msg_back_proc_off_flag: i32,
    return_to_sel_enable_flag: i32,
    return_to_sel_exist_flag: i32,
    return_to_menu_enable_flag: i32,
    return_to_menu_exist_flag: i32,
    save_enable_flag: i32,
    save_exist_flag: i32,
    load_enable_flag: i32,
    load_exist_flag: i32,
    end_game_enable_flag: i32,
    end_game_exist_flag: i32,
    game_end_flag: i32,
    game_end_no_warning_flag: i32,
    game_end_save_done_flag: i32,
    no_wipe_anime_onoff_flag: i32,
    skip_wipe_anime_onoff_flag: i32,
    script_skip_unread_message_flag: i32,
    script_stage_time_stop_flag: i32,
    system_wipe_flag: i32,
    do_frame_action_flag: i32,
    do_load_after_call_flag: i32,
    last_pc: usize,
    last_line_no: i32,
    last_scene: String,
}

/// Default flag array size matching C++ engine (32 elements each for A-G, X, Z).
const FLAG_LIST_SIZE: usize = 32;

pub struct Vm {
    pub scene: String,
    pub lexer: SceneLexer,
    pub stack: IfcStack,
    frames: Vec<Frame>,
    pub max_steps: u64,
    pub steps: u64,
    pub halted: bool,
    scene_title: String,

    pub options: VmOptions,
    pub stats: VmStats,

    // Scene user-prop storage (best-effort). Indexed by element code.
    user_prop_forms: Vec<i32>,
    user_prop_values: Vec<PropValue>,

    // FrameAction elements (headless best-effort). These are normally driven by the engine frame loop.
    frame_action: FrameAction,
    frame_action_ch: Vec<FrameAction>,

    // ----- Flag system (mirrors C++ Gp_flag) -----
    flags_a: Vec<i32>,
    flags_b: Vec<i32>,
    flags_c: Vec<i32>,
    flags_d: Vec<i32>,
    flags_e: Vec<i32>,
    flags_f: Vec<i32>,
    flags_x: Vec<i32>,
    flags_g: Vec<i32>,
    flags_z: Vec<i32>,
    flags_s: Vec<String>,
    flags_m: Vec<String>,
    global_namae: Vec<String>,
    local_namae: Vec<String>,

    // ----- Save / selection point state (C++ local/sel-save approximation) -----
    save_point_set: bool,
    sel_point_set: bool,
    save_point_snapshot: Option<VmPersistentState>,
    sel_point_snapshot: Option<VmPersistentState>,
    sel_point_stock: Option<VmPersistentState>,

    // ----- Current message window element path (mirrors C++ Gp_local->cur_mwnd) -----
    cur_mwnd_element: Vec<i32>,
    cur_sel_mwnd_element: Vec<i32>,
    last_sel_msg: String,

    // ----- Minimal system/syscom/script/input state for system scenes -----
    hide_mwnd_onoff_flag: i32,
    hide_mwnd_enable_flag: i32,
    hide_mwnd_exist_flag: i32,
    read_skip_onoff_flag: i32,
    read_skip_enable_flag: i32,
    read_skip_exist_flag: i32,
    auto_mode_onoff_flag: i32,
    auto_mode_enable_flag: i32,
    auto_mode_exist_flag: i32,
    msg_back_enable_flag: i32,
    msg_back_exist_flag: i32,
    msg_back_open_flag: i32,
    msg_back_has_message: i32,
    msg_back_disable_flag: i32,
    msg_back_off_flag: i32,
    msg_back_disp_off_flag: i32,
    msg_back_proc_off_flag: i32,
    return_to_sel_enable_flag: i32,
    return_to_sel_exist_flag: i32,
    return_to_menu_enable_flag: i32,
    return_to_menu_exist_flag: i32,
    save_enable_flag: i32,
    save_exist_flag: i32,
    load_enable_flag: i32,
    load_exist_flag: i32,
    end_game_enable_flag: i32,
    end_game_exist_flag: i32,
    game_end_flag: i32,
    game_end_no_warning_flag: i32,
    game_end_save_done_flag: i32,
    no_wipe_anime_onoff_flag: i32,
    skip_wipe_anime_onoff_flag: i32,
    script_skip_unread_message_flag: i32,
    script_stage_time_stop_flag: i32,
    system_wipe_flag: i32,
    do_frame_action_flag: i32,
    do_load_after_call_flag: i32,
    system_extra_int_values: Vec<i32>,
    system_extra_str_values: Vec<String>,
    return_scene_once: Option<(String, i32)>,

    // ----- Global wipe state (best-effort timing alignment with C++ flow) -----
    wipe_end_at: Option<Instant>,

    last_pc: usize,
    last_line_no: i32,
    last_scene: String,

    // ----- Local save slots (Rust-native UX-compatible save/load) -----
    local_save_slots: BTreeMap<i32, LocalSaveSlot>,
    quick_save_slots: BTreeMap<i32, LocalSaveSlot>,
    inner_save_slots: BTreeMap<i32, LocalSaveSlot>,
    end_save_slots: BTreeMap<i32, LocalSaveSlot>,
}

fn make_user_props(dat: &SceneDat) -> (Vec<i32>, Vec<PropValue>) {
    let mut forms = Vec::with_capacity(dat.scn_props.len());
    let mut values = Vec::with_capacity(dat.scn_props.len());

    // NOTE: Scene.dat scn_prop_list entries are (form_code, size) in the original engine.
    for &(form_code, size) in &dat.scn_props {
        let f = form_code;
        forms.push(f);
        let v = if f == crate::elm::form::INT {
            PropValue::Int(0)
        } else if f == crate::elm::form::STR {
            PropValue::Str(String::new())
        } else if f == crate::elm::form::INTLIST {
            let n = size.max(0) as usize;
            PropValue::IntList(vec![0; n])
        } else if f == crate::elm::form::STRLIST {
            let n = size.max(0) as usize;
            PropValue::StrList(vec![String::new(); n])
        } else {
            // For now, treat unknown user-prop forms as "element-like" placeholders.
            PropValue::Element(Vec::new())
        };
        values.push(v);
    }

    (forms, values)
}
