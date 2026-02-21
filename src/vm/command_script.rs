// Complete script sub-command routing — aligns with C++ cmd_script.cpp
use super::*;

impl Vm {
    /// Route `global.script.<sub>` commands. Returns `true` if handled.
    pub(super) fn try_command_script(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        let method = element[0];

        // Helper: read first int arg, default 0
        let arg_int = |idx: usize| -> i32 {
            match args.get(idx).map(|p| &p.value) {
                Some(PropValue::Int(v)) => *v,
                _ => 0,
            }
        };
        let arg_str = |idx: usize| -> String {
            match args.get(idx).map(|p| &p.value) {
                Some(PropValue::Str(s)) => s.clone(),
                _ => String::new(),
            }
        };

        use crate::elm::script::*;

        match method {
            // ----- savepoint -----
            ELM_SCRIPT_SET_AUTO_SAVEPOINT_OFF => {
                self.script_dont_set_save_point = true;
            }
            ELM_SCRIPT_SET_AUTO_SAVEPOINT_ON => {
                self.script_dont_set_save_point = false;
            }

            // ----- skip control -----
            ELM_SCRIPT_SET_SKIP_DISABLE => {
                self.script_skip_disable = true;
            }
            ELM_SCRIPT_SET_SKIP_ENABLE => {
                self.script_skip_disable = false;
            }
            ELM_SCRIPT_SET_SKIP_DISABLE_FLAG => {
                self.script_skip_disable = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_SKIP_DISABLE_FLAG => {
                self.stack.push_int(if self.script_skip_disable { 1 } else { 0 });
                return true;
            }
            ELM_SCRIPT_SET_CTRL_SKIP_DISABLE => {
                self.script_ctrl_disable = true;
            }
            ELM_SCRIPT_SET_CTRL_SKIP_ENABLE => {
                self.script_ctrl_disable = false;
            }
            ELM_SCRIPT_SET_CTRL_SKIP_DISABLE_FLAG => {
                self.script_ctrl_disable = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_CTRL_SKIP_DISABLE_FLAG => {
                self.stack.push_int(if self.script_ctrl_disable { 1 } else { 0 });
                return true;
            }
            ELM_SCRIPT_CHECK_SKIP => {
                // Headless: never skipping
                self.stack.push_int(0);
                return true;
            }
            ELM_SCRIPT_SET_STOP_SKIP_BY_KEY_DISABLE => {
                self.script_not_stop_skip_by_click = true;
            }
            ELM_SCRIPT_SET_STOP_SKIP_BY_KEY_ENABLE => {
                self.script_not_stop_skip_by_click = false;
            }
            ELM_SCRIPT_SET_END_MSG_BY_KEY_DISABLE => {
                self.script_not_skip_msg_by_click = true;
            }
            ELM_SCRIPT_SET_END_MSG_BY_KEY_ENABLE => {
                self.script_not_skip_msg_by_click = false;
            }

            // ----- skip unread message -----
            ELM_SCRIPT_SET_SKIP_UNREAD_MESSAGE_FLAG => {
                self.script_skip_unread_message_flag = arg_int(0);
            }
            ELM_SCRIPT_GET_SKIP_UNREAD_MESSAGE_FLAG => {
                self.stack.push_int(self.script_skip_unread_message_flag);
                return true;
            }

            // ----- auto mode -----
            ELM_SCRIPT_START_AUTO_MODE => {
                self.script_auto_mode_flag = true;
            }
            ELM_SCRIPT_END_AUTO_MODE => {
                self.script_auto_mode_flag = false;
            }
            ELM_SCRIPT_SET_AUTO_MODE_MOJI_WAIT => {
                self.script_auto_mode_moji_wait = arg_int(0);
            }
            ELM_SCRIPT_SET_AUTO_MODE_MOJI_WAIT_DEFAULT => {
                self.script_auto_mode_moji_wait = -1;
            }
            ELM_SCRIPT_GET_AUTO_MODE_MOJI_WAIT => {
                self.stack.push_int(self.script_auto_mode_moji_wait);
                return true;
            }
            ELM_SCRIPT_SET_AUTO_MODE_MIN_WAIT => {
                self.script_auto_mode_min_wait = arg_int(0);
            }
            ELM_SCRIPT_SET_AUTO_MODE_MIN_WAIT_DEFAULT => {
                self.script_auto_mode_min_wait = -1;
            }
            ELM_SCRIPT_GET_AUTO_MODE_MIN_WAIT => {
                self.stack.push_int(self.script_auto_mode_min_wait);
                return true;
            }
            ELM_SCRIPT_SET_AUTO_MODE_MOJI_CNT => {
                self.script_auto_mode_moji_cnt = arg_int(0);
            }

            // ----- mouse cursor hide -----
            ELM_SCRIPT_SET_MOUSE_CURSOR_HIDE_ONOFF => {
                self.script_mouse_cursor_hide_onoff = arg_int(0);
            }
            ELM_SCRIPT_SET_MOUSE_CURSOR_HIDE_ONOFF_DEFAULT => {
                self.script_mouse_cursor_hide_onoff = -1;
            }
            ELM_SCRIPT_GET_MOUSE_CURSOR_HIDE_ONOFF => {
                self.stack.push_int(self.script_mouse_cursor_hide_onoff);
                return true;
            }
            ELM_SCRIPT_SET_MOUSE_CURSOR_HIDE_TIME => {
                self.script_mouse_cursor_hide_time = arg_int(0);
            }
            ELM_SCRIPT_SET_MOUSE_CURSOR_HIDE_TIME_DEFAULT => {
                self.script_mouse_cursor_hide_time = -1;
            }
            ELM_SCRIPT_GET_MOUSE_CURSOR_HIDE_TIME => {
                self.stack.push_int(self.script_mouse_cursor_hide_time);
                return true;
            }

            // ----- message speed / nowait -----
            ELM_SCRIPT_SET_MESSAGE_SPEED => {
                self.script_msg_speed = arg_int(0);
            }
            ELM_SCRIPT_SET_MESSAGE_SPEED_DEFAULT => {
                self.script_msg_speed = -1;
            }
            ELM_SCRIPT_GET_MESSAGE_SPEED => {
                self.stack.push_int(self.script_msg_speed);
                return true;
            }
            ELM_SCRIPT_SET_MESSAGE_NOWAIT_FLAG => {
                self.script_msg_nowait = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_MESSAGE_NOWAIT_FLAG => {
                self.stack.push_int(if self.script_msg_nowait { 1 } else { 0 });
                return true;
            }

            // ----- msg async mode -----
            ELM_SCRIPT_SET_MSG_ASYNC_MODE_ON => {
                self.script_async_msg_mode = true;
                self.script_async_msg_mode_once = false;
            }
            ELM_SCRIPT_SET_MSG_ASYNC_MODE_ON_ONCE => {
                self.script_async_msg_mode = true;
                self.script_async_msg_mode_once = true;
            }
            ELM_SCRIPT_SET_MSG_ASYNC_MODE_OFF => {
                self.script_async_msg_mode = false;
                self.script_async_msg_mode_once = false;
            }

            // ----- hide mwnd -----
            ELM_SCRIPT_SET_HIDE_MWND_DISABLE => {
                self.script_hide_mwnd_disable = true;
            }
            ELM_SCRIPT_SET_HIDE_MWND_ENABLE => {
                self.script_hide_mwnd_disable = false;
            }

            // ----- msg_back (already partially handled in command_head) -----
            ELM_SCRIPT_SET_MSG_BACK_DISABLE => {
                self.msg_back_disable_flag = 1;
            }
            ELM_SCRIPT_SET_MSG_BACK_ENABLE => {
                self.msg_back_disable_flag = 0;
            }
            ELM_SCRIPT_SET_MSG_BACK_OFF => {
                self.msg_back_off_flag = 1;
            }
            ELM_SCRIPT_SET_MSG_BACK_ON => {
                self.msg_back_off_flag = 0;
            }
            ELM_SCRIPT_SET_MSG_BACK_DISP_OFF => {
                self.msg_back_disp_off_flag = 1;
                host.on_msg_back_state(false);
                host.on_msg_back_display(false);
            }
            ELM_SCRIPT_SET_MSG_BACK_DISP_ON => {
                self.msg_back_disp_off_flag = 0;
                host.on_msg_back_display(true);
                if self.msg_back_open_flag != 0 {
                    host.on_msg_back_state(true);
                }
            }
            ELM_SCRIPT_SET_MSG_BACK_PROC_OFF => {
                self.msg_back_proc_off_flag = 1;
            }
            ELM_SCRIPT_SET_MSG_BACK_PROC_ON => {
                self.msg_back_proc_off_flag = 0;
            }

            // ----- mouse disp -----
            ELM_SCRIPT_SET_MOUSE_DISP_OFF => {
                self.script_cursor_disp_off = true;
            }
            ELM_SCRIPT_SET_MOUSE_DISP_ON => {
                self.script_cursor_disp_off = false;
            }

            // ----- mouse move by key -----
            ELM_SCRIPT_SET_MOUSE_MOVE_BY_KEY_DISABLE => {
                self.script_cursor_move_by_key_disable = true;
            }
            ELM_SCRIPT_SET_MOUSE_MOVE_BY_KEY_ENABLE => {
                self.script_cursor_move_by_key_disable = false;
            }

            // ----- key disable -----
            ELM_SCRIPT_SET_KEY_DISABLE => {
                let key = arg_int(0);
                if (0..256).contains(&key) {
                    self.script_key_disable[key as usize] = true;
                }
            }
            ELM_SCRIPT_SET_KEY_ENABLE => {
                let key = arg_int(0);
                if (0..256).contains(&key) {
                    self.script_key_disable[key as usize] = false;
                }
            }

            // ----- mwnd anime -----
            ELM_SCRIPT_SET_MWND_ANIME_OFF_FLAG => {
                self.script_mwnd_anime_off_flag = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_MWND_ANIME_OFF_FLAG => {
                self.stack.push_int(if self.script_mwnd_anime_off_flag { 1 } else { 0 });
                return true;
            }
            ELM_SCRIPT_SET_MWND_ANIME_ON_FLAG => {
                self.script_mwnd_anime_on_flag = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_MWND_ANIME_ON_FLAG => {
                self.stack.push_int(if self.script_mwnd_anime_on_flag { 1 } else { 0 });
                return true;
            }

            // ----- mwnd disp off -----
            ELM_SCRIPT_SET_MWND_DISP_OFF_FLAG => {
                self.script_mwnd_disp_off_flag = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_MWND_DISP_OFF_FLAG => {
                self.stack.push_int(if self.script_mwnd_disp_off_flag { 1 } else { 0 });
                return true;
            }

            // ----- koe dont stop -----
            ELM_SCRIPT_SET_KOE_DONT_STOP_ON_FLAG => {
                self.script_koe_dont_stop_on_flag = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_KOE_DONT_STOP_ON_FLAG => {
                self.stack.push_int(if self.script_koe_dont_stop_on_flag { 1 } else { 0 });
                return true;
            }
            ELM_SCRIPT_SET_KOE_DONT_STOP_OFF_FLAG => {
                self.script_koe_dont_stop_off_flag = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_KOE_DONT_STOP_OFF_FLAG => {
                self.stack.push_int(if self.script_koe_dont_stop_off_flag { 1 } else { 0 });
                return true;
            }

            // ----- shortcut -----
            ELM_SCRIPT_SET_SHORTCUT_ENABLE => {
                self.script_shortcut_disable = false;
            }
            ELM_SCRIPT_SET_SHORTCUT_DISABLE => {
                self.script_shortcut_disable = true;
            }

            // ----- quake stop -----
            ELM_SCRIPT_SET_QUAKE_STOP_FLAG => {
                self.script_quake_stop_flag = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_QUAKE_STOP_FLAG => {
                self.stack.push_int(if self.script_quake_stop_flag { 1 } else { 0 });
                return true;
            }

            // ----- emote mouth stop -----
            ELM_SCRIPT_SET_EMOTE_MOUTH_STOP_FLAG => {
                self.script_emote_mouth_stop_flag = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_EMOTE_MOUTH_STOP_FLAG => {
                self.stack.push_int(if self.script_emote_mouth_stop_flag { 1 } else { 0 });
                return true;
            }

            // ----- bgmfade -----
            ELM_SCRIPT_START_BGMFADE => {
                self.script_bgmfade_flag = true;
            }
            ELM_SCRIPT_END_BGMFADE => {
                self.script_bgmfade_flag = false;
            }

            // ----- vsync -----
            ELM_SCRIPT_SET_VSYNC_WAIT_OFF_FLAG => {
                self.script_vsync_wait_off_flag = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_VSYNC_WAIT_OFF_FLAG => {
                self.stack.push_int(if self.script_vsync_wait_off_flag { 1 } else { 0 });
                return true;
            }

            // ----- skip trigger -----
            ELM_SCRIPT_SET_SKIP_TRIGGER => {
                self.script_skip_trigger = true;
            }

            // ----- ignore R -----
            ELM_SCRIPT_IGNORE_R_ON => {
                self.script_ignore_r_flag = true;
            }
            ELM_SCRIPT_IGNORE_R_OFF => {
                self.script_ignore_r_flag = false;
            }

            // ----- cursor no -----
            ELM_SCRIPT_SET_CURSOR_NO => {
                self.script_cursor_no = arg_int(0);
            }
            ELM_SCRIPT_GET_CURSOR_NO => {
                self.stack.push_int(self.script_cursor_no);
                return true;
            }

            // ----- time stop flags -----
            ELM_SCRIPT_SET_TIME_STOP_FLAG => {
                self.script_time_stop_flag = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_TIME_STOP_FLAG => {
                self.stack.push_int(if self.script_time_stop_flag { 1 } else { 0 });
                return true;
            }
            ELM_SCRIPT_SET_COUNTER_TIME_STOP_FLAG => {
                self.script_counter_time_stop_flag = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_COUNTER_TIME_STOP_FLAG => {
                self.stack.push_int(if self.script_counter_time_stop_flag { 1 } else { 0 });
                return true;
            }
            ELM_SCRIPT_SET_FRAME_ACTION_TIME_STOP_FLAG => {
                self.script_frame_action_time_stop_flag = arg_int(0) != 0;
            }
            ELM_SCRIPT_GET_FRAME_ACTION_TIME_STOP_FLAG => {
                self.stack.push_int(if self.script_frame_action_time_stop_flag { 1 } else { 0 });
                return true;
            }
            ELM_SCRIPT_SET_STAGE_TIME_STOP_FLAG => {
                self.script_stage_time_stop_flag = arg_int(0);
            }
            ELM_SCRIPT_GET_STAGE_TIME_STOP_FLAG => {
                self.stack.push_int(self.script_stage_time_stop_flag);
                return true;
            }

            // ----- font -----
            ELM_SCRIPT_SET_FONT_NAME => {
                self.script_font_name = arg_str(0);
            }
            ELM_SCRIPT_SET_FONT_NAME_DEFAULT => {
                self.script_font_name.clear();
            }
            ELM_SCRIPT_GET_FONT_NAME => {
                self.stack.push_str(self.script_font_name.clone());
                return true;
            }
            ELM_SCRIPT_SET_FONT_BOLD => {
                self.script_font_bold = arg_int(0);
            }
            ELM_SCRIPT_SET_FONT_BOLD_DEFAULT => {
                self.script_font_bold = -1;
            }
            ELM_SCRIPT_GET_FONT_BOLD => {
                self.stack.push_int(self.script_font_bold);
                return true;
            }
            ELM_SCRIPT_SET_FONT_SHADOW => {
                self.script_font_shadow = arg_int(0);
            }
            ELM_SCRIPT_SET_FONT_SHADOW_DEFAULT => {
                self.script_font_shadow = -1;
            }
            ELM_SCRIPT_GET_FONT_SHADOW => {
                self.stack.push_int(self.script_font_shadow);
                return true;
            }

            // ----- joypad mode (stub) -----
            ELM_SCRIPT_SET_ALLOW_JOYPAD_MODE_ONOFF => {
                self.script_allow_joypad_mode_onoff = arg_int(0);
            }
            ELM_SCRIPT_SET_ALLOW_JOYPAD_MODE_ONOFF_DEFAULT => {
                self.script_allow_joypad_mode_onoff = -1;
            }
            ELM_SCRIPT_GET_ALLOW_JOYPAD_MODE_ONOFF => {
                self.stack.push_int(self.script_allow_joypad_mode_onoff);
                return true;
            }

            _ => {
                host.on_error("無効なコマンドが指定されました。(script)");
                return true;
            }
        }

        // Commands that didn't early-return with a push
        if ret_form == crate::elm::form::INT {
            // C++ script commands generally don't push, but if caller expects INT we
            // behave defensively.
        }
        true
    }
}
