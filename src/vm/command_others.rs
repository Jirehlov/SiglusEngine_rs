// Counter / Database / CgTable / BgmTable / G00Buf / Mask / File routing
// Aligns with C++ cmd_others.cpp
use super::*;
impl Vm {
    fn mask_event_owner_id(mask_index: usize, sub: i32) -> i32 {
        ((mask_index as i32) << 8) | (sub & 0xFF)
    }

    fn trace_excall_counter_command(&self, counter_idx: usize, action: &str, value: i32) {
        if counter_idx < Self::EXCALL_COUNTER_BASE || !Self::excall_counter_trace_enabled() {
            return;
        }
        let phase = match action {
            "start" => crate::vm::VmExcallCounterPhase::Start,
            "stop" => crate::vm::VmExcallCounterPhase::Stop,
            "reset" => crate::vm::VmExcallCounterPhase::Reset,
            "wait" => crate::vm::VmExcallCounterPhase::Wait,
            "wait_key" => crate::vm::VmExcallCounterPhase::WaitKey,
            "check_value" => crate::vm::VmExcallCounterPhase::CheckValue,
            "check_active" => crate::vm::VmExcallCounterPhase::CheckActive,
            _ => crate::vm::VmExcallCounterPhase::Tick,
        };
        let line = crate::vm::format_excall_counter_trace(
            counter_idx,
            phase,
            value,
            self.counter_active
                .get(counter_idx)
                .copied()
                .unwrap_or(false),
        );
        log::debug!("{}", line);
    }

    fn ensure_counter_slot(&mut self, idx: usize) {
        if self.counter_values.len() <= idx {
            self.counter_values.resize(idx + 1, 0);
            self.counter_active.resize(idx + 1, false);
        }
    }

    fn resolve_counter_index(
        &self,
        idx_raw: i32,
        list_size: usize,
        list_name: &str,
        disp: bool,
        host: &mut dyn Host,
    ) -> Option<usize> {
        if idx_raw < 0 || (idx_raw as usize) >= list_size {
            if disp {
                host.on_error(&format!(
                    "範囲外の {}[{}] を参照しました。",
                    list_name, idx_raw
                ));
            }
            return None;
        }
        Some(idx_raw as usize)
    }

    fn ensure_database_slot(&mut self, idx: usize) {
        if self.database_tables.len() <= idx {
            self.database_tables.resize(idx + 1, Vec::new());
        }
        if self.database_row_calls.len() <= idx {
            self.database_row_calls.resize(idx + 1, Vec::new());
        }
        if self.database_col_calls.len() <= idx {
            self.database_col_calls.resize(idx + 1, Vec::new());
        }
        if self.database_col_types.len() <= idx {
            self.database_col_types.resize(idx + 1, Vec::new());
        }
    }

    fn resolve_db_index(arg_no: i32, call_list: &[i32], max_len: usize) -> Option<usize> {
        if arg_no < 0 {
            return None;
        }
        let idx = arg_no as usize;
        if idx < max_len {
            return Some(idx);
        }
        call_list.iter().position(|v| *v == arg_no)
    }

    fn report_resource_file_not_found(
        host: &mut dyn Host,
        path: &str,
        tag: &str,
        kind: VmResourceKind,
    ) {
        if path.is_empty() {
            return;
        }
        if !host.on_resource_exists_with_kind(path, kind) {
            host.on_error_file_not_found(&format!(
                "ファイル \"{}\" が見つかりません。({})",
                path, tag
            ));
        }
    }

    pub(super) fn try_command_counter_list(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        self.try_command_counter_list_with_options(
            element,
            args,
            ret_form,
            host,
            self.counter_list_size,
            self.options.disp_out_of_range_error,
            "counter",
        )
    }

