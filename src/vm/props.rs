use super::opcode::cd;
use super::*;


impl Vm {
    pub(super) fn reload_user_props_from_current_scene(&mut self) {
        let (forms, values) = make_user_props(&self.lexer.dat);
        self.user_prop_forms = forms;
        self.user_prop_values = values;
    }

    pub(super) fn push_vm_value(&mut self, form: i32, v: PropValue) {
        match form {
            f if f == crate::elm::form::INT => {
                let n = match v {
                    PropValue::Int(x) => x,
                    _ => 0,
                };
                self.stack.push_int(n);
            }
            f if f == crate::elm::form::STR => {
                let s = match v {
                    PropValue::Str(x) => x,
                    _ => String::new(),
                };
                self.stack.push_str(s);
            }
            _ => {
                // Best-effort: push as element.
                if let PropValue::Element(el) = v {
                    self.stack.push_element(&el);
                } else {
                    self.stack.push_element(&[]);
                }
            }
        }
    }

    pub(super) fn proc_dec_prop(&mut self, form_code: i32, prop_id: i32, size: i32) {
        let value = if form_code == crate::elm::form::INT {
            PropValue::Int(0)
        } else if form_code == crate::elm::form::STR {
            PropValue::Str(String::new())
        } else if form_code == crate::elm::form::INTLIST {
            let n = if size > 0 { size as usize } else { 0 };
            PropValue::IntList(vec![0; n])
        } else if form_code == crate::elm::form::STRLIST {
            let n = if size > 0 { size as usize } else { 0 };
            PropValue::StrList(vec![String::new(); n])
        } else {
            // For ref/element types, keep placeholder element.
            PropValue::Element(Vec::new())
        };

        if let Some(frame) = self.frames.last_mut() {
            frame.call.user_props.push(CallProp {
                prop_id,
                form: form_code,
                value,
            });
        }
    }

    pub(super) fn proc_arg(&mut self) -> Result<()> {
        let (frame_action_flag, arg_cnt) = self
            .frames
            .last()
            .map(|f| (f.frame_action_flag, f.arg_cnt))
            .unwrap_or((false, 0));

        let len = self
            .frames
            .last()
            .map(|f| f.call.user_props.len())
            .unwrap_or(0);

        // FrameAction calls have a stricter calling convention: the first declared call prop
        // must be a FRAMEACTION, and the number of arguments must match the number of declared
        // call props. (See tnm_expand_arg_into_call_flag in the C++ original.)
        if frame_action_flag {
            if len == 0
                || self.frames.last().unwrap().call.user_props[0].form
                    != crate::elm::form::FRAMEACTION
            {
                bail!("vm: frame_action call requires first arg to be FM_FRAMEACTION");
            }
            if arg_cnt != len {
                bail!(
                    "vm: frame_action call arg_cnt mismatch: got {}, expected {}",
                    arg_cnt,
                    len
                );
            }
        }

        for i in (0..len).rev() {
            let form = self.frames.last().unwrap().call.user_props[i].form;
            let _prop_id = self.frames.last().unwrap().call.user_props[i].prop_id;

            // In the original engine, non-scalar argument types are passed as elements.
            // For local list props (FM_INTLIST/FM_STRLIST) we keep our backing storage,
            // but still pop the element to keep the stack balanced.
            if form == crate::elm::form::INTLIST || form == crate::elm::form::STRLIST {
                let _ = self.stack.pop_element()?;
                continue;
            }

            let value = if form == crate::elm::form::INT {
                PropValue::Int(self.stack.pop_int()?)
            } else if form == crate::elm::form::STR {
                PropValue::Str(self.stack.pop_str()?)
            } else {
                PropValue::Element(self.stack.pop_element()?)
            };

            self.frames.last_mut().unwrap().call.user_props[i].value = value;
        }

        Ok(())
    }

