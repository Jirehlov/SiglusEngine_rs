use super::*;

include!("command_call_excall_frame_action.rs");

#[allow(dead_code)]
impl Vm {
    /// Route `global.call.<sub>` (call_list level). Returns `true` if handled.
    pub(super) fn try_command_call_list(
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
        match element[0] {
            x if x == crate::elm::ELM_ARRAY => {
                // call[idx].<sub>
                if element.len() > 2 {
                    self.try_command_call(&element[2..], arg_list_id, args, ret_form, host);
                }
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。（calllist）");
                true
            }
        }
    }

    /// Route `global.cur_call.<sub>` or `call[idx].<sub>`. Returns `true` if handled.
    pub(super) fn try_command_cur_call(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        self.try_command_call(element, arg_list_id, args, ret_form, host);
        true
    }

    fn try_command_call(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        _host: &mut dyn Host,
    ) {
        if element.is_empty() {
            return;
        }
        match element[0] {
            x if x == crate::elm::call::ELM_CALL_L => {
                // call.L → int_list on current call's L
                // This is handled by the existing flag list machinery.
                // For now just accept + default return.
                if element.len() > 1 {
                    let sub = &element[1..];
                    if !sub.is_empty() && sub[0] == crate::elm::ELM_ARRAY {
                        let idx = sub.get(1).copied().unwrap_or(0);
                        let cur = self.frames.last();
                        if arg_list_id == 0 {
                            // get
                            let val = cur
                                .and_then(|f| f.call.l.get(idx as usize).copied())
                                .unwrap_or(0);
                            self.stack.push_int(val);
                        } else if arg_list_id == 1 {
                            // set
                            if let Some(PropValue::Int(v)) = args.first().map(|p| &p.value) {
                                if let Some(f) = self.frames.last_mut() {
                                    let idx = idx as usize;
                                    if idx < f.call.l.len() {
                                        f.call.l[idx] = *v;
                                    }
                                }
                            }
                        }
                    } else if crate::elm::list::is_intlist_get_size(sub[0]) {
                        let len = self
                            .frames
                            .last()
                            .map(|f| f.call.l.len() as i32)
                            .unwrap_or(0);
                        self.stack.push_int(len);
                    } else if crate::elm::list::is_intlist_init(sub[0]) {
                        if let Some(f) = self.frames.last_mut() {
                            for v in f.call.l.iter_mut() {
                                *v = 0;
                            }
                        }
                    } else if crate::elm::list::is_intlist_resize(sub[0]) {
                        let n = match args.first().map(|p| &p.value) {
                            Some(PropValue::Int(v)) => (*v).max(0) as usize,
                            _ => 0,
                        };
                        if let Some(f) = self.frames.last_mut() {
                            f.call.l.resize(n, 0);
                        }
                    }
                }
            }
            x if x == crate::elm::call::ELM_CALL_K => {
                // call.K → str_list on current call's K
                if element.len() > 1 {
                    let sub = &element[1..];
                    if !sub.is_empty() && sub[0] == crate::elm::ELM_ARRAY {
                        let idx = sub.get(1).copied().unwrap_or(0) as usize;
                        let cur = self.frames.last();
                        if arg_list_id == 0 {
                            let val = cur
                                .and_then(|f| f.call.k.get(idx).cloned())
                                .unwrap_or_default();
                            self.stack.push_str(val);
                        } else if arg_list_id == 1 {
                            if let Some(PropValue::Str(v)) = args.first().map(|p| &p.value) {
                                if let Some(f) = self.frames.last_mut() {
                                    if idx < f.call.k.len() {
                                        f.call.k[idx] = v.clone();
                                    }
                                }
                            }
                        }
                    } else if crate::elm::list::is_strlist_get_size(sub[0]) {
                        let len = self
                            .frames
                            .last()
                            .map(|f| f.call.k.len() as i32)
                            .unwrap_or(0);
                        self.stack.push_int(len);
                    } else if crate::elm::list::is_strlist_init(sub[0]) {
                        if let Some(f) = self.frames.last_mut() {
                            for v in f.call.k.iter_mut() {
                                v.clear();
                            }
                        }
                    } else if crate::elm::list::is_strlist_resize(sub[0]) {
                        let n = match args.first().map(|p| &p.value) {
                            Some(PropValue::Int(v)) => (*v).max(0) as usize,
                            _ => 0,
                        };
                        if let Some(f) = self.frames.last_mut() {
                            f.call.k.resize(n, String::new());
                        }
                    }
                }
            }
            _ => {
                // call_prop or unknown — accept
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                } else if ret_form == crate::elm::form::STR {
                    self.stack.push_str(String::new());
                }
            }
        }
    }

    fn excall_slot_state(&self, slot: Option<usize>) -> bool {
        match slot {
            Some(i) if i < self.excall_allocated.len() => self.excall_allocated[i],
            _ => self.excall_allocated.iter().any(|v| *v),
        }
    }

    /// Route `global.excall.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_excall(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        self.try_command_excall_with_slot(element, arg_list_id, args, ret_form, host, None)
    }

    fn try_command_excall_with_slot(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
        excall_slot: Option<usize>,
    ) -> bool {
        use crate::elm::excall::*;
        if element.is_empty() {
            return true;
        }

        if let Some(scope) = self.resolve_excall_scope(excall_slot, host) {
            if scope != 0
                && !self.excall_scope_ready(scope)
                && Self::excall_scope_requires_ready(element[0])
            {
                self.excall_report_not_ready(host);
                return true;
            }
        }

        match element[0] {
            x if x == crate::elm::ELM_ARRAY => {
                // excall[0] / excall[1] dispatch
                if element.len() > 2 {
                    let slot = usize::try_from(element[1]).ok();
                    return self.try_command_excall_with_slot(
                        &element[2..],
                        arg_list_id,
                        args,
                        ret_form,
                        host,
                        slot,
                    );
                }
                true
            }
            ELM_EXCALL_ALLOC => {
                if let Some(scope) = self.resolve_excall_scope(excall_slot, host) {
                    if !Self::excall_scope_supports_lifecycle(scope) {
                        self.excall_report_invalid_global_lifecycle(host, "alloc");
                        return true;
                    }
                    if scope < self.excall_allocated.len() {
                        self.excall_allocated[scope] = true;
                        self.excall_counter_list_size[scope] = self.counter_list_size;
                        self.reset_excall_scope_counter_slots(scope);
                        self.reset_excall_scope_runtime_state(scope, true);
                    }
                }
                true
            }
            ELM_EXCALL_FREE => {
                if let Some(scope) = self.resolve_excall_scope(excall_slot, host) {
                    if !Self::excall_scope_supports_lifecycle(scope) {
                        self.excall_report_invalid_global_lifecycle(host, "free");
                        return true;
                    }
                    if scope < self.excall_allocated.len() {
                        self.excall_allocated[scope] = false;
                        self.excall_counter_list_size[scope] = 0;
                        self.reset_excall_scope_counter_slots(scope);
                        self.reset_excall_scope_runtime_state(scope, false);
                    }
                }
                true
            }
            ELM_EXCALL_IS_EXCALL => {
                self.stack
                    .push_int(self.excall_allocated.iter().any(|v| *v) as i32);
                true
            }
            ELM_EXCALL_CHECK_ALLOC => {
                if let Some(scope) = self.resolve_excall_scope(excall_slot, host) {
                    if !Self::excall_scope_supports_lifecycle(scope) {
                        self.excall_report_invalid_global_lifecycle(host, "check_alloc");
                        self.stack.push_int(0);
                    } else {
                        self.stack.push_int(self.excall_scope_ready(scope) as i32);
                    }
                } else {
                    self.stack.push_int(0);
                }
                true
            }
            ELM_EXCALL_F => {
                if let Some(scope) = self.resolve_excall_scope(excall_slot, host) {
                    return self.try_command_excall_f_scoped(
                        &element[1..],
                        arg_list_id,
                        args,
                        ret_form,
                        host,
                        scope,
                    );
                }
                true
            }
            ELM_EXCALL_COUNTER => {
                if element.len() > 1 {
                    if let Some(scope) = self.resolve_excall_scope(excall_slot, host) {
                        return self.try_command_excall_counter_scoped(
                            &element[1..],
                            args,
                            ret_form,
                            host,
                            scope,
                        );
                    }
                }
                true
            }
            ELM_EXCALL_FRAME_ACTION => {
                if let Some(scope) = self.resolve_excall_scope(excall_slot, host) {
                    return self.try_command_excall_frame_action_scoped(
                        &element[1..],
                        args,
                        ret_form,
                        host,
                        scope,
                    );
                }
                true
            }
            ELM_EXCALL_FRAME_ACTION_CH => {
                if let Some(scope) = self.resolve_excall_scope(excall_slot, host) {
                    return self.try_command_excall_frame_action_ch_scoped(
                        &element[1..],
                        args,
                        ret_form,
                        host,
                        scope,
                    );
                }
                true
            }
            ELM_EXCALL_STAGE | ELM_EXCALL_FRONT | ELM_EXCALL_BACK | ELM_EXCALL_NEXT => {
                // Delegate to stage.
                if element.len() > 1 {
                    let stage_idx = if element[0] == ELM_EXCALL_FRONT {
                        1
                    } else if element[0] == ELM_EXCALL_NEXT {
                        2
                    } else {
                        0
                    };
                    if self.try_command_stage(
                        stage_idx,
                        &element[1..],
                        arg_list_id,
                        args,
                        ret_form,
                        host,
                    ) {
                        return true;
                    }
                }
                true
            }
            ELM_EXCALL_SCRIPT => {
                // C++ cmd_call.cpp routes scope0/scope1 script lane into the same
                // cmd_script.cpp::tnm_command_proc_script_excall storage (Gp_excall font pod).
                if element.len() > 1 {
                    if let Some(scope) = self.resolve_excall_scope(excall_slot, host) {
                        let font_scope = Self::excall_script_font_scope(scope);
                        return self.try_command_script_excall(
                            font_scope,
                            &element[1..],
                            args,
                            ret_form,
                            host,
                        );
                    }
                }
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(excall)");
                true
            }
        }
    }
}
