use super::opcode::cd;
use super::*;

impl Vm {
    pub(super) fn resolve_call_prop_slot(call: &CallContext, cp_code: usize) -> Option<usize> {
        if cp_code < call.user_props.len() {
            return Some(cp_code);
        }
        call.user_props
            .iter()
            .position(|cp| cp.prop_id >= 0 && cp.prop_id as usize == cp_code)
    }

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

    pub(super) fn proc_arg(&mut self, host: &mut dyn Host) -> Result<()> {
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
                self.report_vm_fatal_with_context(
                    host,
                    "vm: frame_action call requires first arg to be FM_FRAMEACTION",
                );
                bail!("vm: frame_action call requires first arg to be FM_FRAMEACTION");
            }
            if arg_cnt != len {
                self.report_vm_fatal_with_context(
                    host,
                    &format!(
                        "vm: frame_action call arg_cnt mismatch: got {}, expected {}",
                        arg_cnt, len
                    ),
                );
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
                let src_element_raw = self.stack.pop_element().map_err(|e| {
                    self.report_vm_fatal_with_context(
                        host,
                        &format!("CD_ARG: pop list source element failed: {}", e),
                    );
                    e
                })?;
                let src_element = self.resolve_command_element_alias(&src_element_raw);
                if form == crate::elm::form::INTLIST {
                    if let Some(v) = self.resolve_intlist_source(&src_element) {
                        self.frames.last_mut().unwrap().call.user_props[i].value =
                            PropValue::IntList(v);
                    } else {
                        self.report_vm_fatal_with_context(
                            host,
                            "CD_ARG: unresolved INTLIST source element",
                        );
                        bail!("CD_ARG: unresolved INTLIST source element");
                    }
                } else if let Some(v) = self.resolve_strlist_source(&src_element) {
                    self.frames.last_mut().unwrap().call.user_props[i].value =
                        PropValue::StrList(v);
                } else {
                    self.report_vm_fatal_with_context(
                        host,
                        "CD_ARG: unresolved STRLIST source element",
                    );
                    bail!("CD_ARG: unresolved STRLIST source element");
                }
                continue;
            }

            let value = if form == crate::elm::form::INT {
                PropValue::Int(self.stack.pop_int().map_err(|e| {
                    self.report_vm_fatal_with_context(
                        host,
                        &format!("CD_ARG: pop int failed: {}", e),
                    );
                    e
                })?)
            } else if form == crate::elm::form::STR {
                PropValue::Str(self.stack.pop_str().map_err(|e| {
                    self.report_vm_fatal_with_context(
                        host,
                        &format!("CD_ARG: pop str failed: {}", e),
                    );
                    e
                })?)
            } else {
                PropValue::Element(self.stack.pop_element().map_err(|e| {
                    self.report_vm_fatal_with_context(
                        host,
                        &format!("CD_ARG: pop element failed: {}", e),
                    );
                    e
                })?)
            };

