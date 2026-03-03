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

    fn stage_index_from_property_root(root: i32) -> Option<i32> {
        if crate::elm::global::is_stage(root) {
            return None;
        }
        if crate::elm::global::is_back(root) {
            return Some(0);
        }
        if crate::elm::global::is_front(root) {
            return Some(1);
        }
        if crate::elm::global::is_next(root) {
            return Some(2);
        }
        None
    }

    fn group_query_is_int(sub: i32) -> bool {
        use crate::elm::group::*;
        matches!(
            sub,
            ELM_GROUP_GET_HIT_NO
                | ELM_GROUP_GET_PUSHED_NO
                | ELM_GROUP_GET_DECIDED_NO
                | ELM_GROUP_GET_RESULT
                | ELM_GROUP_GET_RESULT_BUTTON_NO
                | ELM_GROUP_ORDER
                | ELM_GROUP_LAYER
                | ELM_GROUP_CANCEL_PRIORITY
        )
    }

    pub(super) fn object_query_is_str(sub: i32) -> bool {
        use crate::elm::objectlist::*;
        matches!(
            sub,
            ELM_OBJECT_GET_FILE_NAME | ELM_OBJECT_GET_STRING | ELM_OBJECT_GET_ELEMENT_NAME
        )
    }

    pub(super) fn object_query_is_int(sub: i32) -> bool {
        Self::object_property_is_settable_int(sub) || Self::object_query_is_readonly_int(sub)
    }

    fn object_query_is_readonly_int(sub: i32) -> bool {
        use crate::elm::objectlist::*;
        matches!(
            sub,
            ELM_OBJECT_GET_PAT_CNT
                | ELM_OBJECT_GET_SIZE_X
                | ELM_OBJECT_GET_SIZE_Y
                | ELM_OBJECT_GET_SIZE_Z
                | ELM_OBJECT_GET_PIXEL_COLOR_R
                | ELM_OBJECT_GET_PIXEL_COLOR_G
                | ELM_OBJECT_GET_PIXEL_COLOR_B
                | ELM_OBJECT_GET_PIXEL_COLOR_A
                | ELM_OBJECT_EXIST_TYPE
                | ELM_OBJECT_GET_TYPE
                | ELM_OBJECT_GET_NUMBER
                | ELM_OBJECT_GET_MOVIE_SEEK_TIME
                | ELM_OBJECT_FRAME_ACTION
                | ELM_OBJECT_FRAME_ACTION_CH
                | ELM_OBJECT_X_REP
                | ELM_OBJECT_Y_REP
                | ELM_OBJECT_Z_REP
                | ELM_OBJECT_TR_REP
                | ELM_OBJECT_CHECK_MOVIE
                | ELM_OBJECT_GET_BUTTON_STATE
                | ELM_OBJECT_GET_BUTTON_HIT_STATE
                | ELM_OBJECT_GET_BUTTON_REAL_STATE
                | ELM_OBJECT_GET_BUTTON_PUSHKEEP
                | ELM_OBJECT_GET_BUTTON_ALPHA_TEST
                | ELM_OBJECT_GET_BUTTON_NO
                | ELM_OBJECT_GET_BUTTON_GROUP_NO
                | ELM_OBJECT_GET_BUTTON_ACTION_NO
                | ELM_OBJECT_GET_BUTTON_SE_NO
                | ELM_OBJECT_EMOTE_CHECK_PLAYING
                | ELM_OBJECT__IAPP_DUMMY
        )
    }

    fn object_query_is_frame_action_sub(sub: i32) -> bool {
        use crate::elm::objectlist::*;
        matches!(sub, ELM_OBJECT_FRAME_ACTION | ELM_OBJECT_FRAME_ACTION_CH)
    }

    fn try_property_object_all_eve(
        list_id: i32,
        obj_idx: i32,
        tail: &[i32],
        stage_idx: i32,
        host: &mut dyn Host,
    ) -> Option<(PropValue, i32)> {
        use crate::elm::allevent::{ELM_ALLEVENT_CHECK, ELM_ALLEVENT_END, ELM_ALLEVENT_WAIT};

        if tail.is_empty() {
            host.on_error_fatal("CD_PROPERTY stage.object.all_eve: missing method");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail[0] == crate::elm::ELM_UP {
            host.on_error_fatal("CD_PROPERTY stage.object.all_eve: ELM_UP is not supported");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail[0] == crate::elm::ELM_ARRAY {
            host.on_error_fatal("CD_PROPERTY stage.object.all_eve: array access is invalid");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail.len() > 1 {
            host.on_error_fatal("CD_PROPERTY stage.object.all_eve: unexpected nested tail");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }

        match tail[0] {
            ELM_ALLEVENT_CHECK => Some((
                PropValue::Int(host.on_object_get(
                    list_id,
                    obj_idx,
                    crate::elm::objectlist::ELM_OBJECT_ALL_EVE,
                    Some(stage_idx),
                )),
                crate::elm::form::INT,
            )),
            ELM_ALLEVENT_END | ELM_ALLEVENT_WAIT => {
                host.on_error_fatal(
                    "CD_PROPERTY stage.object.all_eve: mutating method requires command lane",
                );
                Some((PropValue::Int(0), crate::elm::form::INT))
            }
            _ => {
                host.on_error_fatal("無効なコマンドが指定されました。(allevent)");
                Some((PropValue::Int(0), crate::elm::form::INT))
            }
        }
    }

    fn object_composite_is_int_event_list_sub(sub: i32) -> bool {
        use crate::elm::objectlist::*;
        matches!(
            sub,
            ELM_OBJECT_X_REP_EVE
                | ELM_OBJECT_Y_REP_EVE
                | ELM_OBJECT_Z_REP_EVE
                | ELM_OBJECT_TR_REP_EVE
        )
    }

    fn object_composite_is_int_event_sub(sub: i32) -> bool {
        use crate::elm::objectlist::*;
        matches!(
            sub,
            ELM_OBJECT_X_EVE
                | ELM_OBJECT_Y_EVE
                | ELM_OBJECT_Z_EVE
                | ELM_OBJECT_SCALE_Z_EVE
                | ELM_OBJECT_SCALE_X_EVE
                | ELM_OBJECT_SCALE_Y_EVE
                | ELM_OBJECT_ROTATE_X_EVE
                | ELM_OBJECT_ROTATE_Y_EVE
                | ELM_OBJECT_ROTATE_Z_EVE
                | ELM_OBJECT_TR_EVE
                | ELM_OBJECT_MONO_EVE
                | ELM_OBJECT_REVERSE_EVE
                | ELM_OBJECT_BRIGHT_EVE
                | ELM_OBJECT_DARK_EVE
                | ELM_OBJECT_CENTER_X_EVE
                | ELM_OBJECT_CENTER_Y_EVE
                | ELM_OBJECT_CENTER_Z_EVE
                | ELM_OBJECT_CENTER_REP_X_EVE
                | ELM_OBJECT_CENTER_REP_Y_EVE
                | ELM_OBJECT_CENTER_REP_Z_EVE
                | ELM_OBJECT_COLOR_RATE_EVE
                | ELM_OBJECT_COLOR_ADD_R_EVE
                | ELM_OBJECT_COLOR_ADD_G_EVE
                | ELM_OBJECT_COLOR_ADD_B_EVE
                | ELM_OBJECT_COLOR_R_EVE
                | ELM_OBJECT_COLOR_G_EVE
                | ELM_OBJECT_COLOR_B_EVE
                | ELM_OBJECT_PATNO_EVE
                | ELM_OBJECT_CLIP_LEFT_EVE
                | ELM_OBJECT_CLIP_TOP_EVE
                | ELM_OBJECT_CLIP_RIGHT_EVE
                | ELM_OBJECT_CLIP_BOTTOM_EVE
                | ELM_OBJECT_SRC_CLIP_LEFT_EVE
                | ELM_OBJECT_SRC_CLIP_TOP_EVE
                | ELM_OBJECT_SRC_CLIP_RIGHT_EVE
                | ELM_OBJECT_SRC_CLIP_BOTTOM_EVE
        )
    }

    fn int_event_method_is_query(method: i32) -> bool {
        use crate::elm::intevent::{ELM_INTEVENT_CHECK, ELM_INTEVENT_GET_EVENT_VALUE};
        method == ELM_INTEVENT_CHECK || method == ELM_INTEVENT_GET_EVENT_VALUE
    }

    fn int_event_method_is_known(method: i32) -> bool {
        use crate::elm::intevent::*;
        matches!(
            method,
            ELM_INTEVENT_SET
                | ELM_INTEVENT_LOOP
                | ELM_INTEVENT_TURN
                | ELM_INTEVENT_END
                | ELM_INTEVENT_WAIT
                | ELM_INTEVENT_CHECK
                | ELM_INTEVENT__SET
                | ELM_INTEVENT_SET_REAL
                | ELM_INTEVENT_LOOP_REAL
                | ELM_INTEVENT_TURN_REAL
                | ELM_INTEVENT_WAIT_KEY
                | ELM_INTEVENT_YURE
                | ELM_INTEVENT_YURE_REAL
                | ELM_INTEVENT_GET_EVENT_VALUE
        )
    }

    fn child_property_requires_runtime_args(sub: i32) -> bool {
        use crate::elm::objectlist::*;
        matches!(
            sub,
            ELM_OBJECT_GET_SIZE_X
                | ELM_OBJECT_GET_SIZE_Y
                | ELM_OBJECT_GET_SIZE_Z
                | ELM_OBJECT_GET_PIXEL_COLOR_R
                | ELM_OBJECT_GET_PIXEL_COLOR_G
                | ELM_OBJECT_GET_PIXEL_COLOR_B
                | ELM_OBJECT_GET_PIXEL_COLOR_A
        )
    }

    fn composite_int_event_owner_id(sub: i32, list_index: Option<i32>) -> i32 {
        // Keep owner-id stable for host hooks, and encode list index for *_rep_eve paths.
        match list_index {
            Some(idx) => ((sub & 0xFFFF) << 16) | (idx & 0xFFFF),
            None => sub,
        }
    }

    fn try_property_object_child_composite(
        list_id: i32,
        obj_idx: i32,
        tail: &[i32],
        stage_idx: i32,
        disp_out_of_range_error: bool,
        host: &mut dyn Host,
    ) -> Option<(PropValue, i32)> {
        use crate::elm::objectlist::ELM_OBJECTLIST_GET_SIZE;

        if tail.is_empty() {
            host.on_error_fatal("CD_PROPERTY stage.object.child: missing method");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail[0] == crate::elm::ELM_UP {
            host.on_error_fatal("CD_PROPERTY stage.object.child: ELM_UP is not supported");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail[0] == ELM_OBJECTLIST_GET_SIZE {
            if tail.len() > 1 {
                host.on_error_fatal(
                    "CD_PROPERTY stage.object.child.get_size: unexpected nested tail",
                );
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            let size = host.on_object_child_list_get_size(list_id, obj_idx, Some(stage_idx));
            return Some((PropValue::Int(size.max(0)), crate::elm::form::INT));
        }
        if tail[0] == crate::elm::ELM_ARRAY {
            if tail.len() < 2 {
                host.on_error_fatal("CD_PROPERTY stage.object.child: missing array index");
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            let idx = tail[1];
            let size = host.on_object_child_list_get_size(list_id, obj_idx, Some(stage_idx));
            if idx < 0 || (size >= 0 && idx >= size) {
                if disp_out_of_range_error {
                    host.on_error_fatal(
                        "範囲外のオブジェクト番号が指定されました。(object_list child)",
                    );
                }
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if !host.on_object_child_is_use(list_id, obj_idx, idx, Some(stage_idx)) {
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if tail.len() == 2 {
                return Some((PropValue::Int(1), crate::elm::form::INT));
            }

            let sub = tail[2];
            if tail.len() > 3 {
                host.on_error_fatal("CD_PROPERTY stage.object.child[idx]: unexpected nested tail");
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if Self::object_query_is_str(sub) {
                let v = host.on_object_child_get_str(list_id, obj_idx, idx, sub, Some(stage_idx));
                return Some((PropValue::Str(v), crate::elm::form::STR));
            }
            if Self::object_query_is_int(sub) {
                if Self::child_property_requires_runtime_args(sub) {
                    host.on_error_fatal(
                        "CD_PROPERTY stage.object.child[idx]: property requires runtime args",
                    );
                    return Some((PropValue::Int(0), crate::elm::form::INT));
                }
                let v = if Self::object_query_uses_query_api(sub) {
                    host.on_object_child_query(list_id, obj_idx, idx, sub, &[], Some(stage_idx))
                } else {
                    host.on_object_child_get(list_id, obj_idx, idx, sub, Some(stage_idx))
                };
                return Some((PropValue::Int(v), crate::elm::form::INT));
            }

            host.on_error_fatal("CD_PROPERTY stage.object.child[idx]: command-only route");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }

        host.on_error_fatal("CD_PROPERTY stage.object.child: command-only route");
        Some((PropValue::Int(0), crate::elm::form::INT))
    }

    fn try_property_object_int_event_composite(
        sub: i32,
        tail: &[i32],
        disp_out_of_range_error: bool,
        proc_depth: i32,
        proc_top: i32,
        host: &mut dyn Host,
    ) -> Option<(PropValue, i32)> {
        if tail.is_empty() {
            host.on_error_fatal("CD_PROPERTY stage.object.*_eve: missing method");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail[0] == crate::elm::ELM_UP {
            host.on_error_fatal("CD_PROPERTY stage.object.*_eve: ELM_UP is not supported");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }

        let (method, list_idx) = if Self::object_composite_is_int_event_list_sub(sub) {
            if tail[0] != crate::elm::ELM_ARRAY {
                host.on_error_fatal("CD_PROPERTY stage.object.*_rep_eve: array index required");
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if tail.len() < 2 {
                host.on_error_fatal("CD_PROPERTY stage.object.*_rep_eve: missing array index");
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if tail[1] < 0 && disp_out_of_range_error {
                host.on_error_fatal("範囲外のイベント番号が指定されました。(int_event_list)");
            }
            if tail.len() < 3 {
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if tail[2] == crate::elm::ELM_ARRAY || tail[2] == crate::elm::ELM_UP {
                host.on_error_fatal("CD_PROPERTY stage.object.*_rep_eve[idx]: invalid nested tail");
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if tail.len() > 3 {
                host.on_error_fatal(
                    "CD_PROPERTY stage.object.*_rep_eve[idx]: unexpected nested tail",
                );
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            (tail[2], Some(tail[1]))
        } else {
            if tail[0] == crate::elm::ELM_ARRAY {
                host.on_error_fatal("CD_PROPERTY stage.object.*_eve: array access is invalid");
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if tail.len() > 1 {
                host.on_error_fatal("CD_PROPERTY stage.object.*_eve: unexpected nested tail");
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            (tail[0], None)
        };

        let owner_id = Self::composite_int_event_owner_id(sub, list_idx);
        if Self::int_event_method_is_query(method) {
            let v = if method == crate::elm::intevent::ELM_INTEVENT_CHECK {
                if host.on_int_event_check(owner_id) {
                    1
                } else {
                    0
                }
            } else {
                host.on_int_event_get_value(owner_id)
            };
            return Some((PropValue::Int(v), crate::elm::form::INT));
        }
        if method == crate::elm::intevent::ELM_INTEVENT_WAIT
            || method == crate::elm::intevent::ELM_INTEVENT_WAIT_KEY
        {
            // flow_proc.cpp alignment: WAIT is proc-driven, so property lane exposes
            // a poll-style observable state instead of mutating command completion.
            // Consumer example: script can poll until `EVE_WAIT_DONE` / `EVE_WAIT_KEY_SKIPPED`
            // and keep stepping while `EVE_WAIT_PENDING` is returned.
            let key_skip = method == crate::elm::intevent::ELM_INTEVENT_WAIT_KEY;
            let before_wait = host.on_int_event_check(owner_id);
            if !before_wait {
                host.on_int_event_wait_status(owner_id, key_skip, crate::vm::EVE_WAIT_DONE);
                host.on_int_event_wait_status_with_proc(
                    owner_id,
                    key_skip,
                    crate::vm::EVE_WAIT_DONE,
                    proc_depth,
                    proc_top,
                );
                return Some((
                    PropValue::Int(crate::vm::EVE_WAIT_DONE),
                    crate::elm::form::INT,
                ));
            }
            if key_skip && host.should_skip_wait() {
                host.on_int_event_wait_status(owner_id, key_skip, crate::vm::EVE_WAIT_KEY_SKIPPED);
                host.on_int_event_wait_status_with_proc(
                    owner_id,
                    key_skip,
                    crate::vm::EVE_WAIT_KEY_SKIPPED,
                    proc_depth,
                    proc_top,
                );
                return Some((
                    PropValue::Int(crate::vm::EVE_WAIT_KEY_SKIPPED),
                    crate::elm::form::INT,
                ));
            }
            host.on_int_event_wait(owner_id, key_skip);
            host.on_wait_frame();
            let still_waiting = host.on_int_event_check(owner_id);
            let status = if still_waiting {
                crate::vm::EVE_WAIT_PENDING
            } else {
                crate::vm::EVE_WAIT_DONE
            };
            host.on_int_event_wait_status(owner_id, key_skip, status);
            host.on_int_event_wait_status_with_proc(
                owner_id, key_skip, status, proc_depth, proc_top,
            );
            return Some((PropValue::Int(status), crate::elm::form::INT));
        }
        if Self::int_event_method_is_known(method) {
            host.on_error_fatal(
                "CD_PROPERTY stage.object.*_eve: mutating method requires command lane",
            );
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }

        host.on_error_fatal("無効なコマンドが指定されました。(intevent)");
        Some((PropValue::Int(0), crate::elm::form::INT))
    }

    fn try_property_object_known_command_only_composite(
        list_id: i32,
        obj_idx: i32,
        sub: i32,
        tail: &[i32],
        stage_idx: i32,
        disp_out_of_range_error: bool,
        proc_depth: i32,
        proc_top: i32,
        host: &mut dyn Host,
    ) -> Option<(PropValue, i32)> {
        if sub == crate::elm::objectlist::ELM_OBJECT_CHILD {
            return Self::try_property_object_child_composite(
                list_id,
                obj_idx,
                tail,
                stage_idx,
                disp_out_of_range_error,
                host,
            );
        }
        if Self::object_composite_is_int_event_sub(sub)
            || Self::object_composite_is_int_event_list_sub(sub)
        {
            return Self::try_property_object_int_event_composite(
                sub,
                tail,
                disp_out_of_range_error,
                proc_depth,
                proc_top,
                host,
            );
        }

        if tail.is_empty() {
            host.on_error_fatal("CD_PROPERTY stage.object composite: missing method");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail[0] == crate::elm::ELM_UP {
            host.on_error_fatal("CD_PROPERTY stage.object composite: ELM_UP is not supported");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail[0] == crate::elm::ELM_ARRAY {
            host.on_error_fatal("CD_PROPERTY stage.object composite: array access is invalid");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }

        host.on_error_fatal("CD_PROPERTY stage.object composite: command-only route");
        Some((PropValue::Int(0), crate::elm::form::INT))
    }

    fn object_query_is_known_composite_sub(sub: i32) -> bool {
        use crate::elm::objectlist::*;
        matches!(
            sub,
            ELM_OBJECT_X_EVE
                | ELM_OBJECT_Y_EVE
                | ELM_OBJECT_Z_EVE
                | ELM_OBJECT_SCALE_Z_EVE
                | ELM_OBJECT_SCALE_X_EVE
                | ELM_OBJECT_SCALE_Y_EVE
                | ELM_OBJECT_ROTATE_X_EVE
                | ELM_OBJECT_ROTATE_Y_EVE
                | ELM_OBJECT_ROTATE_Z_EVE
                | ELM_OBJECT_TR_EVE
                | ELM_OBJECT_MONO_EVE
                | ELM_OBJECT_REVERSE_EVE
                | ELM_OBJECT_BRIGHT_EVE
                | ELM_OBJECT_DARK_EVE
                | ELM_OBJECT_CENTER_X_EVE
                | ELM_OBJECT_CENTER_Y_EVE
                | ELM_OBJECT_CENTER_Z_EVE
                | ELM_OBJECT_CENTER_REP_X_EVE
                | ELM_OBJECT_CENTER_REP_Y_EVE
                | ELM_OBJECT_CENTER_REP_Z_EVE
                | ELM_OBJECT_COLOR_RATE_EVE
                | ELM_OBJECT_COLOR_ADD_R_EVE
                | ELM_OBJECT_COLOR_ADD_G_EVE
                | ELM_OBJECT_COLOR_ADD_B_EVE
                | ELM_OBJECT_COLOR_R_EVE
                | ELM_OBJECT_COLOR_G_EVE
                | ELM_OBJECT_COLOR_B_EVE
                | ELM_OBJECT_PATNO_EVE
                | ELM_OBJECT_CLIP_LEFT_EVE
                | ELM_OBJECT_CLIP_TOP_EVE
                | ELM_OBJECT_CLIP_RIGHT_EVE
                | ELM_OBJECT_CLIP_BOTTOM_EVE
                | ELM_OBJECT_X_REP_EVE
                | ELM_OBJECT_Y_REP_EVE
                | ELM_OBJECT_Z_REP_EVE
                | ELM_OBJECT_TR_REP_EVE
                | ELM_OBJECT_SRC_CLIP_LEFT_EVE
                | ELM_OBJECT_SRC_CLIP_TOP_EVE
                | ELM_OBJECT_SRC_CLIP_RIGHT_EVE
                | ELM_OBJECT_SRC_CLIP_BOTTOM_EVE
                | ELM_OBJECT_ALL_EVE
                | ELM_OBJECT_CHILD
        )
    }

    fn validate_object_frame_action_tail(
        &self,
        sub: i32,
        tail: &[i32],
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::frameaction::{is_frameactionlist_get_size, is_frameactionlist_resize};
        use crate::elm::objectlist::ELM_OBJECT_FRAME_ACTION_CH;

        if tail.is_empty() {
            // C++ frame_action/frame_action_list: empty tail returns element ptr lane;
            // property lane degrades to typed default without fatal.
            return true;
        }

        let head = tail[0];
        if head == crate::elm::ELM_UP {
            host.on_error_fatal("無効なコマンドが指定されました。(frame_action)");
            return false;
        }

        if sub == ELM_OBJECT_FRAME_ACTION_CH {
            if head == crate::elm::ELM_ARRAY {
                if tail.len() < 2 {
                    host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                    return false;
                }
                let idx = tail[1];
                if idx < 0 && self.options.disp_out_of_range_error {
                    host.on_error_fatal(
                        "範囲外のフレームアクション番号が指定されました。(frame_action_ch)",
                    );
                }
                if idx < 0 {
                    return false;
                }
                if tail.len() >= 3 {
                    let nested = tail[2];
                    if nested == crate::elm::ELM_UP {
                        if tail.len() == 3 {
                            host.on_error_fatal(
                                "無効なコマンドが指定されました。(frame_action_ch)",
                            );
                            return false;
                        }
                        if tail.len() >= 5
                            && (is_frameactionlist_get_size(tail[3])
                                || is_frameactionlist_resize(tail[3]))
                        {
                            host.on_error_fatal(
                                "無効なコマンドが指定されました。(frame_action_ch)",
                            );
                            return false;
                        }
                        if tail.len() > 5
                            && tail[3] == crate::elm::frameaction::ELM_FRAMEACTION_COUNTER
                        {
                            host.on_error_fatal(
                                "無効なコマンドが指定されました。(frame_action_ch)",
                            );
                            return false;
                        }
                    } else if is_frameactionlist_get_size(nested)
                        || is_frameactionlist_resize(nested)
                    {
                        host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                        return false;
                    } else if tail.len() > 4
                        && nested == crate::elm::frameaction::ELM_FRAMEACTION_COUNTER
                    {
                        host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                        return false;
                    }
                }
                return true;
            }

            if is_frameactionlist_get_size(head) || is_frameactionlist_resize(head) {
                if tail.len() > 1 {
                    host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                    return false;
                }
                return true;
            }
        }

        true
    }

    fn default_frame_action_property(
        &self,
        sub: i32,
        tail: &[i32],
        host: &mut dyn Host,
    ) -> Option<(PropValue, i32)> {
        use crate::elm::frameaction::{
            is_frameaction_end, is_frameaction_is_end_action, is_frameaction_start,
            is_frameactionlist_get_size, is_frameactionlist_resize,
        };
        use crate::elm::objectlist::{ELM_OBJECT_FRAME_ACTION, ELM_OBJECT_FRAME_ACTION_CH};

        let err = |host: &mut dyn Host, msg: &str| {
            host.on_error_fatal(msg);
            (PropValue::Int(0), crate::elm::form::INT)
        };

        if tail.is_empty() {
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }

        if sub == ELM_OBJECT_FRAME_ACTION_CH {
            if tail[0] == crate::elm::ELM_ARRAY {
                if tail.len() < 3 {
                    return Some((PropValue::Int(0), crate::elm::form::INT));
                }
                if tail[2] == crate::elm::ELM_UP {
                    if tail.len() == 3 {
                        return Some(err(
                            host,
                            "無効なコマンドが指定されました。(frame_action_ch)",
                        ));
                    }
                    if is_frameactionlist_get_size(tail[3]) {
                        if tail.len() > 4 {
                            return Some(err(
                                host,
                                "無効なコマンドが指定されました。(frame_action_ch)",
                            ));
                        }
                        return Some((PropValue::Int(0), crate::elm::form::INT));
                    }
                    if is_frameactionlist_resize(tail[3]) {
                        return Some(err(
                            host,
                            "無効なコマンドが指定されました。(frame_action_ch)",
                        ));
                    }
                    if tail[3] == crate::elm::frameaction::ELM_FRAMEACTION_COUNTER {
                        if tail.len() == 4 {
                            return Some(err(
                                host,
                                "無効なコマンドが指定されました。(frame_action.counter)",
                            ));
                        }
                        if tail.len() > 5 {
                            return Some(err(
                                host,
                                "無効なコマンドが指定されました。(frame_action_ch)",
                            ));
                        }
                        return Some(err(
                            host,
                            "無効なコマンドが指定されました。(frame_action_ch)",
                        ));
                    }
                }
            }
            if (is_frameactionlist_get_size(tail[0]) || is_frameactionlist_resize(tail[0]))
                && tail.len() > 1
            {
                return Some(err(
                    host,
                    "無効なコマンドが指定されました。(frame_action_ch)",
                ));
            }
        }

        let method = if sub == ELM_OBJECT_FRAME_ACTION_CH && tail[0] == crate::elm::ELM_ARRAY {
            if tail.len() >= 4 && tail[2] == crate::elm::ELM_UP {
                tail[3]
            } else if tail.len() >= 3 {
                tail[2]
            } else {
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
        } else {
            tail[0]
        };

        if sub == ELM_OBJECT_FRAME_ACTION_CH
            && tail.first().copied() == Some(crate::elm::ELM_ARRAY)
            && tail.get(2).copied() == Some(crate::elm::frameaction::ELM_FRAMEACTION_COUNTER)
            && tail.len() > 4
        {
            return Some(err(
                host,
                "無効なコマンドが指定されました。(frame_action_ch)",
            ));
        }

        if is_frameaction_is_end_action(method) {
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }

        if is_frameaction_start(method) || is_frameaction_end(method) {
            return Some(err(host, "無効なコマンドが指定されました。(frame_action)"));
        }

        if sub == ELM_OBJECT_FRAME_ACTION_CH {
            if is_frameactionlist_get_size(tail[0]) {
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if is_frameactionlist_resize(tail[0]) {
                return Some(err(
                    host,
                    "無効なコマンドが指定されました。(frame_action_ch)",
                ));
            }
        }

        if sub == ELM_OBJECT_FRAME_ACTION || sub == ELM_OBJECT_FRAME_ACTION_CH {
            return Some(err(host, "無効なコマンドが指定されました。(frame_action)"));
        }

        None
    }

    fn object_query_uses_query_api(sub: i32) -> bool {
        use crate::elm::objectlist::*;
        matches!(
            sub,
            ELM_OBJECT_GET_MOVIE_SEEK_TIME
                | ELM_OBJECT_CHECK_MOVIE
                | ELM_OBJECT_FRAME_ACTION
                | ELM_OBJECT_FRAME_ACTION_CH
                | ELM_OBJECT__IAPP_DUMMY
        )
    }

    pub(super) fn object_property_is_settable_int(sub: i32) -> bool {
        use crate::elm::objectlist::*;
        matches!(
            sub,
            ELM_OBJECT_WIPE_COPY
                | ELM_OBJECT_WIPE_ERASE
                | ELM_OBJECT_CLICK_DISABLE
                | ELM_OBJECT_DISP
                | ELM_OBJECT_PATNO
                | ELM_OBJECT_WORLD
                | ELM_OBJECT_ORDER
                | ELM_OBJECT_LAYER
                | ELM_OBJECT_X
                | ELM_OBJECT_Y
                | ELM_OBJECT_Z
                | ELM_OBJECT_CENTER_X
                | ELM_OBJECT_CENTER_Y
                | ELM_OBJECT_CENTER_Z
                | ELM_OBJECT_CENTER_REP_X
                | ELM_OBJECT_CENTER_REP_Y
                | ELM_OBJECT_CENTER_REP_Z
                | ELM_OBJECT_SCALE_X
                | ELM_OBJECT_SCALE_Y
                | ELM_OBJECT_SCALE_Z
                | ELM_OBJECT_ROTATE_X
                | ELM_OBJECT_ROTATE_Y
                | ELM_OBJECT_ROTATE_Z
                | ELM_OBJECT_CLIP_USE
                | ELM_OBJECT_CLIP_LEFT
                | ELM_OBJECT_CLIP_TOP
                | ELM_OBJECT_CLIP_RIGHT
                | ELM_OBJECT_CLIP_BOTTOM
                | ELM_OBJECT_SRC_CLIP_USE
                | ELM_OBJECT_SRC_CLIP_LEFT
                | ELM_OBJECT_SRC_CLIP_TOP
                | ELM_OBJECT_SRC_CLIP_RIGHT
                | ELM_OBJECT_SRC_CLIP_BOTTOM
                | ELM_OBJECT_TR
                | ELM_OBJECT_MONO
                | ELM_OBJECT_REVERSE
                | ELM_OBJECT_BRIGHT
                | ELM_OBJECT_DARK
                | ELM_OBJECT_COLOR_R
                | ELM_OBJECT_COLOR_G
                | ELM_OBJECT_COLOR_B
                | ELM_OBJECT_COLOR_RATE
                | ELM_OBJECT_COLOR_ADD_R
                | ELM_OBJECT_COLOR_ADD_G
                | ELM_OBJECT_COLOR_ADD_B
                | ELM_OBJECT_MASK_NO
                | ELM_OBJECT_TONECURVE_NO
                | ELM_OBJECT_CULLING
                | ELM_OBJECT_ALPHA_TEST
                | ELM_OBJECT_ALPHA_BLEND
                | ELM_OBJECT_BLEND
                | ELM_OBJECT_LIGHT_NO
                | ELM_OBJECT_FOG_USE
                | ELM_OBJECT_EMOTE_KOE_CHARA_NO
                | ELM_OBJECT_EMOTE_MOUTH_VOLUME
        )
    }

    fn try_property_stage_object_group(
        &mut self,
        element: &[i32],
        host: &mut dyn Host,
    ) -> Option<(PropValue, i32)> {
        use crate::elm::objectlist::{
            ELM_OBJECTLIST_GET_SIZE, ELM_STAGE_OBJBTNGROUP, ELM_STAGE_OBJECT,
        };

        if element.is_empty() {
            return None;
        }

        let (stage_idx, mut pos) = if crate::elm::global::is_stage(element[0]) {
            if element.len() < 3 || element[1] != crate::elm::ELM_ARRAY {
                return None;
            }
            (element[2], 3)
        } else if let Some(idx) = Self::stage_index_from_property_root(element[0]) {
            (idx, 1)
        } else {
            return None;
        };

        let stage_size = host.on_stage_list_get_size();
        if stage_size >= 0 && (stage_idx < 0 || stage_idx >= stage_size) {
            if self.options.disp_out_of_range_error {
                host.on_error_fatal("範囲外のステージ番号が指定されました。(stage_list)");
            }
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }

        if pos >= element.len() {
            return None;
        }

        if element[pos] == ELM_STAGE_OBJECT {
            pos += 1;
            if pos < element.len() && element[pos] == ELM_OBJECTLIST_GET_SIZE {
                let n = host.on_object_list_get_size(ELM_STAGE_OBJECT, Some(stage_idx));
                return Some((PropValue::Int(n.max(0)), crate::elm::form::INT));
            }
            if pos + 3 <= element.len() && element[pos] == crate::elm::ELM_ARRAY {
                let obj_idx = element[pos + 1];
                let obj_size = host.on_object_list_get_size(ELM_STAGE_OBJECT, Some(stage_idx));
                if obj_size >= 0 && (obj_idx < 0 || obj_idx >= obj_size) {
                    if self.options.disp_out_of_range_error {
                        host.on_error_fatal(
                            "範囲外のオブジェクト番号が指定されました。(object_list)",
                        );
                    }
                    return Some((PropValue::Int(0), crate::elm::form::INT));
                }
                if !host.on_object_is_use(ELM_STAGE_OBJECT, obj_idx, Some(stage_idx)) {
                    return Some((PropValue::Int(0), crate::elm::form::INT));
                }
                let sub = element[pos + 2];
                if element.len() != pos + 3 {
                    let tail = &element[pos + 3..];
                    if Self::object_query_is_frame_action_sub(sub) {
                        if !self.validate_object_frame_action_tail(sub, tail, host) {
                            return Some((PropValue::Int(0), crate::elm::form::INT));
                        }
                        if let Some((v, form)) = self.default_frame_action_property(sub, tail, host)
                        {
                            return Some((v, form));
                        }
                        if let Some((v, form)) = host.on_object_frame_action_property(
                            ELM_STAGE_OBJECT,
                            obj_idx,
                            sub,
                            tail,
                            Some(stage_idx),
                        ) {
                            return Some((v, form));
                        }
                        host.on_error_fatal("無効なコマンドが指定されました。(frame_action)");
                        return Some((PropValue::Int(0), crate::elm::form::INT));
                    }
                    if Self::object_query_is_int(sub) || Self::object_query_is_str(sub) {
                        host.on_error_fatal("CD_PROPERTY stage.object: invalid composite target");
                        return Some((PropValue::Int(0), crate::elm::form::INT));
                    }
                    if sub == crate::elm::objectlist::ELM_OBJECT_ALL_EVE {
                        return Self::try_property_object_all_eve(
                            ELM_STAGE_OBJECT,
                            obj_idx,
                            tail,
                            stage_idx,
                            host,
                        );
                    }
                    if Self::object_query_is_known_composite_sub(sub) {
                        let (proc_depth, proc_top) = self.observe_proc_stack_tuple();
                        return Self::try_property_object_known_command_only_composite(
                            ELM_STAGE_OBJECT,
                            obj_idx,
                            sub,
                            tail,
                            stage_idx,
                            self.options.disp_out_of_range_error,
                            proc_depth,
                            proc_top,
                            host,
                        );
                    }
                    return None;
                }
                if Self::object_query_is_str(sub) {
                    return Some((
                        PropValue::Str(host.on_object_get_str(
                            ELM_STAGE_OBJECT,
                            obj_idx,
                            sub,
                            Some(stage_idx),
                        )),
                        crate::elm::form::STR,
                    ));
                }
                if Self::object_query_is_int(sub) {
                    let v = if Self::object_query_uses_query_api(sub) {
                        host.on_object_query(ELM_STAGE_OBJECT, obj_idx, sub, &[], Some(stage_idx))
                    } else {
                        host.on_object_get(ELM_STAGE_OBJECT, obj_idx, sub, Some(stage_idx))
                    };
                    return Some((PropValue::Int(v), crate::elm::form::INT));
                }
            }
            return None;
        }

        if element[pos] == ELM_STAGE_OBJBTNGROUP {
            pos += 1;
            if pos < element.len() && element[pos] == ELM_OBJECTLIST_GET_SIZE {
                let n = host.on_group_list_get_size(stage_idx);
                return Some((PropValue::Int(n.max(0)), crate::elm::form::INT));
            }

            if pos + 3 <= element.len()
                && element[pos] == crate::elm::ELM_ARRAY
                && pos + 2 < element.len()
            {
                let group_idx = element[pos + 1];
                let sub = element[pos + 2];
                let group_size = host.on_group_list_get_size(stage_idx);
                if group_size >= 0 && (group_idx < 0 || group_idx >= group_size) {
                    if self.options.disp_out_of_range_error {
                        host.on_error_fatal("範囲外のグループ番号が指定されました。(group_list)");
                    }
                    return Some((PropValue::Int(0), crate::elm::form::INT));
                }
                if Self::group_query_is_int(sub) {
                    return Some((
                        PropValue::Int(host.on_group_get(stage_idx, group_idx, sub)),
                        crate::elm::form::INT,
                    ));
                }
            }
            return None;
        }

        None
    }

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
            return Ok(Some((
                match form {
                    crate::elm::form::INT => PropValue::Int(ret.int),
                    crate::elm::form::STR => PropValue::Str(ret.str_),
                    _ => PropValue::Int(ret.int),
                },
                form,
            )));
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
