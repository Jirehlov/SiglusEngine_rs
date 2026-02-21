use super::*;

impl Vm {
    pub(super) fn snapshot_local_state(&self) -> VmLocalState {
        VmLocalState {
            scene: self.scene.clone(),
            lexer: self.lexer.clone(),
            stack: self.stack.clone(),
            frames: self.frames.clone(),
            scene_title: self.scene_title.clone(),
            user_prop_forms: self.user_prop_forms.clone(),
            user_prop_values: self.user_prop_values.clone(),
            frame_action: self.frame_action.clone(),
            frame_action_ch: self.frame_action_ch.clone(),
            key_wait_proc: self.key_wait_proc,
            flags_a: self.flags_a.clone(),
            flags_b: self.flags_b.clone(),
            flags_c: self.flags_c.clone(),
            flags_d: self.flags_d.clone(),
            flags_e: self.flags_e.clone(),
            flags_f: self.flags_f.clone(),
            flags_x: self.flags_x.clone(),
            flags_g: self.flags_g.clone(),
            flags_z: self.flags_z.clone(),
            flags_s: self.flags_s.clone(),
            flags_m: self.flags_m.clone(),
            global_namae: self.global_namae.clone(),
            local_namae: self.local_namae.clone(),
            save_point_set: self.save_point_set,
            sel_point_set: self.sel_point_set,
            save_point_snapshot: self.save_point_snapshot.clone(),
            sel_point_snapshot: self.sel_point_snapshot.clone(),
            sel_point_stock: self.sel_point_stock.clone(),
            cur_mwnd_element: self.cur_mwnd_element.clone(),
            cur_sel_mwnd_element: self.cur_sel_mwnd_element.clone(),
            last_sel_msg: self.last_sel_msg.clone(),
            hide_mwnd_onoff_flag: self.hide_mwnd_onoff_flag,
            hide_mwnd_enable_flag: self.hide_mwnd_enable_flag,
            hide_mwnd_exist_flag: self.hide_mwnd_exist_flag,
            read_skip_onoff_flag: self.read_skip_onoff_flag,
            read_skip_enable_flag: self.read_skip_enable_flag,
            read_skip_exist_flag: self.read_skip_exist_flag,
            auto_mode_onoff_flag: self.auto_mode_onoff_flag,
            auto_mode_enable_flag: self.auto_mode_enable_flag,
            auto_mode_exist_flag: self.auto_mode_exist_flag,
            msg_back_enable_flag: self.msg_back_enable_flag,
            msg_back_exist_flag: self.msg_back_exist_flag,
            msg_back_open_flag: self.msg_back_open_flag,
            msg_back_has_message: self.msg_back_has_message,
            msg_back_disable_flag: self.msg_back_disable_flag,
            msg_back_off_flag: self.msg_back_off_flag,
            msg_back_disp_off_flag: self.msg_back_disp_off_flag,
            msg_back_proc_off_flag: self.msg_back_proc_off_flag,
            return_to_sel_enable_flag: self.return_to_sel_enable_flag,
            return_to_sel_exist_flag: self.return_to_sel_exist_flag,
            return_to_menu_enable_flag: self.return_to_menu_enable_flag,
            return_to_menu_exist_flag: self.return_to_menu_exist_flag,
            save_enable_flag: self.save_enable_flag,
            save_exist_flag: self.save_exist_flag,
            load_enable_flag: self.load_enable_flag,
            load_exist_flag: self.load_exist_flag,
            end_game_enable_flag: self.end_game_enable_flag,
            end_game_exist_flag: self.end_game_exist_flag,
            game_end_flag: self.game_end_flag,
            game_end_no_warning_flag: self.game_end_no_warning_flag,
            game_end_save_done_flag: self.game_end_save_done_flag,
            no_wipe_anime_onoff_flag: self.no_wipe_anime_onoff_flag,
            skip_wipe_anime_onoff_flag: self.skip_wipe_anime_onoff_flag,
            script_skip_unread_message_flag: self.script_skip_unread_message_flag,
            script_stage_time_stop_flag: self.script_stage_time_stop_flag,
            system_wipe_flag: self.system_wipe_flag,
            do_frame_action_flag: self.do_frame_action_flag,
            do_load_after_call_flag: self.do_load_after_call_flag,
            game_timer_move_flag: self.game_timer_move_flag,
            syscom_menu_disable_flag: self.syscom_menu_disable_flag,
            last_pc: self.last_pc,
            last_line_no: self.last_line_no,
            last_scene: self.last_scene.clone(),
            // ----- Script runtime flags -----
            script_dont_set_save_point: self.script_dont_set_save_point,
            script_skip_disable: self.script_skip_disable,
            script_ctrl_disable: self.script_ctrl_disable,
            script_not_stop_skip_by_click: self.script_not_stop_skip_by_click,
            script_not_skip_msg_by_click: self.script_not_skip_msg_by_click,
            script_auto_mode_flag: self.script_auto_mode_flag,
            script_auto_mode_moji_wait: self.script_auto_mode_moji_wait,
            script_auto_mode_min_wait: self.script_auto_mode_min_wait,
            script_auto_mode_moji_cnt: self.script_auto_mode_moji_cnt,
            script_mouse_cursor_hide_onoff: self.script_mouse_cursor_hide_onoff,
            script_mouse_cursor_hide_time: self.script_mouse_cursor_hide_time,
            script_msg_speed: self.script_msg_speed,
            script_msg_nowait: self.script_msg_nowait,
            script_async_msg_mode: self.script_async_msg_mode,
            script_async_msg_mode_once: self.script_async_msg_mode_once,
            script_hide_mwnd_disable: self.script_hide_mwnd_disable,
            script_cursor_disp_off: self.script_cursor_disp_off,
            script_cursor_move_by_key_disable: self.script_cursor_move_by_key_disable,
            script_key_disable: self.script_key_disable,
            script_mwnd_anime_off_flag: self.script_mwnd_anime_off_flag,
            script_mwnd_anime_on_flag: self.script_mwnd_anime_on_flag,
            script_mwnd_disp_off_flag: self.script_mwnd_disp_off_flag,
            script_koe_dont_stop_on_flag: self.script_koe_dont_stop_on_flag,
            script_koe_dont_stop_off_flag: self.script_koe_dont_stop_off_flag,
            script_shortcut_disable: self.script_shortcut_disable,
            script_quake_stop_flag: self.script_quake_stop_flag,
            script_emote_mouth_stop_flag: self.script_emote_mouth_stop_flag,
            script_bgmfade_flag: self.script_bgmfade_flag,
            script_vsync_wait_off_flag: self.script_vsync_wait_off_flag,
            script_skip_trigger: self.script_skip_trigger,
            script_ignore_r_flag: self.script_ignore_r_flag,
            script_cursor_no: self.script_cursor_no,
            script_time_stop_flag: self.script_time_stop_flag,
            script_counter_time_stop_flag: self.script_counter_time_stop_flag,
            script_frame_action_time_stop_flag: self.script_frame_action_time_stop_flag,
            script_font_name: self.script_font_name.clone(),
            script_font_bold: self.script_font_bold,
            script_font_shadow: self.script_font_shadow,
            script_allow_joypad_mode_onoff: self.script_allow_joypad_mode_onoff,
        }
    }
    pub(super) fn apply_local_state(&mut self, st: &VmLocalState) {
        self.scene = st.scene.clone();
        self.lexer = st.lexer.clone();
        self.stack = st.stack.clone();
        self.frames = st.frames.clone();
        self.scene_title = st.scene_title.clone();
        self.user_prop_forms = st.user_prop_forms.clone();
        self.user_prop_values = st.user_prop_values.clone();
        self.frame_action = st.frame_action.clone();
        self.frame_action_ch = st.frame_action_ch.clone();
        self.key_wait_proc = st.key_wait_proc;
        self.flags_a = st.flags_a.clone();
        self.flags_b = st.flags_b.clone();
        self.flags_c = st.flags_c.clone();
        self.flags_d = st.flags_d.clone();
        self.flags_e = st.flags_e.clone();
        self.flags_f = st.flags_f.clone();
        self.flags_x = st.flags_x.clone();
        self.flags_g = st.flags_g.clone();
        self.flags_z = st.flags_z.clone();
        self.flags_s = st.flags_s.clone();
        self.flags_m = st.flags_m.clone();
        self.global_namae = st.global_namae.clone();
        self.local_namae = st.local_namae.clone();
        self.save_point_set = st.save_point_set;
        self.sel_point_set = st.sel_point_set;
        self.cur_mwnd_element = st.cur_mwnd_element.clone();
        self.cur_sel_mwnd_element = st.cur_sel_mwnd_element.clone();
        self.last_sel_msg = st.last_sel_msg.clone();
        self.hide_mwnd_onoff_flag = st.hide_mwnd_onoff_flag;
        self.hide_mwnd_enable_flag = st.hide_mwnd_enable_flag;
        self.hide_mwnd_exist_flag = st.hide_mwnd_exist_flag;
        self.read_skip_onoff_flag = st.read_skip_onoff_flag;
        self.read_skip_enable_flag = st.read_skip_enable_flag;
        self.read_skip_exist_flag = st.read_skip_exist_flag;
        self.auto_mode_onoff_flag = st.auto_mode_onoff_flag;
        self.auto_mode_enable_flag = st.auto_mode_enable_flag;
        self.auto_mode_exist_flag = st.auto_mode_exist_flag;
        self.msg_back_enable_flag = st.msg_back_enable_flag;
        self.msg_back_exist_flag = st.msg_back_exist_flag;
        self.msg_back_open_flag = st.msg_back_open_flag;
        self.msg_back_has_message = st.msg_back_has_message;
        self.msg_back_disable_flag = st.msg_back_disable_flag;
        self.msg_back_off_flag = st.msg_back_off_flag;
        self.msg_back_disp_off_flag = st.msg_back_disp_off_flag;
        self.msg_back_proc_off_flag = st.msg_back_proc_off_flag;
        self.return_to_sel_enable_flag = st.return_to_sel_enable_flag;
        self.return_to_sel_exist_flag = st.return_to_sel_exist_flag;
        self.return_to_menu_enable_flag = st.return_to_menu_enable_flag;
        self.return_to_menu_exist_flag = st.return_to_menu_exist_flag;
        self.save_enable_flag = st.save_enable_flag;
        self.save_exist_flag = st.save_exist_flag;
        self.load_enable_flag = st.load_enable_flag;
        self.load_exist_flag = st.load_exist_flag;
        self.end_game_enable_flag = st.end_game_enable_flag;
        self.end_game_exist_flag = st.end_game_exist_flag;
        self.game_end_flag = st.game_end_flag;
        self.game_end_no_warning_flag = st.game_end_no_warning_flag;
        self.game_end_save_done_flag = st.game_end_save_done_flag;
        self.no_wipe_anime_onoff_flag = st.no_wipe_anime_onoff_flag;
        self.skip_wipe_anime_onoff_flag = st.skip_wipe_anime_onoff_flag;
        self.options.no_wipe_anime = self.no_wipe_anime_onoff_flag != 0;
        self.options.skip_wipe_anime = self.skip_wipe_anime_onoff_flag != 0;
        self.script_skip_unread_message_flag = st.script_skip_unread_message_flag;
        self.script_stage_time_stop_flag = st.script_stage_time_stop_flag;
        self.system_wipe_flag = st.system_wipe_flag;
        self.do_frame_action_flag = st.do_frame_action_flag;
        self.do_load_after_call_flag = st.do_load_after_call_flag;
        self.game_timer_move_flag = st.game_timer_move_flag;
        self.syscom_menu_disable_flag = st.syscom_menu_disable_flag;
        self.last_pc = st.last_pc;
        self.last_line_no = st.last_line_no;
        self.last_scene = st.last_scene.clone();
        // ----- Script runtime flags -----
        self.script_dont_set_save_point = st.script_dont_set_save_point;
        self.script_skip_disable = st.script_skip_disable;
        self.script_ctrl_disable = st.script_ctrl_disable;
        self.script_not_stop_skip_by_click = st.script_not_stop_skip_by_click;
        self.script_not_skip_msg_by_click = st.script_not_skip_msg_by_click;
        self.script_auto_mode_flag = st.script_auto_mode_flag;
        self.script_auto_mode_moji_wait = st.script_auto_mode_moji_wait;
        self.script_auto_mode_min_wait = st.script_auto_mode_min_wait;
        self.script_auto_mode_moji_cnt = st.script_auto_mode_moji_cnt;
        self.script_mouse_cursor_hide_onoff = st.script_mouse_cursor_hide_onoff;
        self.script_mouse_cursor_hide_time = st.script_mouse_cursor_hide_time;
        self.script_msg_speed = st.script_msg_speed;
        self.script_msg_nowait = st.script_msg_nowait;
        self.script_async_msg_mode = st.script_async_msg_mode;
        self.script_async_msg_mode_once = st.script_async_msg_mode_once;
        self.script_hide_mwnd_disable = st.script_hide_mwnd_disable;
        self.script_cursor_disp_off = st.script_cursor_disp_off;
        self.script_cursor_move_by_key_disable = st.script_cursor_move_by_key_disable;
        self.script_key_disable = st.script_key_disable;
        self.script_mwnd_anime_off_flag = st.script_mwnd_anime_off_flag;
        self.script_mwnd_anime_on_flag = st.script_mwnd_anime_on_flag;
        self.script_mwnd_disp_off_flag = st.script_mwnd_disp_off_flag;
        self.script_koe_dont_stop_on_flag = st.script_koe_dont_stop_on_flag;
        self.script_koe_dont_stop_off_flag = st.script_koe_dont_stop_off_flag;
        self.script_shortcut_disable = st.script_shortcut_disable;
        self.script_quake_stop_flag = st.script_quake_stop_flag;
        self.script_emote_mouth_stop_flag = st.script_emote_mouth_stop_flag;
        self.script_bgmfade_flag = st.script_bgmfade_flag;
        self.script_vsync_wait_off_flag = st.script_vsync_wait_off_flag;
        self.script_skip_trigger = st.script_skip_trigger;
        self.script_ignore_r_flag = st.script_ignore_r_flag;
        self.script_cursor_no = st.script_cursor_no;
        self.script_time_stop_flag = st.script_time_stop_flag;
        self.script_counter_time_stop_flag = st.script_counter_time_stop_flag;
        self.script_frame_action_time_stop_flag = st.script_frame_action_time_stop_flag;
        self.script_font_name = st.script_font_name.clone();
        self.script_font_bold = st.script_font_bold;
        self.script_font_shadow = st.script_font_shadow;
        self.script_allow_joypad_mode_onoff = st.script_allow_joypad_mode_onoff;
        self.save_point_snapshot = st.save_point_snapshot.clone();
        self.sel_point_snapshot = st.sel_point_snapshot.clone();
        self.sel_point_stock = st.sel_point_stock.clone();
        self.wipe_end_at = None;
        self.halted = false;
    }
    pub fn set_options(&mut self, options: VmOptions) {
        self.options = options;
        self.no_wipe_anime_onoff_flag = if self.options.no_wipe_anime { 1 } else { 0 };
        self.skip_wipe_anime_onoff_flag = if self.options.skip_wipe_anime { 1 } else { 0 };
        self.system_extra_int_values = self.options.system_extra_int_values.clone();
        self.system_extra_str_values = self.options.system_extra_str_values.clone();
    }
    pub fn snapshot_persistent_state(&self) -> VmPersistentState {
        VmPersistentState {
            flags_a: self.flags_a.clone(),
            flags_b: self.flags_b.clone(),
            flags_c: self.flags_c.clone(),
            flags_d: self.flags_d.clone(),
            flags_e: self.flags_e.clone(),
            flags_f: self.flags_f.clone(),
            flags_x: self.flags_x.clone(),
            flags_g: self.flags_g.clone(),
            flags_z: self.flags_z.clone(),
            flags_s: self.flags_s.clone(),
            flags_m: self.flags_m.clone(),
            global_namae: self.global_namae.clone(),
            local_namae: self.local_namae.clone(),
            save_point_set: self.save_point_set,
            sel_point_set: self.sel_point_set,
        }
    }
    pub fn snapshot_end_save_state(&self) -> VmEndSaveState {
        VmEndSaveState {
            scene_title: self.scene_title.clone(),
            message: self.last_sel_msg.clone(),
            persistent: self.snapshot_persistent_state(),
            // STUB(C++: eng_syscom.cpp::tnm_saveload_proc_end_save local payload)
            // Gap: full VmLocalState parity is still pending (e.g. additional mwnd/msg-back side paths and failure semantics).
            // Expected: end-load should eventually match C++ local continuation semantics branch-by-branch.
            // Validation: run END_SAVE -> END_LOAD cross-process and verify call/user-prop/frame-action continuity.
            runtime: Some(self.snapshot_end_save_runtime_state()),
        }
    }
    pub fn apply_persistent_state(&mut self, st: &VmPersistentState) {
        fn restore_fixed_i32(dst: &mut Vec<i32>, src: &[i32]) {
            if src.len() == FLAG_LIST_SIZE {
                *dst = src.to_vec();
            }
        }
        fn restore_fixed_string(dst: &mut Vec<String>, src: &[String]) {
            if src.len() == FLAG_LIST_SIZE {
                *dst = src.to_vec();
            }
        }
        restore_fixed_i32(&mut self.flags_a, &st.flags_a);
        restore_fixed_i32(&mut self.flags_b, &st.flags_b);
        restore_fixed_i32(&mut self.flags_c, &st.flags_c);
        restore_fixed_i32(&mut self.flags_d, &st.flags_d);
        restore_fixed_i32(&mut self.flags_e, &st.flags_e);
        restore_fixed_i32(&mut self.flags_f, &st.flags_f);
        restore_fixed_i32(&mut self.flags_x, &st.flags_x);
        restore_fixed_i32(&mut self.flags_g, &st.flags_g);
        restore_fixed_i32(&mut self.flags_z, &st.flags_z);
        restore_fixed_string(&mut self.flags_s, &st.flags_s);
        restore_fixed_string(&mut self.flags_m, &st.flags_m);
        restore_fixed_string(&mut self.global_namae, &st.global_namae);
        restore_fixed_string(&mut self.local_namae, &st.local_namae);
        self.save_point_set = st.save_point_set;
        self.sel_point_set = st.sel_point_set;
        self.save_point_snapshot = if self.save_point_set {
            Some(self.snapshot_persistent_state())
        } else {
            None
        };
        self.sel_point_snapshot = if self.sel_point_set {
            Some(self.snapshot_persistent_state())
        } else {
            None
        };
        self.sel_point_stock = None;
    }
    pub(super) fn clear_transient_flow_state(&mut self) {
        self.stack = IfcStack::default();
        self.last_sel_msg.clear();
        self.sel_point_stock = None;
        self.return_scene_once = None;
        self.key_wait_proc = KeyWaitProc::default();
    }
}
