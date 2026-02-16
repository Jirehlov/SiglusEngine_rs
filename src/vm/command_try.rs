use super::*;


impl Vm {
    pub(super) fn try_command(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        _named_arg_cnt: i32,
        ret_form: i32,
        provider: &mut dyn SceneProvider,
        host: &mut dyn Host,
    ) -> Result<bool> {
        if !element.is_empty() {}
        if element.is_empty() {
            return Ok(false);
        }

        if let Some(ret) = self.try_command_global_head(
            element,
            arg_list_id,
            args,
            ret_form,
            provider,
            host,
        )? {
            return Ok(ret);
        }

        if let Some(ret) = self.try_command_global_tail(element, arg_list_id, args, ret_form, host)? {
            return Ok(ret);
        }

        if crate::elm::owner::is_user_cmd(element[0]) {
            let user_cmd_id = elm_code(element[0]) as i32;
            self.proc_user_cmd_call(user_cmd_id, args, ret_form, false, provider)?;
            return Ok(true);
        }

        // -----------------------------------------------------------------
        // Internal list helpers for call.L / call.K and local call props
        // -----------------------------------------------------------------
        if element.len() >= 3 && crate::elm::call::is_cur_call(element[0]) {
            // call.L (intlist)
            if crate::elm::call::is_call_l(element[1]) {
                let method = element[2];
                if crate::elm::list::is_intlist_get_size(method) {
                    let n = self.frames.last().map(|f| f.call.l.len()).unwrap_or(0) as i32;
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(n);
                    }
                    return Ok(true);
                }
                if crate::elm::list::is_intlist_init(method) {
                    if let Some(frame) = self.frames.last_mut() {
                        for x in &mut frame.call.l {
                            *x = 0;
                        }
                    }
                    return Ok(true);
                }
                if crate::elm::list::is_intlist_resize(method) {
                    let n = match args.get(0).map(|p| &p.value) {
                        Some(PropValue::Int(v)) => *v,
                        _ => 0,
                    };
                    let n = if n > 0 { n as usize } else { 0 };
                    if let Some(frame) = self.frames.last_mut() {
                        frame.call.l.resize(n, 0);
                    }
                    return Ok(true);
                }
                if crate::elm::list::is_intlist_sets(method) {
                    let start = match args.get(0).map(|p| &p.value) {
                        Some(PropValue::Int(v)) => *v,
                        _ => 0,
                    };
                    let start = if start > 0 { start as usize } else { 0 };
                    let values: Vec<i32> = args
                        .iter()
                        .skip(1)
                        .map(|p| match &p.value {
                            PropValue::Int(v) => *v,
                            _ => 0,
                        })
                        .collect();
                    if let Some(frame) = self.frames.last_mut() {
                        let list = &mut frame.call.l;
                        let need = start.saturating_add(values.len());
                        if list.len() < need {
                            list.resize(need, 0);
                        }
                        for (i, v) in values.iter().enumerate() {
                            list[start + i] = *v;
                        }
                    }
                    return Ok(true);
                }
                if crate::elm::list::is_intlist_clear(method) {
                    let a = match args.get(0).map(|p| &p.value) {
                        Some(PropValue::Int(v)) => *v,
                        _ => 0,
                    };
                    let b = match args.get(1).map(|p| &p.value) {
                        Some(PropValue::Int(v)) => *v,
                        _ => a,
                    };
                    let fill = match args.get(2).map(|p| &p.value) {
                        Some(PropValue::Int(v)) => *v,
                        _ => 0,
                    };
                    let start = if a > 0 { a as usize } else { 0 };
                    let end = if b > 0 { b as usize } else { 0 };
                    if let Some(frame) = self.frames.last_mut() {
                        let list = &mut frame.call.l;
                        if list.is_empty() {
                            return Ok(true);
                        }
                        if start >= list.len() {
                            return Ok(true);
                        }
                        let mut end_incl = end;
                        if end_incl >= list.len() {
                            end_incl = list.len().saturating_sub(1);
                        }
                        if start > end_incl {
                            return Ok(true);
                        }
                        for i in start..=end_incl {
                            list[i] = fill;
                        }
                    }
                    return Ok(true);
                }
            }

            // call.K (strlist)
            if crate::elm::call::is_call_k(element[1]) {
                let method = element[2];
                if crate::elm::list::is_strlist_get_size(method) {
                    let n = self.frames.last().map(|f| f.call.k.len()).unwrap_or(0) as i32;
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(n);
                    }
                    return Ok(true);
                }
                if crate::elm::list::is_strlist_init(method) {
                    if let Some(frame) = self.frames.last_mut() {
                        for x in &mut frame.call.k {
                            x.clear();
                        }
                    }
                    return Ok(true);
                }
                if crate::elm::list::is_strlist_resize(method) {
                    let n = match args.get(0).map(|p| &p.value) {
                        Some(PropValue::Int(v)) => *v,
                        _ => 0,
                    };
                    let n = if n > 0 { n as usize } else { 0 };
                    if let Some(frame) = self.frames.last_mut() {
                        frame.call.k.resize(n, String::new());
                    }
                    return Ok(true);
                }
            }