    pub(super) fn try_command_counter_list_with_options(
        &mut self,
        element: &[i32],
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
        list_size: usize,
        disp_out_of_range_error: bool,
        list_name: &str,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        match element[0] {
            x if x == crate::elm::ELM_ARRAY => {
                let idx_raw = element.get(1).copied().unwrap_or(0);
                let Some(idx) = self.resolve_counter_index(
                    idx_raw,
                    list_size,
                    list_name,
                    disp_out_of_range_error,
                    host,
                ) else {
                    return true;
                };
                self.ensure_counter_slot(idx);
                if element.len() == 2 {
                    return true;
                }
                self.try_command_counter(idx, &element[2..], args, ret_form, host);
                true
            }
            x if x == crate::elm::counterlist::ELM_COUNTERLIST_GET_SIZE => {
                self.stack.push_int(list_size as i32);
                true
            }
            _ => {
                host.on_error(&format!("無効なコマンドが指定されました。{}", list_name));
                true
            }
        }
    }

    fn try_command_counter(
        &mut self,
        counter_idx: usize,
        element: &[i32],
        args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) {
        use crate::elm::counter::*;
        if element.is_empty() {
            return;
        }
        let method = element[0];
        let Some(int_args) = Self::counter_collect_int_args(method, args, host) else {
            return;
        };
        self.ensure_counter_slot(counter_idx);
        self.maybe_emit_excall_counter_aggregate_hint(host);
        match method {
            ELM_COUNTER_SET => {
                self.counter_values[counter_idx] = int_args[0];
            }
            ELM_COUNTER_GET => {
                self.stack.push_int(self.counter_values[counter_idx]);
            }
            ELM_COUNTER_RESET => {
                self.counter_values[counter_idx] = 0;
                self.counter_active[counter_idx] = false;
                self.trace_excall_counter_command(counter_idx, "reset", 0);
                host.on_counter_action(method, args);
            }
            ELM_COUNTER_START
            | ELM_COUNTER_START_REAL
            | ELM_COUNTER_START_FRAME
            | ELM_COUNTER_START_FRAME_REAL
            | ELM_COUNTER_START_FRAME_LOOP
            | ELM_COUNTER_START_FRAME_LOOP_REAL
            | ELM_COUNTER_RESUME => {
                self.counter_active[counter_idx] = true;
                self.trace_excall_counter_command(
                    counter_idx,
                    "start",
                    self.counter_values[counter_idx],
                );
                host.on_counter_action(method, args);
            }
            ELM_COUNTER_STOP => {
                self.counter_active[counter_idx] = false;
                self.trace_excall_counter_command(
                    counter_idx,
                    "stop",
                    self.counter_values[counter_idx],
                );
                host.on_counter_action(method, args);
            }
            ELM_COUNTER_WAIT | ELM_COUNTER_WAIT_KEY => {
                let option = int_args[0];
                self.trace_excall_counter_command(
                    counter_idx,
                    if method == ELM_COUNTER_WAIT {
                        "wait"
                    } else {
                        "wait_key"
                    },
                    self.counter_values[counter_idx],
                );
                self.counter_emit_wait_observation(counter_idx, method, option, host);
                host.on_counter_action(method, args);
            }
            ELM_COUNTER_CHECK_VALUE => {
                let time = int_args[0];
                let ok = if self.counter_values[counter_idx] - time >= 0 {
                    1
                } else {
                    0
                };
                self.trace_excall_counter_command(counter_idx, "check_value", ok);
                self.stack.push_int(ok);
            }
            ELM_COUNTER_CHECK_ACTIVE => {
                let _option = int_args[0];
                let active = if self.counter_active[counter_idx] {
                    1
                } else {
                    0
                };
                self.trace_excall_counter_command(counter_idx, "check_active", active);
                self.stack.push_int(active);
            }
            _ => host.on_error("無効なコマンドが指定されました。(counter)"),
        }
    }

    pub(super) fn try_command_database_list(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        match element[0] {
            x if x == crate::elm::ELM_ARRAY => {
                let db_idx = element.get(1).copied().unwrap_or(0).max(0) as usize;
                self.ensure_database_slot(db_idx);
                if element.len() > 2 {
                    self.try_command_database(db_idx, &element[2..], args, ret_form, host);
                }
                true
            }
            x if x == crate::elm::databaselist::ELM_DATABASELIST_GET_SIZE => {
                self.stack.push_int(self.database_tables.len() as i32);
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(databaselist)");
                true
            }
        }
    }

