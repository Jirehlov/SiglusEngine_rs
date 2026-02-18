use super::command_syscom_return::{SyscomProc, SyscomProcType};
use super::*;

impl Vm {
    fn run_end_save_core(&mut self, se_play: bool, host: &mut dyn Host) -> bool {
        // C++ reference: eng_syscom.cpp::tnm_syscom_end_save_is_enable.
        if self.save_enable_flag == 0 || self.save_exist_flag == 0 {
            return false;
        }
        // C++ reference: eng_syscom.cpp::tnm_syscom_end_save(save_cnt + quick_save_cnt).
        self.end_save_slots.insert(0, self.make_local_slot());
        let end_save_state = self.snapshot_end_save_state();
        host.on_syscom_end_save_snapshot(0, &end_save_state);
        self.game_end_save_done_flag = 1;
        if se_play {
            host.on_syscom_play_se(crate::elm::syscom::SE_KIND_SAVE);
        }
        true
    }

    pub(super) fn run_end_save_for_end_game(&mut self, host: &mut dyn Host) {
        // C++ reference: eng_syscom.cpp::tnm_syscom_end_game -> tnm_syscom_end_save(false, false).
        let _ = self.run_end_save_core(false, host);
    }

    pub(super) fn handle_syscom_end_save(
        &mut self,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> Result<Option<bool>> {
        let warning = Self::arg_int(args, 0) != 0;
        let se_play = Self::arg_int(args, 1) != 0;
        if warning && !host.on_syscom_end_save_warning() {
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(0);
            }
            return Ok(Some(true));
        }

        let ok = self.run_end_save_core(se_play, host);
        if ret_form == crate::elm::form::INT {
            self.stack.push_int(if ok { 1 } else { 0 });
        }
        Ok(Some(true))
    }

    pub(super) fn handle_syscom_end_load(
        &mut self,
        args: &[Prop],
        ret_form: i32,
        provider: &mut dyn SceneProvider,
        host: &mut dyn Host,
    ) -> Result<Option<bool>> {
        let warning = Self::arg_int(args, 0) != 0;
        let se_play = Self::arg_int(args, 1) != 0;
        let fade_out = Self::arg_int(args, 2) != 0;
        // C++ reference: eng_syscom.cpp::tnm_syscom_end_load_is_enable.
        let host_end_save_exist = host.on_syscom_end_save_exist(0).unwrap_or(false);
        if self.load_enable_flag == 0
            || self.load_exist_flag == 0
            || (!self.end_save_slots.contains_key(&0) && !host_end_save_exist)
        {
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(0);
            }
            return Ok(Some(true));
        }
        if warning && !host.on_syscom_end_load_warning() {
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(0);
            }
            return Ok(Some(true));
        }

        host.on_game_timer_move(false);
        let mut proc_queue = Vec::new();
        if fade_out {
            // C++ push order: END_LOAD, GAME_END_WIPE, DISP (stack semantics => exec DISP first).
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
            proc_type: SyscomProcType::EndLoad,
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
            host.on_syscom_play_se(crate::elm::syscom::SE_KIND_LOAD);
        }
        self.run_syscom_proc_queue(&proc_queue, provider, host)?;
        if ret_form == crate::elm::form::INT {
            // C++ reference: eng_syscom.cpp::tnm_syscom_end_load returns true once accepted;
            // later tnm_end_load_proc internal load failures do not retroactively change cmd return.
            self.stack.push_int(1);
        }
        Ok(Some(true))
    }
}
