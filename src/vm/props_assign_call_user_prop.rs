impl Vm {

    fn validate_assign_ref_tail_shape(
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
                    host.on_error_fatal(&format!("{}: array -> up nested tail is invalid", err_tag));
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
    fn try_assign_via_ref_target(
        &mut self,
        ref_element: &[i32],
        tail: &[i32],
        rhs: &Prop,
        host: &mut dyn Host,
        err_tag: &str,
    ) -> Result<bool> {
        if tail.is_empty() {
            return Ok(false);
        }
        if !self.validate_assign_ref_tail_shape(tail, host, err_tag) {
            return Ok(true);
        }
        let mut target = self.resolve_command_element_alias(ref_element);
        if target.is_empty() {
            host.on_error_fatal(err_tag);
            return Ok(true);
        }
        target.extend_from_slice(tail);
        if target == ref_element {
            host.on_error_fatal(err_tag);
            return Ok(true);
        }
        match self.try_assign_internal(&target, 1, rhs, host) {
            Ok(v) => {
                if !v {
                    host.on_error_fatal(err_tag);
                }
                Ok(true)
            }
            Err(e) => {
                host.on_error_fatal(&format!("{}: {}", err_tag, e));
                Ok(true)
            }
        }
    }

    fn try_assign_user_or_call_props(
        &mut self,
        element: &[i32],
        rhs: &Prop,
        host: &mut dyn Host,
    ) -> Result<bool> {
        // scene user-prop assign: <user_prop>[idx] / <user_prop>
        if !element.is_empty() && crate::elm::owner::is_user_prop(element[0]) {
            let up_idx = elm_code(element[0]) as usize;
            if up_idx >= self.user_prop_forms.len() || up_idx >= self.user_prop_values.len() {
                host.on_error_fatal("CD_ASSIGN user_prop: index out of range");
                return Ok(true);
            }

            let form = self.user_prop_forms[up_idx];
            if element.len() >= 3 && element[1] == crate::elm::ELM_ARRAY {
                let idx = element[2] as isize;
                if idx < 0 {
                    host.on_error_fatal("CD_ASSIGN user_prop[idx]: negative index");
                    return Ok(true);
                }
                let idx = idx as usize;
                match (&mut self.user_prop_values[up_idx], &rhs.value) {
                    (PropValue::IntList(v), PropValue::Int(x)) => {
                        if idx >= v.len() {
                            host.on_error_fatal("CD_ASSIGN user_prop[idx]: index out of range");
                            return Ok(true);
                        }
                        v[idx] = *x;
                        return Ok(true);
                    }
                    (PropValue::StrList(v), PropValue::Str(x)) => {
                        if idx >= v.len() {
                            host.on_error_fatal("CD_ASSIGN user_prop[idx]: index out of range");
                            return Ok(true);
                        }
                        v[idx] = x.clone();
                        return Ok(true);
                    }
                    (PropValue::IntList(_), _) => {
                        host.on_error_fatal("CD_ASSIGN user_prop[idx]: rhs type mismatch INTLIST");
                        return Ok(true);
                    }
                    (PropValue::StrList(_), _) => {
                        host.on_error_fatal("CD_ASSIGN user_prop[idx]: rhs type mismatch STRLIST");
                        return Ok(true);
                    }
                    _ => {
                        host.on_error_fatal("CD_ASSIGN user_prop[idx]: target is not list prop");
                        return Ok(true);
                    }
                }
            }

            // direct set
            let next = match form {
                f if f == crate::elm::form::INT => self
                    .resolve_assign_int_rhs(rhs, host)
                    .map(PropValue::Int)
                    .map_err(|e| anyhow::anyhow!("CD_ASSIGN user_prop: {}", e)),
                f if f == crate::elm::form::STR => self
                    .resolve_assign_str_rhs(rhs, host)
                    .map(PropValue::Str)
                    .map_err(|e| anyhow::anyhow!("CD_ASSIGN user_prop: {}", e)),
                f if f == crate::elm::form::INTLIST => self
                    .resolve_assign_intlist_rhs(rhs)
                    .map(PropValue::IntList)
                    .map_err(|e| {
                        if matches!(rhs.value, PropValue::IntList(_) | PropValue::Element(_)) {
                            e
                        } else {
                            anyhow::anyhow!("CD_ASSIGN user_prop intlist: rhs type mismatch")
                        }
                    }),
                f if f == crate::elm::form::STRLIST => self
                    .resolve_assign_strlist_rhs(rhs)
                    .map(PropValue::StrList)
                    .map_err(|e| {
                        if matches!(rhs.value, PropValue::StrList(_) | PropValue::Element(_)) {
                            e
                        } else {
                            anyhow::anyhow!("CD_ASSIGN user_prop strlist: rhs type mismatch")
                        }
                    }),
                f if matches!(
                    f,
                    crate::elm::form::INTREF
                        | crate::elm::form::STRREF
                        | crate::elm::form::INTLISTREF
                        | crate::elm::form::STRLISTREF
                ) =>
                {
                    let ref_target = match &self.user_prop_values[up_idx] {
                        PropValue::Element(el) => el.clone(),
                        _ => vec![element[0]],
                    };
                    if element.len() > 1 {
                        let _ = self.try_assign_via_ref_target(
                            &ref_target,
                            &element[1..],
                            rhs,
                            host,
                            "CD_ASSIGN user_prop ref: invalid tail route",
                        )?;
                        return Ok(true);
                    }
                    match &rhs.value {
                        PropValue::Element(el) => Ok(PropValue::Element(el.clone())),
                        _ => Err(anyhow::anyhow!(
                            "CD_ASSIGN user_prop ref: rhs type mismatch ELEMENT"
                        )),
                    }
                }
                _ => Ok(rhs.value.clone()),
            };

            match next {
                Ok(v) => {
                    self.user_prop_values[up_idx] = v;
                }
                Err(e) => {
                    host.on_error_fatal(&format!("{}", e));
                }
            }
            return Ok(true);
        }

        // cur_call.L = <intlist/element>
        if element.len() >= 2
            && crate::elm::call::is_cur_call(element[0])
            && crate::elm::call::is_call_l(element[1])
            && (element.len() == 2 || element[2] != crate::elm::ELM_ARRAY)
        {
            let src = match self.resolve_assign_intlist_rhs(rhs) {
                Ok(v) => v,
                Err(e) => {
                    host.on_error_fatal(&format!("{}", e));
                    return Ok(true);
                }
            };
            if let Some(frame) = self.frames.last_mut() {
                let dst = &mut frame.call.l;
                for (i, v) in src.iter().copied().enumerate() {
                    if i < dst.len() {
                        dst[i] = v;
                    }
                }
                if src.len() < dst.len() {
                    for v in &mut dst[src.len()..] {
                        *v = 0;
                    }
                }
                return Ok(true);
            }
            host.on_error_fatal("CD_ASSIGN call.L: no current frame");
            return Ok(true);
        }

        // cur_call.K = <strlist/element>
        if element.len() >= 2
            && crate::elm::call::is_cur_call(element[0])
            && crate::elm::call::is_call_k(element[1])
            && (element.len() == 2 || element[2] != crate::elm::ELM_ARRAY)
        {
            let src = match self.resolve_assign_strlist_rhs(rhs) {
                Ok(v) => v,
                Err(e) => {
                    host.on_error_fatal(&format!("{}", e));
                    return Ok(true);
                }
            };
            if let Some(frame) = self.frames.last_mut() {
                let dst = &mut frame.call.k;
                for (i, v) in src.iter().enumerate() {
                    if i < dst.len() {
                        dst[i] = v.clone();
                    }
                }
                if src.len() < dst.len() {
                    for v in &mut dst[src.len()..] {
                        *v = String::new();
                    }
                }
                return Ok(true);
            }
            host.on_error_fatal("CD_ASSIGN call.K: no current frame");
            return Ok(true);
        }

        // cur_call.L[idx] = int
        if element.len() >= 4
            && crate::elm::call::is_cur_call(element[0])
            && crate::elm::call::is_call_l(element[1])
            && element[2] == crate::elm::ELM_ARRAY
        {
            let idx = element[3] as isize;
            if idx < 0 {
                host.on_error_fatal("CD_ASSIGN call.L[idx]: negative index");
                return Ok(true);
            }
            let idx = idx as usize;
            let v = match &rhs.value {
                PropValue::Int(x) => *x,
                _ => {
                    host.on_error_fatal("CD_ASSIGN call.L[idx]: rhs type mismatch");
                    return Ok(true);
                }
            };
            if let Some(frame) = self.frames.last_mut() {
                if idx < frame.call.l.len() {
                    frame.call.l[idx] = v;
                    return Ok(true);
                }
                host.on_error_fatal("CD_ASSIGN call.L[idx]: index out of range");
                return Ok(true);
            }
            host.on_error_fatal("CD_ASSIGN call.L[idx]: no current frame");
            return Ok(true);
        }

        // cur_call.K[idx] = str
        if element.len() >= 4
            && crate::elm::call::is_cur_call(element[0])
            && crate::elm::call::is_call_k(element[1])
            && element[2] == crate::elm::ELM_ARRAY
        {
            let idx = element[3] as isize;
            if idx < 0 {
                host.on_error_fatal("CD_ASSIGN call.K[idx]: negative index");
                return Ok(true);
            }
            let idx = idx as usize;
            let v = match &rhs.value {
                PropValue::Str(s) => s.clone(),
                _ => {
                    host.on_error_fatal("CD_ASSIGN call.K[idx]: rhs type mismatch");
                    return Ok(true);
                }
            };
            if let Some(frame) = self.frames.last_mut() {
                if idx < frame.call.k.len() {
                    frame.call.k[idx] = v;
                    return Ok(true);
                }
                host.on_error_fatal("CD_ASSIGN call.K[idx]: index out of range");
                return Ok(true);
            }
            host.on_error_fatal("CD_ASSIGN call.K[idx]: no current frame");
            return Ok(true);
        }

        // cur_call.<call_prop> = (int/str/intlist/strlist)
        if element.len() >= 2 && crate::elm::call::is_cur_call(element[0]) {
            let head = element[1];
            if crate::elm::owner::is_call_prop(head) {
                let idx = elm_code(head) as usize;
                let (slot, form) = if let Some(frame) = self.frames.last() {
                    if let Some(slot) = Self::resolve_call_prop_slot(&frame.call, idx) {
                        (slot, frame.call.user_props[slot].form)
                    } else {
                        host.on_error_fatal("CD_ASSIGN cur_call.<prop>: prop not found");
                        return Ok(true);
                    }
                } else {
                    host.on_error_fatal("CD_ASSIGN cur_call.<prop>: no current frame");
                    return Ok(true);
                };

                let value = if form == crate::elm::form::INT {
                    let v = self
                        .resolve_assign_int_rhs(rhs, host)
                        .map_err(|e| anyhow::anyhow!("CD_ASSIGN cur_call.<prop>: {}", e))?;
                    PropValue::Int(v)
                } else if form == crate::elm::form::STR {
                    let v = self
                        .resolve_assign_str_rhs(rhs, host)
                        .map_err(|e| anyhow::anyhow!("CD_ASSIGN cur_call.<prop>: {}", e))?;
                    PropValue::Str(v)
                } else if form == crate::elm::form::INTLIST {
                    let v = self.resolve_assign_intlist_rhs(rhs).map_err(|e| {
                        if matches!(rhs.value, PropValue::IntList(_) | PropValue::Element(_)) {
                            e
                        } else {
                            anyhow::anyhow!("CD_ASSIGN cur_call.<prop>: rhs type mismatch INTLIST")
                        }
                    })?;
                    PropValue::IntList(v)
                } else if form == crate::elm::form::STRLIST {
                    let v = self.resolve_assign_strlist_rhs(rhs).map_err(|e| {
                        if matches!(rhs.value, PropValue::StrList(_) | PropValue::Element(_)) {
                            e
                        } else {
                            anyhow::anyhow!("CD_ASSIGN cur_call.<prop>: rhs type mismatch STRLIST")
                        }
                    })?;
                    PropValue::StrList(v)
                } else if matches!(
                    form,
                    crate::elm::form::INTREF
                        | crate::elm::form::STRREF
                        | crate::elm::form::INTLISTREF
                        | crate::elm::form::STRLISTREF
                ) {
                    let ref_target = if let Some(frame) = self.frames.last() {
                        match &frame.call.user_props[slot].value {
                            PropValue::Element(el) => el.clone(),
                            _ => vec![element[0], head],
                        }
                    } else {
                        vec![element[0], head]
                    };
                    if element.len() > 2 {
                        let _ = self.try_assign_via_ref_target(
                            &ref_target,
                            &element[2..],
                            rhs,
                            host,
                            "CD_ASSIGN cur_call.<prop> ref: invalid tail route",
                        )?;
                        return Ok(true);
                    }
                    match &rhs.value {
                        PropValue::Element(el) => PropValue::Element(el.clone()),
                        _ => {
                            host.on_error_fatal(
                                "CD_ASSIGN cur_call.<prop>: rhs type mismatch ELEMENT",
                            );
                            return Ok(true);
                        }
                    }
                } else {
                    host.on_error_fatal(&format!(
                        "CD_ASSIGN cur_call.<prop>: unsupported form {}",
                        form
                    ));
                    return Ok(true);
                };

                if let Some(frame) = self.frames.last_mut() {
                    frame.call.user_props[slot].value = value;
                    return Ok(true);
                }
                host.on_error_fatal("CD_ASSIGN cur_call.<prop>: no current frame");
                return Ok(true);
            }
        }

        // cur_call.<call_prop>[idx] = (int/str) (best-effort for local list props)
        if element.len() >= 4 && crate::elm::call::is_cur_call(element[0]) {
            let head = element[1];
            if crate::elm::owner::is_call_prop(head) && element[2] == crate::elm::ELM_ARRAY {
                let cp_idx = elm_code(head) as usize;
                let idx = element[3] as isize;
                if idx < 0 {
                    host.on_error_fatal("CD_ASSIGN cur_call.<prop>[idx]: negative index");
                    return Ok(true);
                }
                let idx = idx as usize;
                if let Some(frame) = self.frames.last_mut() {
                    if let Some(slot) = Self::resolve_call_prop_slot(&frame.call, cp_idx) {
                        match &mut frame.call.user_props[slot].value {
                            PropValue::IntList(v) => {
                                let vv = match &rhs.value {
                                    PropValue::Int(x) => *x,
                                    _ => {
                                        host.on_error_fatal(
                                            "CD_ASSIGN cur_call.<prop>[idx]: rhs type mismatch INT",
                                        );
                                        return Ok(true);
                                    }
                                };
                                if idx < v.len() {
                                    v[idx] = vv;
                                    return Ok(true);
                                }
                                host.on_error_fatal(
                                    "CD_ASSIGN cur_call.<prop>[idx]: index out of range",
                                );
                                return Ok(true);
                            }
                            PropValue::StrList(v) => {
                                let vv = match &rhs.value {
                                    PropValue::Str(s) => s.clone(),
                                    _ => {
                                        host.on_error_fatal(
                                            "CD_ASSIGN cur_call.<prop>[idx]: rhs type mismatch STR",
                                        );
                                        return Ok(true);
                                    }
                                };
                                if idx < v.len() {
                                    v[idx] = vv;
                                    return Ok(true);
                                }
                                host.on_error_fatal(
                                    "CD_ASSIGN cur_call.<prop>[idx]: index out of range",
                                );
                                return Ok(true);
                            }
                            _ => {}
                        }
                    }
                }
                if self.frames.last().is_none() {
                    host.on_error_fatal("CD_ASSIGN cur_call.<prop>[idx]: no current frame");
                    return Ok(true);
                }
                host.on_error_fatal(
                    "CD_ASSIGN cur_call.<prop>[idx]: prop not found or invalid list target",
                );
                return Ok(true);
            }
        }

        Ok(false)
    }
}
