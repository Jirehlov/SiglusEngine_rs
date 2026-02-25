#[derive(Debug, Clone)]
pub enum PropValue {
    Int(i32),
    Str(String),
    /// Argument list (FM_LIST)
    List(Vec<Prop>),
    /// Raw element chain (fallback / refs)
    Element(Vec<i32>),
    /// Call/local intlist storage (best-effort)
    IntList(Vec<i32>),
    /// Call/local strlist storage (best-effort)
    StrList(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct Prop {
    pub id: i32,
    pub form: i32,
    pub value: PropValue,
}

/// Unified observable state for `*_eve.wait` / `*_eve.wait_key` in property lane.
///
/// - `EVE_WAIT_DONE`: event already finished in this poll cycle.
/// - `EVE_WAIT_PENDING`: event still waiting after one wait-frame tick.
/// - `EVE_WAIT_KEY_SKIPPED`: `wait_key` was skipped by key input policy.
pub const EVE_WAIT_DONE: i32 = 0;
pub const EVE_WAIT_PENDING: i32 = 1;
pub const EVE_WAIT_KEY_SKIPPED: i32 = 2;

/// Syscom wait-observation owner-id mapping (flow_proc.cpp/ifc_proc_stack alignment).
///
/// Hosts can use this table to classify VM wait-status signals coming from syscom flow.
pub const SYSCOM_WAIT_OWNER_PROC_BASE: i32 = -10_000;
pub const SYSCOM_WAIT_OWNER_PROC_RETURN_TO_MENU: i32 = -10_011;
pub const SYSCOM_WAIT_OWNER_PROC_RETURN_TO_SEL: i32 = -10_012;
pub const SYSCOM_WAIT_OWNER_PROC_END_GAME: i32 = -10_013;
pub const SYSCOM_WAIT_OWNER_END_LOAD_PRE_QUEUE: i32 = -10_201;
pub const SYSCOM_WAIT_OWNER_END_LOAD_POST_QUEUE: i32 = -10_202;

include!("api_syscom_wait.rs");

include!("api_excall_counter_trace.rs");

/// Flatten command arguments into selectable string options.
///
/// Siglus selection commands may provide options either as direct string props
/// or nested lists of string props.
pub fn extract_selection_options(args: &[Prop]) -> Vec<String> {
    let mut options = Vec::new();
    for arg in args {
        match &arg.value {
            PropValue::Str(s) => options.push(s.clone()),
            PropValue::List(list) => {
                for item in list {
                    if let PropValue::Str(s) = &item.value {
                        options.push(s.clone());
                    }
                }
            }
            _ => {}
        }
    }
    options
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmResourceKind {
    Generic,
    Image,
    Movie,
    Text,
}

#[derive(Debug, Clone, Default)]
pub struct VmCaptureFlagsSpec {
    pub element: Vec<i32>,
    pub index: i32,
    pub count: i32,
}

#[derive(Debug, Clone, Default)]
pub struct VmCaptureFlagPayload {
    pub int_values: Vec<i32>,
    pub str_values: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VmCaptureFileOp {
    pub file_name: String,
    pub extension: String,
    pub dialog_flag: bool,
    pub dialog_title: String,
    pub int_flags: VmCaptureFlagsSpec,
    pub str_flags: VmCaptureFlagsSpec,
    pub int_values: Vec<i32>,
    pub str_values: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct HostReturn {
    pub int: i32,
    pub str_: String,
    pub element: Vec<i32>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VmLoadFlowState {
    pub system_wipe_flag: bool,
    pub do_frame_action_flag: bool,
    pub do_load_after_call_flag: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmQuakeKind {
    Vec,
    Dir,
    Zoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmQuakeRequest {
    pub sub: i32,
    pub kind: VmQuakeKind,
    pub time_ms: i32,
    pub cnt: i32,
    pub end_cnt: i32,
    pub begin_order: i32,
    pub end_order: i32,
    pub wait_flag: bool,
    pub key_flag: bool,
    pub power: i32,
    pub vec: i32,
    pub center_x: i32,
    pub center_y: i32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VmInputButtonState {
    pub on_down: bool,
    pub on_up: bool,
    pub on_down_up: bool,
    pub is_down: bool,
    pub is_up: bool,
    pub on_flick: bool,
    pub on_repeat: bool,
    pub flick_angle: i32,
    pub flick_pixel: i32,
    pub flick_mm: i32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct VmFlickState {
    pub has_flick_stock: bool,
    pub angle_radian: f64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VmInputMouseState {
    pub pos_x: i32,
    pub pos_y: i32,
    pub wheel_delta: i32,
    pub left: VmInputButtonState,
    pub right: VmInputButtonState,
}

pub trait Host {
    fn on_name(&mut self, _name: &str) {}
    fn on_text(&mut self, _text: &str, _read_flag_no: i32) {}

    fn on_command(
        &mut self,
        _element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        _named_arg_cnt: i32,
        _ret_form: i32,
    ) -> HostReturn {
        HostReturn::default()
    }

    /// Called when the VM evaluates a property.
    ///
    /// If you need to return a non-INT value, prefer implementing `on_property_typed`.
    fn on_property(&mut self, _element: &[i32]) -> HostReturn {
        HostReturn::default()
    }

    /// Optional typed property hook. If it returns `Some((ret, form))`, the VM will
    /// push `ret` using `form`.
    fn on_property_typed(&mut self, _element: &[i32]) -> Option<(HostReturn, i32)> {
        None
    }

    fn on_assign(&mut self, _element: &[i32], _al_id: i32, _rhs: &Prop) {}

    /// Optional verbose trace lines from the VM.
    fn on_trace(&mut self, _msg: &str) {}

    fn on_error(&mut self, _msg: &str) {}

    /// C++ tnm_set_error(TNM_ERROR_TYPE_FATAL, ...).
    fn on_error_fatal(&mut self, msg: &str) {
        self.on_error(msg);
    }

    /// C++ tnm_set_error(TNM_ERROR_TYPE_FILE_NOT_FOUND, ...).
    fn on_error_file_not_found(&mut self, msg: &str) {
        self.on_error(msg);
    }

    /// Best-effort resource existence probe using host-specific search paths.
    fn on_resource_exists(&mut self, path: &str) -> bool {
        self.on_resource_exists_with_kind(path, VmResourceKind::Generic)
    }

    /// Resource existence probe with command-level type hint.
    ///
    /// C++ cmd routing often probes file names with family-specific fallback rules
    /// (e.g. image/movie extension candidates). Hosts may use `kind` to align those
    /// resolution paths without changing VM-side command semantics.
    fn on_resource_exists_with_kind(&mut self, path: &str, _kind: VmResourceKind) -> bool {
        std::path::Path::new(path).exists()
    }

    /// Optional host-side text loading via resource resolver.
    /// Returning `Some` means VM should consume this value directly; `None` falls
    /// back to local filesystem best-effort loading for headless/default hosts.
    fn on_resource_read_text(&mut self, _path: &str) -> Option<String> {
        None
    }

    /// Called when VM location/title context changes (for window caption, overlays, etc.).
    fn on_location(&mut self, _scene_title: &str, _scene: &str, _line_no: i32, _pc: usize) {}

    /// Notify host that msg_back open-state changed.
    fn on_msg_back_state(&mut self, _open: bool) {}

    /// Notify host that msg_back display availability changed (script disp_off/on).
    fn on_msg_back_display(&mut self, _enabled: bool) {}

    /// Notify host that syscom requested opening the tweet dialog.
    fn on_open_tweet_dialog(&mut self) {}

    /// C++ reference: cmd_syscom.cpp::ELM_SYSCOM_RETURN_TO_MENU warning branch.
    /// Return `false` to emulate user-cancel in warning dialog.
    fn on_syscom_return_to_menu_warning(&mut self) -> bool {
        true
    }

    /// C++ reference: cmd_syscom.cpp::ELM_SYSCOM_RETURN_TO_SEL warning branch.
    fn on_syscom_return_to_sel_warning(&mut self) -> bool {
        true
    }

    /// C++ reference: cmd_syscom.cpp::ELM_SYSCOM_END_GAME warning branch.
    fn on_syscom_end_game_warning(&mut self) -> bool {
        true
    }

    /// C++ reference: eng_syscom.cpp::tnm_syscom_end_save warning branch.
    fn on_syscom_end_save_warning(&mut self) -> bool {
        true
    }

    /// C++ reference: eng_syscom.cpp::tnm_syscom_end_load warning branch.
    fn on_syscom_end_load_warning(&mut self) -> bool {
        true
    }

    /// C++ reference: eng_syscom.cpp syscom SE triggers.
    /// `kind` uses `elm::syscom::SE_KIND_*` constants.
    fn on_syscom_play_se(&mut self, _kind: i32) {}

    /// C++ reference: eng_syscom.cpp fade-out branches -> TNM_PROC_TYPE_DISP.
    fn on_syscom_proc_disp(&mut self) {}

    /// C++ reference: flow_proc.cpp::tnm_game_end_wipe_proc.
    fn on_syscom_proc_game_end_wipe(&mut self, _wipe_type: i32, _wipe_time_ms: u64) {}

    /// C++ reference: flow_proc.cpp::tnm_game_start_wipe_proc.
    fn on_syscom_proc_game_start_wipe(&mut self, _wipe_type: i32, _wipe_time_ms: u64) {}

    /// C++ reference: flow_proc.cpp::tnm_return_to_sel_proc.
    fn on_syscom_proc_return_to_sel(&mut self) {}

    /// C++ reference: flow_proc.cpp::tnm_end_game_proc.
    fn on_syscom_proc_end_game(&mut self) {}

    /// C++ reference: flow_proc.cpp::tnm_end_load_proc + eng_scene.cpp::tnm_saveload_proc_end_load.
    /// Reports the internal end-load restore result. Note that cmd-level END_LOAD may already
    /// have returned success once the proc is accepted (C++ queue semantics).
    fn on_syscom_proc_end_load_result(&mut self, _ok: bool) {}

    /// C++ reference: flow_proc.cpp load-family procs update
    /// system_wipe_flag/do_frame_action_flag/do_load_after_call_flag.
    fn on_syscom_load_flow_state(&mut self, _state: VmLoadFlowState) {}

    /// C++ reference: eng_syscom.cpp::tnm_syscom_end_game -> tnm_syscom_end_save(false, false).
    /// Called immediately when END_GAME command is accepted.
    fn on_syscom_end_game_save_flush(&mut self, _state: &crate::vm::VmPersistentState) {}

    /// Host-side optional end-save persistence hook (slot-indexed).
    fn on_syscom_end_save_snapshot(&mut self, _slot_no: i32, _state: &crate::vm::VmEndSaveState) {}

    /// Host-side optional end-save existence query.
    fn on_syscom_end_save_exist(&mut self, _slot_no: i32) -> Option<bool> {
        None
    }

    /// Host-side optional end-save load hook.
    fn on_syscom_end_load_snapshot(&mut self, _slot_no: i32) -> Option<crate::vm::VmEndSaveState> {
        None
    }

    /// C++ reference: eng_syscom.cpp::tnm_syscom_return_to_menu -> tnm_save_global_on_file().
    /// Called before return-menu scene restart to allow host persistence sync.
    fn on_syscom_return_to_menu_save_global(&mut self, _state: &crate::vm::VmPersistentState) {}

    /// C++ reference: eng_syscom.cpp::tnm_syscom_return_to_menu and
    /// flow_proc.cpp::tnm_game_timer_start_proc.
    /// `moving=false` is emitted before jump; `moving=true` after restart.
    fn on_game_timer_move(&mut self, _moving: bool) {}

    /// Called periodically by the VM to check if execution should be aborted.
    fn should_interrupt(&self) -> bool {
        false
    }

    /// C++ break/step emulation flags used by CD_NL flow control.
    fn is_breaking(&self) -> bool {
        false
    }

    fn break_step_flag(&self) -> bool {
        false
    }

    fn on_break_step_line_advanced(&mut self) {}

    /// C++ cmd_sound.cpp: BGM play/oneshot/wait/ready.
    fn on_bgm_play(
        &mut self,
        _name: &str,
        _loop_flag: bool,
        _wait_flag: bool,
        _fade_in: i32,
        _fade_out: i32,
        _start_pos: i32,
        _ready: bool,
    ) {
    }
    /// C++ cmd_sound.cpp: BGM stop.
    fn on_bgm_stop(&mut self, _fade_out: i32) {}
    /// C++ cmd_sound.cpp: BGM pause.
    fn on_bgm_pause(&mut self, _fade: i32) {}
    /// C++ cmd_sound.cpp: BGM resume.
    fn on_bgm_resume(&mut self, _fade: i32, _wait: bool, _delay_time: i32) {}
    /// C++ cmd_sound.cpp: BGM set volume.
    fn on_bgm_set_volume(&mut self, _sub: i32, _vol: i32) {}
    /// C++ cmd_sound.cpp: PCM play.
    fn on_pcm_play(&mut self, _name: &str) {}
    /// C++ cmd_sound.cpp: PCM stop.
    fn on_pcm_stop(&mut self) {}
    /// C++ cmd_sound.cpp: SE play.
    fn on_se_play(&mut self, _id: i32, _name: &str) {}
    /// C++ cmd_sound.cpp: SE stop.
    fn on_se_stop(&mut self, _fade: i32) {}
    /// C++ cmd_sound.cpp: MOV play.
    fn on_mov_play(&mut self, _name: &str) {}
    /// C++ cmd_sound.cpp: MOV stop.
    fn on_mov_stop(&mut self) {}

    /// C++ eng_frame.cpp::frame_action_proc — load-after-call farcall trigger.
    /// Called when `do_load_after_call_flag` is consumed and a farcall to
    /// `load_after_call_scene` / `load_after_call_z_no` is about to execute.
    fn on_frame_action_load_after_call(&mut self, _scene: &str, _z_no: i32) {}

    /// C++ flow_script.cpp fatal parse/eof path hook.
    ///
    /// Expected host-side side effect: switch process to NONE-equivalent.
    fn on_script_fatal(&mut self, msg: &str) {
        self.on_error(msg);
    }

    /// Called by wait-related commands to allow host-level skip/fast-forward.
    fn should_skip_wait(&self) -> bool {
        false
    }

    /// Wait one frame-equivalent tick for proc-based waits (KEY_WAIT/int_event/etc).
    ///
    /// Default keeps legacy VM single-run behavior by sleeping a short duration.
    fn on_wait_frame(&mut self) {
        std::thread::sleep(std::time::Duration::from_millis(8));
    }
    /// C++ elm_counter.cpp::update_time inputs used by frame-mode counters.
    ///
    /// Returns `(past_game_time, past_real_time)` deltas for one wait/update turn.
    /// Default keeps legacy VM behavior with unit-step increments.
    fn on_frame_counter_elapsed(&mut self) -> (i32, i32) {
        (1, 1)
    }

    /// C++ reference: cmd_input.cpp::ELM_INPUT_CLEAR (input root clear all queues).
    fn on_input_clear(&mut self) {}

    /// C++ reference: cmd_input.cpp::ELM_INPUT_NEXT (input root advance all queues).
    fn on_input_next(&mut self) {}

    /// C++ reference: cmd_input.cpp::ELM_MOUSE_CLEAR (mouse-only clear).
    fn on_input_mouse_clear(&mut self) {
        self.on_input_clear();
    }

    /// C++ reference: cmd_input.cpp::ELM_MOUSE_NEXT (mouse-only frame advance).
    fn on_input_mouse_next(&mut self) {
        self.on_input_next();
    }

    /// C++ reference: cmd_input.cpp::ELM_KEYLIST_CLEAR (keyboard-only clear).
    fn on_input_keylist_clear(&mut self) {
        self.on_input_clear();
    }

    /// C++ reference: cmd_input.cpp::ELM_KEYLIST_NEXT (keyboard-only frame advance).
    fn on_input_keylist_next(&mut self) {
        self.on_input_next();
    }

    /// C++ reference: cmd_input.cpp::ELM_KEYLIST_WAIT(_FORCE).
    fn on_input_key_wait(&mut self, _force_skip_disable: bool) {}

    /// C++ reference: cmd_input.cpp::TNM_PROC_TYPE_KEY_WAIT polling condition.
    ///
    /// Returns true when any key/mouse decide/cancel down-stock is pending.
    fn on_input_key_wait_has_press_stock(&mut self) -> bool {
        false
    }

    /// C++ reference: cmd_input.cpp::TNM_PROC_TYPE_KEY_WAIT completion path.
    ///
    /// Called once KEY_WAIT detects input and consumes one frame-equivalent stock.
    fn on_input_key_wait_consume_frame(&mut self) {
        self.on_input_next();
    }

    /// C++ reference: cmd_input.cpp::ELM_MOUSE_SET_POS.
    fn on_input_set_mouse_pos(&mut self, _x: i32, _y: i32) {}

    /// C++ reference: cmd_input.cpp::tnm_command_proc_mouse.
    fn on_input_get_mouse_state(&mut self) -> VmInputMouseState {
        VmInputMouseState::default()
    }

    /// C++ reference: cmd_input.cpp::tnm_command_proc_key (regular key path).
    fn on_input_get_key_state(&mut self, _key_no: i32) -> VmInputButtonState {
        VmInputButtonState::default()
    }

    /// C++ reference: cmd_input.cpp::tnm_command_proc_key (VK_EX_DECIDE branch).
    fn on_input_get_decide_state(&mut self) -> VmInputButtonState {
        VmInputButtonState::default()
    }

    /// C++ reference: cmd_input.cpp::tnm_command_proc_key (VK_EX_CANCEL branch).
    fn on_input_get_cancel_state(&mut self) -> VmInputButtonState {
        VmInputButtonState::default()
    }

    /// C++ reference: eng_frame.cpp flick scene branch (`mouse.left.check_flick_stock/get_flick_angle`).
    fn on_input_get_left_flick_state(&mut self) -> VmFlickState {
        VmFlickState::default()
    }

    /// C++ reference: eng_frame.cpp flick scene consume path (`mouse.left.use_flick_stock`).
    fn on_input_consume_left_flick_stock(&mut self) -> bool {
        false
    }

    /// C++ eng_frame.cpp flick gating (`m_mov.is_playing`).
    fn on_movie_is_playing(&mut self) -> bool {
        false
    }

    // Screen / Effect / Quake Host callbacks (cmd_effect.cpp alignment)

    /// C++ cmd_effect.cpp: screen-level property set (x/y/z/mono/etc on effect_list[0]).
    fn on_screen_property(&mut self, _property_id: i32, _value: i32) {}

    /// C++ cmd_effect.cpp: per-effect property set.
    fn on_effect_property(&mut self, _property_id: i32, _value: i32) {}

    /// C++ cmd_effect.cpp: effect reinit.
    fn on_effect_init(&mut self) {}

    /// C++ cmd_effect.cpp: quake start (vec/dir/zoom variants).
    fn on_quake_start(&mut self, _req: VmQuakeRequest) {}

    /// C++ cmd_effect.cpp: quake end.
    fn on_quake_end(&mut self) {}

    /// C++ cmd_effect.cpp: quake->check() for WAIT/CHECK polling branches.
    fn on_quake_is_active(&mut self) -> bool {
        false
    }

    // World Host callbacks (cmd_world.cpp alignment)

    /// C++ cmd_world.cpp: world property set.
    fn on_world_property(&mut self, _property_id: i32, _value: i32) {}

    /// C++ cmd_world.cpp: create_world.
    fn on_world_create(&mut self) {}

    /// C++ cmd_world.cpp: destroy_world.
    fn on_world_destroy(&mut self) {}

    /// C++ cmd_world.cpp: world reinit.
    fn on_world_init(&mut self) {}

    /// C++ cmd_world.cpp: set_camera_eye / set_camera_pint / set_camera_up.
    fn on_world_set_camera(&mut self, _sub: i32, _x: i32, _y: i32, _z: i32) {}

    /// C++ cmd_world.cpp: calc_camera_eye / calc_camera_pint.
    fn on_world_calc_camera(&mut self, _sub: i32, _distance: i32, _rotate_h: i32, _rotate_v: i32) {}

    // PCMCH Host callbacks (cmd_sound.cpp alignment)

    /// C++ cmd_sound.cpp: PCMCH play with full named-arg parameters.
    fn on_pcmch_play(
        &mut self,
        _ch: i32,
        _name: &str,
        _loop_flag: bool,
        _wait_flag: bool,
        _fade_in: i32,
        _volume_type: i32,
        _chara_no: i32,
        _ready: bool,
    ) {
    }

    /// C++ cmd_sound.cpp: PCMCH stop.
    fn on_pcmch_stop(&mut self, _ch: i32, _fade: i32) {}

    /// C++ cmd_sound.cpp: PCMCH pause.
    fn on_pcmch_pause(&mut self, _ch: i32, _fade: i32) {}

    /// C++ cmd_sound.cpp: PCMCH resume.
    fn on_pcmch_resume(&mut self, _ch: i32, _fade: i32, _wait: bool) {}

    /// C++ cmd_sound.cpp: PCMCH set_volume.
    fn on_pcmch_set_volume(&mut self, _ch: i32, _sub: i32, _vol: i32) {}

    // Stage / Group Host callbacks (cmd_stage.cpp alignment)

    /// C++ cmd_stage.cpp: stage_list->get_sub(index, disp_out_of_range_error).
    ///
    /// Return negative when stage count is unknown.
    fn on_stage_list_get_size(&mut self) -> i32 {
        -1
    }

    /// C++ cmd_stage.cpp: group sel / sel_cancel.
    fn on_group_sel(&mut self, _stage_idx: i32, _group_idx: i32, _sub: i32) {}

    /// C++ cmd_stage.cpp: SEL/START cancel branch flags.
    fn on_group_set_cancel(
        &mut self,
        _stage_idx: i32,
        _group_idx: i32,
        _enabled: bool,
        _se_no: i32,
    ) {
    }

    /// C++ cmd_stage.cpp: group init (reinit).
    fn on_group_init(&mut self, _stage_idx: i32, _group_idx: i32) {}

    /// C++ cmd_stage.cpp: group start / start_cancel.
    fn on_group_start(&mut self, _stage_idx: i32, _group_idx: i32, _sub: i32) {}

    /// C++ cmd_stage.cpp: group on_hit_no.
    fn on_group_on_hit_no(&mut self, _stage_idx: i32, _group_idx: i32, _button_no: i32) {}

    /// C++ cmd_stage.cpp: group on_pushed_no.
    fn on_group_on_pushed_no(&mut self, _stage_idx: i32, _group_idx: i32, _button_no: i32) {}

    /// C++ cmd_stage.cpp: group on_decided_no.
    fn on_group_on_decided_no(&mut self, _stage_idx: i32, _group_idx: i32, _button_no: i32) {}

    /// C++ cmd_stage.cpp: group end.
    fn on_group_end(&mut self, _stage_idx: i32, _group_idx: i32) {}

    /// C++ cmd_stage.cpp: group_list alloc (clear + resize).
    fn on_group_alloc(&mut self, _stage_idx: i32, _count: i32) {}

    /// C++ cmd_stage.cpp: group_list free (clear).
    fn on_group_free(&mut self, _stage_idx: i32) {}

    /// C++ cmd_stage.cpp: group_list->get_sub(index, disp_out_of_range_error).
    ///
    /// Return negative when the host cannot provide a concrete size yet.
    fn on_group_list_get_size(&mut self, _stage_idx: i32) -> i32 {
        -1
    }

    /// C++ cmd_mwnd.cpp: mwnd_list->get_sub(index, disp_out_of_range_error).
    fn on_mwnd_list_get_size(&mut self) -> i32 {
        -1
    }

    /// C++ cmd_world.cpp: world_list->get_sub(index, disp_out_of_range_error).
    fn on_world_list_get_size(&mut self) -> i32 {
        -1
    }

    /// C++ cmd_effect.cpp: effect_list->get_sub(index, disp_out_of_range_error).
    fn on_effect_list_get_size(&mut self) -> i32 {
        -1
    }
    fn on_effect_list_resize(&mut self, _size: i32) {}

    /// C++ cmd_effect.cpp: quake_list->get_sub(index, disp_out_of_range_error).
    fn on_quake_list_get_size(&mut self) -> i32 {
        -1
    }
    fn on_quake_list_resize(&mut self, _size: i32) {}

    /// C++ cmd_others.cpp: int_event_list->get_sub(index, disp_out_of_range_error).
    fn on_int_event_list_get_size(&mut self, _owner_id: i32) -> i32 {
        -1
    }
    fn on_int_event_list_resize(&mut self, _owner_id: i32, _size: i32) {}

    /// C++ cmd_stage.cpp: group property/query get.
    fn on_group_get(&mut self, _stage_idx: i32, _group_idx: i32, _query_id: i32) -> i32 {
        -1
    }

    /// C++ cmd_stage.cpp: group property set (order/layer/cancel_priority).
    fn on_group_property(
        &mut self,
        _stage_idx: i32,
        _group_idx: i32,
        _property_id: i32,
        _value: i32,
    ) {
    }

    /// C++ cmd_stage.cpp SEL_BTN_OBJ proc-style polling.
    fn on_group_wait_result(&mut self, _stage_idx: i32, _group_idx: i32) -> Option<i32> {
        None
    }

    /// C++ cmd_syscom.cpp: create_capture_buffer.
    fn on_syscom_create_capture_buffer(&mut self, _width: i32, _height: i32) {}

    /// C++ cmd_syscom.cpp: destroy_capture_buffer.
    fn on_syscom_destroy_capture_buffer(&mut self) {}

    /// C++ cmd_syscom.cpp: capture_to_capture_buffer / capture_and_save_buffer_to_png.
    fn on_syscom_capture_to_buffer(&mut self, _x: i32, _y: i32, _save_png_path: &str) {}

    /// C++ cmd_syscom.cpp: save_capture_buffer_to_file.
    fn on_syscom_save_capture_buffer_to_file(&mut self, _req: &VmCaptureFileOp) -> bool {
        false
    }

    /// C++ cmd_syscom.cpp: load_flag_from_capture_file.
    fn on_syscom_load_flag_from_capture_file(
        &mut self,
        _req: &VmCaptureFileOp,
    ) -> Option<VmCaptureFlagPayload> {
        None
    }

    include!("api_int_event_hooks.rs");

    include!("api_object_hooks.rs");

    // Mwnd Host callbacks (cmd_mwnd.cpp alignment)

    /// C++ cmd_mwnd.cpp: mwnd action command (sub_id identifies the command).
    fn on_mwnd_action(&mut self, _sub_id: i32, _args: &[Prop]) {}

    /// C++ cmd_mwnd.cpp: mwnd property get.
    fn on_mwnd_get(&mut self, _sub_id: i32) -> i32 {
        0
    }

    // Counter / Database / Others Host callbacks

    /// C++ cmd_others.cpp: counter action (set/reset/start/stop/resume/wait).
    fn on_counter_action(&mut self, _action: i32, _args: &[Prop]) {}
}
