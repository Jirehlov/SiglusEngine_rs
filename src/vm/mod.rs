use std::{collections::BTreeMap, sync::Arc, time::Instant};

use anyhow::{Context, Result, bail};

use crate::{dat::SceneDat, lexer::SceneLexer, stack::IfcStack};

use self::syscom_config_state::VmSyscomConfigState;

mod api;
mod command_call;
mod command_effect;
mod command_head;
mod command_input;
mod command_int_event;
mod command_misc;
mod command_mwnd;
mod command_object;
mod command_others;
mod command_others_cg;
mod command_others_counter_matrix;
mod command_script;
mod command_sound;
mod command_sound_bgm;
mod command_stage;
mod command_syscom;
mod command_syscom_capture;
mod command_syscom_endio;
mod command_syscom_misc;
mod command_syscom_misc_lowfreq;
mod command_syscom_return;
mod command_syscom_slot;
mod command_tail;
mod command_try;
mod command_world;
mod core;
mod core_flow;
mod end_save_runtime;
mod end_save_state;
mod local_state;
mod opcode;
mod persistent;
mod props;
mod props_assign;
mod stack_ops;
mod syscom_config_state;

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
    /// C++ tnm_ini.cpp: CONFIG.GLOBAL_EXTRA_SWITCH default onoff by index.
    pub default_global_extra_switch: BTreeMap<i32, i32>,
    /// C++ tnm_ini.cpp: CONFIG.GLOBAL_EXTRA_MODE default mode by index.
    pub default_global_extra_mode: BTreeMap<i32, i32>,
    /// C++ tnm_ini.cpp: CONFIG.OBJECT_DISP default onoff by index.
    pub default_object_disp: BTreeMap<i32, i32>,
    /// C++ tnm_ini.cpp: CONFIG.LOCAL_EXTRA_MODE defaults by index.
    pub default_local_extra_mode_value: BTreeMap<i32, i32>,
    pub default_local_extra_mode_enable: BTreeMap<i32, i32>,
    pub default_local_extra_mode_exist: BTreeMap<i32, i32>,
    /// C++ tnm_ini.cpp: CONFIG.LOCAL_EXTRA_SWITCH defaults by index.
    pub default_local_extra_switch_onoff: BTreeMap<i32, i32>,
    pub default_local_extra_switch_enable: BTreeMap<i32, i32>,
    pub default_local_extra_switch_exist: BTreeMap<i32, i32>,
    pub default_global_extra_switch_cnt: usize,
    pub default_global_extra_mode_cnt: usize,
    pub default_object_disp_cnt: usize,
    pub default_local_extra_mode_cnt: usize,
    pub default_local_extra_switch_cnt: usize,
    pub default_charakoe_cnt: usize,
    pub default_charakoe_onoff: BTreeMap<i32, i32>,
    pub default_charakoe_volume: BTreeMap<i32, i32>,
    /// C++ tnm_ini.h: LOAD_AFTER_CALL scene name (empty / None = disabled).
    pub load_after_call_scene: Option<String>,
    /// C++ tnm_ini.h: LOAD_AFTER_CALL z-label index (default 0).
    pub load_after_call_z_no: i32,
    /// C++ tnm_ini.cpp: FLICK_SCENE routing table.
    pub flick_scene_routes: Vec<FlickSceneRoute>,
    /// C++ S_tnm_command_proc_arg_struct::disp_out_of_range_error equivalent.
    /// When false, out-of-range element access returns defaults without emitting VM error text.
    pub disp_out_of_range_error: bool,
    /// Preloaded database rows by database index (from INI/gameexe resource bootstrap).
    pub preloaded_database_tables: Vec<Vec<Vec<PropValue>>>,
    /// Optional DB row call_no mapping by database index.
    pub preloaded_database_row_calls: Vec<Vec<i32>>,
    /// Optional DB column call_no mapping by database index.
    pub preloaded_database_col_calls: Vec<Vec<i32>>,
    /// Optional DB column data_type (from DBS header; b'V'/b'S').
    pub preloaded_database_col_types: Vec<Vec<u8>>,
    /// Preloaded CG table flag count (from INI/gameexe bootstrap).
    pub preloaded_cg_flag_count: usize,
    /// Preloaded CG name -> flag index map.
    pub preloaded_cg_name_to_flag: BTreeMap<String, i32>,
    /// Preloaded CG group code by list index (CGTABLE2).
    pub preloaded_cg_group_codes: Vec<[i32; 5]>,
    /// Preloaded CG code_exist_cnt by list index (CGTABLE2).
    pub preloaded_cg_code_exist_cnt: Vec<i32>,
    /// Preloaded BGM name list (listened defaults to false).
    pub preloaded_bgm_names: Vec<String>,
    /// C++ tnm_ini.cpp: COUNTER.CNT (counter_list default size).
    pub preloaded_counter_count: usize,
    /// C++ tnm_ini.cpp: FRAME_ACTION_CH.CNT (excall frame_action_ch default size).
    pub preloaded_frame_action_ch_count: usize,
}

