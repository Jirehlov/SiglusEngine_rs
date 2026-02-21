/// Object command routing.
///
/// C++ reference: cmd_object.cpp
///
/// Covers:
///   - object_list: array access, resize, get_size
///   - per-object: property get/set, event routing, lifecycle, create, query, movie, emote, button, etc.
///
/// Property get commands push default 0 values (or delegate to host).
/// Property set commands delegate to Host callbacks.
/// Event properties (*_EVE) delegate to command_int_event sub-router.
use super::*;

impl Vm {
    // ---------------------------------------------------------------
    // Object list: stage.object / mwnd.object / etc.
    // ---------------------------------------------------------------

    /// Route object_list commands matching C++ `tnm_command_proc_object_list`.
    pub(super) fn try_command_object_list(
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

        if element[0] == crate::elm::ELM_ARRAY {
            // Indexed: object_list[idx].sub
            if element.len() >= 2 {
                let _obj_idx = element[1];
                let rest = if element.len() > 2 { &element[2..] } else { &[] };
                return self.try_command_object(rest, arg_list_id, args, ret_form, host);
            }
            // bare indexed — push default
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(0);
            }
            return true;
        }

        use crate::elm::objectlist::*;
        match element[0] {
            ELM_OBJECTLIST_RESIZE => {
                // C++ p_object_list->resize(arg0)
                host.on_object_action(element[0], args);
                true
            }
            ELM_OBJECTLIST_GET_SIZE => {
                // C++ tnm_stack_push_int(p_object_list->get_size())
                self.stack.push_int(0);
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(object_list)");
                true
            }
        }
    }

    // ---------------------------------------------------------------
    // Per-object: object_list[idx].<sub>
    // ---------------------------------------------------------------

    /// Route per-object commands matching C++ `tnm_command_proc_object`.
    fn try_command_object(
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
        use crate::elm::objectlist::*;

        match sub {
            // =============================================================
            // Simple int property get/set (al_id=0 → push, al_id=1 → set)
            // =============================================================
            ELM_OBJECT_WIPE_COPY | ELM_OBJECT_WIPE_ERASE | ELM_OBJECT_CLICK_DISABLE
            | ELM_OBJECT_DISP | ELM_OBJECT_PATNO | ELM_OBJECT_WORLD | ELM_OBJECT_ORDER
            | ELM_OBJECT_LAYER | ELM_OBJECT_X | ELM_OBJECT_Y | ELM_OBJECT_Z
            | ELM_OBJECT_CENTER_X | ELM_OBJECT_CENTER_Y | ELM_OBJECT_CENTER_Z
            | ELM_OBJECT_CENTER_REP_X | ELM_OBJECT_CENTER_REP_Y | ELM_OBJECT_CENTER_REP_Z
            | ELM_OBJECT_SCALE_X | ELM_OBJECT_SCALE_Y | ELM_OBJECT_SCALE_Z
            | ELM_OBJECT_ROTATE_X | ELM_OBJECT_ROTATE_Y | ELM_OBJECT_ROTATE_Z
            | ELM_OBJECT_CLIP_USE | ELM_OBJECT_CLIP_LEFT | ELM_OBJECT_CLIP_TOP
            | ELM_OBJECT_CLIP_RIGHT | ELM_OBJECT_CLIP_BOTTOM
            | ELM_OBJECT_SRC_CLIP_USE | ELM_OBJECT_SRC_CLIP_LEFT | ELM_OBJECT_SRC_CLIP_TOP
            | ELM_OBJECT_SRC_CLIP_RIGHT | ELM_OBJECT_SRC_CLIP_BOTTOM
            | ELM_OBJECT_TR | ELM_OBJECT_MONO | ELM_OBJECT_REVERSE
            | ELM_OBJECT_BRIGHT | ELM_OBJECT_DARK
            | ELM_OBJECT_COLOR_R | ELM_OBJECT_COLOR_G | ELM_OBJECT_COLOR_B
            | ELM_OBJECT_COLOR_RATE
            | ELM_OBJECT_COLOR_ADD_R | ELM_OBJECT_COLOR_ADD_G | ELM_OBJECT_COLOR_ADD_B
            | ELM_OBJECT_MASK_NO | ELM_OBJECT_TONECURVE_NO | ELM_OBJECT_CULLING
            | ELM_OBJECT_ALPHA_TEST | ELM_OBJECT_ALPHA_BLEND | ELM_OBJECT_BLEND
            | ELM_OBJECT_LIGHT_NO | ELM_OBJECT_FOG_USE => {
                if arg_list_id == 0 {
                    self.stack.push_int(0); // get: default value
                } else {
                    host.on_object_property(sub, Self::int_arg(args, 0));
                }
                true
            }

            // =============================================================
            // Compound set commands (2-3 positional args)
            // =============================================================
            ELM_OBJECT_SET_POS => {
                // C++ al_id 0: set_pos_x, set_pos_y; al_id 1: +set_pos_z
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_SET_CENTER => {
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_SET_CENTER_REP => {
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_SET_SCALE => {
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_SET_ROTATE => {
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_SET_CLIP => {
                // C++ 5 args: use, left, top, right, bottom
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_SET_SRC_CLIP => {
                host.on_object_action(sub, args);
                true
            }

            // =============================================================
            // Int list / rep properties (host manages int_list data)
            // =============================================================
            ELM_OBJECT_X_REP | ELM_OBJECT_Y_REP | ELM_OBJECT_Z_REP | ELM_OBJECT_TR_REP => {
                // C++ tnm_command_proc_int_list — accept, host handles
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_F => {
                // C++ tnm_command_proc_int_list(&p_obj->F(), 32, ...)
                host.on_object_action(sub, args);
                true
            }

            // =============================================================
            // Event properties → int_event sub-router
            // =============================================================
            ELM_OBJECT_PATNO_EVE | ELM_OBJECT_X_EVE | ELM_OBJECT_Y_EVE | ELM_OBJECT_Z_EVE
            | ELM_OBJECT_CENTER_X_EVE | ELM_OBJECT_CENTER_Y_EVE | ELM_OBJECT_CENTER_Z_EVE
            | ELM_OBJECT_CENTER_REP_X_EVE | ELM_OBJECT_CENTER_REP_Y_EVE | ELM_OBJECT_CENTER_REP_Z_EVE
            | ELM_OBJECT_SCALE_X_EVE | ELM_OBJECT_SCALE_Y_EVE | ELM_OBJECT_SCALE_Z_EVE
            | ELM_OBJECT_ROTATE_X_EVE | ELM_OBJECT_ROTATE_Y_EVE | ELM_OBJECT_ROTATE_Z_EVE
            | ELM_OBJECT_CLIP_LEFT_EVE | ELM_OBJECT_CLIP_TOP_EVE | ELM_OBJECT_CLIP_RIGHT_EVE | ELM_OBJECT_CLIP_BOTTOM_EVE
            | ELM_OBJECT_SRC_CLIP_LEFT_EVE | ELM_OBJECT_SRC_CLIP_TOP_EVE | ELM_OBJECT_SRC_CLIP_RIGHT_EVE | ELM_OBJECT_SRC_CLIP_BOTTOM_EVE
            | ELM_OBJECT_TR_EVE | ELM_OBJECT_MONO_EVE | ELM_OBJECT_REVERSE_EVE
            | ELM_OBJECT_BRIGHT_EVE | ELM_OBJECT_DARK_EVE
            | ELM_OBJECT_COLOR_R_EVE | ELM_OBJECT_COLOR_G_EVE | ELM_OBJECT_COLOR_B_EVE
            | ELM_OBJECT_COLOR_RATE_EVE
            | ELM_OBJECT_COLOR_ADD_R_EVE | ELM_OBJECT_COLOR_ADD_G_EVE | ELM_OBJECT_COLOR_ADD_B_EVE => {
                self.try_command_int_event(&element[1..], arg_list_id, args, ret_form, host, sub)
            }

            // Event list properties → int_event_list sub-router
            ELM_OBJECT_X_REP_EVE | ELM_OBJECT_Y_REP_EVE | ELM_OBJECT_Z_REP_EVE | ELM_OBJECT_TR_REP_EVE => {
                self.try_command_int_event_list(&element[1..], arg_list_id, args, ret_form, host, sub)
            }

            // =============================================================
            // ALL_EVE: end/wait/check
            // =============================================================
            ELM_OBJECT_ALL_EVE => {
                if element.len() >= 2 {
                    use crate::elm::allevent::*;
                    match element[1] {
                        ELM_ALLEVENT_END => {
                            host.on_object_action(sub, args);
                        }
                        ELM_ALLEVENT_WAIT => {
                            // C++ pushes proc TNM_PROC_TYPE_ALL_EVENT_WAIT
                            host.on_object_action(sub, args);
                        }
                        ELM_ALLEVENT_CHECK => {
                            // C++ tnm_stack_push_int(p_obj->check_all_event() ? 1 : 0)
                            self.stack.push_int(0);
                        }
                        _ => {
                            host.on_error("無効なコマンドが指定されました。(allevent)");
                        }
                    }
                }
                true
            }

            // =============================================================
            // Query commands (push values)
            // =============================================================
            ELM_OBJECT_GET_PAT_CNT => {
                // C++ tnm_stack_push_int(p_obj->get_pat_cnt())
                self.stack.push_int(0);
                true
            }
            ELM_OBJECT_GET_SIZE_X | ELM_OBJECT_GET_SIZE_Y | ELM_OBJECT_GET_SIZE_Z => {
                // C++ tnm_stack_push_int(p_obj->get_size_*(pat_no))
                self.stack.push_int(0);
                true
            }
            ELM_OBJECT_GET_PIXEL_COLOR_R | ELM_OBJECT_GET_PIXEL_COLOR_G
            | ELM_OBJECT_GET_PIXEL_COLOR_B | ELM_OBJECT_GET_PIXEL_COLOR_A => {
                self.stack.push_int(0);
                true
            }
            ELM_OBJECT_GET_FILE_NAME => {
                // C++ tnm_stack_push_str(p_obj->get_file_path())
                self.stack.push_str(String::new());
                true
            }
            ELM_OBJECT_EXIST_TYPE => {
                // C++ tnm_stack_push_int(type != NONE ? 1 : 0)
                self.stack.push_int(0);
                true
            }
            ELM_OBJECT_GET_ELEMENT_NAME => {
                self.stack.push_str(String::new());
                true
            }
            ELM_OBJECT_GET_TYPE => {
                self.stack.push_int(0);
                true
            }

            // =============================================================
            // Lifecycle commands
            // =============================================================
            ELM_OBJECT_INIT | ELM_OBJECT_FREE | ELM_OBJECT_INIT_PARAM => {
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_CREATE | ELM_OBJECT_CREATE_RECT | ELM_OBJECT_CREATE_STRING
            | ELM_OBJECT_CREATE_NUMBER | ELM_OBJECT_CREATE_WEATHER | ELM_OBJECT_CREATE_MESH
            | ELM_OBJECT_CREATE_BILLBOARD | ELM_OBJECT_CREATE_SAVE_THUMB
            | ELM_OBJECT_CREATE_CAPTURE_THUMB | ELM_OBJECT_CREATE_CAPTURE
            | ELM_OBJECT_CREATE_COPY_FROM | ELM_OBJECT_CREATE_EMOTE
            | ELM_OBJECT_CREATE_FROM_CAPTURE_FILE => {
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_CREATE_MOVIE | ELM_OBJECT_CREATE_MOVIE_LOOP
            | ELM_OBJECT_CREATE_MOVIE_WAIT | ELM_OBJECT_CREATE_MOVIE_WAIT_KEY => {
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_CHANGE_FILE => {
                host.on_object_action(sub, args);
                true
            }

            // =============================================================
            // String / Number object commands
            // =============================================================
            ELM_OBJECT_SET_STRING | ELM_OBJECT_SET_STRING_PARAM
            | ELM_OBJECT_SET_NUMBER | ELM_OBJECT_SET_NUMBER_PARAM => {
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_GET_STRING => {
                self.stack.push_str(String::new());
                true
            }
            ELM_OBJECT_GET_NUMBER => {
                self.stack.push_int(0);
                true
            }

            // =============================================================
            // Movie commands
            // =============================================================
            ELM_OBJECT_PAUSE_MOVIE | ELM_OBJECT_RESUME_MOVIE | ELM_OBJECT_SEEK_MOVIE
            | ELM_OBJECT_WAIT_MOVIE | ELM_OBJECT_WAIT_MOVIE_KEY | ELM_OBJECT_END_MOVIE_LOOP
            | ELM_OBJECT_SET_MOVIE_AUTO_FREE => {
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_GET_MOVIE_SEEK_TIME => {
                self.stack.push_int(0);
                true
            }
            ELM_OBJECT_CHECK_MOVIE => {
                self.stack.push_int(0);
                true
            }

            // =============================================================
            // Weather param commands
            // =============================================================
            ELM_OBJECT_SET_WEATHER_PARAM_TYPE_A | ELM_OBJECT_SET_WEATHER_PARAM_TYPE_B => {
                host.on_object_action(sub, args);
                true
            }

            // =============================================================
            // Button commands
            // =============================================================
            ELM_OBJECT_CLEAR_BUTTON | ELM_OBJECT_SET_BUTTON | ELM_OBJECT_SET_BUTTON_GROUP
            | ELM_OBJECT_SET_BUTTON_PUSHKEEP | ELM_OBJECT_SET_BUTTON_ALPHA_TEST
            | ELM_OBJECT_SET_BUTTON_STATE_NORMAL | ELM_OBJECT_SET_BUTTON_STATE_SELECT
            | ELM_OBJECT_SET_BUTTON_STATE_DISABLE
            | ELM_OBJECT_SET_BUTTON_CALL | ELM_OBJECT_CLEAR_BUTTON_CALL => {
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_GET_BUTTON_STATE | ELM_OBJECT_GET_BUTTON_HIT_STATE | ELM_OBJECT_GET_BUTTON_REAL_STATE
            | ELM_OBJECT_GET_BUTTON_PUSHKEEP | ELM_OBJECT_GET_BUTTON_ALPHA_TEST
            | ELM_OBJECT_GET_BUTTON_NO | ELM_OBJECT_GET_BUTTON_GROUP_NO
            | ELM_OBJECT_GET_BUTTON_ACTION_NO | ELM_OBJECT_GET_BUTTON_SE_NO => {
                self.stack.push_int(0);
                true
            }

            // Note: ELM_OBJECT_LOAD_GAN (0) and ELM_OBJECT_START_GAN (1)
            // overlap with ELM_OBJECT_DISP/PATNO — in C++ they are sub-dispatched
            // from within FRAME_ACTION, not from the main object switch.
            // Frame action commands
            // =============================================================
            ELM_OBJECT_FRAME_ACTION | ELM_OBJECT_FRAME_ACTION_CH => {
                host.on_object_action(sub, args);
                true
            }

            // =============================================================
            // Emote commands
            // =============================================================
            ELM_OBJECT_EMOTE_PLAY_TIMELINE | ELM_OBJECT_EMOTE_STOP_TIMELINE
            | ELM_OBJECT_EMOTE_WAIT_PLAYING | ELM_OBJECT_EMOTE_WAIT_PLAYING_KEY
            | ELM_OBJECT_EMOTE_SKIP | ELM_OBJECT_EMOTE_PASS => {
                host.on_object_action(sub, args);
                true
            }
            ELM_OBJECT_EMOTE_CHECK_PLAYING => {
                self.stack.push_int(0);
                true
            }
            ELM_OBJECT_EMOTE_KOE_CHARA_NO | ELM_OBJECT_EMOTE_MOUTH_VOLUME => {
                if arg_list_id == 0 {
                    self.stack.push_int(0); // get
                } else {
                    host.on_object_property(sub, Self::int_arg(args, 0));
                }
                true
            }

            // =============================================================
            // Hints
            // =============================================================
            ELM_OBJECT_ADD_HINTS | ELM_OBJECT_CLEAR_HINTS => {
                host.on_object_action(sub, args);
                true
            }

            // =============================================================
            // Child object list
            // =============================================================
            ELM_OBJECT_CHILD => {
                self.try_command_object_list(&element[1..], arg_list_id, args, ret_form, host)
            }

            // =============================================================
            // Child sort type
            // =============================================================
            ELM_OBJECT_SET_CHILD_SORT_TYPE_DEFAULT | ELM_OBJECT_SET_CHILD_SORT_TYPE_TEST => {
                host.on_object_action(sub, args);
                true
            }

            // =============================================================
            // iapp dummy
            // =============================================================
            ELM_OBJECT__IAPP_DUMMY => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                true
            }

            _ => {
                host.on_error("無効なコマンドが指定されました。(object)");
                true
            }
        }
    }
}
