use super::*;

#[allow(dead_code)]
impl Vm {
    fn capture_req_from_args(&self, args: &[Prop]) -> VmCaptureFileOp {
        let mut req = VmCaptureFileOp {
            file_name: match args.first().map(|p| &p.value) {
                Some(PropValue::Str(v)) => v.clone(),
                _ => String::new(),
            },
            extension: match args.get(1).map(|p| &p.value) {
                Some(PropValue::Str(v)) => v.clone(),
                _ => String::new(),
            },
            dialog_flag: false,
            dialog_title: String::new(),
            int_flags: VmCaptureFlagsSpec::default(),
            str_flags: VmCaptureFlagsSpec::default(),
            int_values: Vec::new(),
            str_values: Vec::new(),
        };
        if req.dialog_title.is_empty() {
            req.dialog_title = if req.file_name.is_empty() {
                "ファイルの選択".to_string()
            } else {
                "保存先の選択".to_string()
            };
        }
        for arg in args {
            match arg.id {
                0 => req.dialog_flag = matches!(arg.value, PropValue::Int(v) if v != 0),
                1 => {
                    if let PropValue::Str(v) = &arg.value {
                        req.dialog_title = v.clone();
                    }
                }
                2 => {
                    if let PropValue::Element(v) = &arg.value {
                        req.int_flags.element = v.clone();
                    }
                }
                3 => {
                    req.int_flags.index = if let PropValue::Int(v) = arg.value {
                        v
                    } else {
                        0
                    }
                }
                4 => {
                    req.int_flags.count = if let PropValue::Int(v) = arg.value {
                        v
                    } else {
                        0
                    }
                }
                5 => {
                    if let PropValue::Element(v) = &arg.value {
                        req.str_flags.element = v.clone();
                    }
                }
                6 => {
                    req.str_flags.index = if let PropValue::Int(v) = arg.value {
                        v
                    } else {
                        0
                    }
                }
                7 => {
                    req.str_flags.count = if let PropValue::Int(v) = arg.value {
                        v
                    } else {
                        0
                    }
                }
                _ => {}
            }
        }
        req
    }

    fn collect_capture_int_values(&mut self, spec: &VmCaptureFlagsSpec) -> Vec<i32> {
        if spec.element.is_empty() || spec.count <= 0 {
            return Vec::new();
        }
        let head = spec.element[0];
        let Some(list) = self.get_intflag_mut(head) else {
            return Vec::new();
        };
        let start = spec.index.max(0) as usize;
        let count = spec.count.max(0) as usize;
        (0..count)
            .map(|off| list.get(start + off).copied().unwrap_or(0))
            .collect()
    }

    fn collect_capture_str_values(&mut self, spec: &VmCaptureFlagsSpec) -> Vec<String> {
        if spec.element.is_empty() || spec.count <= 0 {
            return Vec::new();
        }
        let head = spec.element[0];
        let Some(list) = self.get_strflag_mut(head) else {
            return Vec::new();
        };
        let start = spec.index.max(0) as usize;
        let count = spec.count.max(0) as usize;
        (0..count)
            .map(|off| list.get(start + off).cloned().unwrap_or_default())
            .collect()
    }

    fn apply_capture_payload(&mut self, req: &VmCaptureFileOp, payload: &VmCaptureFlagPayload) {
        if !req.int_flags.element.is_empty() && req.int_flags.count > 0 {
            if let Some(list) = self.get_intflag_mut(req.int_flags.element[0]) {
                let start = req.int_flags.index.max(0) as usize;
                for (off, value) in payload.int_values.iter().copied().enumerate() {
                    if off >= req.int_flags.count.max(0) as usize {
                        break;
                    }
                    let idx = start + off;
                    if idx < list.len() {
                        list[idx] = value;
                    }
                }
            }
        }
        if !req.str_flags.element.is_empty() && req.str_flags.count > 0 {
            if let Some(list) = self.get_strflag_mut(req.str_flags.element[0]) {
                let start = req.str_flags.index.max(0) as usize;
                for (off, value) in payload.str_values.iter().cloned().enumerate() {
                    if off >= req.str_flags.count.max(0) as usize {
                        break;
                    }
                    let idx = start + off;
                    if idx < list.len() {
                        list[idx] = value;
                    }
                }
            }
        }
    }

    pub(super) fn try_command_syscom_capture(
        &mut self,
        x: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> Option<bool> {
        match x {
            y if y == crate::elm::syscom::ELM_SYSCOM_CREATE_CAPTURE_BUFFER => {
                host.on_syscom_create_capture_buffer(
                    Self::arg_int(args, 0),
                    Self::arg_int(args, 1),
                );
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_DESTROY_CAPTURE_BUFFER => {
                host.on_syscom_destroy_capture_buffer();
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_CAPTURE_TO_CAPTURE_BUFFER => {
                host.on_syscom_capture_to_buffer(
                    Self::arg_int(args, 0),
                    Self::arg_int(args, 1),
                    "",
                );
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_CAPTURE_AND_SAVE_BUFFER_TO_PNG => {
                let path = match args.get(2).map(|p| &p.value) {
                    Some(PropValue::Str(v)) => v.clone(),
                    _ => String::new(),
                };
                host.on_syscom_capture_to_buffer(
                    Self::arg_int(args, 0),
                    Self::arg_int(args, 1),
                    &path,
                );
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SAVE_CAPTURE_BUFFER_TO_FILE => {
                let mut req = self.capture_req_from_args(args);
                req.int_values = self.collect_capture_int_values(&req.int_flags);
                req.str_values = self.collect_capture_str_values(&req.str_flags);
                let ok = host.on_syscom_save_capture_buffer_to_file(&req);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(if ok { 1 } else { 0 });
                }
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_LOAD_FLAG_FROM_CAPTURE_FILE => {
                let mut req = self.capture_req_from_args(args);
                if req.dialog_title.is_empty() {
                    req.dialog_title = "ファイルの選択".to_string();
                }
                if !req.dialog_flag
                    && !req.file_name.is_empty()
                    && !host.on_resource_exists_with_kind(&req.file_name, VmResourceKind::Generic)
                {
                    host.on_error_file_not_found(&format!(
                        "ファイル \"{}\" が見つかりません。(syscom.load_flag_from_capture_file)",
                        req.file_name
                    ));
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(0);
                    }
                    return Some(true);
                }
                let payload = host.on_syscom_load_flag_from_capture_file(&req);
                let ok = if let Some(payload) = payload {
                    self.apply_capture_payload(&req, &payload);
                    true
                } else {
                    false
                };
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(if ok { 1 } else { 0 });
                }
                Some(true)
            }
            _ => None,
        }
    }
}
