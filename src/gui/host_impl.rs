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

            if matches!(
                elm,
                siglus::elm::global::ELM_GLOBAL_SEL
                    | siglus::elm::global::ELM_GLOBAL_SEL_CANCEL
                    | siglus::elm::global::ELM_GLOBAL_SELMSG
                    | siglus::elm::global::ELM_GLOBAL_SELMSG_CANCEL
                    | siglus::elm::global::ELM_GLOBAL_SELBTN_START
                    | ELM_GLOBAL_SELBTN
                    | ELM_GLOBAL_SELBTN_CANCEL
            ) && ret_form == siglus::elm::form::INT
            {
                let is_button_select = matches!(
                    elm,
                    siglus::elm::global::ELM_GLOBAL_SELBTN_START
                        | ELM_GLOBAL_SELBTN
                        | ELM_GLOBAL_SELBTN_CANCEL
                );
                let options = siglus::vm::extract_selection_options(args);
                if is_button_select && options.is_empty() {
                    // Align with Siglus button-selection polling behavior:
                    // when no concrete choice is made yet, return -1 instead
                    // of auto-selecting the first entry.
                    return siglus::vm::HostReturn {
                        int: -1,
                        ..siglus::vm::HostReturn::default()
                    };
                }
                let _ = self.event_tx.send(HostEvent::Selection(options));
                let selected = self.selection_rx.recv().unwrap_or(0);
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

    fn on_trace(&mut self, _msg: &str) {}

    fn on_error(&mut self, msg: &str) {
        // Log error to file instead of showing in UI
        error!("VM Error: {}", msg);
        // We still send it to UI thread if we want to handle it there (e.g. flash taskbar?)
        // but for now let's just log it. The user specifically asked to remove on-screen error.
        // let _ = self.event_tx.send(HostEvent::Error(msg.to_string()));
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

    fn on_msg_back_state(&mut self, open: bool) {
        let _ = self.event_tx.send(HostEvent::MsgBackState(open));
    }

    fn on_msg_back_display(&mut self, enabled: bool) {
        let _ = self.event_tx.send(HostEvent::MsgBackDisplayEnabled(enabled));
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
        info!("syscom game_end_wipe wipe_type={} wipe_time_ms={}", wipe_type, wipe_time_ms);
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
            error!("syscom end_save snapshot flush failed ({}): {:#}", path.display(), e);
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
                error!("syscom end_load snapshot read failed ({}): {:#}", path.display(), e);
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

    fn on_location(&mut self, scene_title: &str, scene: &str, line_no: i32) {
        let _ = self.event_tx.send(HostEvent::Location {
            scene_title: scene_title.to_string(),
            scene: scene.to_string(),
            line_no,
        });
    }

    fn on_bgm_play(&mut self, name: &str, loop_flag: bool, _wait_flag: bool, fade_in: i32, _fade_out: i32, _start_pos: i32, _ready: bool) {
        let _ = self.event_tx.send(HostEvent::PlayBgm { name: name.to_string(), loop_flag, fade_in_ms: fade_in });
    }
    fn on_bgm_stop(&mut self, fade_out: i32) {
        let _ = self.event_tx.send(HostEvent::StopBgm { fade_out_ms: fade_out });
    }
    fn on_pcm_play(&mut self, name: &str) {
        let _ = self.event_tx.send(HostEvent::PlayPcm { ch: 0, name: name.to_string(), loop_flag: false });
    }
    fn on_pcm_stop(&mut self) {
        let _ = self.event_tx.send(HostEvent::StopPcm { ch: 0 });
    }
    fn on_se_play(&mut self, _id: i32, name: &str) {
        let _ = self.event_tx.send(HostEvent::PlaySe { name: name.to_string() });
    }
    fn on_se_stop(&mut self, _fade: i32) {
        let _ = self.event_tx.send(HostEvent::StopSe);
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
        let _ = self.event_tx.send(HostEvent::PlayPcm { ch, name: name.to_string(), loop_flag });
    }
    fn on_pcmch_stop(&mut self, ch: i32, _fade: i32) {
        let _ = self.event_tx.send(HostEvent::StopPcm { ch });
    }

    fn on_int_event_set(&mut self, owner_id: i32, start: i32, end: i32, time: i32, _delay: i32, realtime: i32, value_override: Option<i32>) {
        let actual_start = value_override.unwrap_or(start);
        if time <= 0 {
            self.int_events.remove(&owner_id);
        } else {
            let duration_ms = if realtime > 0 { time } else { time * 16 };
            self.int_events.insert(owner_id, IntEventState {
                start_val: actual_start,
                end_val: end,
                duration_ms,
                started_at: std::time::Instant::now(),
            });
        }
    }

    fn on_int_event_loop(&mut self, owner_id: i32, start: i32, end: i32, time: i32, _delay: i32, _count: i32, realtime: i32) {
        let duration_ms = if realtime > 0 { time } else { time * 16 };
        self.int_events.insert(owner_id, IntEventState {
            start_val: start,
            end_val: end,
            duration_ms: duration_ms.max(1),
            started_at: std::time::Instant::now(),
        });
    }

    fn on_int_event_turn(&mut self, owner_id: i32, start: i32, end: i32, time: i32, _delay: i32, _count: i32, realtime: i32) {
        let duration_ms = if realtime > 0 { time } else { time * 16 };
        self.int_events.insert(owner_id, IntEventState {
            start_val: start,
            end_val: end,
            duration_ms: duration_ms.max(1),
            started_at: std::time::Instant::now(),
        });
    }

    fn on_int_event_end(&mut self, owner_id: i32) {
        self.int_events.remove(&owner_id);
    }

    fn on_int_event_wait(&mut self, _owner_id: i32, _key_skip: bool) {}

    fn on_int_event_check(&mut self, owner_id: i32) -> bool {
        if let Some(state) = self.int_events.get(&owner_id) {
            state.started_at.elapsed().as_millis() < state.duration_ms as u128
        } else {
            false
        }
    }

    fn on_int_event_get_value(&mut self, owner_id: i32) -> i32 {
        if let Some(state) = self.int_events.get(&owner_id) {
            let elapsed = state.started_at.elapsed().as_millis() as i32;
            if elapsed >= state.duration_ms {
                return state.end_val;
            }
            let progress = elapsed as f32 / state.duration_ms as f32;
            state.start_val + ((state.end_val - state.start_val) as f32 * progress) as i32
        } else {
            0
        }
    }
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
