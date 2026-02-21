// Call / Excall command routing — aligns with C++ cmd_call.cpp
use super::*;

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

    /// Route `global.excall.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_excall(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::excall::*;
        if element.is_empty() {
            return true;
        }
        match element[0] {
            x if x == crate::elm::ELM_ARRAY => {
                // excall[0] / excall[1] dispatch
                if element.len() > 2 {
                    return self.try_command_excall(
                        &element[2..],
                        _arg_list_id,
                        _args,
                        ret_form,
                        host,
                    );
                }
                true
            }
            ELM_EXCALL_ALLOC | ELM_EXCALL_FREE => {
                // accept
                true
            }
            ELM_EXCALL_IS_EXCALL => {
                self.stack.push_int(0);
                true
            }
            ELM_EXCALL_CHECK_ALLOC => {
                self.stack.push_int(0);
                true
            }
            ELM_EXCALL_F => {
                // excall.F → int_list; stub accept
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                true
            }
            ELM_EXCALL_COUNTER => {
                // Delegate to counter_list
                if element.len() > 1 {
                    return self.try_command_counter_list(
                        &element[1..],
                        _arg_list_id,
                        _args,
                        ret_form,
                        host,
                    );
                }
                true
            }
            ELM_EXCALL_FRAME_ACTION | ELM_EXCALL_FRAME_ACTION_CH => {
                // accept
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                true
            }
            ELM_EXCALL_STAGE | ELM_EXCALL_FRONT | ELM_EXCALL_BACK | ELM_EXCALL_NEXT => {
                // Delegate to stage; for now accept
                if element.len() > 1 {
                    if self.try_command_stage(&element[1..], _arg_list_id, _args, ret_form, host) {
                        return true;
                    }
                }
                // Fallback to host
                false
            }
            ELM_EXCALL_SCRIPT => {
                // Delegate to script
                if element.len() > 1 {
                    return self.try_command_script(
                        &element[1..],
                        _arg_list_id,
                        _args,
                        ret_form,
                        host,
                    );
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
