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
                let _ = self.event_tx.send(HostEvent::StartWipe { duration_ms });
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
