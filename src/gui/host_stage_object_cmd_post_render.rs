impl GuiHost {
    fn movie_wait_trace_enabled() -> bool {
        std::env::var("SIGLUS_MOVIE_WAIT_TRACE").map(|v| v != "0").unwrap_or(false)
    }

    fn wait_movie_gate(&mut self, plane: StagePlane, object_index: i32, key_skip: bool) {
        if Self::movie_wait_trace_enabled() {
            log::debug!("vm.movie_wait_trace gate_start stage={:?} index={} key_skip={} state={:?}", plane, object_index, key_skip, self.object_movie_wait_state(plane, object_index));
        }
        while !self.is_object_movie_wait_ready(plane, object_index) {
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }
            self.refresh_movie_lifecycle();
            if key_skip && self.consume_movie_wait_key_skip_stock() {
                if Self::movie_wait_trace_enabled() {
                    log::debug!("vm.movie_wait_trace key_skip_consumed stage={:?} index={} state={:?}", plane, object_index, self.object_movie_wait_state(plane, object_index));
                }
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(8));
        }
        if Self::movie_wait_trace_enabled() {
            log::debug!("vm.movie_wait_trace gate_end stage={:?} index={} key_skip={} state={:?}", plane, object_index, key_skip, self.object_movie_wait_state(plane, object_index));
        }
    }

    fn handle_object_command_post_render(
        &mut self,
        plane: StagePlane,
        object_index: i32,
        cmd: i32,
        args: &[siglus::vm::Prop],
    ) -> bool {
        let handled = matches!(
            cmd,
            x if x == siglus::elm::objectlist::ELM_OBJECT_FREE
                || x == siglus::elm::objectlist::ELM_OBJECT_INIT
                || x == siglus::elm::objectlist::ELM_OBJECT_RESUME_MOVIE
                || x == siglus::elm::objectlist::ELM_OBJECT_PAUSE_MOVIE
                || x == siglus::elm::objectlist::ELM_OBJECT_END_MOVIE_LOOP
                || x == siglus::elm::objectlist::ELM_OBJECT_WAIT_MOVIE
                || x == siglus::elm::objectlist::ELM_OBJECT_WAIT_MOVIE_KEY
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE_LOOP
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE_WAIT
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE_WAIT_KEY
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_MOVIE_AUTO_FREE
                || x == siglus::elm::objectlist::ELM_OBJECT_SEEK_MOVIE
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_WEATHER_PARAM_TYPE_A
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_WEATHER_PARAM_TYPE_B
                || x == siglus::elm::objectlist::ELM_OBJECT_CLEAR_BUTTON
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_GROUP
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_PUSHKEEP
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_ALPHA_TEST
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_STATE_NORMAL
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_STATE_SELECT
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_STATE_DISABLE
                || x == siglus::elm::objectlist::ELM_OBJECT_CLEAR_BUTTON_CALL
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_CALL
        );

        match cmd {
            x if x == siglus::elm::objectlist::ELM_OBJECT_FREE
                || x == siglus::elm::objectlist::ELM_OBJECT_INIT =>
            {
                // Lifecycle matrix: FREE/INIT starts a clean slot and must drop prior terminal movie flags.
                let st = self.get_or_create_object_state(plane, object_index);
                reset_object_state_preserve_seq(st);
                self.movie_playing_objects.remove(&(plane, object_index));
                self.movie_ready_objects.remove(&(plane, object_index));
                self.movie_generations.remove(&(plane, object_index));
                self.movie_auto_free_ms.remove(&(plane, object_index));
                self.clear_movie_terminal_state(plane, object_index);
                self.clear_object_string_state(plane, object_index);
                self.clear_object_string_style_state(plane, object_index);
                self.clear_object_number_state(plane, object_index);
                self.clear_object_number_style_state(plane, object_index);
                self.clear_object_button_state(plane, object_index);
                self.clear_object_weather_state(plane, object_index);
                self.clear_object_movie_seek_state(plane, object_index);
                let _ = self.event_tx.send(HostEvent::SetObjectVisible {
                    stage: plane,
                    index: object_index,
                    visible: false,
                });
                let _ = self.event_tx.send(HostEvent::RemoveObject {
                    stage: plane,
                    index: object_index,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_RESUME_MOVIE => {
                // Lifecycle matrix: RESUME dispatches a fresh generation; clear stale terminal snapshots first.
                self.movie_playing_objects.insert((plane, object_index));
                self.movie_ready_objects.remove(&(plane, object_index));
                let duration_ms = self
                    .movie_auto_free_ms
                    .get(&(plane, object_index))
                    .copied()
                    .unwrap_or(3_000)
                    .max(1);
                let generation = self.next_movie_generation;
                self.next_movie_generation = self.next_movie_generation.saturating_add(1);
                self.movie_generations
                    .insert((plane, object_index), generation);
                self.clear_movie_terminal_state(plane, object_index);
                let file_name = self
                    .objects
                    .get(&(plane, object_index))
                    .map(|o| o.file_name.clone())
                    .unwrap_or_default();
                let _ = self.event_tx.send(HostEvent::PlayObjectMovie {
                    stage: plane,
                    index: object_index,
                    file_name,
                    duration_ms,
                    generation,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_PAUSE_MOVIE
                || x == siglus::elm::objectlist::ELM_OBJECT_END_MOVIE_LOOP =>
            {
                self.movie_playing_objects.remove(&(plane, object_index));
                self.movie_ready_objects.remove(&(plane, object_index));
                let generation = self
                    .movie_generations
                    .remove(&(plane, object_index))
                    .unwrap_or(0);
                let _ = self.event_tx.send(HostEvent::StopObjectMovie {
                    stage: plane,
                    index: object_index,
                    generation,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_WAIT_MOVIE => {
                self.wait_movie_gate(plane, object_index, false);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_WAIT_MOVIE_KEY => {
                self.wait_movie_gate(plane, object_index, true);
            }
            // Lifecycle matrix: CREATE_MOVIE* dispatches a fresh generation; terminal state must be reset.
            x if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE_LOOP
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE_WAIT
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE_WAIT_KEY =>
            {
                let file_name = args
                    .first()
                    .and_then(|p| match &p.value {
                        siglus::vm::PropValue::Str(v) => Some(v.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let resolved_name = {
                    let state = self.get_or_create_object_state(plane, object_index);
                    if !file_name.is_empty() {
                        state.file_name = file_name;
                    }
                    state.file_name.clone()
                };
                let duration_ms = self
                    .movie_auto_free_ms
                    .get(&(plane, object_index))
                    .copied()
                    .unwrap_or(3_000)
                    .max(1);
                let generation = self.next_movie_generation;
                self.next_movie_generation = self.next_movie_generation.saturating_add(1);
                self.movie_generations
                    .insert((plane, object_index), generation);
                self.movie_playing_objects.insert((plane, object_index));
                self.movie_ready_objects.remove(&(plane, object_index));
                self.clear_movie_terminal_state(plane, object_index);
                let _ = self.event_tx.send(HostEvent::PlayObjectMovie {
                    stage: plane,
                    index: object_index,
                    file_name: resolved_name,
                    duration_ms,
                    generation,
                });
                if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE_WAIT {
                    self.wait_movie_gate(plane, object_index, false);
                } else if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE_WAIT_KEY {
                    self.wait_movie_gate(plane, object_index, true);
                }
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_MOVIE_AUTO_FREE => {
                let ms = args
                    .first()
                    .and_then(|p| p.as_int())
                    .unwrap_or(3_000)
                    .max(1);
                self.movie_auto_free_ms.insert((plane, object_index), ms);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SEEK_MOVIE => {
                let seek = args.first().and_then(|p| p.as_int()).unwrap_or(0);
                self.set_object_movie_seek_state(plane, object_index, seek.max(0));
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_WEATHER_PARAM_TYPE_A
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_WEATHER_PARAM_TYPE_B =>
            {
                let mut raw = Vec::with_capacity(args.len());
                for a in args {
                    raw.push(a.as_int().unwrap_or(0));
                }
                self.set_object_weather_state(
                    plane,
                    object_index,
                    ObjectWeatherState {
                        last_type: cmd,
                        params: raw,
                    },
                );
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CLEAR_BUTTON => {
                self.clear_object_button_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON => {
                let mut st = self.get_object_button_state(plane, object_index);
                st.button_no = args.first().and_then(|p| p.as_int()).unwrap_or(0);
                st.group_no = args.get(1).and_then(|p| p.as_int()).unwrap_or(0);
                st.action_no = args.get(2).and_then(|p| p.as_int()).unwrap_or(0);
                st.se_no = args.get(3).and_then(|p| p.as_int()).unwrap_or(0);
                self.set_object_button_state(plane, object_index, st);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_GROUP => {
                let mut st = self.get_object_button_state(plane, object_index);
                st.group_no = args.first().and_then(|p| p.as_int()).unwrap_or(st.group_no);
                self.set_object_button_state(plane, object_index, st);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_PUSHKEEP => {
                let mut st = self.get_object_button_state(plane, object_index);
                st.push_keep = args.first().and_then(|p| p.as_int()).unwrap_or(0);
                self.set_object_button_state(plane, object_index, st);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_ALPHA_TEST => {
                let mut st = self.get_object_button_state(plane, object_index);
                st.alpha_test = args.first().and_then(|p| p.as_int()).unwrap_or(0);
                self.set_object_button_state(plane, object_index, st);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_STATE_NORMAL => {
                let mut st = self.get_object_button_state(plane, object_index);
                st.state = 0;
                st.real_state = 0;
                self.set_object_button_state(plane, object_index, st);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_STATE_SELECT => {
                let mut st = self.get_object_button_state(plane, object_index);
                st.state = 1;
                st.real_state = 1;
                self.set_object_button_state(plane, object_index, st);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_STATE_DISABLE => {
                let mut st = self.get_object_button_state(plane, object_index);
                st.state = 2;
                st.real_state = 2;
                self.set_object_button_state(plane, object_index, st);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CLEAR_BUTTON_CALL
                || x == siglus::elm::objectlist::ELM_OBJECT_SET_BUTTON_CALL => {}
            _ => {}
        }
        handled
    }
}
