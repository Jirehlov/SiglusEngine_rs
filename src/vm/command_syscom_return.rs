use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SyscomProcType {
    Disp,
    GameEndWipe,
    GameStartWipe,
    ReturnToMenu,
    ReturnToSel,
    GameTimerStart,
    EndGame,
    EndLoad,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SyscomProc {
    pub(super) proc_type: SyscomProcType,
    pub(super) option: i32,
}

impl Vm {
    fn notify_load_flow_state(&mut self, host: &mut dyn Host) {
        host.on_syscom_load_flow_state(VmLoadFlowState {
            system_wipe_flag: self.system_wipe_flag != 0,
            do_frame_action_flag: self.do_frame_action_flag != 0,
            do_load_after_call_flag: self.do_load_after_call_flag != 0,
        });
    }

    pub(super) fn run_syscom_proc_queue(
        &mut self,
        queue: &[SyscomProc],
        provider: &mut dyn SceneProvider,
        host: &mut dyn Host,
    ) -> Result<()> {
        for proc in queue.iter().copied() {
            match proc.proc_type {
                SyscomProcType::Disp => {
                    // C++ reference: eng_syscom.cpp fade-out branches push TNM_PROC_TYPE_DISP.
                    host.on_syscom_proc_disp();
                }
                SyscomProcType::GameEndWipe => {
                    // C++ reference: flow_proc.cpp::tnm_game_end_wipe_proc.
                    self.system_wipe_flag = 1;
                    self.notify_load_flow_state(host);
                    host.on_syscom_proc_game_end_wipe(
                        self.options.load_wipe_type,
                        self.options.load_wipe_time_ms,
                    );
                }
                SyscomProcType::GameStartWipe => {
                    // C++ reference: flow_proc.cpp::tnm_game_start_wipe_proc.
                    self.system_wipe_flag = 0;
                    self.notify_load_flow_state(host);
                    host.on_syscom_proc_game_start_wipe(
                        self.options.load_wipe_type,
                        self.options.load_wipe_time_ms,
                    );
                }
                SyscomProcType::ReturnToMenu => {
                    self.system_wipe_flag = 0;
                    self.notify_load_flow_state(host);
                    let leave_msgbk = proc.option == 1;
                    let target = self
                        .return_scene_once
                        .take()
                        .or_else(|| self.options.return_menu_scene.clone());
                    if let Some((scene, z)) = target {
                        self.proc_jump(&scene, z, provider)?;
                        if self.frames.len() > 1 {
                            self.frames.truncate(1);
                        }
                        if !leave_msgbk {
                            // C++ reference: flow_proc.cpp::tnm_return_to_menu_proc +
                            // eng_init.cpp local reinit path when reinit_msgbk_except_flag=false.
                            self.msg_back_has_message = 0;
                            self.msg_back_open_flag = 0;
                            self.msg_back_disable_flag = 0;
                            self.msg_back_off_flag = 0;
                            self.msg_back_disp_off_flag = 0;
                            self.msg_back_proc_off_flag = 0;
                            host.on_msg_back_state(false);
                            host.on_msg_back_display(true);
                        }
                        self.clear_transient_flow_state();
                    } else {
                        self.halted = true;
                    }
                }
                SyscomProcType::ReturnToSel => {
                    // C++ reference: flow_proc.cpp::tnm_return_to_sel_proc -> tnm_saveload_proc_return_to_sel().
                    // C++ no-sel-save path returns false without reinit/reload side effects.
                    if let Some(sel_state) = self.sel_point_snapshot.clone() {
                        self.apply_persistent_state(&sel_state);
                        self.clear_transient_flow_state();
                    }
                    self.system_wipe_flag = 1;
                    self.do_frame_action_flag = 1;
                    self.do_load_after_call_flag = 1;
                    self.notify_load_flow_state(host);
                    host.on_syscom_proc_return_to_sel();
                }
                SyscomProcType::GameTimerStart => {
                    // C++ reference: flow_proc.cpp::tnm_game_timer_start_proc.
                    host.on_game_timer_move(true);
                }
                SyscomProcType::EndGame => {
                    // C++ reference: flow_proc.cpp::tnm_end_game_proc.
                    self.game_end_flag = 1;
                    self.game_end_no_warning_flag = 1;
                    self.game_end_save_done_flag = 1;
                    host.on_syscom_proc_end_game();
                    self.halted = true;
                }
                SyscomProcType::EndLoad => {
                    // C++ reference: flow_proc.cpp::tnm_end_load_proc.
                    // C++ keeps running proc queue after calling tnm_saveload_proc_end_load(),
                    // so this hook is observational and does not abort subsequent procs.
                    let ok = if let Some(slot) = self.end_save_slots.get(&0).cloned() {
                        self.apply_local_state(&slot.state);
                        true
                    } else if let Some(st) = host.on_syscom_end_load_snapshot(0) {
                        self.apply_end_save_state_with_provider(&st, provider)?
                    } else {
                        false
                    };
                    host.on_syscom_proc_end_load_result(ok);
                    self.system_wipe_flag = 1;
                    self.do_frame_action_flag = 1;
                    self.do_load_after_call_flag = 1;
                    self.notify_load_flow_state(host);
                    self.clear_transient_flow_state();
                }
            }
        }
        Ok(())
    }

    pub(super) fn handle_syscom_return_to_menu(
        &mut self,
        args: &[Prop],
        ret_form: i32,
        provider: &mut dyn SceneProvider,
        host: &mut dyn Host,
    ) -> Result<Option<bool>> {
        let warning = Self::arg_int(args, 0) != 0;
        let se_play = Self::arg_int(args, 1) != 0;
        let fade_out = Self::arg_int(args, 2) != 0;
        let leave_msgbk = args
            .iter()
            .find(|arg| arg.id == 0)
            .and_then(|arg| match &arg.value {
                PropValue::Int(v) => Some(*v),
                _ => None,
            })
            .is_some_and(|v| v != 0);
        if warning && !host.on_syscom_return_to_menu_warning() {
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(0);
            }
            return Ok(Some(true));
        }
        let global_state = self.snapshot_persistent_state();
        host.on_syscom_return_to_menu_save_global(&global_state);
        host.on_game_timer_move(false);
        let mut proc_queue = Vec::new();
        if fade_out {
            // C++ push order: RETURN_TO_MENU, GAME_END_WIPE, DISP (stack semantics => exec DISP first).
            proc_queue.push(SyscomProc {
                proc_type: SyscomProcType::Disp,
                option: 0,
            });
            proc_queue.push(SyscomProc {
                proc_type: SyscomProcType::GameEndWipe,
                option: 0,
            });
        }
        proc_queue.push(SyscomProc {
            proc_type: SyscomProcType::ReturnToMenu,
            option: if leave_msgbk { 1 } else { 0 },
        });
        proc_queue.push(SyscomProc {
            proc_type: SyscomProcType::GameTimerStart,
            option: 0,
        });

        if se_play {
            host.on_syscom_play_se(crate::elm::syscom::SE_KIND_MENU);
        }
        self.run_syscom_proc_queue(&proc_queue, provider, host)?;
        if ret_form == crate::elm::form::INT {
            self.stack.push_int(0);
        }
        Ok(Some(true))
    }

    pub(super) fn handle_syscom_return_to_sel(
        &mut self,
        args: &[Prop],
        ret_form: i32,
        provider: &mut dyn SceneProvider,
        host: &mut dyn Host,
    ) -> Result<Option<bool>> {
        let warning = Self::arg_int(args, 0) != 0;
        let se_play = Self::arg_int(args, 1) != 0;
        let fade_out = Self::arg_int(args, 2) != 0;

        if warning && !host.on_syscom_return_to_sel_warning() {
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(0);
            }
            return Ok(Some(true));
        }

        host.on_game_timer_move(false);

        let mut proc_queue = Vec::new();
        if fade_out {
            // C++ push order: RETURN_TO_SEL, GAME_END_WIPE, DISP (stack semantics => exec DISP first).
            proc_queue.push(SyscomProc {
                proc_type: SyscomProcType::Disp,
                option: 0,
            });
            proc_queue.push(SyscomProc {
                proc_type: SyscomProcType::GameEndWipe,
                option: 0,
            });
        }
        proc_queue.push(SyscomProc {
            proc_type: SyscomProcType::ReturnToSel,
            option: 0,
        });
        proc_queue.push(SyscomProc {
            proc_type: SyscomProcType::GameStartWipe,
            option: 0,
        });
        proc_queue.push(SyscomProc {
            proc_type: SyscomProcType::GameTimerStart,
            option: 0,
        });

        if se_play {
            host.on_syscom_play_se(crate::elm::syscom::SE_KIND_PREV_SEL);
        }
        self.run_syscom_proc_queue(&proc_queue, provider, host)?;
        if ret_form == crate::elm::form::INT {
            self.stack.push_int(0);
        }
        Ok(Some(true))
    }

    pub(super) fn handle_syscom_end_game(
        &mut self,
        args: &[Prop],
        ret_form: i32,
        provider: &mut dyn SceneProvider,
        host: &mut dyn Host,
    ) -> Result<Option<bool>> {
        let warning = Self::arg_int(args, 0) != 0;
        let _se_play = Self::arg_int(args, 1) != 0;
        let fade_out = Self::arg_int(args, 2) != 0;

        if warning && !host.on_syscom_end_game_warning() {
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(0);
            }
            return Ok(Some(true));
        }

        host.on_game_timer_move(false);
        self.game_end_no_warning_flag = 1;
        // C++ reference: eng_syscom.cpp::tnm_syscom_end_game -> tnm_syscom_end_save(false, false).
        self.run_end_save_for_end_game(host);
        let persistent_state = self.snapshot_persistent_state();
        host.on_syscom_end_game_save_flush(&persistent_state);

        let mut proc_queue = Vec::new();
        if fade_out {
            // C++ push order: DISP, END_GAME, GAME_END_WIPE (stack semantics => exec GAME_END_WIPE first).
            proc_queue.push(SyscomProc {
                proc_type: SyscomProcType::GameEndWipe,
                option: 0,
            });
        }
        proc_queue.push(SyscomProc {
            proc_type: SyscomProcType::EndGame,
            option: 0,
        });
        proc_queue.push(SyscomProc {
            proc_type: SyscomProcType::Disp,
            option: 0,
        });

        // C++ currently ignores END_GAME se_play parameter in tnm_syscom_end_game().
        self.run_syscom_proc_queue(&proc_queue, provider, host)?;
        if ret_form == crate::elm::form::INT {
            self.stack.push_int(0);
        }
        Ok(Some(true))
    }
}
