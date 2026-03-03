impl GuiHost {
    pub(super) fn drain_movie_playback_events(&mut self) {
        while let Ok(evt) = self.movie_event_rx.try_recv() {
            match evt {
                MoviePlaybackEvent::ObjectStarted {
                    stage,
                    index,
                    generation,
                } => {
                    if self.movie_generations.get(&(stage, index)).copied() == Some(generation) {
                        self.movie_playing_objects.insert((stage, index));
                        self.movie_ready_objects.insert((stage, index));
                        self.clear_movie_terminal_state(stage, index);
                    }
                }
                MoviePlaybackEvent::ObjectFinished {
                    stage,
                    index,
                    generation,
                } => {
                    if self.movie_generations.get(&(stage, index)).copied() == Some(generation) {
                        self.movie_playing_objects.remove(&(stage, index));
                        self.movie_ready_objects.remove(&(stage, index));
                        self.clear_movie_terminal_state(stage, index);
                    }
                }
                MoviePlaybackEvent::ObjectFailed {
                    stage,
                    index,
                    generation,
                    info,
                } => {
                    if self.movie_generations.get(&(stage, index)).copied() == Some(generation) {
                        self.movie_playing_objects.remove(&(stage, index));
                        self.movie_ready_objects.remove(&(stage, index));
                        self.movie_last_failure.insert((stage, index), info.clone());
                        self.movie_interrupted_objects.remove(&(stage, index));
                        log::warn!(
                            "movie failed stage={:?} index={} generation={} category={:?} backend={:?} unrecoverable={} detail={} counters=({}, {}, {})",
                            stage,
                            index,
                            generation,
                            info.category,
                            info.backend,
                            info.unrecoverable,
                            info.detail,
                            info.spawn_fail,
                            info.wait_fail,
                            info.exit_fail
                        );
                    }
                }
                MoviePlaybackEvent::ObjectInterrupted {
                    stage,
                    index,
                    generation,
                } => {
                    if self.movie_generations.get(&(stage, index)).copied() == Some(generation) {
                        self.movie_playing_objects.remove(&(stage, index));
                        self.movie_ready_objects.remove(&(stage, index));
                        self.movie_interrupted_objects.insert((stage, index));
                    }
                }
            }
        }
    }

    pub(super) fn play_cancel_se(&mut self, se_no: i32) {
        if se_no < 0 {
            return;
        }
        if let Some(mapped) = self.cancel_se_map.get(&se_no).cloned() {
            let _ = self.event_tx.send(HostEvent::PlaySe { name: mapped });
            return;
        }
        let candidates = [
            format!("SE_{:03}", se_no),
            format!("se_{:03}", se_no),
            format!("se{:03}", se_no),
            format!("sys_se_{:03}", se_no),
            format!("{:03}", se_no),
        ];
        let exts = ["ogg", "wav", "mp3", "flac"];
        for name in candidates {
            let found = exts.iter().any(|ext| {
                let direct = self.base_dir.join(format!("{name}.{ext}"));
                let se_dir = self.base_dir.join("SE").join(format!("{name}.{ext}"));
                direct.exists() || se_dir.exists()
            });
            if found {
                let _ = self.event_tx.send(HostEvent::PlaySe { name });
                return;
            }
        }
    }

    pub(super) fn refresh_movie_lifecycle(&mut self) {
        self.drain_movie_playback_events();
    }


    fn clear_movie_terminal_state(&mut self, plane: StagePlane, object_index: i32) {
        self.movie_last_failure.remove(&(plane, object_index));
        self.movie_interrupted_objects.remove(&(plane, object_index));
    }


    fn object_positional_int_args(args: &[siglus::vm::Prop]) -> Vec<i32> {
        args.iter()
            .filter(|p| p.id < 0)
            .map(|p| p.as_int().unwrap_or(0))
            .collect()
    }

    fn object_named_int_arg(args: &[siglus::vm::Prop], id: i32) -> Option<i32> {
        args.iter()
            .rev()
            .find(|p| p.id == id)
            .and_then(|p| p.as_int())
    }

    fn object_tail_value(args: &[siglus::vm::Prop], positional_idx: usize, named_id: i32, default: i32) -> i32 {
        let positional = Self::object_positional_int_args(args);
        let v = positional.get(positional_idx).copied().unwrap_or(default);
        Self::object_named_int_arg(args, named_id).unwrap_or(v)
    }

    fn object_movie_option_flags(args: &[siglus::vm::Prop]) -> (bool, bool, bool) {
        let auto_init = Self::object_named_int_arg(args, 0).unwrap_or(1) != 0;
        let real_time = Self::object_named_int_arg(args, 1).unwrap_or(1) != 0;
        let ready_only = Self::object_named_int_arg(args, 2).unwrap_or(0) != 0;
        (auto_init, real_time, ready_only)
    }



    pub(super) fn object_movie_wait_state(&self, plane: StagePlane, object_index: i32) -> MovieWaitState {
        let playing = self.movie_playing_objects.contains(&(plane, object_index));
        let ready_only = self
            .objects
            .get(&(plane, object_index))
            .map(|st| st.movie_ready_only)
            .unwrap_or(false);
        if playing {
            if !ready_only {
                return MovieWaitState::Pending;
            }
            if self.movie_ready_objects.contains(&(plane, object_index)) {
                return MovieWaitState::Ready;
            }
            return MovieWaitState::Pending;
        }
        if self.movie_last_failure.contains_key(&(plane, object_index)) {
            return MovieWaitState::Failed;
        }
        if self.movie_interrupted_objects.contains(&(plane, object_index)) {
            return MovieWaitState::Interrupted;
        }
        if self.movie_generations.contains_key(&(plane, object_index)) {
            // Pre-start gate: play command already dispatched but backend has not reported started/finished yet.
            return MovieWaitState::Pending;
        }
        MovieWaitState::Ready
    }

    fn object_check_movie_failed_code(&self, plane: StagePlane, object_index: i32) -> i32 {
        let Some(info) = self.movie_last_failure.get(&(plane, object_index)) else {
            return -1;
        };
        // Align with iapp selector category domain (1..5): map to -11..-15.
        -(10 + info.category_code())
    }

    fn is_object_movie_wait_ready(&self, plane: StagePlane, object_index: i32) -> bool {
        !matches!(
            self.object_movie_wait_state(plane, object_index),
            MovieWaitState::Pending
        )
    }

    fn consume_movie_wait_key_skip_stock(&self) -> bool {
        if let Ok(mut state) = self.input_state.lock() {
            if state.has_key_wait_press_stock() {
                let _ = state.consume_key_wait_press_stock();
                return true;
            }
        }
        false
    }

    pub(super) fn apply_object_command(
        &mut self,
        plane: StagePlane,
        object_index: i32,
        cmd: i32,
        args: &[siglus::vm::Prop],
    ) {
        self.refresh_movie_lifecycle();

        let arg_str = |idx: usize| -> Option<&str> {
            args.get(idx).and_then(|p| match &p.value {
                siglus::vm::PropValue::Str(v) => Some(v.as_str()),
                _ => None,
            })
        };

        match cmd {
            x if x == siglus::elm::objectlist::ELM_OBJECT_CREATE
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_NUMBER
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_WEATHER
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MESH
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_BILLBOARD =>
            {
                self.reset_object_runtime_state_for_create(plane, object_index);
                if let Some(file_name) = arg_str(0) {
                    if file_name.is_empty() {
                        return;
                    }
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.file_name = file_name.to_owned();
                    self.clear_object_string_state(plane, object_index);
                    let disp = Self::object_tail_value(args, 1, 0, 1) != 0;
                    let pos_x = Self::object_tail_value(args, 2, 1, 0) as f32;
                    let pos_y = Self::object_tail_value(args, 3, 2, 0) as f32;
                    let pat_no = Self::object_tail_value(args, 4, 3, 0).max(0) as usize;
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.visible = disp;
                    state.x = pos_x;
                    state.y = pos_y;
                    state.pat_no = pat_no;
                    self.emit_object_sort_and_visibility(plane, object_index);
                    self.emit_object_image(plane, object_index);
                }
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_STRING
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_SAVE_THUMB
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_CAPTURE_THUMB
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_RECT =>
            {
                self.reset_object_runtime_state_for_create(plane, object_index);
                if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_STRING {
                    let text = args
                        .first()
                        .and_then(|p| match &p.value { siglus::vm::PropValue::Str(v) => Some(v.as_str()), _ => None })
                        .unwrap_or("")
                        .to_owned();
                    self.set_object_string_state(plane, object_index, text.clone());
                    self.apply_create_tail_disp_xy_pat(plane, object_index, 1, 2, 3, None, args);
                    self.emit_generated_object_image(
                        plane,
                        object_index,
                        Self::build_string_raster_image(
                            &text,
                            &self.get_object_string_style_state(plane, object_index),
                        ),
                    );
                    return;
                }
                if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_RECT {
                    self.clear_object_string_state(plane, object_index);
                    self.apply_create_tail_disp_xy_pat(plane, object_index, 8, 9, 10, None, args);
                    self.emit_generated_object_image(
                        plane,
                        object_index,
                        Self::build_rect_image(args),
                    );
                    return;
                }
                self.clear_object_string_state(plane, object_index);
                self.apply_create_tail_disp_xy_pat(plane, object_index, 1, 2, 3, None, args);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_CAPTURE => {
                self.reset_object_runtime_state_for_create(plane, object_index);
                self.apply_create_tail_disp_xy_pat(plane, object_index, 0, 1, 2, None, args);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_EMOTE => {
                self.reset_object_runtime_state_for_create(plane, object_index);
                if let Some(file_name) = arg_str(2) {
                    if file_name.is_empty() {
                        return;
                    }
                    let rep_x = Self::object_named_int_arg(args, 0).unwrap_or(0);
                    let rep_y = Self::object_named_int_arg(args, 1).unwrap_or(0);
                    log::debug!("create_emote rep_pos=({}, {}) stage={:?} index={}", rep_x, rep_y, plane, object_index);
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.file_name = file_name.to_owned();
                    state.emote_rep_x = rep_x;
                    state.emote_rep_y = rep_y;
                    self.clear_object_string_state(plane, object_index);
                    self.apply_create_tail_disp_xy_pat(plane, object_index, 3, 4, 5, None, args);
                    self.emit_object_image(plane, object_index);
                }
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE_LOOP
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE_WAIT
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_MOVIE_WAIT_KEY =>
            {
                self.reset_object_runtime_state_for_create(plane, object_index);
                if let Some(file_name) = arg_str(0) {
                    if file_name.is_empty() {
                        return;
                    }
                    let (auto_init, real_time, ready_only) = Self::object_movie_option_flags(args);
                    log::debug!("create_movie opts auto_init={} real_time={} ready_only={} stage={:?} index={}", auto_init, real_time, ready_only, plane, object_index);
                    if std::env::var("SIGLUS_MOVIE_WAIT_TRACE").map(|v| v != "0").unwrap_or(false) {
                        log::debug!(
                            "vm.failed_code_trace create stage={:?} index={} auto_init={} real_time={} ready_only={}",
                            plane,
                            object_index,
                            auto_init,
                            real_time,
                            ready_only
                        );
                    }
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.file_name = file_name.to_owned();
                    state.movie_auto_init = auto_init;
                    state.movie_real_time = real_time;
                    state.movie_ready_only = ready_only;
                    self.clear_object_string_state(plane, object_index);
                    self.apply_create_tail_disp_xy_pat(plane, object_index, 1, 2, 3, None, args);
                    self.emit_object_image(plane, object_index);
                }
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CHANGE_FILE => {
                if let Some(siglus::vm::Prop {
                    value: siglus::vm::PropValue::Str(file_name),
                    ..
                }) = args.first()
                {
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.file_name = file_name.clone();
                    self.clear_object_string_state(plane, object_index);
                    self.emit_object_image(plane, object_index);
                }
            }
            // Some scripts emit object.create as command id 38 (not exposed in constants.rs).
            38 => {
                if let Some(siglus::vm::Prop {
                    value: siglus::vm::PropValue::Str(file_name),
                    ..
                }) = args.first()
                {
                    let visible = args.get(1).and_then(|p| p.as_int()).unwrap_or(1) != 0;
                    {
                        let state = self.get_or_create_object_state(plane, object_index);
                        state.file_name = file_name.clone();
                        state.visible = visible;
                    }
                    self.clear_object_string_state(plane, object_index);
                    self.emit_object_image(plane, object_index);
                }
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_POS => {
                let (x, y) = (
                    args.first().and_then(|p| p.as_int()).unwrap_or(0) as f32,
                    args.get(1).and_then(|p| p.as_int()).unwrap_or(0) as f32,
                );
                let state = self.get_or_create_object_state(plane, object_index);
                state.x = x;
                state.y = y;
                let _ = self.event_tx.send(HostEvent::SetObjectPos {
                    stage: plane,
                    index: object_index,
                    x,
                    y,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_X => {
                let x = args.first().and_then(|p| p.as_int()).unwrap_or(0) as f32;
                let y = {
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.x = x;
                    state.y
                };
                let _ = self.event_tx.send(HostEvent::SetObjectPos {
                    stage: plane,
                    index: object_index,
                    x,
                    y,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_Y => {
                let y = args.first().and_then(|p| p.as_int()).unwrap_or(0) as f32;
                let x = {
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.y = y;
                    state.x
                };
                let _ = self.event_tx.send(HostEvent::SetObjectPos {
                    stage: plane,
                    index: object_index,
                    x,
                    y,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_DISP => {
                let visible = args.first().and_then(|p| p.as_int()).unwrap_or(1) != 0;
                let state = self.get_or_create_object_state(plane, object_index);
                state.visible = visible;
                if visible {
                    self.emit_object_sort_and_visibility(plane, object_index);
                } else {
                    let _ = self.event_tx.send(HostEvent::SetObjectVisible {
                        stage: plane,
                        index: object_index,
                        visible: false,
                    });
                }
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_PATNO => {
                if let Some(pat_no) = args.first().and_then(|p| p.as_int()) {
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.pat_no = pat_no.max(0) as usize;
                    self.emit_object_image(plane, object_index);
                }
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_SCALE => {
                let sx = args.first().and_then(|p| p.as_int()).unwrap_or(1000) as f32 / 1000.0;
                let sy = args.get(1).and_then(|p| p.as_int()).unwrap_or(1000) as f32 / 1000.0;
                let state = self.get_or_create_object_state(plane, object_index);
                state.scale_x = sx;
                state.scale_y = sy;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_INIT_PARAM => {
                let st = self.get_or_create_object_state(plane, object_index);
                let keep_file = st.file_name.clone();
                let keep_pat = st.pat_no;
                reset_object_state_preserve_seq(st);
                st.file_name = keep_file;
                st.pat_no = keep_pat;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_COPY_FROM => {
                if let Some(siglus::vm::Prop {
                    value: siglus::vm::PropValue::Element(src_elm),
                    ..
                }) = args.first()
                {
                    if let Some((sp, si, _)) = parse_stage_object_prop(src_elm) {
                        let src = self.objects.get(&(sp, si)).cloned();
                        if let Some(src_state) = src {
                            let dst = self.get_or_create_object_state(plane, object_index);
                            copy_object_state_preserve_seq(dst, &src_state);
                            self.refresh_object_image(plane, object_index);
                        }
                    }
                }
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_FROM_CAPTURE_FILE => {
                // Treat these creation commands as lifecycle reset points.
                let st = self.get_or_create_object_state(plane, object_index);
                reset_object_state_preserve_seq(st);
                let _ = self.event_tx.send(HostEvent::RemoveObject {
                    stage: plane,
                    index: object_index,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_STRING => {
                let text = args
                    .first()
                    .and_then(|p| match &p.value { siglus::vm::PropValue::Str(v) => Some(v.as_str()), _ => None })
                    .unwrap_or("")
                    .to_owned();
                self.set_object_string_state(plane, object_index, text.clone());
                let style = self.get_object_string_style_state(plane, object_index);
                self.emit_generated_object_image(
                    plane,
                    object_index,
                    Self::build_string_raster_image(&text, &style),
                );
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_STRING_PARAM => {
                let style = ObjectStringStyleState {
                    moji_size: args.first().and_then(|p| p.as_int()).unwrap_or(18),
                    moji_space_x: args.get(1).and_then(|p| p.as_int()).unwrap_or(0),
                    moji_space_y: args.get(2).and_then(|p| p.as_int()).unwrap_or(0),
                    moji_cnt: args.get(3).and_then(|p| p.as_int()).unwrap_or(0),
                    moji_color: args.get(4).and_then(|p| p.as_int()).unwrap_or(0xFFFFFF),
                    shadow_color: args.get(5).and_then(|p| p.as_int()).unwrap_or(0x000000),
                    fuchi_color: args.get(6).and_then(|p| p.as_int()).unwrap_or(0x000000),
                    shadow_mode: args.get(7).and_then(|p| p.as_int()).unwrap_or(-1),
                };
                self.set_object_string_style_state(plane, object_index, style.clone());
                let text = self.get_object_string_state(plane, object_index);
                self.emit_generated_object_image(
                    plane,
                    object_index,
                    Self::build_string_raster_image(&text, &style),
                );
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_NUMBER => {
                let num = args.first().and_then(|p| p.as_int()).unwrap_or(0);
                self.set_object_number_state(plane, object_index, num);
                let nstyle = self.get_object_number_style_state(plane, object_index);
                self.emit_generated_object_image(
                    plane,
                    object_index,
                    Self::build_number_raster_image(num, &nstyle),
                );
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_NUMBER_PARAM => {
                let nstyle = ObjectNumberStyleState {
                    keta_max: args.first().and_then(|p| p.as_int()).unwrap_or(0),
                    disp_zero: args.get(1).and_then(|p| p.as_int()).unwrap_or(0),
                    disp_sign: args.get(2).and_then(|p| p.as_int()).unwrap_or(0),
                    tumeru_sign: args.get(3).and_then(|p| p.as_int()).unwrap_or(0),
                    space_mod: args.get(4).and_then(|p| p.as_int()).unwrap_or(0),
                    space: args.get(5).and_then(|p| p.as_int()).unwrap_or(0),
                };
                self.set_object_number_style_state(plane, object_index, nstyle.clone());
                let num = self.get_object_number_state(plane, object_index);
                self.emit_generated_object_image(
                    plane,
                    object_index,
                    Self::build_number_raster_image(num, &nstyle),
                );
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_CENTER => {
                let cx = args.first().and_then(|p| p.as_int()).unwrap_or(0) as f32;
                let cy = args.get(1).and_then(|p| p.as_int()).unwrap_or(0) as f32;
                let state = self.get_or_create_object_state(plane, object_index);
                state.center_x = cx;
                state.center_y = cy;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_ROTATE => {
                let rz = args.get(2).and_then(|p| p.as_int()).unwrap_or(0) as f32 / 10.0;
                let state = self.get_or_create_object_state(plane, object_index);
                state.rotate_z_deg = rz;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_CLIP => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.dst_clip_use = args.first().and_then(|p| p.as_int()).unwrap_or(1) != 0;
                state.dst_clip_left = args.get(1).and_then(|p| p.as_int()).unwrap_or(0) as f32;
                state.dst_clip_top = args.get(2).and_then(|p| p.as_int()).unwrap_or(0) as f32;
                state.dst_clip_right = args.get(3).and_then(|p| p.as_int()).unwrap_or(0) as f32;
                state.dst_clip_bottom = args.get(4).and_then(|p| p.as_int()).unwrap_or(0) as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CLIP_USE => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.dst_clip_use = args.first().and_then(|p| p.as_int()).unwrap_or(1) != 0;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SET_SRC_CLIP => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.src_clip_use = args.first().and_then(|p| p.as_int()).unwrap_or(1) != 0;
                state.src_clip_left = args.get(1).and_then(|p| p.as_int()).unwrap_or(0) as f32;
                state.src_clip_top = args.get(2).and_then(|p| p.as_int()).unwrap_or(0) as f32;
                state.src_clip_right = args.get(3).and_then(|p| p.as_int()).unwrap_or(0) as f32;
                state.src_clip_bottom = args.get(4).and_then(|p| p.as_int()).unwrap_or(0) as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SRC_CLIP_USE => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.src_clip_use = args.first().and_then(|p| p.as_int()).unwrap_or(1) != 0;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_ALPHA_BLEND => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.alpha_blend = args.first().and_then(|p| p.as_int()).unwrap_or(1) != 0;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_TR => {
                let state = self.get_or_create_object_state(plane, object_index);
                let v = args.first().and_then(|p| p.as_int()).unwrap_or(255) as f32;
                state.alpha = (v / 255.0).clamp(0.0, 1.0);
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_RATE => {
                let state = self.get_or_create_object_state(plane, object_index);
                let v = args.first().and_then(|p| p.as_int()).unwrap_or(1000) as f32;
                state.color_rate = (v / 1000.0).clamp(0.0, 4.0);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_R => {
                let state = self.get_or_create_object_state(plane, object_index);
                let v = args.first().and_then(|p| p.as_int()).unwrap_or(255) as f32;
                state.color_r = (v / 255.0).clamp(0.0, 4.0);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_G => {
                let state = self.get_or_create_object_state(plane, object_index);
                let v = args.first().and_then(|p| p.as_int()).unwrap_or(255) as f32;
                state.color_g = (v / 255.0).clamp(0.0, 4.0);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_B => {
                let state = self.get_or_create_object_state(plane, object_index);
                let v = args.first().and_then(|p| p.as_int()).unwrap_or(255) as f32;
                state.color_b = (v / 255.0).clamp(0.0, 4.0);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_ADD_R => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.color_add_r = args.first().and_then(|p| p.as_int()).unwrap_or(0) as f32;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_ADD_G => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.color_add_g = args.first().and_then(|p| p.as_int()).unwrap_or(0) as f32;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_ADD_B => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.color_add_b = args.first().and_then(|p| p.as_int()).unwrap_or(0) as f32;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_BRIGHT => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.bright = args.first().and_then(|p| p.as_int()).unwrap_or(0) as f32;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_DARK => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.dark = args.first().and_then(|p| p.as_int()).unwrap_or(0) as f32;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_MONO => {
                let state = self.get_or_create_object_state(plane, object_index);
                let v = args.first().and_then(|p| p.as_int()).unwrap_or(0) as f32;
                state.mono = (v / 255.0).clamp(0.0, 1.0);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_REVERSE => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.reverse = args.first().and_then(|p| p.as_int()).unwrap_or(0) != 0;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_ORDER => {
                if let Some(order) = args.first().and_then(|p| p.as_int()) {
                    let (order_v, layer_v, seq_v) = {
                        let state = self.get_or_create_object_state(plane, object_index);
                        state.order = order;
                        (state.order, state.layer, state.seq)
                    };
                    let _ = self.event_tx.send(HostEvent::SetObjectSort {
                        stage: plane,
                        index: object_index,
                        order: order_v,
                        layer: layer_v,
                        seq: seq_v,
                    });
                    if self
                        .objects
                        .get(&(plane, object_index))
                        .map(|v| v.visible)
                        .unwrap_or(false)
                    {
                        let _ = self.event_tx.send(HostEvent::SetObjectVisible {
                            stage: plane,
                            index: object_index,
                            visible: true,
                        });
                    }
                }
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_LAYER => {
                if let Some(layer) = args.first().and_then(|p| p.as_int()) {
                    let (order_v, layer_v, seq_v) = {
                        let state = self.get_or_create_object_state(plane, object_index);
                        state.layer = layer;
                        (state.order, state.layer, state.seq)
                    };
                    let _ = self.event_tx.send(HostEvent::SetObjectSort {
                        stage: plane,
                        index: object_index,
                        order: order_v,
                        layer: layer_v,
                        seq: seq_v,
                    });
                    if self
                        .objects
                        .get(&(plane, object_index))
                        .map(|v| v.visible)
                        .unwrap_or(false)
                    {
                        let _ = self.event_tx.send(HostEvent::SetObjectVisible {
                            stage: plane,
                            index: object_index,
                            visible: true,
                        });
                    }
                }
            }
            _ => {
                if self.handle_object_command_post_render(plane, object_index, cmd, args) {
                    return;
                }
            }
        }
    }
}
