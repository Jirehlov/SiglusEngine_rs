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

    /// Called when VM location/title context changes (for window caption, overlays, etc.).
    fn on_location(&mut self, _scene_title: &str, _scene: &str, _line_no: i32) {}

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
    fn on_bgm_play(&mut self, _name: &str, _loop_flag: bool, _wait_flag: bool, _fade_in: i32, _fade_out: i32, _ready: bool) {}
    /// C++ cmd_sound.cpp: BGM stop.
    fn on_bgm_stop(&mut self, _fade_out: i32) {}
    /// C++ cmd_sound.cpp: BGM pause.
    fn on_bgm_pause(&mut self, _fade: i32) {}
    /// C++ cmd_sound.cpp: BGM resume.
    fn on_bgm_resume(&mut self, _fade: i32, _wait: bool) {}
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

    // ---------------------------------------------------------------
    // Screen / Effect / Quake Host callbacks (cmd_effect.cpp alignment)
    // ---------------------------------------------------------------

    /// C++ cmd_effect.cpp: screen-level property set (x/y/z/mono/etc on effect_list[0]).
    fn on_screen_property(&mut self, _property_id: i32, _value: i32) {}

    /// C++ cmd_effect.cpp: per-effect property set.
    fn on_effect_property(&mut self, _property_id: i32, _value: i32) {}

    /// C++ cmd_effect.cpp: effect reinit.
    fn on_effect_init(&mut self) {}

    /// C++ cmd_effect.cpp: quake start (sub identifies variant).
    fn on_quake_start(&mut self, _sub: i32) {}

    /// C++ cmd_effect.cpp: quake end.
    fn on_quake_end(&mut self) {}

    // ---------------------------------------------------------------
    // World Host callbacks (cmd_world.cpp alignment)
    // ---------------------------------------------------------------

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

    // ---------------------------------------------------------------
    // PCMCH Host callbacks (cmd_sound.cpp alignment)
    // ---------------------------------------------------------------

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
    ) {}

    /// C++ cmd_sound.cpp: PCMCH stop.
    fn on_pcmch_stop(&mut self, _ch: i32, _fade: i32) {}

    /// C++ cmd_sound.cpp: PCMCH pause.
    fn on_pcmch_pause(&mut self, _ch: i32, _fade: i32) {}

    /// C++ cmd_sound.cpp: PCMCH resume.
    fn on_pcmch_resume(&mut self, _ch: i32, _fade: i32, _wait: bool) {}

    /// C++ cmd_sound.cpp: PCMCH set_volume.
    fn on_pcmch_set_volume(&mut self, _ch: i32, _sub: i32, _vol: i32) {}
}