#[derive(Debug, Clone)]
pub struct FlickSceneRoute {
    pub scene: String,
    pub z_no: i32,
    pub angle_type: i32,
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
            default_global_extra_switch: BTreeMap::new(),
            default_global_extra_mode: BTreeMap::new(),
            default_object_disp: BTreeMap::new(),
            default_local_extra_mode_value: BTreeMap::new(),
            default_local_extra_mode_enable: BTreeMap::new(),
            default_local_extra_mode_exist: BTreeMap::new(),
            default_local_extra_switch_onoff: BTreeMap::new(),
            default_local_extra_switch_enable: BTreeMap::new(),
            default_local_extra_switch_exist: BTreeMap::new(),
            default_global_extra_switch_cnt: 0,
            default_global_extra_mode_cnt: 0,
            default_object_disp_cnt: 0,
            default_local_extra_mode_cnt: 0,
            default_local_extra_switch_cnt: 0,
            default_charakoe_cnt: 0,
            default_charakoe_onoff: BTreeMap::new(),
            default_charakoe_volume: BTreeMap::new(),
            load_after_call_scene: None,
            load_after_call_z_no: 0,
            flick_scene_routes: Vec::new(),
            disp_out_of_range_error: true,
            preloaded_database_tables: Vec::new(),
            preloaded_database_row_calls: Vec::new(),
            preloaded_database_col_calls: Vec::new(),
            preloaded_database_col_types: Vec::new(),
            preloaded_cg_flag_count: 0,
            preloaded_cg_name_to_flag: BTreeMap::new(),
            preloaded_cg_group_codes: Vec::new(),
            preloaded_cg_code_exist_cnt: Vec::new(),
            preloaded_bgm_names: Vec::new(),
            preloaded_counter_count: FLAG_LIST_SIZE,
            preloaded_frame_action_ch_count: 0,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VmCallType {
    None = 0,
    Gosub = 1,
    Farcall = 2,
    UserCmd = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VmProcType {
    None = 0,
    Script = 1,
}

#[derive(Debug, Clone)]
struct FrameAction {
    end_time: i32,
    real_flag: i32,
    scn_name: String,
    cmd_name: String,
    args: Vec<Prop>,
    end_action_flag: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct KeyWaitProc {
    active: bool,
    force_skip_disable: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct GroupWaitProc {
    active: bool,
    stage_idx: i32,
    group_idx: i32,
}

#[derive(Debug, Clone, Default)]
struct MaskSlotState {
    name: String,
    x: i32,
    y: i32,
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
    /// C++ C_elm_call::call_type metadata.
    call_type: VmCallType,
    /// C++ C_elm_call::excall_flag metadata.
    excall_flag: bool,
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

include!("local_state_struct.rs");

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
    excall_frame_action: FrameAction,
    excall_frame_action_ch: Vec<FrameAction>,
    key_wait_proc: KeyWaitProc,
    group_wait_proc: GroupWaitProc,
    // C++ cmd_call.cpp excall allocation state (`excall.is_excall/check_alloc`).
    excall_allocated: [bool; 2],

    // ----- Flag system (mirrors C++ Gp_flag) -----
    flags_a: Vec<i32>,
    flags_b: Vec<i32>,
    flags_c: Vec<i32>,
    flags_d: Vec<i32>,
    flags_e: Vec<i32>,
    flags_f: Vec<i32>,
    excall_flags_f: [Vec<i32>; 2],
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
    syscom_cfg: VmSyscomConfigState,
    no_wipe_anime_onoff_flag: i32,
    skip_wipe_anime_onoff_flag: i32,
    script_skip_unread_message_flag: i32,
    script_stage_time_stop_flag: i32,
    system_wipe_flag: i32,
    do_frame_action_flag: i32,
    do_load_after_call_flag: i32,
    game_timer_move_flag: i32,
    syscom_menu_disable_flag: i32,
    system_extra_int_values: Vec<i32>,
    system_extra_str_values: Vec<String>,
    return_scene_once: Option<(String, i32)>,

    // ----- Global wipe state (best-effort timing alignment with C++ flow) -----
    wipe_end_at: Option<Instant>,

    last_pc: usize,
    last_line_no: i32,
    last_scene: String,
    proc_stack: Vec<VmProcType>,

    script_dont_set_save_point: bool,
    script_skip_disable: bool,
    script_ctrl_disable: bool,
    script_not_stop_skip_by_click: bool,
    script_not_skip_msg_by_click: bool,
    script_auto_mode_flag: bool,
    script_auto_mode_moji_wait: i32,
    script_auto_mode_min_wait: i32,
    script_auto_mode_moji_cnt: i32,
    script_mouse_cursor_hide_onoff: i32,
    script_mouse_cursor_hide_time: i32,
    script_msg_speed: i32,
    script_msg_nowait: bool,
    script_async_msg_mode: bool,
    script_async_msg_mode_once: bool,
    script_hide_mwnd_disable: bool,
    script_cursor_disp_off: bool,
    script_cursor_move_by_key_disable: bool,
    script_key_disable: [bool; 256],
    script_mwnd_anime_off_flag: bool,
    script_mwnd_anime_on_flag: bool,
    script_mwnd_disp_off_flag: bool,
    script_koe_dont_stop_on_flag: bool,
    script_koe_dont_stop_off_flag: bool,
    script_shortcut_disable: bool,
    script_quake_stop_flag: bool,
    script_emote_mouth_stop_flag: bool,
    script_bgmfade_flag: bool,
    script_vsync_wait_off_flag: bool,
    script_skip_trigger: bool,
    script_ignore_r_flag: bool,
    script_cursor_no: i32,
    script_time_stop_flag: bool,
    script_counter_time_stop_flag: bool,
    script_frame_action_time_stop_flag: bool,
    script_font_name: String,
    script_font_bold: i32,
    script_font_shadow: i32,
    script_allow_joypad_mode_onoff: i32,
    excall_script_font_name: [String; 2],
    excall_script_font_bold: [i32; 2],
    excall_script_font_shadow: [i32; 2],
    counter_list_size: usize,
    excall_counter_list_size: [usize; 2],
    counter_values: Vec<i32>,
    counter_active: Vec<bool>,
    database_tables: Vec<Vec<Vec<PropValue>>>,
    database_row_calls: Vec<Vec<i32>>,
    database_col_calls: Vec<Vec<i32>>,
    database_col_types: Vec<Vec<u8>>,
    cg_table_off_flag: bool,
    cg_flags: Vec<i32>,
    cg_name_to_flag: BTreeMap<String, i32>,
    cg_group_codes: Vec<[i32; 5]>,
    cg_code_exist_cnt: Vec<i32>,
    bgm_name_listened: BTreeMap<String, bool>,
    g00buf_loaded: Vec<Option<String>>,
    mask_slots: Vec<MaskSlotState>,
    object_gan_loaded_path: BTreeMap<(i32, i32, i32), String>,
    object_gan_started_set: BTreeMap<(i32, i32, i32), i32>,

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
