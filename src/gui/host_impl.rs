include!("host_impl_input.rs");
include!("host_stage_hit_test.rs");
include!("host_impl_stage_object.rs");
include!("host_impl_syscom_capture.rs");

impl GuiHost {
    fn build_vm_error_context(&self) -> VmErrorContext {
        VmErrorContext {
            scene: self.vm_scene.clone(),
            line_no: self.vm_line_no,
            pc: self.vm_pc,
            element: self.vm_element.clone(),
        }
    }

    fn parse_selbtn_request(
        &self,
        elm: i32,
        args: &[siglus::vm::Prop],
    ) -> SelectionRequest {
        let mut req = SelectionRequest::default();
        let positional: Vec<&siglus::vm::Prop> = args.iter().filter(|p| p.id == 0).collect();
        let mut idx = 0usize;
        if matches!(
            positional.first().map(|p| &p.value),
            Some(siglus::vm::PropValue::Int(_))
        ) {
            idx = 1; // template_no
        }

        let mut pending: Option<SelectionOption> = None;
        let mut item_arg_no = 0;
        while idx < positional.len() {
            match &positional[idx].value {
                siglus::vm::PropValue::Str(text) => {
                    if let Some(prev) = pending.take() {
                        req.options.push(prev);
                    }
                    pending = Some(SelectionOption {
                        text: text.clone(),
                        item_type: 1,
                        color: -1,
                    });
                    item_arg_no = 0;
                }
                siglus::vm::PropValue::Int(v) => {
                    if let Some(cur) = pending.as_mut() {
                        match item_arg_no {
                            1 => cur.item_type = *v,
                            2 => cur.color = *v,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            item_arg_no += 1;
            idx += 1;
        }
        if let Some(prev) = pending.take() {
            req.options.push(prev);
        }
        let mut named = SelBtnNamedArgs::default();
        named.read_flag_scene = self.vm_scene.clone();
        named.read_flag_line_no = self.vm_line_no;
        named.cancel_enable = matches!(
            elm,
            siglus::elm::global::ELM_GLOBAL_SELBTN_CANCEL
                | siglus::elm::global::ELM_GLOBAL_SELBTN_CANCEL_READY
        );
        for p in args.iter().filter(|p| p.id > 0) {
            match p.id {
                1 => {
                    if let siglus::vm::PropValue::Int(v) = p.value {
                        named.capture_flag = v != 0;
                    }
                }
                2 => {
                    if let siglus::vm::PropValue::Str(ref v) = p.value {
                        named.sel_start_call_scn = v.clone();
                    }
                }
                3 => {
                    if let siglus::vm::PropValue::Int(v) = p.value {
                        named.sel_start_call_z_no = v;
                    }
                }
                4 => {
                    if let siglus::vm::PropValue::Int(v) = p.value {
                        named.sync_type = v;
                    }
                }
                5 => {
                    if let siglus::vm::PropValue::Str(ref v) = p.value {
                        named.read_flag_scene = v.clone();
                    }
                }
                6 => {
                    if let siglus::vm::PropValue::Int(v) = p.value {
                        named.read_flag_line_no = v;
                    }
                }
                _ => {}
            }
        }
        if matches!(
            elm,
            siglus::elm::global::ELM_GLOBAL_SELBTN
                | siglus::elm::global::ELM_GLOBAL_SELBTN_READY
                | siglus::elm::global::ELM_GLOBAL_SELBTN_CANCEL
                | siglus::elm::global::ELM_GLOBAL_SELBTN_CANCEL_READY
                | siglus::elm::global::ELM_GLOBAL_SELBTN_START
        ) {
            req.selbtn = Some(named);
        }
        req
    }

    fn cache_selbtn_ready_options(&mut self, elm: i32, args: &[siglus::vm::Prop]) {
        let req = self.parse_selbtn_request(elm, args);
        let sync_type = req.selbtn.as_ref().map(|v| v.sync_type).unwrap_or(0);
        self.emit_selbtn_sync_checkpoint(sync_type, req.selbtn.as_ref().map(|v| v.cancel_enable).unwrap_or(false), "ready_cached", req.options.len(), None);
        self.pending_selbtn_request = Some(req);
    }

    fn resolve_selbtn_start_request(&self, elm: i32, args: &[siglus::vm::Prop]) -> SelectionRequest {
        let mut req = self.parse_selbtn_request(elm, args);
        if req.options.is_empty() {
            if let Some(prev) = &self.pending_selbtn_request {
                req.options = prev.options.clone();
                if req.selbtn.is_none() {
                    req.selbtn = prev.selbtn.clone();
                } else if let (Some(cur), Some(old)) = (req.selbtn.as_mut(), prev.selbtn.as_ref()) {
                    if cur.sel_start_call_scn.is_empty() {
                        cur.sel_start_call_scn = old.sel_start_call_scn.clone();
                    }
                    if cur.sel_start_call_z_no < 0 {
                        cur.sel_start_call_z_no = old.sel_start_call_z_no;
                    }
                    if cur.sync_type == 0 {
                        cur.sync_type = old.sync_type;
                    }
                    if !cur.capture_flag {
                        cur.capture_flag = old.capture_flag;
                    }
                    if !cur.cancel_enable {
                        cur.cancel_enable = old.cancel_enable;
                    }
                    if cur.read_flag_scene.is_empty() {
                        cur.read_flag_scene = old.read_flag_scene.clone();
                    }
                    if cur.read_flag_line_no < 0 {
                        cur.read_flag_line_no = old.read_flag_line_no;
                    }
                }
            }
        }
        req
    }

    fn emit_selbtn_sync_checkpoint(
        &mut self,
        sync_type: i32,
        cancel_enable: bool,
        phase: &'static str,
        option_count: usize,
        selected: Option<i32>,
    ) {
        let _ = self.event_tx.send(HostEvent::SelBtnSyncCheckpoint {
            sync_type,
            cancel_enable,
            phase,
            option_count,
            selected,
        });
    }

    fn run_selection_wait(&mut self, req: SelectionRequest) -> i32 {
        let sync_type = req.selbtn.as_ref().map(|v| v.sync_type).unwrap_or(-1);
        let cancel_enable = req.selbtn.as_ref().map(|v| v.cancel_enable).unwrap_or(false);
        let option_count = req.options.len();
        if sync_type >= 0 {
            self.emit_selbtn_sync_checkpoint(sync_type, cancel_enable, "wait_enter", option_count, None);
        }
        let _ = self.event_tx.send(HostEvent::Selection(req));
        let selected = self.selection_rx.recv().unwrap_or(0);
        if sync_type >= 0 {
            self.emit_selbtn_sync_checkpoint(sync_type, cancel_enable, "choice_received", option_count, Some(selected));
            match sync_type {
                0 => self.emit_selbtn_sync_checkpoint(sync_type, cancel_enable, "sync0_close_end", option_count, Some(selected)),
                1 => self.emit_selbtn_sync_checkpoint(sync_type, cancel_enable, "sync1_close_start", option_count, Some(selected)),
                2 => self.emit_selbtn_sync_checkpoint(sync_type, cancel_enable, "sync2_decide", option_count, Some(selected)),
                _ => self.emit_selbtn_sync_checkpoint(sync_type, cancel_enable, "sync_unknown", option_count, Some(selected)),
            }
        }
        if let Some(selbtn) = req.selbtn.as_ref() {
            if selected < 0 && selbtn.cancel_enable {
                self.emit_selbtn_sync_checkpoint(sync_type.max(0), true, "cancel_input", option_count, Some(selected));
                self.emit_selbtn_sync_checkpoint(sync_type.max(0), true, "cancel_complete", option_count, Some(selected));
            }
            if selected >= 0 {
                self.emit_selbtn_sync_checkpoint(sync_type.max(0), cancel_enable, "read_flag_mark", option_count, Some(selected));
                self.emit_selbtn_sync_checkpoint(sync_type.max(0), cancel_enable, "read_flag_complete", option_count, Some(selected));
            } else {
                self.emit_selbtn_sync_checkpoint(sync_type.max(0), cancel_enable, "read_flag_skip_cancel", option_count, Some(selected));
            }
            if selbtn.capture_flag && selected >= 0 {
                self.emit_selbtn_sync_checkpoint(sync_type.max(0), cancel_enable, "capture_requested", option_count, Some(selected));
                self.emit_selbtn_sync_checkpoint(sync_type.max(0), cancel_enable, "capture_finished", option_count, Some(selected));
                if !selbtn.sel_start_call_scn.is_empty() && selbtn.sel_start_call_z_no >= 0 {
                    self.emit_selbtn_sync_checkpoint(
                        sync_type.max(0),
                        cancel_enable,
                        "sel_start_call_queued",
                        option_count,
                        Some(selected),
                    );
                }
            }
        }
        selected
    }
}

impl siglus::vm::Host for GuiHost {
    fn on_name(&mut self, name: &str) {
        let _ = self.event_tx.send(HostEvent::Name(name.to_string()));
    }
    fn on_text(&mut self, text: &str, _read_flag_no: i32) {
        let _ = self.event_tx.send(HostEvent::Text {
            text: text.to_string(),
        });

        // If skip mode is off, wait for user click to advance
        if !self.skip_mode.load(Ordering::Relaxed) {
            loop {
                if self.shutdown.load(Ordering::Relaxed) {
                    return;
                }
                match self
                    .advance_rx
                    .recv_timeout(std::time::Duration::from_millis(50))
                {
                    Ok(AdvanceSignal::Proceed) => break,
                    Ok(AdvanceSignal::Shutdown) => return,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Check skip in case it was toggled while waiting
                        if self.skip_mode.load(Ordering::Relaxed) {
                            break;
                        }
                        continue;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                }
            }
        }
    }
    fn on_command(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[siglus::vm::Prop],
        _named_arg_cnt: i32,
        ret_form: i32,
    ) -> siglus::vm::HostReturn {
        self.vm_element = element.to_vec();

        if is_visual_or_flow_command(element) {
            debug!(
                "VM command: element={:?} args={} ret_form={}",
                element,
                summarize_props(args),
                ret_form
            );
        }

        if let Some(&elm) = element.first() {
            if matches!(
                elm,
                siglus::elm::global::ELM_GLOBAL_OPEN
                    | siglus::elm::global::ELM_GLOBAL_OPEN_NOWAIT
                    | siglus::elm::global::ELM_GLOBAL_OPEN_WAIT
            ) {
                let _ = self.event_tx.send(HostEvent::MessageWindowVisible(true));
            } else if matches!(
                elm,
                siglus::elm::global::ELM_GLOBAL_CLOSE
                    | siglus::elm::global::ELM_GLOBAL_CLOSE_NOWAIT
                    | siglus::elm::global::ELM_GLOBAL_CLOSE_WAIT
            ) {
                let _ = self.event_tx.send(HostEvent::MessageWindowVisible(false));
            }

            const ELM_GLOBAL_SELBTN: i32 = 76;
            const ELM_GLOBAL_SELBTN_CANCEL: i32 = 77;
            const ELM_GLOBAL_SELBTN_READY: i32 = 126;
            const ELM_GLOBAL_SELBTN_START: i32 = 127;
            const ELM_GLOBAL_SELBTN_CANCEL_READY: i32 = 128;

            if element.len() == 1
                && matches!(
                    elm,
                    siglus::elm::global::ELM_GLOBAL_STAGE
                        | siglus::elm::global::ELM_GLOBAL_BACK
                        | siglus::elm::global::ELM_GLOBAL_FRONT
                        | siglus::elm::global::ELM_GLOBAL_NEXT
                )
            {
                // STAGE(file_name, transition_time, ...)
                // checking args
                let plane = if elm == siglus::elm::global::ELM_GLOBAL_FRONT {
                    StagePlane::Front
                } else if elm == siglus::elm::global::ELM_GLOBAL_NEXT {
                    StagePlane::Next
                } else {
                    // Keep previous behavior as default for STAGE/BACK.
                    StagePlane::Back
                };

                if let Some(arg) = args.first() {
                    if let siglus::vm::PropValue::Str(s) = &arg.value {
                        // Load image
                        match load_stage_like_cpp(&self.base_dir, &self.append_dirs, s, 0) {
                            Ok(img) => {
                                let _ = self.event_tx.send(HostEvent::LoadPlaneImage {
                                    stage: plane,
                                    image: Arc::new(img.clone()),
                                });
                                let _ = self.event_tx.send(HostEvent::LoadImage {
                                    image: Arc::new(img),
                                });
                            }
                            Err(e) => {
                                error!("Failed to load stage image {}: {}", s, e);
                                let _ = self.event_tx.send(HostEvent::VmError {
                                    level: VmErrorLevel::FileNotFound,
                                    message: format!("ファイル \"{}\" が見つかりません。(screen:{})", s, elm),
                                    context: self.build_vm_error_context(),
                                });
                                let _ = self.event_tx.send(HostEvent::MissingPlaneImage {
                                    stage: plane,
                                    name: s.clone(),
                                });
                            }
                        }
                    }
                }
                return siglus::vm::HostReturn::default();
            }

            if let Some((plane, object_index, cmd)) = parse_stage_object_command(element) {
                self.apply_object_command(plane, object_index, cmd, args);
                return siglus::vm::HostReturn::default();
            } else if looks_like_stage_object_path(element) {
                warn!(
                    "unhandled stage-object command path: element={:?} args={}",
                    element,
                    summarize_props(args)
                );
            }

            if let Some((plane, stage_cmd)) = parse_stage_plane_command(element) {
                self.apply_stage_plane_command(plane, stage_cmd, args);
                return siglus::vm::HostReturn::default();
            }

            if matches!(elm, ELM_GLOBAL_SELBTN_READY | ELM_GLOBAL_SELBTN_CANCEL_READY) {
                // C++ cmd_global.cpp: *_READY only prepares button-selection candidates.
                // Start/return wiring is triggered later by SELBTN_START.
                self.cache_selbtn_ready_options(elm, args);
                return siglus::vm::HostReturn::default();
            }

            if matches!(elm, ELM_GLOBAL_SELBTN | ELM_GLOBAL_SELBTN_CANCEL)
                && ret_form == siglus::elm::form::INT
            {
                // C++ cmd_global.cpp executes READY + START in one command for SELBTN/CANCEL.
                let req = self.resolve_selbtn_start_request(elm, args);
                let sync_type = req.selbtn.as_ref().map(|v| v.sync_type).unwrap_or(0);
                self.emit_selbtn_sync_checkpoint(sync_type, req.selbtn.as_ref().map(|v| v.cancel_enable).unwrap_or(false), "start_resolved", req.options.len(), None);
                if req.options.is_empty() {
                    return siglus::vm::HostReturn {
                        int: -1,
                        ..siglus::vm::HostReturn::default()
                    };
                }
                self.pending_selbtn_request = Some(req.clone());
                let selected = self.run_selection_wait(req);
                return siglus::vm::HostReturn {
                    int: selected,
                    ..siglus::vm::HostReturn::default()
                };
            }

            if elm == ELM_GLOBAL_SELBTN_START && ret_form == siglus::elm::form::INT {
                // C++ start path consumes previously prepared ready-state when no args are supplied.
                let req = self.resolve_selbtn_start_request(elm, args);
                let sync_type = req.selbtn.as_ref().map(|v| v.sync_type).unwrap_or(0);
                self.emit_selbtn_sync_checkpoint(sync_type, req.selbtn.as_ref().map(|v| v.cancel_enable).unwrap_or(false), "start_resolved", req.options.len(), None);
                if req.options.is_empty() {
                    return siglus::vm::HostReturn {
                        int: -1,
                        ..siglus::vm::HostReturn::default()
                    };
                }
                let selected = self.run_selection_wait(req);
                return siglus::vm::HostReturn {
                    int: selected,
                    ..siglus::vm::HostReturn::default()
                };
            }

            if matches!(
                elm,
                siglus::elm::global::ELM_GLOBAL_SEL
                    | siglus::elm::global::ELM_GLOBAL_SEL_CANCEL
                    | siglus::elm::global::ELM_GLOBAL_SELMSG
                    | siglus::elm::global::ELM_GLOBAL_SELMSG_CANCEL
            ) && ret_form == siglus::elm::form::INT
            {
                let req = SelectionRequest {
                    options: siglus::vm::extract_selection_options(args)
                        .into_iter()
                        .map(|text| SelectionOption {
                            text,
                            item_type: 1,
                            color: -1,
                        })
                        .collect(),
                    selbtn: None,
                };
                let selected = self.run_selection_wait(req);
                return siglus::vm::HostReturn {
                    int: selected,
                    ..siglus::vm::HostReturn::default()
                };
            }

            if siglus::elm::global::is_wipe_start_command(elm) {
                // C++ source of truth: cmd_wipe.cpp::tnm_command_proc_wipe
                let duration_ms = parse_wipe_duration_from_cpp(elm, args);
                let wipe_type = parse_wipe_type_from_cpp(elm, args);
                let _ = self.event_tx.send(HostEvent::StartWipe {
                    duration_ms,
                    wipe_type,
                    wipe_direction: WipeDirection::Normal,
                });
                return siglus::vm::HostReturn::default();
            }
        }

        siglus::vm::HostReturn::default()
    }
    fn on_property(&mut self, _element: &[i32]) -> siglus::vm::HostReturn {
        siglus::vm::HostReturn::default()
    }
    fn on_assign(&mut self, element: &[i32], _al_id: i32, rhs: &siglus::vm::Prop) {
        if let Some((plane, object_index, prop)) = parse_stage_object_prop(element) {
            self.apply_object_assign(plane, object_index, prop, rhs);
        }
    }
    include!("host_impl_trace.rs");

    fn on_error(&mut self, msg: &str) {
        error!("VM Error: {}", msg);
        let _ = self.event_tx.send(HostEvent::VmError {
            level: VmErrorLevel::Fatal,
            message: msg.to_string(),
            context: self.build_vm_error_context(),
        });
    }
    fn on_error_fatal(&mut self, msg: &str) {
        error!("VM Fatal: {}", msg);
        let _ = self.event_tx.send(HostEvent::VmError {
            level: VmErrorLevel::Fatal,
            message: msg.to_string(),
            context: self.build_vm_error_context(),
        });
    }
    fn on_error_file_not_found(&mut self, msg: &str) {
        error!("VM FileNotFound: {}", msg);
        let _ = self.event_tx.send(HostEvent::VmError {
            level: VmErrorLevel::FileNotFound,
            message: msg.to_string(),
            context: self.build_vm_error_context(),
        });
    }
    fn on_resource_exists(&mut self, path: &str) -> bool {
        resource_exists_like_cpp(&self.base_dir, &self.append_dirs, path)
    }

    fn on_resource_exists_with_kind(&mut self, path: &str, kind: siglus::vm::VmResourceKind) -> bool {
        resource_exists_like_cpp_with_kind(&self.base_dir, &self.append_dirs, path, kind)
    }

    fn on_resource_read_text(&mut self, path: &str) -> Option<String> {
        read_text_like_cpp(&self.base_dir, &self.append_dirs, path)
    }
    fn on_script_fatal(&mut self, msg: &str) {
        // C++ flow_script.cpp fatal path pushes TNM_PROC_TYPE_NONE and stops script flow.
        // GUI host maps this to worker shutdown to stop VM loop deterministically.
        self.on_error(msg);
        self.shutdown.store(true, Ordering::Relaxed);
    }

    fn should_interrupt(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    fn should_skip_wait(&self) -> bool {
        self.skip_mode.load(Ordering::Relaxed)
    }
    fn on_wait_frame(&mut self) {
        self.refresh_movie_lifecycle();
        if self.shutdown.load(Ordering::Relaxed) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(8));
    }

    fn on_movie_is_playing(&mut self) -> bool {
        self.refresh_movie_lifecycle();
        !self.movie_playing_objects.is_empty() || self.global_mov_playing
    }

    fn on_mwnd_list_get_size(&mut self) -> i32 {
        self.mwnd_list_size
    }

    fn on_world_list_get_size(&mut self) -> i32 {
        self.world_list_size
    }

    fn on_world_create(&mut self) {
        self.world_list_size = self.world_list_size.saturating_add(1);
    }

    fn on_world_destroy(&mut self) {
        self.world_list_size = self.world_list_size.saturating_sub(1);
    }

    fn on_effect_list_get_size(&mut self) -> i32 {
        self.effect_list_size
    }

    fn on_effect_list_resize(&mut self, size: i32) {
        self.effect_list_size = size.max(0);
    }

    fn on_quake_list_get_size(&mut self) -> i32 {
        self.quake_list_size
    }

    fn on_quake_list_resize(&mut self, size: i32) {
        self.quake_list_size = size.max(0);
    }

    impl_host_quake_methods!();

    fn on_int_event_list_get_size(&mut self, _owner_id: i32) -> i32 {
        self.int_event_list_sizes
            .get(&_owner_id)
            .copied()
            .unwrap_or(1)
    }

    fn on_int_event_list_resize(&mut self, owner_id: i32, size: i32) {
        self.int_event_list_sizes.insert(owner_id, size.max(0));
    }
    include!("host_impl_int_event.rs");
    impl_host_input_methods!();
    fn on_msg_back_state(&mut self, open: bool) {
        let _ = self.event_tx.send(HostEvent::MsgBackState(open));
    }
    fn on_msg_back_display(&mut self, enabled: bool) {
        let _ = self
            .event_tx
            .send(HostEvent::MsgBackDisplayEnabled(enabled));
    }
    fn on_syscom_return_to_menu_warning(&mut self) -> bool {
        // C++ reference: eng_syscom.cpp::tnm_syscom_return_to_menu warning branch.
        // Block VM until GUI returns YES/NO equivalent.
        let _ = self.event_tx.send(HostEvent::ConfirmReturnToMenuWarning);
        loop {
            if self.shutdown.load(Ordering::Relaxed) {
                return true;
            }
            match self
                .return_to_menu_warning_rx
                .recv_timeout(std::time::Duration::from_millis(50))
            {
                Ok(v) => return v,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => return true,
            }
        }
    }
    fn on_syscom_return_to_sel_warning(&mut self) -> bool {
        // TODO(C++: eng_syscom.cpp::tnm_syscom_return_to_sel warning branch)
        // Missing reason: GUI currently only has dedicated return_to_menu warning dialog wiring.
        // Expected behavior: dedicated warning text + YES/NO path for return_to_sel.
        true
    }
    fn on_syscom_end_game_warning(&mut self) -> bool {
        // TODO(C++: eng_syscom.cpp::tnm_syscom_end_game warning branch)
        // Missing reason: GUI currently only has dedicated return_to_menu warning dialog wiring.
        // Expected behavior: dedicated warning text + YES/NO path for end_game.
        true
    }
    fn on_syscom_play_se(&mut self, kind: i32) {
        // STUB(C++: eng_syscom.cpp syscom SE types: MENU/PREV_SEL/END_GAME)
        // Missing reason: Rust GUI host has no menu/syscom SE playback path yet.
        // Expected behavior: play corresponding SE once when each syscom command is confirmed.
        info!("syscom requested se kind={} (stub)", kind);
    }
    fn on_syscom_proc_disp(&mut self) {
        // C++ reference: eng_syscom.cpp fade-out branches push TNM_PROC_TYPE_DISP.
        info!("syscom DISP proc");
    }
    fn on_syscom_proc_game_end_wipe(&mut self, wipe_type: i32, wipe_time_ms: u64) {
        // C++ reference: flow_proc.cpp::tnm_game_end_wipe_proc.
        let wipe_time_ms = wipe_time_ms.max(1);
        info!(
            "syscom game_end_wipe wipe_type={} wipe_time_ms={}",
            wipe_type, wipe_time_ms
        );
        let _ = self.event_tx.send(HostEvent::StartWipe {
            duration_ms: wipe_time_ms,
            wipe_type,
            wipe_direction: WipeDirection::SystemOut,
        });
        let started = std::time::Instant::now();
        while started.elapsed() < std::time::Duration::from_millis(wipe_time_ms) {
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    fn on_syscom_proc_game_start_wipe(&mut self, wipe_type: i32, wipe_time_ms: u64) {
        // TODO(C++: flow_proc.cpp::tnm_game_start_wipe_proc)
        // Missing reason: start/end wipe ranges are not separated in current renderer path.
        // Expected behavior: run SYSTEM_IN wipe semantics distinct from GAME_END_WIPE.
        let wipe_time_ms = wipe_time_ms.max(1);
        info!(
            "syscom game_start_wipe wipe_type={} wipe_time_ms={}",
            wipe_type, wipe_time_ms
        );
        let _ = self.event_tx.send(HostEvent::StartWipe {
            duration_ms: wipe_time_ms,
            wipe_type,
            wipe_direction: WipeDirection::SystemIn,
        });
        let started = std::time::Instant::now();
        while started.elapsed() < std::time::Duration::from_millis(wipe_time_ms) {
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    fn on_syscom_proc_return_to_sel(&mut self) {
        // TODO(C++: flow_proc.cpp::tnm_return_to_sel_proc -> saveload return path)
        // Missing reason: Rust VM currently restores sel snapshot best-effort without dedicated scene transition event.
        info!("syscom return_to_sel proc");
    }
    fn on_syscom_proc_end_game(&mut self) {
        // TODO(C++: flow_proc.cpp::tnm_end_game_proc)
        // Missing reason: GUI host lacks dedicated global game-end state pipeline.
        // Expected behavior: set game_end flags and trigger application-level termination flow.
        info!("syscom end_game proc");
    }
    fn on_syscom_proc_end_load_result(&mut self, ok: bool) {
        // C++ reference: flow_proc.cpp::tnm_end_load_proc invokes tnm_saveload_proc_end_load(),
        // but proc queue continues regardless; host can still observe actual restore success/failure.
        info!("syscom end_load restore result <- {}", ok);
    }
    fn on_syscom_load_flow_state(&mut self, state: siglus::vm::VmLoadFlowState) {
        // C++ reference: flow_proc.cpp load/return proc family updates these global flags,
        // consumed later by eng_frame.cpp::frame_action_proc.
        info!(
            "syscom load flow state <- wipe={} frame_action={} load_after_call={}",
            state.system_wipe_flag, state.do_frame_action_flag, state.do_load_after_call_flag
        );
    }
    fn on_syscom_end_save_snapshot(&mut self, slot_no: i32, state: &siglus::vm::VmEndSaveState) {
        let path = self
            .persistent_state_path
            .with_file_name(format!("siglus_end_save_{slot_no}.bin"));
        if let Err(e) = save_end_save_state(&path, state) {
            error!(
                "syscom end_save snapshot flush failed ({}): {:#}",
                path.display(),
                e
            );
        }
    }
    fn on_syscom_end_save_exist(&mut self, slot_no: i32) -> Option<bool> {
        let path = self
            .persistent_state_path
            .with_file_name(format!("siglus_end_save_{slot_no}.bin"));
        Some(path.exists())
    }
    fn on_syscom_end_load_snapshot(&mut self, slot_no: i32) -> Option<siglus::vm::VmEndSaveState> {
        let path = self
            .persistent_state_path
            .with_file_name(format!("siglus_end_save_{slot_no}.bin"));
        match load_end_save_state(&path) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    "syscom end_load snapshot read failed ({}): {:#}",
                    path.display(),
                    e
                );
                None
            }
        }
    }
    fn on_syscom_end_game_save_flush(&mut self, state: &siglus::vm::VmPersistentState) {
        // STUB(C++: eng_syscom.cpp::tnm_syscom_end_game -> tnm_syscom_end_save(false, false))
        // Current gap: still no dedicated C++-style end-save local slot file/capture pipeline.
        // Implemented now: flush persistent snapshot immediately to reduce callback-only drift.
        if let Err(e) = save_persistent_state(&self.persistent_state_path, state) {
            error!(
                "syscom end_game save flush failed ({}): {:#}",
                self.persistent_state_path.display(),
                e
            );
        } else {
            info!(
                "syscom end_game save flushed to {}",
                self.persistent_state_path.display()
            );
        }
    }
    fn on_syscom_return_to_menu_save_global(&mut self, state: &siglus::vm::VmPersistentState) {
        // C++ reference: eng_syscom.cpp::tnm_syscom_return_to_menu -> tnm_save_global_on_file().
        // Rust path: flush VM persistent snapshot immediately at return_to_menu trigger time.
        if let Err(e) = save_persistent_state(&self.persistent_state_path, state) {
            error!(
                "syscom return_to_menu immediate global save failed ({}): {:#}",
                self.persistent_state_path.display(),
                e
            );
        }
    }
    fn on_game_timer_move(&mut self, moving: bool) {
        // C++ reference: eng_syscom.cpp::tnm_syscom_return_to_menu +
        // flow_proc.cpp::tnm_game_timer_start_proc.
        // Rust has no separate game-timer proc queue yet; keep observable timer flag transition.
        info!("game_timer_move_flag <- {}", moving);
    }
    fn on_frame_action_load_after_call(&mut self, scene: &str, z_no: i32) {
        info!(
            "frame_action_proc: load_after_call farcall scene={} z={}",
            scene, z_no
        );
    }
    fn on_open_tweet_dialog(&mut self) {
        // C++ reference: cmd_syscom.cpp::ELM_SYSCOM_OPEN_TWEET_DIALOG -> tnm_twitter_start().
        // Rust currently opens a minimal placeholder dialog (no real tweet/upload pipeline yet).
        info!("syscom open_tweet_dialog requested (opening placeholder dialog)");
        let _ = self.event_tx.send(HostEvent::OpenTweetDialog);
    }
    fn on_location(&mut self, scene_title: &str, scene: &str, line_no: i32, pc: usize) {
        self.vm_scene = scene.to_string();
        self.vm_line_no = line_no;
        self.vm_pc = pc;
        let _ = self.event_tx.send(HostEvent::Location {
            scene_title: scene_title.to_string(),
            scene: scene.to_string(),
            line_no,
            pc,
        });
    }
    fn on_bgm_play(
        &mut self,
        name: &str,
        loop_flag: bool,
        _wait_flag: bool,
        fade_in: i32,
        _fade_out: i32,
        _start_pos: i32,
        _ready: bool,
    ) {
        let _ = self.event_tx.send(HostEvent::PlayBgm {
            name: name.to_string(),
            loop_flag,
            fade_in_ms: fade_in,
        });
    }
    fn on_bgm_stop(&mut self, fade_out: i32) {
        let _ = self.event_tx.send(HostEvent::StopBgm {
            fade_out_ms: fade_out,
        });
    }
    fn on_pcm_play(&mut self, name: &str) {
        let _ = self.event_tx.send(HostEvent::PlayPcm {
            ch: 0,
            name: name.to_string(),
            loop_flag: false,
        });
    }
    fn on_pcm_stop(&mut self) {
        let _ = self.event_tx.send(HostEvent::StopPcm { ch: 0 });
    }
    fn on_se_play(&mut self, _id: i32, name: &str) {
        let _ = self.event_tx.send(HostEvent::PlaySe {
            name: name.to_string(),
        });
    }
    fn on_se_stop(&mut self, _fade: i32) {
        let _ = self.event_tx.send(HostEvent::StopSe);
    }
    fn on_mov_play(&mut self, _name: &str) {
        self.global_mov_playing = true;
    }
    fn on_mov_stop(&mut self) {
        self.global_mov_playing = false;
    }
    fn on_pcmch_play(
        &mut self,
        ch: i32,
        name: &str,
        loop_flag: bool,
        _wait_flag: bool,
        _fade_in: i32,
        _volume_type: i32,
        _chara_no: i32,
        _ready: bool,
    ) {
        let _ = self.event_tx.send(HostEvent::PlayPcm {
            ch,
            name: name.to_string(),
            loop_flag,
        });
    }
    fn on_pcmch_stop(&mut self, ch: i32, _fade: i32) {
        let _ = self.event_tx.send(HostEvent::StopPcm { ch });
    }
    impl_host_stage_object_methods!();
    impl_host_syscom_capture_methods!();

}

fn parse_wipe_type_from_cpp(elm: i32, args: &[siglus::vm::Prop]) -> i32 {
    // C++ source of truth: cmd_wipe.cpp::tnm_command_proc_wipe
    // positional arg: WIPE(...): arg0, MASK_WIPE(...): arg1
    let default_pos = if elm == siglus::elm::global::ELM_GLOBAL_MASK_WIPE
        || elm == siglus::elm::global::ELM_GLOBAL_MASK_WIPE_ALL
    {
        1
    } else {
        0
    };
    let mut wipe_type = args.get(default_pos).and_then(|p| p.as_int()).unwrap_or(0);
    // named override: id=0
    for arg in args {
        if arg.id == 0 {
            if let Some(v) = arg.as_int() {
                wipe_type = v;
            }
            break;
        }
    }
    wipe_type
}

fn parse_wipe_duration_from_cpp(elm: i32, args: &[siglus::vm::Prop]) -> u64 {
    let mut wipe_time = 500i32;
    let mut start_time = 0i32;

    let time_pos = if elm == siglus::elm::global::ELM_GLOBAL_MASK_WIPE
        || elm == siglus::elm::global::ELM_GLOBAL_MASK_WIPE_ALL
    {
        2
    } else {
        1
    };
    if let Some(v) = args.get(time_pos).and_then(|p| p.as_int()) {
        wipe_time = v;
    }

    for arg in args {
        match arg.id {
            1 => {
                if let Some(v) = arg.as_int() {
                    wipe_time = v;
                }
            }
            11 => {
                if let Some(v) = arg.as_int() {
                    start_time = v;
                }
            }
            _ => {}
        }
    }

    (wipe_time as i64 - start_time as i64).max(0) as u64
}

// ── GUI Application ─────────────────────────────────────────────────────