            // cur_call.<call_prop> list methods (best-effort)
            let head = element[1];
            if crate::elm::owner::is_call_prop(head) {
                let cp_idx = elm_code(head) as usize;
                let method = element[2];

                let mut ret_int: Option<i32> = None;

                {
                    let frame = match self.frames.last_mut() {
                        Some(f) => f,
                        None => return Ok(false),
                    };
                    if cp_idx >= frame.call.user_props.len() {
                        return Ok(true);
                    }

                    match &mut frame.call.user_props[cp_idx].value {
                        PropValue::IntList(v) => {
                            if crate::elm::list::is_intlist_get_size(method) {
                                ret_int = Some(v.len() as i32);
                            } else if crate::elm::list::is_intlist_init(method) {
                                for x in v.iter_mut() {
                                    *x = 0;
                                }
                            } else if crate::elm::list::is_intlist_resize(method) {
                                let n = match args.get(0).map(|p| &p.value) {
                                    Some(PropValue::Int(x)) => *x,
                                    _ => 0,
                                };
                                let n = if n > 0 { n as usize } else { 0 };
                                v.resize(n, 0);
                            } else if crate::elm::list::is_intlist_sets(method) {
                                let start = match args.get(0).map(|p| &p.value) {
                                    Some(PropValue::Int(x)) => *x,
                                    _ => 0,
                                };
                                let start = if start > 0 { start as usize } else { 0 };
                                let values: Vec<i32> = args
                                    .iter()
                                    .skip(1)
                                    .map(|p| match &p.value {
                                        PropValue::Int(x) => *x,
                                        _ => 0,
                                    })
                                    .collect();
                                let need = start.saturating_add(values.len());
                                if v.len() < need {
                                    v.resize(need, 0);
                                }
                                for (i, vv) in values.iter().enumerate() {
                                    v[start + i] = *vv;
                                }
                            } else if crate::elm::list::is_intlist_clear(method) {
                                let a = match args.get(0).map(|p| &p.value) {
                                    Some(PropValue::Int(x)) => *x,
                                    _ => 0,
                                };
                                let b = match args.get(1).map(|p| &p.value) {
                                    Some(PropValue::Int(x)) => *x,
                                    _ => a,
                                };
                                let fill = match args.get(2).map(|p| &p.value) {
                                    Some(PropValue::Int(x)) => *x,
                                    _ => 0,
                                };
                                let start = if a > 0 { a as usize } else { 0 };
                                let end = if b > 0 { b as usize } else { 0 };
                                if !v.is_empty() && start < v.len() {
                                    let mut end_incl = end;
                                    if end_incl >= v.len() {
                                        end_incl = v.len().saturating_sub(1);
                                    }
                                    if start <= end_incl {
                                        for i in start..=end_incl {
                                            v[i] = fill;
                                        }
                                    }
                                }
                            } else {
                                return Ok(false);
                            }
                        }
                        PropValue::StrList(v) => {
                            if crate::elm::list::is_strlist_get_size(method) {
                                ret_int = Some(v.len() as i32);
                            } else if crate::elm::list::is_strlist_init(method) {
                                for x in v.iter_mut() {
                                    x.clear();
                                }
                            } else if crate::elm::list::is_strlist_resize(method) {
                                let n = match args.get(0).map(|p| &p.value) {
                                    Some(PropValue::Int(x)) => *x,
                                    _ => 0,
                                };
                                let n = if n > 0 { n as usize } else { 0 };
                                v.resize(n, String::new());
                            } else {
                                return Ok(false);
                            }
                        }
                        _ => return Ok(false),
                    }
                }

                if let Some(v) = ret_int {
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(v);
                    }
                    return Ok(true);
                }

                return Ok(true);
            }
        }

        Ok(false)
    }
}
