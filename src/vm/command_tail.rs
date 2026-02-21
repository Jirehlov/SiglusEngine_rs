use super::*;

impl Vm {
    fn parse_wipe_start_params_from_cpp(elm: i32, args: &[Prop]) -> (u64, bool, i32) {
        // C++ source of truth: cmd_wipe.cpp::tnm_command_proc_wipe
        // defaults + positional args + named args override
        let mut wipe_time = 500i32;
        let mut start_time = 0i32;
        let mut wait_flag = true;
        let mut key_wait_mode = -1i32;

        let time_pos = if elm == crate::elm::global::ELM_GLOBAL_MASK_WIPE
            || elm == crate::elm::global::ELM_GLOBAL_MASK_WIPE_ALL
        {
            2
        } else {
            1
        };
        if let Some(PropValue::Int(v)) = args.get(time_pos).map(|p| &p.value) {
            wipe_time = *v;
        }

        for arg in args {
            match arg.id {
                1 => {
                    if let PropValue::Int(v) = arg.value {
                        wipe_time = v;
                    }
                }
                8 => {
                    if let PropValue::Int(v) = arg.value {
                        wait_flag = v != 0;
                    }
                }
                9 => {
                    if let PropValue::Int(v) = arg.value {
                        key_wait_mode = v;
                    }
                }
                11 => {
                    if let PropValue::Int(v) = arg.value {
                        start_time = v;
                    }
                }
                _ => {}
            }
        }

        (
            ((wipe_time as i64 - start_time as i64).max(0)) as u64,
            wait_flag,
            key_wait_mode,
        )
    }
    fn mwnd_no_from_element_path(path: &[i32]) -> Option<i32> {
        // C++ keeps current mwnd as element path and GET resolves from that element:
        //   global.front.stage_mwnd[idx]
        //   global.stage[front].stage_mwnd[idx]
        // Accept only these canonical forms and treat anything else as invalid.
        const FRONT_STAGE_INDEX: i32 = 1;
        match path {
            [a, b, c, idx]
                if *a == crate::elm::global::ELM_GLOBAL_FRONT
                    && *b == crate::elm::objectlist::ELM_STAGE_MWND
                    && *c == crate::elm::ELM_ARRAY =>
            {
                Some(*idx)
            }
            [a, b, c, d, e, idx]
                if *a == crate::elm::global::ELM_GLOBAL_STAGE
                    && *b == crate::elm::ELM_ARRAY
                    && *c == FRONT_STAGE_INDEX
                    && *d == crate::elm::objectlist::ELM_STAGE_MWND
                    && *e == crate::elm::ELM_ARRAY =>
            {
                Some(*idx)
            }
            _ => None,
        }
    }

    fn resolve_mwnd_no(element_path: &[i32]) -> i32 {
        Self::mwnd_no_from_element_path(element_path).unwrap_or(-1)
    }

    fn canonical_mwnd_element_path_from_no(mwnd_no: i32) -> Vec<i32> {
        vec![
            crate::elm::global::ELM_GLOBAL_FRONT,
            crate::elm::objectlist::ELM_STAGE_MWND,
            crate::elm::ELM_ARRAY,
            mwnd_no,
        ]
    }

    fn canonical_mwnd_element_path_from_arg(path: &[i32]) -> Vec<i32> {
        match Self::mwnd_no_from_element_path(path) {
            Some(idx) => Self::canonical_mwnd_element_path_from_no(idx),
            None => Self::canonical_mwnd_element_path_from_no(-1),
        }
    }