    fn try_command_database(
        &mut self,
        db_idx: usize,
        element: &[i32],
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) {
        use crate::elm::databaselist::*;
        if element.is_empty() {
            return;
        }

        let db = &self.database_tables[db_idx];
        let row_arg = Self::arg_int(args, 0);
        let col_arg = Self::arg_int(args, 1);
        let row_no = Self::resolve_db_index(
            row_arg,
            self.database_row_calls
                .get(db_idx)
                .map(|v| v.as_slice())
                .unwrap_or(&[]),
            db.len(),
        );
        let col_no = Self::resolve_db_index(
            col_arg,
            self.database_col_calls
                .get(db_idx)
                .map(|v| v.as_slice())
                .unwrap_or(&[]),
            db.iter().map(|r| r.len()).max().unwrap_or(0),
        );
        let row = row_no.and_then(|i| db.get(i));
        let cell = row.and_then(|r| col_no.and_then(|c| r.get(c))).cloned();
        let col_type = col_no.and_then(|c| {
            self.database_col_types
                .get(db_idx)
                .and_then(|v| v.get(c))
                .copied()
                .and_then(|dt| match dt {
                    b'V' => Some(1),
                    b'S' => Some(2),
                    _ => None,
                })
                .or_else(|| {
                    db.iter().find_map(|r| match r.get(c) {
                        Some(PropValue::Int(_)) => Some(1),
                        Some(PropValue::Str(_)) => Some(2),
                        _ => None,
                    })
                })
        });

        match element[0] {
            ELM_DATABASE_GET_NUM => match cell {
                Some(PropValue::Int(v)) => self.stack.push_int(v),
                _ => self.stack.push_int(0),
            },
            ELM_DATABASE_GET_STR | ELM_DATABASE_GET_DATA => match cell {
                Some(PropValue::Str(s)) => self.stack.push_str(s.clone()),
                _ => {
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(0);
                    } else {
                        self.stack.push_str(String::new());
                    }
                }
            },
            ELM_DATABASE_CHECK_ITEM => {
                self.stack.push_int(if row_no.is_some() { 1 } else { 0 });
            }
            ELM_DATABASE_CHECK_COLUMN => {
                self.stack.push_int(col_type.unwrap_or(0));
            }
            ELM_DATABASE_FIND_NUM => {
                let target = Self::arg_int(args, 1);
                if col_type != Some(1) {
                    self.stack.push_int(-1);
                } else {
                    let found = db.iter().position(|r| {
                        col_no
                            .and_then(|cn| r.get(cn))
                            .is_some_and(|pv| matches!(pv, PropValue::Int(v) if *v == target))
                    });
                    let ret = found
                        .map(|idx| {
                            self.database_row_calls
                                .get(db_idx)
                                .and_then(|v| v.get(idx))
                                .copied()
                                .unwrap_or(idx as i32)
                        })
                        .unwrap_or(-1);
                    self.stack.push_int(ret);
                }
            }
            ELM_DATABASE_FIND_STR | ELM_DATABASE_FIND_STR_REAL => {
                let target = Self::arg_str_others(args, 1);
                if col_type != Some(2) {
                    self.stack.push_int(-1);
                } else {
                    let target_cmp = target.to_ascii_lowercase();
                    let found = db
                        .iter()
                        .position(|r| match col_no.and_then(|cn| r.get(cn)) {
                            Some(PropValue::Str(v)) => {
                                if element[0] == ELM_DATABASE_FIND_STR_REAL {
                                    v == &target
                                } else {
                                    v.to_ascii_lowercase() == target_cmp
                                }
                            }
                            _ => false,
                        });
                    let ret = found
                        .map(|idx| {
                            self.database_row_calls
                                .get(db_idx)
                                .and_then(|v| v.get(idx))
                                .copied()
                                .unwrap_or(idx as i32)
                        })
                        .unwrap_or(-1);
                    self.stack.push_int(ret);
                }
            }
            _ => host.on_error("無効なコマンドが指定されました。(database)"),
        }
    }

    /// Route `global.bgmtable.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_bgm_table(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::bgmtable::*;
        if element.is_empty() {
            return true;
        }
        match element[0] {
            ELM_BGMTABLE_GET_BGM_CNT => {
                self.stack.push_int(self.bgm_name_listened.len() as i32);
                true
            }
            ELM_BGMTABLE_GET_LISTEN_BY_NAME => {
                let name = Self::arg_str_others(args, 0);
                let listened = self.bgm_name_listened.get(&name).copied().unwrap_or(false);
                self.stack.push_int(if listened { 1 } else { 0 });
                true
            }
            ELM_BGMTABLE_SET_LISTEN_BY_NAME => {
                self.bgm_name_listened
                    .insert(Self::arg_str_others(args, 0), Self::arg_int(args, 1) != 0);
                true
            }
            ELM_BGMTABLE_SET_ALL_FLAG => {
                let v = Self::arg_int(args, 0) != 0;
                for listened in self.bgm_name_listened.values_mut() {
                    *listened = v;
                }
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(bgmtable)");
                true
            }
        }
    }

    // G00Buf / Mask / File — accept + default returns

    /// Route `global.g00buf.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_g00buf(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::g00buf::*;
        use crate::elm::g00buflist::*;
        if element.is_empty() {
            host.on_error_fatal("無効なコマンドが指定されました。(g00buf)");
            return true;
        }
        let push_default = |vm: &mut Vm| {
            if ret_form == crate::elm::form::INT {
                vm.stack.push_int(0);
            } else if ret_form == crate::elm::form::STR {
                vm.stack.push_str(String::new());
            }
        };
        match element[0] {
            ELM_G00BUFLIST_GET_SIZE => {
                self.stack.push_int(self.g00buf_loaded.len() as i32);
            }
            ELM_G00BUFLIST_FREE_ALL => {
                self.g00buf_loaded.fill(None);
            }
            x if x == crate::elm::ELM_ARRAY => {
                let idx = element.get(1).copied().unwrap_or(-1);
                if idx < 0 {
                    if self.options.disp_out_of_range_error {
                        host.on_error_fatal("範囲外の g00buf 番号が指定されました。(g00buf)");
                    }
                    push_default(self);
                    return true;
                }
                let idx = idx as usize;
                if self.g00buf_loaded.len() <= idx {
                    self.g00buf_loaded.resize(idx + 1, None);
                }
                if element.len() <= 2 {
                    return true;
                }
                match element[2] {
                    ELM_G00BUF_LOAD => {
                        let name = Self::arg_str_others(args, 0);
                        if name.is_empty() {
                            self.g00buf_loaded[idx] = None;
                        } else {
                            self.g00buf_loaded[idx] = Some(name.clone());
                            Self::report_resource_file_not_found(
                                host,
                                &name,
                                "g00buf.load",
                                VmResourceKind::Image,
                            );
                        }
                        host.on_trace(&format!("vm: g00buf[{}].load '{}'", idx, name));
                    }
                    ELM_G00BUF_FREE => {
                        self.g00buf_loaded[idx] = None;
                        host.on_trace(&format!("vm: g00buf[{}].free", idx));
                    }
                    _ => {
                        host.on_error_fatal("無効なコマンドが指定されました。(g00buf)");
                        push_default(self);
                    }
                }
            }
            _ => {
                host.on_error_fatal("無効なコマンドが指定されました。(g00buf)");
                push_default(self);
            }
        }
        true
    }

    /// Route `global.mask.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_mask_list(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::mask::*;
        use crate::elm::masklist::*;
        if element.is_empty() {
            host.on_error_fatal("無効なコマンドが指定されました。(mask)");
            return true;
        }
        let push_default = |vm: &mut Vm| {
            if ret_form == crate::elm::form::INT {
                vm.stack.push_int(0);
            } else if ret_form == crate::elm::form::STR {
                vm.stack.push_str(String::new());
            }
        };
        match element[0] {
            ELM_MASKLIST_GET_SIZE => {
                self.stack.push_int(self.mask_slots.len() as i32);
            }
            x if x == crate::elm::ELM_ARRAY => {
                let idx = element.get(1).copied().unwrap_or(-1);
                if idx < 0 {
                    if self.options.disp_out_of_range_error {
                        host.on_error_fatal("範囲外の mask 番号が指定されました。(mask)");
                    }
                    push_default(self);
                    return true;
                }
                let idx = idx as usize;
                if self.mask_slots.len() <= idx {
                    self.mask_slots.resize(idx + 1, MaskSlotState::default());
                }
                if element.len() <= 2 {
                    return true;
                }
                let sub = element[2];
                match sub {
                    ELM_MASK_INIT => {
                        self.mask_slots[idx] = MaskSlotState::default();
                    }
                    ELM_MASK_CREATE => {
                        self.mask_slots[idx].name = Self::arg_str_others(args, 0);
                    }
                    ELM_MASK_X => {
                        if args.is_empty() {
                            self.stack.push_int(self.mask_slots[idx].x);
                        } else {
                            self.mask_slots[idx].x = Self::arg_int(args, 0);
                        }
                    }
                    ELM_MASK_Y => {
                        if args.is_empty() {
                            self.stack.push_int(self.mask_slots[idx].y);
                        } else {
                            self.mask_slots[idx].y = Self::arg_int(args, 0);
                        }
                    }
                    ELM_MASK_X_EVE | ELM_MASK_Y_EVE => {
                        let owner = Self::mask_event_owner_id(idx, sub);
                        let rest = if element.len() > 3 {
                            &element[3..]
                        } else {
                            &[]
                        };
                        self.try_command_int_event(rest, _arg_list_id, args, ret_form, host, owner);
                    }
                    _ => {
                        host.on_error_fatal("無効なコマンドが指定されました。(mask)");
                        push_default(self);
                    }
                }
            }
            _ => {
                host.on_error_fatal("無効なコマンドが指定されました。(mask)");
                push_default(self);
            }
        }
        true
    }

    /// Route `global.file.<sub>`. Returns `true` if handled.
    pub(super) fn try_command_file(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::file::*;
        if element.is_empty() {
            host.on_error_fatal("無効なコマンドが指定されました。(file)");
            return true;
        }
        let path = || Self::arg_str_others(args, 0);
        match element[0] {
            ELM_FILE_LOAD_TXT => {
                let p = path();
                let txt = if p.is_empty() {
                    String::new()
                } else {
                    Self::report_resource_file_not_found(
                        host,
                        &p,
                        "file.load_txt",
                        VmResourceKind::Text,
                    );
                    host.on_resource_read_text(&p)
                        .unwrap_or_else(|| std::fs::read_to_string(&p).unwrap_or_default())
                };
                host.on_trace(&format!(
                    "vm: file.load_txt path='{}' bytes={}",
                    p,
                    txt.len()
                ));
                if ret_form == crate::elm::form::STR {
                    self.stack.push_str(txt);
                } else if ret_form == crate::elm::form::INT {
                    self.stack.push_int(if p.is_empty() { 0 } else { 1 });
                }
            }
            ELM_FILE_PRELOAD_OMV => {
                let p = path();
                host.on_trace(&format!("vm: file.preload_omv path='{}'", p));
                Self::report_resource_file_not_found(
                    host,
                    &p,
                    "file.preload_omv",
                    VmResourceKind::Movie,
                );
            }
            _ => {
                host.on_error_fatal("無効なコマンドが指定されました。(file)");
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                } else if ret_form == crate::elm::form::STR {
                    self.stack.push_str(String::new());
                }
            }
        }
        true
    }
}