    /// Return a mutable reference to the int-flag list for the given element head, or None.
    pub(super) fn get_intflag_mut(&mut self, head: i32) -> Option<&mut Vec<i32>> {
        match crate::elm::global::intflag_slot(head) {
            Some(0) => Some(&mut self.flags_a),
            Some(1) => Some(&mut self.flags_b),
            Some(2) => Some(&mut self.flags_c),
            Some(3) => Some(&mut self.flags_d),
            Some(4) => Some(&mut self.flags_e),
            Some(5) => Some(&mut self.flags_f),
            Some(6) => Some(&mut self.flags_x),
            Some(7) => Some(&mut self.flags_g),
            Some(8) => Some(&mut self.flags_z),
            _ => None,
        }
    }

    /// Return a mutable reference to the str-flag list for the given element head, or None.
    pub(super) fn get_strflag_mut(&mut self, head: i32) -> Option<&mut Vec<String>> {
        match crate::elm::global::strflag_slot(head) {
            Some(0) => Some(&mut self.flags_s),
            Some(1) => Some(&mut self.flags_m),
            Some(2) => Some(&mut self.global_namae),
            Some(3) => Some(&mut self.local_namae),
            _ => None,
        }
    }

    /// Check if element head is a known int-flag element.
    pub(super) fn is_intflag(head: i32) -> bool {
        crate::elm::global::is_intflag_head(head)
    }

    /// Check if element head is a known str-flag element.
    pub(super) fn is_strflag(head: i32) -> bool {
        crate::elm::global::is_strflag_head(head)
    }