    pub(super) fn try_command_global_tail(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> Result<Option<bool>> {
        match element[0] {
            // -----------------------------------------------------------------
            // Flag int-list commands: A...Z list methods
            // -----------------------------------------------------------------
            x if Self::is_intflag(x) => {
                if element.len() >= 2 {
                    let method = element[1];
                    // Handle ELM_ARRAY access (already handled via property/assign, but
                    // command path can occur for method calls on the list)
                    if method == crate::elm::ELM_ARRAY {
                        // Indexed access handled by property/assign; accept command.
                        return Ok(Some(true));
                    }
                    if crate::elm::list::is_intlist_get_size(method) {
                        let len = self.get_intflag_mut(x).map(|l| l.len() as i32).unwrap_or(0);
                        if ret_form == crate::elm::form::INT {
                            self.stack.push_int(len);
                        }
                        return Ok(Some(true));
                    }
                    if crate::elm::list::is_intlist_init(method) {
                        if let Some(list) = self.get_intflag_mut(x) {
                            for v in list.iter_mut() {
                                *v = 0;
                            }
                        }
                        return Ok(Some(true));
                    }
                    if crate::elm::list::is_intlist_resize(method) {
                        let n = match args.get(0).map(|p| &p.value) {
                            Some(PropValue::Int(v)) => (*v).max(0) as usize,
                            _ => 0,
                        };
                        if let Some(list) = self.get_intflag_mut(x) {
                            list.resize(n, 0);
                        }
                        return Ok(Some(true));
                    }
                    if crate::elm::list::is_intlist_sets(method) {
                        let start = match args.get(0).map(|p| &p.value) {
                            Some(PropValue::Int(v)) => (*v).max(0) as usize,
                            _ => 0,
                        };
                        let values: Vec<i32> = args
                            .iter()
                            .skip(1)
                            .map(|p| match &p.value {
                                PropValue::Int(v) => *v,
                                _ => 0,
                            })
                            .collect();
                        if let Some(list) = self.get_intflag_mut(x) {
                            let need = start.saturating_add(values.len());
                            if list.len() < need {
                                list.resize(need, 0);
                            }
                            for (i, v) in values.iter().enumerate() {
                                list[start + i] = *v;
                            }
                        }
                        return Ok(Some(true));
                    }
                    if crate::elm::list::is_intlist_clear(method) {
                        let a = match args.get(0).map(|p| &p.value) {
                            Some(PropValue::Int(v)) => (*v).max(0) as usize,
                            _ => 0,
                        };
                        let b = match args.get(1).map(|p| &p.value) {
                            Some(PropValue::Int(v)) => (*v).max(0) as usize,
                            _ => a,
                        };
                        let fill = match args.get(2).map(|p| &p.value) {
                            Some(PropValue::Int(v)) => *v,
                            _ => 0,
                        };
                        if let Some(list) = self.get_intflag_mut(x) {
                            if !list.is_empty() && a < list.len() {
                                let end = b.min(list.len().saturating_sub(1));
                                if a <= end {
                                    for i in a..=end {
                                        list[i] = fill;
                                    }
                                }
                            }
                        }
                        return Ok(Some(true));
                    }
                }
                // Single-element access = accept
                return Ok(Some(true));
            }
            // -----------------------------------------------------------------
            // Flag str-list commands: S / M / namae_global / namae_local list methods
            // -----------------------------------------------------------------
            x if Self::is_strflag(x) => {
                if element.len() >= 2 {
                    let method = element[1];
                    if method == crate::elm::ELM_ARRAY {
                        return Ok(Some(true));
                    }
                    if crate::elm::list::is_strlist_get_size(method) {
                        let len = self.get_strflag_mut(x).map(|l| l.len() as i32).unwrap_or(0);
                        if ret_form == crate::elm::form::INT {
                            self.stack.push_int(len);
                        }
                        return Ok(Some(true));
                    }
                    if crate::elm::list::is_strlist_init(method) {
                        if let Some(list) = self.get_strflag_mut(x) {
                            for v in list.iter_mut() {
                                v.clear();
                            }
                        }
                        return Ok(Some(true));
                    }
                    if crate::elm::list::is_strlist_resize(method) {
                        let n = match args.get(0).map(|p| &p.value) {
                            Some(PropValue::Int(v)) => (*v).max(0) as usize,
                            _ => 0,
                        };
                        if let Some(list) = self.get_strflag_mut(x) {
                            list.resize(n, String::new());
                        }
                        return Ok(Some(true));
                    }
                }
                return Ok(Some(true));
            }
            // -----------------------------------------------------------------
            // Save / Selection point commands
            // C++ reference: siglus_engine_source/cmd_global.cpp
            // STUB(C++: eng_scene.cpp + eng_save.cpp):
            // - Gap: C++ stores full local/selection save blobs; Rust currently snapshots
            //   VmPersistentState as an approximation for existence/stock semantics.
            // - Expected behavior: preserve command-visible check/stack/drop side effects
            //   (stack source = current local state, drop target = selection state).
            // - Minimal validation direction: run CHECK/STACK/DROP sequences and compare
            //   stack results and sel-point existence transitions with C++.
            // -----------------------------------------------------------------
            x if crate::elm::global::is_savepoint_command(x) => {
                match x {
                    x if crate::elm::global::is_savepoint_set(x) => {
                        // C++ does an internal push/pop and always leaves 0 on stack.
                        self.save_point_snapshot = Some(self.snapshot_persistent_state());
                        self.save_point_set = true;
                        self.stack.push_int(0);
                    }
                    x if crate::elm::global::is_savepoint_clear(x) => {
                        self.save_point_snapshot = None;
                        self.save_point_set = false;
                    }
                    x if crate::elm::global::is_savepoint_check(x) => {
                        // C++ cmd_global.cpp::ELM_GLOBAL_CHECK_SAVEPOINT always pushes int.
                        self.stack.push_int(if self.save_point_snapshot.is_some() {
                            1
                        } else {
                            0
                        });
                    }
                    x if crate::elm::global::is_selpoint_set(x) => {
                        self.sel_point_snapshot = Some(self.snapshot_persistent_state());
                        self.sel_point_set = true;
                    }
                    x if crate::elm::global::is_selpoint_clear(x) => {
                        self.sel_point_snapshot = None;
                        self.sel_point_set = false;
                    }
                    x if crate::elm::global::is_selpoint_check(x) => {
                        // C++ cmd_global.cpp::ELM_GLOBAL_CHECK_SELPOINT always pushes int.
                        self.stack.push_int(if self.sel_point_snapshot.is_some() {
                            1
                        } else {
                            0
                        });
                    }
                    x if crate::elm::global::is_selpoint_stack(x) => {
                        // C++ tnm_stack_sel_point stores current local-save-equivalent state.
                        self.sel_point_stock = Some(self.snapshot_persistent_state());
                    }
                    x if crate::elm::global::is_selpoint_drop(x) => {
                        self.sel_point_snapshot = self.sel_point_stock.clone();
                        self.sel_point_set = self.sel_point_snapshot.is_some();
                        self.sel_point_stock = None;
                    }
                    _ => {}
                }
                return Ok(Some(true));
            }

            // -----------------------------------------------------------------
            // Math built-in commands
            // -----------------------------------------------------------------
            x if crate::elm::global::is_math(x) => {
                if element.len() >= 2 {
                    self.proc_math_command(&element[1..], args, ret_form);
                }
                return Ok(Some(true));
            }
            // -----------------------------------------------------------------
            // Timewait handling (supports realtime wait + skip interrupt)
            // -----------------------------------------------------------------
            x if crate::elm::global::is_timewait_command(x) => {
                if self.options.wait_enabled() {
                    let key_skip_enabled = x == crate::elm::global::ELM_GLOBAL_TIMEWAIT_KEY;
                    if let Some(PropValue::Int(ms)) = args.get(0).map(|p| &p.value) {
                        if *ms > 0 {
                            let total = std::time::Duration::from_millis(*ms as u64);
                            let tick = std::time::Duration::from_millis(16);
                            let start = std::time::Instant::now();
                            while start.elapsed() < total {
                                if host.should_interrupt()
                                    || (key_skip_enabled && host.should_skip_wait())
                                {
                                    break;
                                }
                                let left = total.saturating_sub(start.elapsed());
                                std::thread::sleep(left.min(tick));
                            }
                        }
                    }
                }
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                return Ok(Some(true));
            }
            // -----------------------------------------------------------------
            // Message window management
            // -----------------------------------------------------------------
            x if crate::elm::global::is_set_mwnd(x) => {
                match arg_list_id {
                    0 => {
                        if let Some(PropValue::Element(elm)) = args.get(0).map(|p| &p.value) {
                            self.cur_mwnd_element = Self::canonical_mwnd_element_path_from_arg(elm);
                        }
                    }
                    1 => {
                        if let Some(PropValue::Int(v)) = args.get(0).map(|p| &p.value) {
                            self.cur_mwnd_element = Self::canonical_mwnd_element_path_from_no(*v);
                        }
                    }
                    _ => {}
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_get_mwnd(x) => {
                // C++ cmd_global.cpp::ELM_GLOBAL_GET_MWND always pushes int.
                let mwnd_no = Self::resolve_mwnd_no(&self.cur_mwnd_element);
                self.stack.push_int(mwnd_no);
                return Ok(Some(true));
            }
            x if crate::elm::global::is_set_sel_mwnd(x) => {
                match arg_list_id {
                    0 => {
                        if let Some(PropValue::Element(elm)) = args.get(0).map(|p| &p.value) {
                            self.cur_sel_mwnd_element =
                                Self::canonical_mwnd_element_path_from_arg(elm);
                        }
                    }
                    1 => {
                        if let Some(PropValue::Int(v)) = args.get(0).map(|p| &p.value) {
                            self.cur_sel_mwnd_element =
                                Self::canonical_mwnd_element_path_from_no(*v);
                        }
                    }
                    _ => {}
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_get_sel_mwnd(x) => {
                // C++ cmd_global.cpp::ELM_GLOBAL_GET_SEL_MWND always pushes int.
                let mwnd_no = Self::resolve_mwnd_no(&self.cur_sel_mwnd_element);
                self.stack.push_int(mwnd_no);
                return Ok(Some(true));
            }
            // -----------------------------------------------------------------
            // Display / capture / wipe stubs (accept + no-op or default)
            // -----------------------------------------------------------------
            x if crate::elm::global::is_display_capture_stub(x) => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_wipe_start_command(x) => {
                let (duration_ms, wait_flag, key_wait_mode) =
                    Self::parse_wipe_start_params_from_cpp(x, args);
                if self.no_wipe_anime_onoff_flag != 0 {
                    self.wipe_end_at = None;
                } else {
                    self.wipe_end_at =
                        Some(Instant::now() + std::time::Duration::from_millis(duration_ms));
                }

                if wait_flag {
                    if let Some(deadline) = self.wipe_end_at {
                        let key_skip_enabled = if key_wait_mode == 0 {
                            false
                        } else if key_wait_mode == 1 {
                            true
                        } else {
                            self.skip_wipe_anime_onoff_flag != 0
                        };
                        let mut wipe_completed = false;
                        while Instant::now() < deadline {
                            if host.should_interrupt() {
                                break;
                            }
                            if key_skip_enabled && host.should_skip_wait() {
                                wipe_completed = true;
                                break;
                            }
                            let left = deadline.saturating_duration_since(Instant::now());
                            std::thread::sleep(left.min(std::time::Duration::from_millis(16)));
                        }
                        if Instant::now() >= deadline {
                            wipe_completed = true;
                        }
                        if wipe_completed {
                            self.wipe_end_at = None;
                        }
                    }
                }

                // Let host observe this command too (for GUI-side wipe animation).
                return Ok(Some(false));
            }
            x if crate::elm::global::is_wipe_end(x) => {
                self.wipe_end_at = None;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_wait_wipe(x) => {
                let mut skipped_by_key = false;
                if let Some(deadline) = self.wipe_end_at {
                    let key_wait_mode = args
                        .iter()
                        .find(|p| p.id == 0)
                        .and_then(|p| match p.value {
                            PropValue::Int(v) => Some(v),
                            _ => None,
                        })
                        .unwrap_or(-1);
                    let key_skip_enabled = if key_wait_mode == 0 {
                        false
                    } else if key_wait_mode == 1 {
                        true
                    } else {
                        // C++ key_wait_mode==-1 follows system.skip_wipe_anime_flag.
                        self.skip_wipe_anime_onoff_flag != 0
                    };

                    let mut wipe_completed = false;
                    while Instant::now() < deadline {
                        if host.should_interrupt() {
                            break;
                        }
                        if key_skip_enabled && host.should_skip_wait() {
                            skipped_by_key = true;
                            wipe_completed = true;
                            break;
                        }
                        let left = deadline.saturating_duration_since(Instant::now());
                        std::thread::sleep(left.min(std::time::Duration::from_millis(16)));
                    }
                    if Instant::now() >= deadline {
                        wipe_completed = true;
                    }
                    if wipe_completed {
                        self.wipe_end_at = None;
                    }
                }
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(if skipped_by_key { 1 } else { 0 });
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_check_wipe(x) => {
                let active = match self.wipe_end_at {
                    Some(deadline) => {
                        if Instant::now() >= deadline {
                            self.wipe_end_at = None;
                            false
                        } else {
                            true
                        }
                    }
                    None => false,
                };
                // C++ cmd_global.cpp::ELM_GLOBAL_CHECK_WIPE always pushes int.
                self.stack.push_int(if active { 1 } else { 0 });
                return Ok(Some(true));
            }
            x if x == crate::elm::global::ELM_GLOBAL_CLEAR_MSGBK => {
                // C++ reference: cmd_global.cpp::ELM_GLOBAL_CLEAR_MSGBK -> tnm_msg_back_clear().
                // Clearing msg_back history must immediately affect syscom availability checks.
                self.msg_back_has_message = 0;
                // Keep host-side UI/event behavior for CLEAR_MSGBK.
                return Ok(Some(false));
            }
            x if x == crate::elm::global::ELM_GLOBAL_INSERT_MSGBK_IMG => {
                // C++ reference: cmd_global.cpp::ELM_GLOBAL_INSERT_MSGBK_IMG -> tnm_msg_back_add_pct().
                // Adding an entry to msg_back should make history "exist" for syscom checks.
                self.msg_back_has_message = 1;
                // Keep host-side rendering behavior for inserted msgbk image.
                return Ok(Some(false));
            }
            // -----------------------------------------------------------------
            // Message window commands (pass through to Host; VM accepts them)
            // -----------------------------------------------------------------
            x if crate::elm::global::is_message_window_passthrough(x) => {
                // These are UI commands. Let them fall through to host.on_command().
                // We return false so the outer dispatch calls host.
                return Ok(Some(false));
            }
            // -----------------------------------------------------------------
            // Stage / Back / Front / Next commands (routed through command_stage)
            // -----------------------------------------------------------------
            x if x == crate::elm::global::ELM_GLOBAL_STAGE => {
                if self.try_command_stage_list(&element[1..], arg_list_id, args, ret_form, host) {
                    return Ok(Some(true));
                }
                return Ok(Some(false));
            }
            x if x == crate::elm::global::ELM_GLOBAL_BACK => {
                // Shortcut for stage[TNM_STAGE_BACK=0]
                if self.try_command_stage(&element[1..], arg_list_id, args, ret_form, host) {
                    return Ok(Some(true));
                }
                return Ok(Some(false));
            }
            x if x == crate::elm::global::ELM_GLOBAL_FRONT => {
                // Shortcut for stage[TNM_STAGE_FRONT=1]
                if self.try_command_stage(&element[1..], arg_list_id, args, ret_form, host) {
                    return Ok(Some(true));
                }
                return Ok(Some(false));
            }
            x if x == crate::elm::global::ELM_GLOBAL_NEXT => {
                // Shortcut for stage[TNM_STAGE_NEXT=2]
                if self.try_command_stage(&element[1..], arg_list_id, args, ret_form, host) {
                    return Ok(Some(true));
                }
                return Ok(Some(false));
            }
            // -----------------------------------------------------------------
            // Screen / Effect / Quake commands (routed through command_effect)
            // -----------------------------------------------------------------
            x if x == crate::elm::global::ELM_GLOBAL_SCREEN => {
                if self.try_command_screen(&element[1..], arg_list_id, args, ret_form, host) {
                    return Ok(Some(true));
                }
                return Ok(Some(false));
            }
            // -----------------------------------------------------------------
            // Sound / KOE / BGM / PCM / SE / MOV stubs
            // -----------------------------------------------------------------
            x if crate::elm::global::is_sound_passthrough(x) => {
                // Sound commands: route through dedicated command_sound module.
                if self.try_command_sound(element, arg_list_id, args, ret_form, host) {
                    return Ok(Some(true));
                }
                // Unhandled sub-command: fall through to host.
                return Ok(Some(false));
            }
            x if crate::elm::global::is_koe_get_volume(x) => {
                // C++ cmd_global.cpp::ELM_GLOBAL_KOE_GET_VOLUME always pushes int.
                self.stack.push_int(100); // default volume
                return Ok(Some(true));
            }
            x if crate::elm::global::is_koe_check(x) => {
                // C++ cmd_global.cpp::ELM_GLOBAL_KOE_CHECK always pushes int.
                self.stack.push_int(0); // not playing
                return Ok(Some(true));
            }
            x if crate::elm::global::is_koe_check_pair(x) => {
                // C++ cmd_global.cpp::ELM_GLOBAL_KOE_CHECK_GET_* always pushes int.
                self.stack.push_int(-1);
                return Ok(Some(true));
            }
            x if crate::elm::global::is_koe_check_is_ex(x) => {
                // C++ cmd_global.cpp::ELM_GLOBAL_KOE_CHECK_IS_EX_KOE always pushes int.
                self.stack.push_int(0);
                return Ok(Some(true));
            }
            x if crate::elm::global::is_selbtn_family(x) => {
                // STUB(C++: cmd_global.cpp ELM_GLOBAL_SELBTN/_READY/_CANCEL/_START):
                // - Gap: selection UI flow (ready/start, return flag wiring, last_sel_msg update)
                //   is not implemented in headless VM yet.
                // - Expected behavior: execute selection UI side effects without pushing
                //   synthetic return values on the VM stack.
                // - Minimal validation direction: run a scene with SELBTN->SELBTN_START and
                //   verify stack depth stability plus branch flow parity against C++.
                return Ok(Some(true));
            }
            x if crate::elm::global::is_iapp_dummy(x) => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                } else if ret_form == crate::elm::form::STR {
                    self.stack.push_str(String::new());
                }
                return Ok(Some(true));
            }
            x if crate::elm::global::is_capture_stub_extra(x) => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                return Ok(Some(true));
            }
            // -----------------------------------------------------------------
            // Input / Mouse / Key / Script / Syscom / System (pass through)
            // -----------------------------------------------------------------
            x if crate::elm::global::is_host_passthrough_root(x) => {
                return Ok(Some(false));
            }
            // C++ cmd_global.cpp default branch reports a fatal invalid-global-command error.
            // Keep command accepted at VM dispatch layer, but surface the same error text and
            // avoid synthetic return pushes that would hide parity issues.
            x if crate::elm::global::is_any_global_element(x) => {
                let _ = ret_form;
                host.on_error("無効なコマンドが指定されました。(global)");
                return Ok(Some(true));
            }
            _ => {}
        }
        Ok(None)
    }
}
