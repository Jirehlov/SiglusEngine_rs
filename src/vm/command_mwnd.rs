/// Mwnd (message window) command routing.
///
/// C++ reference: cmd_mwnd.cpp
///
/// Covers:
///   - mwnd_list: close_all variants, array access
///   - per-mwnd: open/close, message text, selection, koe, face, waku,
///     window properties, animation, and sub-object lists (button, face, object)
///
/// Query commands push default values (0 or empty string).
/// Action/set commands delegate to Host callbacks.
use super::*;

impl Vm {
    // ---------------------------------------------------------------
    // Mwnd list: stage.mwnd
    // ---------------------------------------------------------------

    /// Route mwnd_list commands matching C++ `tnm_command_proc_mwnd_list`.
    pub(super) fn try_command_mwnd_list(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }

        use crate::elm::mwnd::*;

        if element[0] == crate::elm::ELM_ARRAY {
            // Indexed: mwnd_list[idx].sub
            if element.len() >= 2 {
                let _mwnd_idx = element[1];
                let rest = if element.len() > 2 { &element[2..] } else { &[] };
                return self.try_command_mwnd(rest, arg_list_id, args, ret_form, host);
            }
            return true;
        }

        match element[0] {
            ELM_MWNDLIST_CLOSE | ELM_MWNDLIST_CLOSE_WAIT => {
                // C++ tnm_msg_proc_close_all(true)
                host.on_mwnd_action(element[0], args);
                true
            }
            ELM_MWNDLIST_CLOSE_NOWAIT => {
                // C++ tnm_msg_proc_close_all(false)
                host.on_mwnd_action(element[0], args);
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(mwnd_list)");
                true
            }
        }
    }

    // ---------------------------------------------------------------
    // Per-mwnd: mwnd_list[idx].<sub>
    // ---------------------------------------------------------------

    /// Route per-mwnd commands matching C++ `tnm_command_proc_mwnd`.
    fn try_command_mwnd(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }

        let sub = element[0];
        use crate::elm::mwnd::*;

        match sub {
            // --- Waku (frame) commands ---
            ELM_MWND_SET_WAKU | ELM_MWND_INIT_WAKU_FILE | ELM_MWND_SET_WAKU_FILE => {
                host.on_mwnd_action(sub, args);
                true
            }
            ELM_MWND_GET_WAKU_FILE => {
                self.stack.push_str(String::new());
                true
            }

            // --- Filter commands ---
            ELM_MWND_INIT_FILTER_FILE | ELM_MWND_SET_FILTER_FILE => {
                host.on_mwnd_action(sub, args);
                true
            }
            ELM_MWND_GET_FILTER_FILE => {
                self.stack.push_str(String::new());
                true
            }

            // --- Open/Close ---
            ELM_MWND_OPEN | ELM_MWND_OPEN_WAIT | ELM_MWND_OPEN_NOWAIT => {
                host.on_mwnd_action(sub, args);
                true
            }
            ELM_MWND_CHECK_OPEN => {
                // C++ tnm_stack_push_int(p_mwnd->get_window_appear_flag() ? 1 : 0)
                self.stack.push_int(host.on_mwnd_get(sub));
                true
            }
            ELM_MWND_CLOSE | ELM_MWND_CLOSE_WAIT | ELM_MWND_CLOSE_NOWAIT | ELM_MWND_END_CLOSE => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Message block / clear ---
            ELM_MWND_MSG_BLOCK | ELM_MWND_MSG_PP_BLOCK | ELM_MWND_CLEAR | ELM_MWND__NOVEL_CLEAR => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Print / overflow print / namae ---
            ELM_MWND_PRINT | ELM_MWND__OVER_FLOW_PRINT | ELM_MWND__OVER_FLOW_NAMAE => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Ruby ---
            ELM_MWND_RUBY => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Wait / flow control ---
            ELM_MWND_WAIT_MSG | ELM_MWND_PP | ELM_MWND_R | ELM_MWND_PAGE => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- NL / NLI ---
            ELM_MWND_NL | ELM_MWND_NLI => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Indent ---
            ELM_MWND_INDENT | ELM_MWND_CLEAR_INDENT => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Multi-message / next ---
            ELM_MWND_MULTI_MSG | ELM_MWND_NEXT_MSG => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Slide message ---
            ELM_MWND_START_SLIDE_MSG | ELM_MWND_END_SLIDE_MSG | ELM_MWND__SLIDE_MSG => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Selection ---
            ELM_MWND_SEL | ELM_MWND_SEL_CANCEL | ELM_MWND_SELMSG | ELM_MWND_SELMSG_CANCEL => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Rep pos / size / color / msgbtn ---
            ELM_MWND_REP_POS | ELM_MWND_SIZE | ELM_MWND_COLOR | ELM_MWND_MSGBTN => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Set namae ---
            ELM_MWND_SET_NAMAE | ELM_MWND_NAMAE => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- KOE ---
            ELM_MWND_KOE | ELM_MWND_KOE_PLAY_WAIT | ELM_MWND_KOE_PLAY_WAIT_KEY
            | ELM_MWND_EXKOE | ELM_MWND_EXKOE_PLAY_WAIT | ELM_MWND_EXKOE_PLAY_WAIT_KEY => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Face ---
            ELM_MWND_CLEAR_FACE | ELM_MWND_SET_FACE => {
                host.on_mwnd_action(sub, args);
                true
            }

            // --- Layer / World ---
            ELM_MWND_LAYER | ELM_MWND_WORLD => {
                if arg_list_id == 0 {
                    self.stack.push_int(host.on_mwnd_get(sub)); // get
                } else {
                    host.on_mwnd_action(sub, args);
                }
                true
            }

            // --- Sub-object lists (button, face, object) → delegate to object_list ---
            ELM_MWND_BUTTON | ELM_MWND_FACE | ELM_MWND_OBJECT => {
                self.try_command_object_list(sub, &element[1..], arg_list_id, args, ret_form, host)
            }

            // --- Window position / size / moji_cnt ---
            ELM_MWND_INIT_WINDOW_POS | ELM_MWND_INIT_WINDOW_SIZE | ELM_MWND_INIT_WINDOW_MOJI_CNT
            | ELM_MWND_SET_WINDOW_POS | ELM_MWND_SET_WINDOW_SIZE | ELM_MWND_SET_WINDOW_MOJI_CNT => {
                host.on_mwnd_action(sub, args);
                true
            }
            ELM_MWND_GET_WINDOW_POS_X | ELM_MWND_GET_WINDOW_POS_Y
            | ELM_MWND_GET_WINDOW_SIZE_X | ELM_MWND_GET_WINDOW_SIZE_Y
            | ELM_MWND_GET_WINDOW_MOJI_CNT_X | ELM_MWND_GET_WINDOW_MOJI_CNT_Y => {
                self.stack.push_int(host.on_mwnd_get(sub));
                true
            }

            // --- Animation type / time ---
            ELM_MWND_INIT_OPEN_ANIME_TYPE | ELM_MWND_INIT_OPEN_ANIME_TIME
            | ELM_MWND_INIT_CLOSE_ANIME_TYPE | ELM_MWND_INIT_CLOSE_ANIME_TIME
            | ELM_MWND_SET_OPEN_ANIME_TYPE | ELM_MWND_SET_OPEN_ANIME_TIME
            | ELM_MWND_SET_CLOSE_ANIME_TYPE | ELM_MWND_SET_CLOSE_ANIME_TIME => {
                host.on_mwnd_action(sub, args);
                true
            }
            ELM_MWND_GET_OPEN_ANIME_TYPE | ELM_MWND_GET_OPEN_ANIME_TIME
            | ELM_MWND_GET_CLOSE_ANIME_TYPE | ELM_MWND_GET_CLOSE_ANIME_TIME
            | ELM_MWND_GET_DEFAULT_OPEN_ANIME_TYPE | ELM_MWND_GET_DEFAULT_OPEN_ANIME_TIME
            | ELM_MWND_GET_DEFAULT_CLOSE_ANIME_TYPE | ELM_MWND_GET_DEFAULT_CLOSE_ANIME_TIME => {
                self.stack.push_int(host.on_mwnd_get(sub));
                true
            }

            _ => {
                host.on_error("無効なコマンドが指定されました。(mwnd)");
                true
            }
        }
    }
}