    pub(super) fn try_property_internal(&mut self, element: &[i32]) -> Option<(PropValue, i32)> {
        // ----- Flag int-list reads: A[idx] ... Z[idx] -----
        if element.len() >= 3 && Self::is_intflag(element[0]) && element[1] == crate::elm::ELM_ARRAY
        {
            let idx = element[2] as isize;
            if let Some(list) = self.get_intflag_mut(element[0]) {
                let val = if idx >= 0 && (idx as usize) < list.len() {
                    list[idx as usize]
                } else {
                    0
                };
                return Some((PropValue::Int(val), crate::elm::form::INT));
            }
        }
        // ----- Flag str-list reads: S[idx] / M[idx] / namae_global[idx] / namae_local[idx] -----
        if element.len() >= 3 && Self::is_strflag(element[0]) && element[1] == crate::elm::ELM_ARRAY
        {
            let idx = element[2] as isize;
            if let Some(list) = self.get_strflag_mut(element[0]) {
                let val = if idx >= 0 && (idx as usize) < list.len() {
                    list[idx as usize].clone()
                } else {
                    String::new()
                };
                return Some((PropValue::Str(val), crate::elm::form::STR));
            }
        }
        // scene user-prop (best-effort): <user_prop>[idx] / <user_prop>
        if !element.is_empty() && crate::elm::owner::is_user_prop(element[0]) {
            let up_idx = elm_code(element[0]) as usize;
            if up_idx < self.user_prop_forms.len() && up_idx < self.user_prop_values.len() {
                let form = self.user_prop_forms[up_idx];
                if element.len() >= 2 && element[1] == crate::elm::ELM_ARRAY {
                    let idx = if element.len() >= 3 {
                        element[2]
                    } else {
                        self.stack.pop_int().unwrap_or(0)
                    } as isize;
                    match &self.user_prop_values[up_idx] {
                        PropValue::IntList(v) => {
                            if idx >= 0 {
                                let out = v.get(idx as usize).copied().unwrap_or(0);
                                return Some((PropValue::Int(out), crate::elm::form::INT));
                            }
                            return Some((PropValue::Int(0), crate::elm::form::INT));
                        }
                        PropValue::StrList(v) => {
                            if idx >= 0 {
                                let out = v.get(idx as usize).cloned().unwrap_or_default();
                                return Some((PropValue::Str(out), crate::elm::form::STR));
                            }
                            return Some((PropValue::Str(String::new()), crate::elm::form::STR));
                        }
                        _ => {}
                    }
                }
                return Some((self.user_prop_values[up_idx].clone(), form));
            }
            // Unknown: default int
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        // cur_call.L[idx]
        if element.len() >= 4
            && crate::elm::call::is_cur_call(element[0])
            && crate::elm::call::is_call_l(element[1])
            && element[2] == crate::elm::ELM_ARRAY
        {
            let idx = element[3] as isize;
            if idx >= 0 {
                let idx = idx as usize;
                let call = &self.frames.last()?.call;
                if idx < call.l.len() {
                    return Some((PropValue::Int(call.l[idx]), crate::elm::form::INT));
                }
            }
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }

        // cur_call.K[idx]
        if element.len() >= 4
            && crate::elm::call::is_cur_call(element[0])
            && crate::elm::call::is_call_k(element[1])
            && element[2] == crate::elm::ELM_ARRAY
        {
            let idx = element[3] as isize;
            if idx >= 0 {
                let idx = idx as usize;
                let call = &self.frames.last()?.call;
                if idx < call.k.len() {
                    return Some((PropValue::Str(call.k[idx].clone()), crate::elm::form::STR));
                }
            }
            return Some((PropValue::Str(String::new()), crate::elm::form::STR));
        }

        // cur_call.<call_prop>
        if element.len() >= 2 && crate::elm::call::is_cur_call(element[0]) {
            let head = element[1];
            if crate::elm::owner::is_call_prop(head) {
                let idx = elm_code(head) as usize;
                let call = &self.frames.last()?.call;
                if idx < call.user_props.len() {
                    let p = &call.user_props[idx];
                    if p.form == crate::elm::form::INT {
                        if let PropValue::Int(v) = &p.value {
                            return Some((PropValue::Int(*v), crate::elm::form::INT));
                        }
                    } else if p.form == crate::elm::form::STR {
                        if let PropValue::Str(s) = &p.value {
                            return Some((PropValue::Str(s.clone()), crate::elm::form::STR));
                        }
                    }
                }
            }
        }

        // cur_call.<call_prop>[idx] (best-effort for local list props)
        if element.len() >= 3 && crate::elm::call::is_cur_call(element[0]) {
            let head = element[1];
            if crate::elm::owner::is_call_prop(head) && element[2] == crate::elm::ELM_ARRAY {
                let cp_idx = elm_code(head) as usize;
                let idx = if element.len() >= 4 {
                    element[3]
                } else {
                    self.stack.pop_int().unwrap_or(0)
                } as isize;
                let call = &self.frames.last()?.call;
                if cp_idx < call.user_props.len() {
                    match &call.user_props[cp_idx].value {
                        PropValue::IntList(v) => {
                            if idx >= 0 {
                                let out = v.get(idx as usize).copied().unwrap_or(0);
                                return Some((PropValue::Int(out), crate::elm::form::INT));
                            }
                            return Some((PropValue::Int(0), crate::elm::form::INT));
                        }
                        PropValue::StrList(v) => {
                            if idx >= 0 {
                                let out = v.get(idx as usize).cloned().unwrap_or_default();
                                return Some((PropValue::Str(out), crate::elm::form::STR));
                            }
                            return Some((PropValue::Str(String::new()), crate::elm::form::STR));
                        }
                        _ => {}
                    }
                }
            }
        }

        // ELM_GLOBAL_NAMAE access
        if element.len() >= 2 && crate::elm::global::is_namae_access(element[0]) {
            // Case 1: NAMAE("key")
            // Can't implement without mapping data. Return key as fallback.
            if element.len() >= 2 && element[1] == cd::CD_TEXT as i32 {
                // This check is intricate as parser output varies.
                // If property access is `NAMAE("str")`, VM sees `[NAMAE, ...]`?
                // Actually, `NAMAE("str")` is likely a command call if not assigned.
                // As a property, it's `NAMAE("str")`.
                // We'll leave this empty for now as most NAMAE usage is `NAMAE = "str"` (set speaker)
                // or `NAMAE_GLOBAL[i]`.
            }
            // Case 2: NAMAE[idx] (treated as global namae access)
            if element.len() >= 3 && element[1] == crate::elm::ELM_ARRAY {
                let idx = element[2] as isize;
                if idx >= 0 && (idx as usize) < self.global_namae.len() {
                    return Some((
                        PropValue::Str(self.global_namae[idx as usize].clone()),
                        crate::elm::form::STR,
                    ));
                }
                return Some((PropValue::Str(String::new()), crate::elm::form::STR));
            }
        }

        None
    }

    pub(super) fn try_assign_internal(&mut self, element: &[i32], al_id: i32, rhs: &Prop) -> Result<bool> {
        // In the original engine, al_id==1 is the common "set" path for properties.
        if al_id != 1 {
            return Ok(false);
        }

        // ----- Flag int-list writes: A[idx] = int -----
        if element.len() >= 3 && Self::is_intflag(element[0]) && element[1] == crate::elm::ELM_ARRAY
        {
            let idx = element[2] as isize;
            if idx >= 0 {
                let v = match &rhs.value {
                    PropValue::Int(x) => *x,
                    _ => 0,
                };
                if let Some(list) = self.get_intflag_mut(element[0]) {
                    let idx = idx as usize;
                    if list.len() <= idx {
                        list.resize(idx + 1, 0);
                    }
                    list[idx] = v;
                }
            }
            return Ok(true);
        }

        // ----- Flag str-list writes: S[idx] = str -----
        if element.len() >= 3 && Self::is_strflag(element[0]) && element[1] == crate::elm::ELM_ARRAY
        {
            let idx = element[2] as isize;
            if idx >= 0 {
                let v = match &rhs.value {
                    PropValue::Str(s) => s.clone(),
                    _ => String::new(),
                };
                if let Some(list) = self.get_strflag_mut(element[0]) {
                    let idx = idx as usize;
                    if list.len() <= idx {
                        list.resize(idx + 1, String::new());
                    }
                    list[idx] = v;
                }
            }
            return Ok(true);
        }

        // ELM_GLOBAL_NAMAE[idx] = "str"
        if element.len() >= 3
            && crate::elm::global::is_namae_access(element[0])
            && element[1] == crate::elm::ELM_ARRAY
        {
            let idx = element[2] as isize;
            if idx >= 0 {
                let v = match &rhs.value {
                    PropValue::Str(s) => s.clone(),
                    _ => String::new(),
                };
                let list = &mut self.global_namae;
                let idx = idx as usize;
                if list.len() <= idx {
                    list.resize(idx + 1, String::new());
                }
                list[idx] = v;
            }
            return Ok(true);
        }

        // scene user-prop (best-effort) assign: <user_prop>[idx] / <user_prop>
        if !element.is_empty() && crate::elm::owner::is_user_prop(element[0]) {
            let up_idx = elm_code(element[0]) as usize;
            if up_idx < self.user_prop_forms.len() && up_idx < self.user_prop_values.len() {
                let form = self.user_prop_forms[up_idx];
                if element.len() >= 3 && element[1] == crate::elm::ELM_ARRAY {
                    let idx = element[2] as isize;
                    if idx >= 0 {
                        let idx = idx as usize;
                        match (&mut self.user_prop_values[up_idx], &rhs.value) {
                            (PropValue::IntList(v), PropValue::Int(x)) => {
                                if v.len() <= idx {
                                    v.resize(idx + 1, 0);
                                }
                                v[idx] = *x;
                                return Ok(true);
                            }
                            (PropValue::StrList(v), PropValue::Str(x)) => {
                                if v.len() <= idx {
                                    v.resize(idx + 1, String::new());
                                }
                                v[idx] = x.clone();
                                return Ok(true);
                            }
                            _ => {}
                        }
                    }
                }

                // direct set
                match form {
                    f if f == crate::elm::form::INT => {
                        let v = match &rhs.value {
                            PropValue::Int(x) => *x,
                            _ => 0,
                        };
                        self.user_prop_values[up_idx] = PropValue::Int(v);
                    }
                    f if f == crate::elm::form::STR => {
                        let v = match &rhs.value {
                            PropValue::Str(x) => x.clone(),
                            _ => String::new(),
                        };
                        self.user_prop_values[up_idx] = PropValue::Str(v);
                    }
                    f if f == crate::elm::form::INTLIST => {
                        // allow assigning intlist via element fallback (rare); otherwise ignore
                        if let PropValue::IntList(v) = &rhs.value {
                            self.user_prop_values[up_idx] = PropValue::IntList(v.clone());
                        }
                    }
                    f if f == crate::elm::form::STRLIST => {
                        if let PropValue::StrList(v) = &rhs.value {
                            self.user_prop_values[up_idx] = PropValue::StrList(v.clone());
                        }
                    }
                    _ => {
                        self.user_prop_values[up_idx] = rhs.value.clone();
                    }
                }
                return Ok(true);
            }
        }

        // cur_call.L[idx] = int
        if element.len() >= 4
            && crate::elm::call::is_cur_call(element[0])
            && crate::elm::call::is_call_l(element[1])
            && element[2] == crate::elm::ELM_ARRAY
        {
            let idx = element[3] as isize;
            if idx >= 0 {
                let idx = idx as usize;
                let v = match &rhs.value {
                    PropValue::Int(x) => *x,
                    _ => 0,
                };
                if let Some(frame) = self.frames.last_mut() {
                    if idx < frame.call.l.len() {
                        frame.call.l[idx] = v;
                        return Ok(true);
                    }
                }
            }
            return Ok(true);
        }

        // cur_call.K[idx] = str
        if element.len() >= 4
            && crate::elm::call::is_cur_call(element[0])
            && crate::elm::call::is_call_k(element[1])
            && element[2] == crate::elm::ELM_ARRAY
        {
            let idx = element[3] as isize;
            if idx >= 0 {
                let idx = idx as usize;
                let v = match &rhs.value {
                    PropValue::Str(s) => s.clone(),
                    _ => String::new(),
                };
                if let Some(frame) = self.frames.last_mut() {
                    if idx < frame.call.k.len() {
                        frame.call.k[idx] = v;
                        return Ok(true);
                    }
                }
            }
            return Ok(true);
        }

        // cur_call.<call_prop> = (int/str)
        if element.len() >= 2 && crate::elm::call::is_cur_call(element[0]) {
            let head = element[1];
            if crate::elm::owner::is_call_prop(head) {
                let idx = elm_code(head) as usize;
                if let Some(frame) = self.frames.last_mut() {
                    if idx < frame.call.user_props.len() {
                        let form = frame.call.user_props[idx].form;
                        if form == crate::elm::form::INT {
                            let v = match &rhs.value {
                                PropValue::Int(x) => *x,
                                _ => 0,
                            };
                            frame.call.user_props[idx].value = PropValue::Int(v);
                            return Ok(true);
                        } else if form == crate::elm::form::STR {
                            let v = match &rhs.value {
                                PropValue::Str(s) => s.clone(),
                                _ => String::new(),
                            };
                            frame.call.user_props[idx].value = PropValue::Str(v);
                            return Ok(true);
                        }
                    }
                }
                return Ok(true);
            }
        }

        // cur_call.<call_prop>[idx] = (int/str) (best-effort for local list props)
        if element.len() >= 4 && crate::elm::call::is_cur_call(element[0]) {
            let head = element[1];
            if crate::elm::owner::is_call_prop(head) && element[2] == crate::elm::ELM_ARRAY {
                let cp_idx = elm_code(head) as usize;
                let idx = element[3] as isize;
                if idx >= 0 {
                    let idx = idx as usize;
                    if let Some(frame) = self.frames.last_mut() {
                        if cp_idx < frame.call.user_props.len() {
                            match &mut frame.call.user_props[cp_idx].value {
                                PropValue::IntList(v) => {
                                    let vv = match &rhs.value {
                                        PropValue::Int(x) => *x,
                                        _ => 0,
                                    };
                                    if idx < v.len() {
                                        v[idx] = vv;
                                    }
                                    return Ok(true);
                                }
                                PropValue::StrList(v) => {
                                    let vv = match &rhs.value {
                                        PropValue::Str(s) => s.clone(),
                                        _ => String::new(),
                                    };
                                    if idx < v.len() {
                                        v[idx] = vv;
                                    }
                                    return Ok(true);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                return Ok(true);
            }
        }

        Ok(false)
    }

}
