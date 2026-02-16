#![allow(unused_imports)]
use super::*;
use std::time::{SystemTime, UNIX_EPOCH};



impl Vm {


    pub(super) fn try_command_global_head(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        provider: &mut dyn SceneProvider,
        host: &mut dyn Host,
    ) -> Result<Option<bool>> {
        match element[0] {
            x if crate::elm::global::is_nop(x) => {
                return Ok(Some(true));
            }
            x if crate::elm::global::is_namae(x) => {
                // NAMAE command (lookup by key usually).
                // In C++ this returns an element. Here we just forward to host if it's a command.
                return Ok(Some(false));
            }
            x if crate::elm::global::is_passthrough_command(x) => {
                return Ok(Some(false));
            }
            x if crate::elm::global::is_get_last_sel_msg(x) => {
                // C++ cmd_global.cpp::ELM_GLOBAL_GET_LAST_SEL_MSG always pushes string.
                self.stack.push_str(self.last_sel_msg.clone());
                return Ok(Some(true));
            }
            x if crate::elm::global::is_test(x) => {
                // C++ cmd_global.cpp::ELM_GLOBAL______TEST has an empty handler.
                // Keep it as a pure no-op: no stack write regardless of return form.
                let _ = ret_form;
                return Ok(Some(true));
            }
            // Call stack management stubs
            x if crate::elm::global::is_init_call_stack(x) => {
                // Clear all frames except the root one
                if self.frames.len() > 1 {
                    self.frames.truncate(1);
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_del_call_stack(x) => {
                if let Some(PropValue::Int(n)) = args.get(0).map(|p| &p.value) {
                    if *n > 0 {
                        let cur = self.frames.len();
                        let dst = cur.saturating_sub(*n as usize).max(1);
                        self.frames.truncate(dst);
                    }
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_get_call_stack_cnt(x) => {
                // C++ eng_scene.cpp::tnm_scene_get_call_stack_cnt returns int unconditionally.
                self.stack.push_int(self.frames.len() as i32);
                return Ok(Some(true));
            }
            x if crate::elm::global::is_set_call_stack_cnt(x) => {
                if let Some(PropValue::Int(dst_cnt)) = args.get(0).map(|p| &p.value) {
                    if *dst_cnt >= 1 {
                        let cur = self.frames.len();
                        let dst = *dst_cnt as usize;
                        if dst < cur {
                            self.frames.truncate(dst);
                        }
                    }
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_owari(x) => {
                self.halted = true;
                return Ok(Some(true));
            }
            x if crate::elm::global::is_returnmenu(x) => {
                // C++ reference: cmd_global.cpp::ELM_GLOBAL_RETURNMENU.
                // al_id==1/2 uses explicit scene/z args; otherwise return-to-menu flow.
                let arg_target = match args.first().map(|p| &p.value) {
                    Some(PropValue::Str(scene)) if !scene.is_empty() => {
                        let z = match args.get(1).map(|p| &p.value) {
                            Some(PropValue::Int(v)) => *v,
                            _ => 0,
                        };
                        Some((scene.clone(), z))
                    }
                    _ => None,
                };

                let target = if let Some(t) = arg_target {
                    Some(t)
                } else if let Some(one_shot) = self.return_scene_once.take() {
                    Some(one_shot)
                } else {
                    self.options.return_menu_scene.clone()
                };

                if let Some((scene, z)) = target {
                    self.proc_jump(&scene, z, provider)?;
                    // RETURNMENU is a hard flow reset to menu: keep only root frame.
                    if self.frames.len() > 1 {
                        self.frames.truncate(1);
                    }
                    // Reset transient per-flow runtime state.
                    self.clear_transient_flow_state();
                } else {
                    self.halted = true;
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_get_scene_name(x) => {
                // C++ cmd_global.cpp::ELM_GLOBAL_GET_SCENE_NAME always pushes string.
                self.stack.push_str(self.scene.clone());
                return Ok(Some(true));
            }
            x if crate::elm::global::is_get_line_no(x) => {
                // C++ cmd_global.cpp::ELM_GLOBAL_GET_LINE_NO always pushes int.
                self.stack.push_int(self.lexer.cur_line_no);
                return Ok(Some(true));
            }
            x if crate::elm::global::is_set_title(x) => {
                if let Some(Prop {
                    value: PropValue::Str(s),
                    ..
                }) = args.get(0)
                {
                    self.scene_title = s.clone();
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_get_title(x) => {
                // C++ cmd_global.cpp::ELM_GLOBAL_GET_TITLE always pushes string.
                self.stack.push_str(self.scene_title.clone());
                return Ok(Some(true));
            }
            // -----------------------------------------------------------------
            // FrameAction (headless best-effort)
            // -----------------------------------------------------------------
            x if crate::elm::global::is_frame_action(x) => {
                let chain = if element.len() > 1 {
                    &element[1..]
                } else {
                    &[]
                };
                if !chain.is_empty() {
                    let scene = self.scene.clone();
                    let stack = &mut self.stack;
                    let fa = &mut self.frame_action;
                    let _ =
                        Self::command_proc_frame_action(fa, chain, args, ret_form, &scene, stack)?;
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_frame_action_ch(x) => {
                if element.len() == 1 {
                    return Ok(Some(true));
                }
                // global.frame_action_ch[ idx ].<frameaction...>
                if element[1] == crate::elm::ELM_ARRAY {
                    let idx = element.get(2).copied().unwrap_or(0) as isize;
                    if idx < 0 {
                        if ret_form == crate::elm::form::INT {
                            self.stack.push_int(0);
                        } else if ret_form == crate::elm::form::STR {
                            self.stack.push_str(String::new());
                        }
                        return Ok(Some(true));
                    }
                    let idx = idx as usize;
                    if idx >= self.frame_action_ch.len() {
                        if ret_form == crate::elm::form::INT {
                            self.stack.push_int(0);
                        } else if ret_form == crate::elm::form::STR {
                            self.stack.push_str(String::new());
                        }
                        return Ok(Some(true));
                    }
                    let chain = if element.len() > 3 {
                        &element[3..]
                    } else {
                        &[]
                    };
                    if !chain.is_empty() {
                        let scene = self.scene.clone();
                        let stack = &mut self.stack;
                        let fa = &mut self.frame_action_ch[idx];
                        let _ = Self::command_proc_frame_action(
                            fa, chain, args, ret_form, &scene, stack,
                        )?;
                    }
                    return Ok(Some(true));
                }

                // list methods
                let method = element[1];
                if crate::elm::frameaction::is_frameactionlist_get_size(method) {
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(self.frame_action_ch.len() as i32);
                    }
                    return Ok(Some(true));
                }
                if crate::elm::frameaction::is_frameactionlist_resize(method) {
                    let n = match args.get(0).map(|p| &p.value) {
                        Some(PropValue::Int(v)) => *v,
                        _ => 0,
                    };
                    let n = if n > 0 { n as usize } else { 0 };
                    self.frame_action_ch.resize(n, FrameAction::default());
                    return Ok(Some(true));
                }

                return Ok(Some(false));
            }
            x if crate::elm::global::is_jump(x) => {
                let scene = match args.get(0).map(|p| &p.value) {
                    Some(PropValue::Str(s)) => s.as_str(),
                    _ => "",
                };
                let z_no = match args.get(1).map(|p| &p.value) {
                    Some(PropValue::Int(v)) => *v,
                    _ => 0,
                };
                if !scene.is_empty() {
                    self.proc_jump(scene, z_no, provider)?;
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_farcall(x) => {
                let scene = match args.get(0).map(|p| &p.value) {
                    Some(PropValue::Str(s)) => s.as_str(),
                    _ => "",
                };
                let z_no = match args.get(1).map(|p| &p.value) {
                    Some(PropValue::Int(v)) => *v,
                    _ => 0,
                };
                if !scene.is_empty() {
                    let call_args = if args.len() >= 2 { &args[2..] } else { &[] };
                    self.proc_farcall_like(
                        scene,
                        z_no,
                        crate::elm::form::INT,
                        call_args,
                        provider,
                    )?;
                }
                return Ok(Some(true));
            }

            x if x == crate::elm::global::ELM_GLOBAL_SYSTEM => {
                let method = element.get(1).copied().unwrap_or(0);
                match method {
                    m if crate::elm::system::is_check_active(m) => {
                        if ret_form == crate::elm::form::INT {
                            self.stack.push_int(1);
                        }
                        return Ok(Some(true));
                    }
                    m if crate::elm::system::is_check_debug_flag(m) => {
                        if ret_form == crate::elm::form::INT {
                            self.stack.push_int(0);
                        }
                        return Ok(Some(true));
                    }
                    m if crate::elm::system::is_check_file_exist(m) => {
                        let exists = match args.first().map(|p| &p.value) {
                            Some(PropValue::Str(path)) => std::path::Path::new(path).exists(),
                            _ => false,
                        };
                        if ret_form == crate::elm::form::INT {
                            self.stack.push_int(if exists { 1 } else { 0 });
                        }
                        return Ok(Some(true));
                    }
                    m if crate::elm::system::is_get_unix_time(m) => {
                        if ret_form == crate::elm::form::INT {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map(|d| d.as_secs() as i32)
                                .unwrap_or(0);
                            self.stack.push_int(now);
                        }
                        return Ok(Some(true));
                    }
                    m if crate::elm::system::is_get_language(m) => {
                        if ret_form == crate::elm::form::STR {
                            self.stack.push_str("ja".to_string());
                        }
                        return Ok(Some(true));
                    }
                    m if crate::elm::system::is_messagebox(m)
                        || crate::elm::system::is_debug_messagebox(m) =>
                    {
                        if ret_form == crate::elm::form::INT {
                            self.stack.push_int(0);
                        }
                        return Ok(Some(true));
                    }
                    m if crate::elm::system::is_chihaya_bench(m) => {
                        if method == crate::elm::system::ELM_SYSTEM_GET_SPEC_INFO_FOR_CHIHAYA_BENCH
                            && ret_form == crate::elm::form::STR
                        {
                            self.stack.push_str(String::new());
                        }
                        return Ok(Some(true));
                    }
                    m if crate::elm::system::is_get_calendar(m)
                        || crate::elm::system::is_shell_open(m)
                        || crate::elm::system::is_debug_write_log(m)
                        || crate::elm::system::is_dummy_file_command(m) =>
                    {
                        if ret_form == crate::elm::form::INT {
                            self.stack.push_int(0);
                        }
                        return Ok(Some(true));
                    }
                    m if crate::elm::system::is_any_system_element(m) => {
                        if ret_form == crate::elm::form::INT {
                            self.stack.push_int(0);
                        } else if ret_form == crate::elm::form::STR {
                            self.stack.push_str(String::new());
                        }
                        return Ok(Some(true));
                    }
                    _ => {
                        // Unknown method under global.system: fallback to host.
                        return Ok(Some(false));
                    }
                }
            }
            x if x == crate::elm::global::ELM_GLOBAL_SCRIPT && element.len() >= 2 => {
                let method = element[1];
                if crate::elm::script::is_set_int_command(method) {
                    let v = match args.get(0).map(|p| &p.value) {
                        Some(PropValue::Int(v)) => *v,
                        _ => 0,
                    };
                    if crate::elm::script::is_set_skip_unread_message_flag(method) {
                        self.script_skip_unread_message_flag = v;
                    } else if crate::elm::script::is_set_msg_back_disable(method) {
                        // C++ reference: cmd_script.cpp::ELM_SCRIPT_SET_MSG_BACK_DISABLE
                        self.msg_back_disable_flag = 1;
                    } else if crate::elm::script::is_set_msg_back_enable(method) {
                        // C++ reference: cmd_script.cpp::ELM_SCRIPT_SET_MSG_BACK_ENABLE
                        self.msg_back_disable_flag = 0;
                    } else if crate::elm::script::is_set_msg_back_off(method) {
                        // C++ reference: cmd_script.cpp::ELM_SCRIPT_SET_MSG_BACK_OFF
                        self.msg_back_off_flag = 1;
                    } else if crate::elm::script::is_set_msg_back_on(method) {
                        // C++ reference: cmd_script.cpp::ELM_SCRIPT_SET_MSG_BACK_ON
                        self.msg_back_off_flag = 0;
                    } else if crate::elm::script::is_set_msg_back_disp_off(method) {
                        // C++ reference: cmd_script.cpp::ELM_SCRIPT_SET_MSG_BACK_DISP_OFF
                        self.msg_back_disp_off_flag = 1;
                        host.on_msg_back_state(false);
                        host.on_msg_back_display(false);
                    } else if crate::elm::script::is_set_msg_back_disp_on(method) {
                        // C++ reference: cmd_script.cpp::ELM_SCRIPT_SET_MSG_BACK_DISP_ON
                        self.msg_back_disp_off_flag = 0;
                        host.on_msg_back_display(true);
                        if self.msg_back_open_flag != 0 {
                            host.on_msg_back_state(true);
                        }
                    } else if crate::elm::script::is_set_msg_back_proc_off(method) {
                        // STUB(C++: ELM_SCRIPT_SET_MSG_BACK_PROC_OFF appears in constants but
                        // runtime-side behavior path is not found in current source snapshot.)
                        self.msg_back_proc_off_flag = 1;
                    } else if crate::elm::script::is_set_msg_back_proc_on(method) {
                        // STUB(C++: ELM_SCRIPT_SET_MSG_BACK_PROC_ON appears in constants but
                        // runtime-side behavior path is not found in current source snapshot.)
                        self.msg_back_proc_off_flag = 0;
                    } else {
                        self.script_stage_time_stop_flag = v;
                    }
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(0);
                    }
                    return Ok(Some(true));
                }
                return Ok(Some(false));
            }
            x if crate::elm::input::is_clear_or_next(x) => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                return Ok(Some(true));
            }
            x if crate::elm::input::is_decide_or_cancel(x)
                || crate::elm::mouse::is_query_int(x) =>
            {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                return Ok(Some(true));
            }
            _ => {
                // Try syscom commands
                if let Some(res) = self.try_command_syscom(element, _arg_list_id, args, ret_form, provider, host)? {
                    return Ok(Some(res));
                }
            }
        }
        Ok(None)
    }
}