            self.frames.last_mut().unwrap().call.user_props[i].value = value;
        }

        Ok(())
    }

    fn get_intflag_ref(&self, head: i32) -> Option<&Vec<i32>> {
        match crate::elm::global::intflag_slot(head) {
            Some(0) => Some(&self.flags_a),
            Some(1) => Some(&self.flags_b),
            Some(2) => Some(&self.flags_c),
            Some(3) => Some(&self.flags_d),
            Some(4) => Some(&self.flags_e),
            Some(5) => Some(&self.flags_f),
            Some(6) => Some(&self.flags_x),
            Some(7) => Some(&self.flags_g),
            Some(8) => Some(&self.flags_z),
            _ => None,
        }
    }

    fn get_strflag_ref(&self, head: i32) -> Option<&Vec<String>> {
        match crate::elm::global::strflag_slot(head) {
            Some(0) => Some(&self.flags_s),
            Some(1) => Some(&self.flags_m),
            Some(2) => Some(&self.global_namae),
            Some(3) => Some(&self.local_namae),
            _ => None,
        }
    }

    pub(super) fn resolve_intlist_source(&self, element: &[i32]) -> Option<Vec<i32>> {
        if element.len() >= 2 && crate::elm::call::is_cur_call(element[0]) {
            if crate::elm::call::is_call_l(element[1]) {
                return self.frames.last().map(|f| f.call.l.clone());
            }
            if crate::elm::owner::is_call_prop(element[1]) {
                let cp = elm_code(element[1]) as usize;
                let call = &self.frames.last()?.call;
                let slot = Self::resolve_call_prop_slot(call, cp)?;
                if let PropValue::IntList(v) = &call.user_props[slot].value {
                    return Some(v.clone());
                }
            }
        }
        if element.len() == 1 {
            if let Some(v) = self.get_intflag_ref(element[0]) {
                return Some(v.clone());
            }
            if crate::elm::owner::is_user_prop(element[0]) {
                let up = elm_code(element[0]) as usize;
                if let Some(PropValue::IntList(v)) = self.user_prop_values.get(up) {
                    return Some(v.clone());
                }
            }
        }
        None
    }

    pub(super) fn resolve_strlist_source(&self, element: &[i32]) -> Option<Vec<String>> {
        if element.len() >= 2 && crate::elm::call::is_cur_call(element[0]) {
            if crate::elm::call::is_call_k(element[1]) {
                return self.frames.last().map(|f| f.call.k.clone());
            }
            if crate::elm::owner::is_call_prop(element[1]) {
                let cp = elm_code(element[1]) as usize;
                let call = &self.frames.last()?.call;
                let slot = Self::resolve_call_prop_slot(call, cp)?;
                if let PropValue::StrList(v) = &call.user_props[slot].value {
                    return Some(v.clone());
                }
            }
        }
        if element.len() == 1 {
            if let Some(v) = self.get_strflag_ref(element[0]) {
                return Some(v.clone());
            }
            if crate::elm::owner::is_user_prop(element[0]) {
                let up = elm_code(element[0]) as usize;
                if let Some(PropValue::StrList(v)) = self.user_prop_values.get(up) {
                    return Some(v.clone());
                }
            }
        }
        None
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

    include!("props_stage_routes.rs");

    /// Check if element head is a known int-flag element.
    pub(super) fn is_intflag(head: i32) -> bool {
        crate::elm::global::is_intflag_head(head)
    }

    /// Check if element head is a known str-flag element.
    pub(super) fn is_strflag(head: i32) -> bool {
        crate::elm::global::is_strflag_head(head)
    }

    pub(super) fn is_known_internal_property_target(element: &[i32]) -> bool {
        if element.is_empty() {
            return false;
        }
        let head = element[0];
        crate::elm::global::is_stage(head)
            || crate::elm::global::is_back(head)
            || crate::elm::global::is_front(head)
            || crate::elm::global::is_next(head)
            || crate::elm::call::is_cur_call(head)
            || crate::elm::owner::is_user_prop(head)
            || Self::is_intflag(head)
            || Self::is_strflag(head)
            || crate::elm::global::is_namae_access(head)
    }

    fn validate_ref_tail_shape(
        &mut self,
        tail: &[i32],
        host: &mut dyn Host,
        err_tag: &str,
    ) -> bool {
        if tail.is_empty() {
            return false;
        }
        let mut pos = 0usize;
        while pos < tail.len() {
            let cur = tail[pos];
            if cur == crate::elm::ELM_UP {
                if pos == 0 {
                    host.on_error_fatal(&format!("{}: ELM_UP is not supported", err_tag));
                } else {
                    host.on_error_fatal(&format!("{}: unexpected ELM_UP in nested tail", err_tag));
                }
                return false;
            }
            if cur == crate::elm::ELM_ARRAY {
                if pos + 1 >= tail.len() {
                    host.on_error_fatal(&format!("{}: missing array index", err_tag));
                    return false;
                }
                if pos + 2 < tail.len() && tail[pos + 2] == crate::elm::ELM_UP {
                    host.on_error_fatal(&format!(
                        "{}: array -> up nested tail is invalid",
                        err_tag
                    ));
                    return false;
                }
                pos += 2;
                continue;
            }
            if pos + 1 < tail.len() {
                host.on_error_fatal(&format!("{}: unexpected nested tail", err_tag));
                return false;
            }
            break;
        }
        true
    }
    fn try_property_via_ref_target(
        &mut self,
        ref_element: &[i32],
        tail: &[i32],
        host: &mut dyn Host,
        err_tag: &str,
    ) -> Result<Option<(PropValue, i32)>> {
        if tail.is_empty() {
            return Ok(None);
        }
        if !self.validate_ref_tail_shape(tail, host, err_tag) {
            return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
        }
        let mut target = self.resolve_command_element_alias(ref_element);
        if target.is_empty() {
            host.on_error_fatal(err_tag);
            return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
        }
        target.extend_from_slice(tail);
        if target == ref_element {
            host.on_error_fatal(err_tag);
            return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
        }
        if let Some(v) = self.try_property_internal(&target, host)? {
            return Ok(Some(v));
        }
        if let Some((ret, form)) = host.on_property_typed(&target) {
            return Ok(Some((ret, form)));
        }
        host.on_error_fatal(err_tag);
        Ok(Some((PropValue::Int(0), crate::elm::form::INT)))
    }

    pub(super) fn try_property_internal(
        &mut self,
        element: &[i32],
        host: &mut dyn Host,
    ) -> Result<Option<(PropValue, i32)>> {
        if let Some(v) = self.try_property_stage_object_group(element, host) {
            return Ok(Some(v));
        }

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
                return Ok(Some((PropValue::Int(val), crate::elm::form::INT)));
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
                return Ok(Some((PropValue::Str(val), crate::elm::form::STR)));
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
                        self.report_vm_fatal_with_context(
                            host,
                            "CD_PROPERTY: user_prop[list] missing index operand",
                        );
                        bail!("CD_PROPERTY: user_prop[list] missing index operand");
                    } as isize;
                    match &self.user_prop_values[up_idx] {
                        PropValue::IntList(v) => {
                            if idx >= 0 {
                                let out = v.get(idx as usize).copied().unwrap_or(0);
                                return Ok(Some((PropValue::Int(out), crate::elm::form::INT)));
                            }
                            return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
                        }
                        PropValue::StrList(v) => {
                            if idx >= 0 {
                                let out = v.get(idx as usize).cloned().unwrap_or_default();
                                return Ok(Some((PropValue::Str(out), crate::elm::form::STR)));
                            }
                            return Ok(Some((
                                PropValue::Str(String::new()),
                                crate::elm::form::STR,
                            )));
                        }
                        _ => {}
                    }
                }
                if form == crate::elm::form::INTLIST || form == crate::elm::form::STRLIST {
                    return Ok(Some((PropValue::Element(element.to_vec()), form)));
                }
                if matches!(
                    form,
                    crate::elm::form::INTREF
                        | crate::elm::form::STRREF
                        | crate::elm::form::INTLISTREF
                        | crate::elm::form::STRLISTREF
                ) {
                    let ref_target = match &self.user_prop_values[up_idx] {
                        PropValue::Element(el) => el.clone(),
                        _ => element.to_vec(),
                    };
                    if element.len() > 1 {
                        return self.try_property_via_ref_target(
                            &ref_target,
                            &element[1..],
                            host,
                            "CD_PROPERTY user_prop ref: invalid tail route",
                        );
                    }
                    return Ok(Some((PropValue::Element(ref_target), form)));
                }
                return Ok(Some((self.user_prop_values[up_idx].clone(), form)));
            }
            // Unknown: default int
            return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
        }
        // cur_call.L / cur_call.K whole-list read: return element reference so CD_ASSIGN can consume list source.
        if element.len() >= 2 && crate::elm::call::is_cur_call(element[0]) && element.len() == 2 {
            if crate::elm::call::is_call_l(element[1]) {
                return Ok(Some((
                    PropValue::Element(vec![element[0], element[1]]),
                    crate::elm::form::INTLIST,
                )));
            }
            if crate::elm::call::is_call_k(element[1]) {
                return Ok(Some((
                    PropValue::Element(vec![element[0], element[1]]),
                    crate::elm::form::STRLIST,
                )));
            }
        }

        // cur_call.L[idx]
        if element.len() >= 4
            && crate::elm::call::is_cur_call(element[0])
            && crate::elm::call::is_call_l(element[1])
            && element[2] == crate::elm::ELM_ARRAY
        {
            let idx = element[3] as isize;
            let Some(frame) = self.frames.last() else {
                host.on_error_fatal("CD_PROPERTY cur_call.L[idx]: no current frame");
                return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
            };
            if idx >= 0 {
                let idx = idx as usize;
                let call = &frame.call;
                if idx < call.l.len() {
                    return Ok(Some((PropValue::Int(call.l[idx]), crate::elm::form::INT)));
                }
            }
            return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
        }

        // cur_call.K[idx]
        if element.len() >= 4
            && crate::elm::call::is_cur_call(element[0])
            && crate::elm::call::is_call_k(element[1])
            && element[2] == crate::elm::ELM_ARRAY
        {
            let idx = element[3] as isize;
            let Some(frame) = self.frames.last() else {
                host.on_error_fatal("CD_PROPERTY cur_call.K[idx]: no current frame");
                return Ok(Some((PropValue::Str(String::new()), crate::elm::form::STR)));
            };
            if idx >= 0 {
                let idx = idx as usize;
                let call = &frame.call;
                if idx < call.k.len() {
                    return Ok(Some((
                        PropValue::Str(call.k[idx].clone()),
                        crate::elm::form::STR,
                    )));
                }
            }
            return Ok(Some((PropValue::Str(String::new()), crate::elm::form::STR)));
        }

        // cur_call.<call_prop>
        if element.len() >= 2 && crate::elm::call::is_cur_call(element[0]) {
            let head = element[1];
            if crate::elm::owner::is_call_prop(head) {
                let idx = elm_code(head) as usize;
                let Some(frame) = self.frames.last() else {
                    host.on_error_fatal("CD_PROPERTY cur_call.<prop>: no current frame");
                    return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
                };
                let call = &frame.call;
                if let Some(slot) = Self::resolve_call_prop_slot(call, idx) {
                    let p = &call.user_props[slot];
                    if p.form == crate::elm::form::INT {
                        if let PropValue::Int(v) = &p.value {
                            return Ok(Some((PropValue::Int(*v), crate::elm::form::INT)));
                        }
                        return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
                    } else if p.form == crate::elm::form::STR {
                        if let PropValue::Str(s) = &p.value {
                            return Ok(Some((PropValue::Str(s.clone()), crate::elm::form::STR)));
                        }
                        return Ok(Some((PropValue::Str(String::new()), crate::elm::form::STR)));
                    } else if p.form == crate::elm::form::INTLIST
                        || p.form == crate::elm::form::STRLIST
                    {
                        return Ok(Some((PropValue::Element(vec![element[0], head]), p.form)));
                    } else if matches!(
                        p.form,
                        crate::elm::form::INTREF
                            | crate::elm::form::STRREF
                            | crate::elm::form::INTLISTREF
                            | crate::elm::form::STRLISTREF
                    ) {
                        let ref_target = match &p.value {
                            PropValue::Element(el) => el.clone(),
                            _ => vec![element[0], head],
                        };
                        if element.len() > 2 {
                            return self.try_property_via_ref_target(
                                &ref_target,
                                &element[2..],
                                host,
                                "CD_PROPERTY cur_call.<prop> ref: invalid tail route",
                            );
                        }
                        return Ok(Some((PropValue::Element(ref_target), p.form)));
                    }
                }
                host.on_error_fatal("CD_PROPERTY cur_call.<prop>: prop not found");
                return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
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
                    self.report_vm_fatal_with_context(
                        host,
                        "CD_PROPERTY: cur_call.<prop>[idx] missing index operand",
                    );
                    bail!("CD_PROPERTY: cur_call.<prop>[idx] missing index operand");
                } as isize;
                let Some(frame) = self.frames.last() else {
                    host.on_error_fatal("CD_PROPERTY cur_call.<prop>[idx]: no current frame");
                    return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
                };
                let call = &frame.call;
                if let Some(slot) = Self::resolve_call_prop_slot(call, cp_idx) {
                    match &call.user_props[slot].value {
                        PropValue::IntList(v) => {
                            if idx >= 0 {
                                let out = v.get(idx as usize).copied().unwrap_or(0);
                                return Ok(Some((PropValue::Int(out), crate::elm::form::INT)));
                            }
                            return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
                        }
                        PropValue::StrList(v) => {
                            if idx >= 0 {
                                let out = v.get(idx as usize).cloned().unwrap_or_default();
                                return Ok(Some((PropValue::Str(out), crate::elm::form::STR)));
                            }
                            return Ok(Some((
                                PropValue::Str(String::new()),
                                crate::elm::form::STR,
                            )));
                        }
                        _ => {}
                    }
                }
                host.on_error_fatal("CD_PROPERTY cur_call.<prop>[idx]: prop not found");
                return Ok(Some((PropValue::Int(0), crate::elm::form::INT)));
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
                    return Ok(Some((
                        PropValue::Str(self.global_namae[idx as usize].clone()),
                        crate::elm::form::STR,
                    )));
                }
                return Ok(Some((PropValue::Str(String::new()), crate::elm::form::STR)));
            }
        }

        Ok(None)
    }
}
