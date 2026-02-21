impl GuiApp {
    fn new(
        event_rx: mpsc::Receiver<HostEvent>,
        selection_tx: mpsc::Sender<i32>,
        return_to_menu_warning_tx: mpsc::Sender<bool>,
        advance_tx: mpsc::Sender<AdvanceSignal>,
        skip_mode: Arc<AtomicBool>,
        shutdown: Arc<AtomicBool>,
        base_title: String,
        scene_size: Option<(i32, i32)>,
        audio_manager: Option<AudioManager>,
    ) -> Self {
        Self {
            event_rx,
            selection_tx,
            return_to_menu_warning_tx,
            advance_tx,
            skip_mode,
            shutdown,
            current_name: String::new(),
            current_text: String::new(),
            waiting_for_click: false,
            queued_advance_stock: 0,
            hide_message_window: false,
            message_window_visible: false,
            pending_options: Vec::new(),
            backlog: Vec::new(),
            done: false,
            show_backlog: false,
            msg_back_display_enabled: true,
            tweet_dialog_open: false,
            tweet_text: String::new(),
            tweet_authorized: false,
            tweet_user_name: String::new(),
            tweet_screen_name: String::new(),
            tweet_status_line: "未認証です。先に認証してください。".to_string(),
            tweet_confirm_empty: false,
            show_return_to_menu_warning: false,
            background_texture: None,
            background_textures: BTreeMap::new(),
            missing_background_names: BTreeMap::new(),
            object_textures: BTreeMap::new(),
            missing_object_names: BTreeMap::new(),
            object_pos: BTreeMap::new(),
            object_visible: BTreeMap::new(),
            object_sort: BTreeMap::new(),
            object_render: BTreeMap::new(),
            base_title: base_title.clone(),
            location_scene_title: String::new(),
            location_scene: String::new(),
            location_line_no: 0,
            last_window_title: format!("{} - Siglus", base_title),
            scene_size,
            wipe_started_at: None,
            wipe_duration_ms: 0,
            wipe_type: 0,
            wipe_direction: WipeDirection::Normal,

            start_time: Instant::now(),
            audio_manager,
        }
    }

    fn stage_transform(&self, screen: egui::Rect) -> (egui::Rect, f32, f32) {
        let Some((sw, sh)) = self.scene_size else {
            return (screen, 1.0, 1.0);
        };
        if sw <= 0 || sh <= 0 {
            return (screen, 1.0, 1.0);
        }

        let base_w = sw as f32;
        let base_h = sh as f32;
        let fit = (screen.width() / base_w)
            .min(screen.height() / base_h)
            .max(0.0001);
        let stage_size = egui::vec2(base_w * fit, base_h * fit);
        let stage_min = egui::pos2(
            screen.center().x - stage_size.x / 2.0,
            screen.center().y - stage_size.y / 2.0,
        );
        (egui::Rect::from_min_size(stage_min, stage_size), fit, fit)
    }

    fn init_logging() -> Result<()> {
        let log_file = std::fs::File::create("debug.log").context("failed to create debug.log")?;
        WriteLogger::init(LevelFilter::Debug, Config::default(), log_file)
            .context("failed to init file logger")?;
        Ok(())
    }

    fn consume_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                HostEvent::Name(name) => self.current_name = name,
                HostEvent::Text { text } => {
                    self.message_window_visible = true;
                    self.current_text = text.clone();
                    let row = if self.current_name.is_empty() {
                        text
                    } else {
                        format!("{}：{}", self.current_name, text)
                    };
                    self.backlog.push(row);
                    if self.backlog.len() > 500 {
                        self.backlog.remove(0);
                    }
                    // VM waits for advance unless skip is on.
                    // Keep a small click-stock so very quick repeated clicks can
                    // continue advancing text like the C++ input stock behavior.
                    if !self.skip_mode.load(Ordering::Relaxed) {
                        if self.queued_advance_stock > 0 {
                            self.queued_advance_stock -= 1;
                            self.waiting_for_click = false;
                        } else {
                            self.waiting_for_click = true;
                        }
                    }
                }
                HostEvent::Selection(options) => {
                    if options.is_empty() {
                        let _ = self.selection_tx.send(0);
                    } else {
                        self.pending_options = options;
                    }
                }
                HostEvent::Done => {
                    self.done = true;
                }
                HostEvent::LoadImage { image } => {
                    // Create texture from image
                    let size = [image.width() as usize, image.height() as usize];
                    let pixels = image.to_rgba8().into_flat_samples();
                    let color_image =
                        egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                    self.background_texture = Some(ctx.load_texture(
                        "background",
                        color_image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
                HostEvent::LoadPlaneImage { stage, image } => {
                    let size = [image.width() as usize, image.height() as usize];
                    let pixels = image.to_rgba8().into_flat_samples();
                    let color_image =
                        egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                    let tex_name = format!("background_{:?}", stage);
                    self.background_textures.insert(
                        stage,
                        ctx.load_texture(tex_name, color_image, egui::TextureOptions::LINEAR),
                    );
                    self.missing_background_names.remove(&stage);
                }
                HostEvent::MissingPlaneImage { stage, name } => {
                    self.background_textures.remove(&stage);
                    self.missing_background_names.insert(stage, name);
                }
                HostEvent::UpsertObjectImage {
                    stage,
                    index,
                    image,
                } => {
                    let size = [image.width() as usize, image.height() as usize];
                    let pixels = image.to_rgba8().into_flat_samples();
                    let color_image =
                        egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                    let tex_name = format!("obj_{:?}_{}", stage, index);
                    self.object_textures.insert(
                        (stage, index),
                        ctx.load_texture(tex_name, color_image, egui::TextureOptions::LINEAR),
                    );
                    self.missing_object_names.remove(&(stage, index));
                }
                HostEvent::MissingObjectImage { stage, index, name } => {
                    self.object_textures.remove(&(stage, index));
                    self.missing_object_names.insert((stage, index), name);
                }
                HostEvent::SetObjectPos { stage, index, x, y } => {
                    self.object_pos.insert((stage, index), egui::pos2(x, y));
                }
                HostEvent::SetObjectVisible {
                    stage,
                    index,
                    visible,
                } => {
                    self.object_visible.insert((stage, index), visible);
                }
                HostEvent::SetObjectSort {
                    stage,
                    index,
                    order,
                    layer,
                    seq,
                } => {
                    self.object_sort.insert((stage, index), (order, layer, seq));
                }
                HostEvent::RemoveObject { stage, index } => {
                    self.object_textures.remove(&(stage, index));
                    self.missing_object_names.remove(&(stage, index));
                    self.object_pos.remove(&(stage, index));
                    self.object_visible.remove(&(stage, index));
                    self.object_sort.remove(&(stage, index));
                    self.object_render.remove(&(stage, index));
                }
                HostEvent::SetObjectRenderState {
                    stage,
                    index,
                    center_x,
                    center_y,
                    scale_x,
                    scale_y,
                    rotate_z_deg,
                    alpha,
                    dst_clip_use,
                    dst_clip_left,
                    dst_clip_top,
                    dst_clip_right,
                    dst_clip_bottom,
                    src_clip_use,
                    src_clip_left,
                    src_clip_top,
                    src_clip_right,
                    src_clip_bottom,
                } => {
                    self.object_render.insert(
                        (stage, index),
                        ObjectRenderState {
                            center_x,
                            center_y,
                            scale_x,
                            scale_y,
                            rotate_z_deg,
                            alpha,
                            dst_clip_use,
                            dst_clip_left,
                            dst_clip_top,
                            dst_clip_right,
                            dst_clip_bottom,
                            src_clip_use,
                            src_clip_left,
                            src_clip_top,
                            src_clip_right,
                            src_clip_bottom,
                        },
                    );
                }
                HostEvent::ClearPlaneObjects { stage } => {
                    self.object_textures.retain(|(s, _), _| *s != stage);
                    self.missing_object_names.retain(|(s, _), _| *s != stage);
                    self.object_pos.retain(|(s, _), _| *s != stage);
                    self.object_visible.retain(|(s, _), _| *s != stage);
                    self.object_sort.retain(|(s, _), _| *s != stage);
                    self.object_render.retain(|(s, _), _| *s != stage);
                }
                HostEvent::Location {
                    scene_title,
                    scene,
                    line_no,
                } => {
                    self.location_scene_title = scene_title;
                    self.location_scene = scene;
                    self.location_line_no = line_no;
                }
                HostEvent::MessageWindowVisible(v) => self.message_window_visible = v,
                HostEvent::MsgBackState(open) => {
                    self.show_backlog = open && self.msg_back_display_enabled;
                    if open {
                        self.hide_message_window = false;
                    }
                }
                HostEvent::MsgBackDisplayEnabled(enabled) => {
                    self.msg_back_display_enabled = enabled;
                    if !enabled {
                        self.show_backlog = false;
                    }
                }
                HostEvent::OpenTweetDialog => {
                    self.tweet_dialog_open = true;
                    self.hide_message_window = false;
                    self.tweet_confirm_empty = false;
                    if self.tweet_authorized {
                        self.tweet_status_line = "投稿可能です。".to_string();
                    } else {
                        self.tweet_status_line = "未認証です。先に認証してください。".to_string();
                    }
                }
                HostEvent::ConfirmReturnToMenuWarning => {
                    self.show_return_to_menu_warning = true;
                }
                HostEvent::StartWipe {
                    duration_ms,
                    wipe_type,
                    wipe_direction,
                } => {
                    self.wipe_duration_ms = duration_ms.max(1);
                    self.wipe_type = wipe_type;
                    self.wipe_direction = wipe_direction;
                    self.wipe_started_at = Some(Instant::now());
                }
                HostEvent::PlayBgm { name, loop_flag, fade_in_ms } => {
                    if let Some(am) = &mut self.audio_manager {
                        am.play_bgm(&name, loop_flag, fade_in_ms);
                    }
                }
                HostEvent::StopBgm { fade_out_ms } => {
                    if let Some(am) = &mut self.audio_manager {
                        am.stop_bgm(fade_out_ms);
                    }
                }
                HostEvent::PlaySe { name } => {
                    if let Some(am) = &mut self.audio_manager {
                        am.play_se(&name);
                    }
                }
                HostEvent::StopSe => {
                    if let Some(am) = &mut self.audio_manager {
                        am.stop_se();
                    }
                }
                HostEvent::PlayPcm { ch, name, loop_flag } => {
                    if let Some(am) = &mut self.audio_manager {
                        am.play_pcmch(ch, &name, loop_flag);
                    }
                }
                HostEvent::StopPcm { ch } => {
                    if let Some(am) = &mut self.audio_manager {
                        am.stop_pcmch(ch);
                    }
                }
            }
        }
    }


    fn compose_window_title(&self) -> String {
        let mut title = self.base_title.clone();
        if !self.location_scene_title.is_empty() {
            title.push(' ');
            title.push_str(&self.location_scene_title);
        }
        if !self.location_scene.is_empty() {
            title.push_str(&format!(" / scene={}", self.location_scene));
            if self.location_line_no > 0 {
                title.push_str(&format!("({})", self.location_line_no));
            }
        }
        title.push_str(" - Siglus");
        title
    }

    /// Queue an advance signal for VM text wait.
    ///
    /// If we're already waiting, this unblocks immediately.
    /// If we're not waiting yet, keep a local stock counter so upcoming text
    /// waits are consumed by rapid clicks.
    fn advance(&mut self) {
        if self.advance_tx.send(AdvanceSignal::Proceed).is_ok() {
            if self.waiting_for_click {
                self.waiting_for_click = false;
            } else {
                self.queued_advance_stock = self.queued_advance_stock.saturating_add(1);
            }
        } else {
            self.waiting_for_click = false;
        }
    }

    /// Handle global input: click to advance, scroll-wheel for backlog, Ctrl for skip.
    fn handle_input(&mut self, ctx: &egui::Context) {
        let primary_clicked = ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary));
        let secondary_clicked =
            ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Secondary));
        let space_pressed = ctx.input(|i| i.key_pressed(egui::Key::Space));
        let escape_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let page_up_pressed = ctx.input(|i| i.key_pressed(egui::Key::PageUp));
        let wheel_up = ctx.input(|i| {
            i.raw_scroll_delta.y > 0.0
                || i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::MouseWheel { delta, .. } if delta.y > 0.0))
        });
        let wheel_down = ctx.input(|i| {
            i.raw_scroll_delta.y < 0.0
                || i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::MouseWheel { delta, .. } if delta.y < 0.0))
        });

        if self.tweet_dialog_open {
            return;
        }

        if self.hide_message_window {
            if primary_clicked || secondary_clicked || space_pressed {
                self.hide_message_window = false;
            }
            return;
        }

        if self.show_backlog && (secondary_clicked || space_pressed || escape_pressed) {
            self.show_backlog = false;
            return;
        }

        if secondary_clicked && self.message_window_visible && !self.show_backlog {
            self.hide_message_window = true;
            self.show_backlog = false;
            return;
        }

        // Ctrl key → temporary skip mode
        let ctrl_held = ctx.input(|i| i.modifiers.ctrl);
        if ctrl_held {
            self.skip_mode.store(true, Ordering::Relaxed);
            // If we're waiting for click, auto-advance
            self.advance();
        } else if self.skip_mode.load(Ordering::Relaxed) {
            self.skip_mode.store(false, Ordering::Relaxed);
        }

        // Mouse wheel up → open backlog (when not in selection mode)
        if self.msg_back_display_enabled
            && self.message_window_visible
            && !self.show_backlog
            && self.pending_options.is_empty()
        {
            if wheel_up || page_up_pressed {
                self.show_backlog = true;
            }
        }

        if self.waiting_for_click
            && !self.show_backlog
            && self.pending_options.is_empty()
            && wheel_down
        {
            self.advance();
        }

        // Mouse wheel down → close backlog if scrolled to bottom
        // (handled inside draw_backlog via close logic)

        // Click anywhere → advance text (when message window is showing and not in backlog/selection)
        if self.waiting_for_click && !self.show_backlog && self.pending_options.is_empty() {
            if primary_clicked {
                self.advance();
            }
        }
    }

}
