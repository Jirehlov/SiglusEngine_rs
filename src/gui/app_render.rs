impl GuiApp {
    fn draw_background(&self, ui: &mut egui::Ui) {
        let screen = ui.max_rect();
        let (rect, _, _) = self.stage_transform(screen);

        ui.painter()
            .rect_filled(screen, 0.0, egui::Color32::from_rgb(8, 8, 16));

        let mut drew_any = false;
        for stage in [StagePlane::Back, StagePlane::Front, StagePlane::Next] {
            if let Some(texture) = self.background_textures.get(&stage) {
                drew_any = true;
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        }

        if !drew_any {
            let mut label = None;
            for stage in [StagePlane::Back, StagePlane::Front, StagePlane::Next] {
                if let Some(name) = self.missing_background_names.get(&stage) {
                    label = Some(format!("missing image: {} ({:?})", name, stage));
                    break;
                }
            }
            if let Some(text) = label {
                ui.painter().rect_stroke(
                    rect.shrink(16.0),
                    6.0,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(180, 90, 90)),
                    egui::StrokeKind::Outside,
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(28.0),
                    egui::Color32::from_rgb(255, 200, 200),
                );
                drew_any = true;
            }
        }

        // Backward compatibility for older events.
        if !drew_any {
            if let Some(texture) = &self.background_texture {
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        }
    }

    fn draw_objects(&self, ui: &mut egui::Ui) {
        let (stage_rect, stage_scale_x, stage_scale_y) = self.stage_transform(ui.max_rect());
        let mut keys: Vec<(StagePlane, i32)> = self
            .object_textures
            .keys()
            .copied()
            .chain(self.missing_object_names.keys().copied())
            .collect();
        keys.sort();
        keys.dedup();
        let mut entries: Vec<_> = keys;
        entries.sort_by_key(|key| {
            let (order, layer, seq) = self.object_sort.get(key).copied().unwrap_or((0, 0, 0));
            let (plane, idx) = *key;
            (order, layer, plane as i32, idx, seq)
        });

        for key in entries {
            if !self.object_visible.get(&key).copied().unwrap_or(true) {
                continue;
            }
            let render = self
                .object_render
                .get(&key)
                .copied()
                .unwrap_or(ObjectRenderState {
                    center_x: 0.0,
                    center_y: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    rotate_z_deg: 0.0,
                    alpha: 1.0,
                    dst_clip_use: false,
                    dst_clip_left: 0.0,
                    dst_clip_top: 0.0,
                    dst_clip_right: 0.0,
                    dst_clip_bottom: 0.0,
                    src_clip_use: false,
                    src_clip_left: 0.0,
                    src_clip_top: 0.0,
                    src_clip_right: 0.0,
                    src_clip_bottom: 0.0,
                });

            let pos = self
                .object_pos
                .get(&key)
                .copied()
                .unwrap_or_else(|| egui::pos2(0.0, 0.0));
            let pos = egui::pos2(
                stage_rect.min.x + pos.x * stage_scale_x,
                stage_rect.min.y + pos.y * stage_scale_y,
            );

            let Some(texture) = self.object_textures.get(&key) else {
                if let Some(name) = self.missing_object_names.get(&key) {
                    let rect = egui::Rect::from_min_size(
                        pos,
                        egui::vec2(220.0 * stage_scale_x, 80.0 * stage_scale_y),
                    );
                    ui.painter().rect_filled(
                        rect,
                        4.0,
                        egui::Color32::from_rgba_premultiplied(70, 20, 20, 220),
                    );
                    ui.painter().rect_stroke(
                        rect,
                        4.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 120, 120)),
                        egui::StrokeKind::Outside,
                    );
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("missing: {}", name),
                        egui::FontId::proportional(16.0),
                        egui::Color32::from_rgb(255, 220, 220),
                    );
                }
                continue;
            };

            let tex_size = texture.size_vec2();
            let src_min = if render.src_clip_use {
                egui::pos2(render.src_clip_left.max(0.0), render.src_clip_top.max(0.0))
            } else {
                egui::pos2(0.0, 0.0)
            };
            let src_max = if render.src_clip_use {
                egui::pos2(
                    render.src_clip_right.clamp(src_min.x, tex_size.x),
                    render.src_clip_bottom.clamp(src_min.y, tex_size.y),
                )
            } else {
                tex_size.to_pos2()
            };
            let src_size = egui::vec2(
                (src_max.x - src_min.x).max(1.0),
                (src_max.y - src_min.y).max(1.0),
            );

            let scale_x = if render.scale_x.abs() < f32::EPSILON {
                0.001
            } else {
                render.scale_x
            };
            let scale_y = if render.scale_y.abs() < f32::EPSILON {
                0.001
            } else {
                render.scale_y
            };

            // C++-like: object position is transformed anchor, drawing origin is (pos - center).
            let left = pos.x - render.center_x * scale_x * stage_scale_x;
            let top = pos.y - render.center_y * scale_y * stage_scale_y;
            let right = left + src_size.x * scale_x * stage_scale_x;
            let bottom = top + src_size.y * scale_y * stage_scale_y;
            let rect = egui::Rect::from_min_max(
                egui::pos2(left.min(right), top.min(bottom)),
                egui::pos2(left.max(right), top.max(bottom)),
            );

            let uv = egui::Rect::from_min_max(
                egui::pos2(
                    src_min.x / tex_size.x.max(1.0),
                    src_min.y / tex_size.y.max(1.0),
                ),
                egui::pos2(
                    src_max.x / tex_size.x.max(1.0),
                    src_max.y / tex_size.y.max(1.0),
                ),
            );
            let uv = egui::Rect::from_min_max(
                egui::pos2(
                    if scale_x < 0.0 { uv.max.x } else { uv.min.x },
                    if scale_y < 0.0 { uv.max.y } else { uv.min.y },
                ),
                egui::pos2(
                    if scale_x < 0.0 { uv.min.x } else { uv.max.x },
                    if scale_y < 0.0 { uv.min.y } else { uv.max.y },
                ),
            );
            let mut rect = rect;
            let mut uv = uv;
            if render.dst_clip_use {
                let clip_rect = egui::Rect::from_min_max(
                    egui::pos2(
                        stage_rect.min.x + render.dst_clip_left * stage_scale_x,
                        stage_rect.min.y + render.dst_clip_top * stage_scale_y,
                    ),
                    egui::pos2(
                        stage_rect.min.x + render.dst_clip_right * stage_scale_x,
                        stage_rect.min.y + render.dst_clip_bottom * stage_scale_y,
                    ),
                );
                let clipped = rect.intersect(clip_rect);
                if clipped.width() <= 0.0 || clipped.height() <= 0.0 {
                    continue;
                }
                let dx0 = (clipped.min.x - rect.min.x) / rect.width().max(1.0);
                let dx1 = (clipped.max.x - rect.min.x) / rect.width().max(1.0);
                let dy0 = (clipped.min.y - rect.min.y) / rect.height().max(1.0);
                let dy1 = (clipped.max.y - rect.min.y) / rect.height().max(1.0);
                let ux0 = uv.min.x + (uv.max.x - uv.min.x) * dx0;
                let ux1 = uv.min.x + (uv.max.x - uv.min.x) * dx1;
                let uy0 = uv.min.y + (uv.max.y - uv.min.y) * dy0;
                let uy1 = uv.min.y + (uv.max.y - uv.min.y) * dy1;
                rect = clipped;
                uv = egui::Rect::from_min_max(egui::pos2(ux0, uy0), egui::pos2(ux1, uy1));
            }

            let alpha = (render.alpha.clamp(0.0, 1.0) * 255.0) as u8;
            let tint = egui::Color32::from_rgba_premultiplied(255, 255, 255, alpha);

            if render.rotate_z_deg.abs() < f32::EPSILON {
                ui.painter().image(texture.id(), rect, uv, tint);
            } else {
                let mut mesh = egui::epaint::Mesh::with_texture(texture.id());
                let center = rect.center();
                let rad = render.rotate_z_deg.to_radians();
                let (s, c) = rad.sin_cos();

                let rot = |p: egui::Pos2| {
                    let dx = p.x - center.x;
                    let dy = p.y - center.y;
                    egui::pos2(center.x + dx * c - dy * s, center.y + dx * s + dy * c)
                };

                let p0 = rot(rect.left_top());
                let p1 = rot(rect.right_top());
                let p2 = rot(rect.right_bottom());
                let p3 = rot(rect.left_bottom());

                mesh.vertices.push(egui::epaint::Vertex {
                    pos: p0,
                    uv: uv.left_top(),
                    color: tint,
                });
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: p1,
                    uv: uv.right_top(),
                    color: tint,
                });
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: p2,
                    uv: uv.right_bottom(),
                    color: tint,
                });
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: p3,
                    uv: uv.left_bottom(),
                    color: tint,
                });
                mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
                ui.painter().add(egui::Shape::mesh(mesh));
            }
        }
    }

    fn draw_message_window(&self, ui: &mut egui::Ui) {
        let screen = ui.max_rect();

        let mw_height = screen.height() * MSG_WINDOW_HEIGHT_RATIO;
        let mw_rect = egui::Rect::from_min_max(
            egui::pos2(
                screen.left() + MSG_WINDOW_MARGIN_X,
                screen.bottom() - MSG_WINDOW_MARGIN_BOTTOM - mw_height,
            ),
            egui::pos2(
                screen.right() - MSG_WINDOW_MARGIN_X,
                screen.bottom() - MSG_WINDOW_MARGIN_BOTTOM,
            ),
        );

        ui.painter()
            .rect_filled(mw_rect, MSG_WINDOW_ROUNDING, MSG_BG);
        ui.painter().rect_stroke(
            mw_rect,
            MSG_WINDOW_ROUNDING,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_premultiplied(100, 140, 200, 60),
            ),
            egui::StrokeKind::Outside,
        );

        // ── Name plate ──
        if !self.current_name.is_empty() {
            let name_font = egui::FontId::proportional(17.0);
            let name_galley = ui.painter().layout_no_wrap(
                self.current_name.clone(),
                name_font,
                egui::Color32::WHITE,
            );
            let name_w = name_galley.size().x + NAME_PLATE_PADDING_X * 2.0;
            let name_rect = egui::Rect::from_min_size(
                egui::pos2(
                    mw_rect.left() + NAME_PLATE_MARGIN_LEFT,
                    mw_rect.top() + NAME_PLATE_OFFSET_Y - NAME_PLATE_HEIGHT,
                ),
                egui::vec2(name_w, NAME_PLATE_HEIGHT),
            );
            ui.painter().rect_filled(name_rect, 6.0, NAME_PLATE_BG);
            ui.painter().rect_stroke(
                name_rect,
                6.0,
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_premultiplied(80, 120, 200, 80),
                ),
                egui::StrokeKind::Outside,
            );
            ui.painter().galley(
                egui::pos2(
                    name_rect.left() + NAME_PLATE_PADDING_X,
                    name_rect.center().y - name_galley.size().y / 2.0,
                ),
                name_galley,
                egui::Color32::WHITE,
            );
        }

        // ── Message text ──
        let text_rect = egui::Rect::from_min_max(
            egui::pos2(
                mw_rect.left() + MSG_TEXT_PADDING_X,
                mw_rect.top() + MSG_TEXT_PADDING_TOP,
            ),
            egui::pos2(
                mw_rect.right() - MSG_TEXT_PADDING_X,
                mw_rect.bottom() - 16.0,
            ),
        );

        if !self.current_text.is_empty() {
            let text_font = egui::FontId::proportional(18.0);
            let text_galley = ui.painter().layout(
                self.current_text.clone(),
                text_font.clone(),
                egui::Color32::from_rgb(230, 235, 245),
                text_rect.width(),
            );
            ui.painter()
                .galley(text_rect.left_top(), text_galley, egui::Color32::WHITE);
        } else if !self.done {
            let text_font = egui::FontId::proportional(16.0);
            let waiting_galley = ui.painter().layout_no_wrap(
                "……".to_string(),
                text_font,
                egui::Color32::from_rgba_premultiplied(150, 160, 180, 120),
            );
            ui.painter()
                .galley(text_rect.left_top(), waiting_galley, egui::Color32::WHITE);
        }

        // ── Click-wait indicator (▼ blinking) ──
        if self.waiting_for_click && self.pending_options.is_empty() {
            let elapsed = self.start_time.elapsed().as_secs_f32();
            let alpha = ((elapsed * 3.0).sin() * 0.5 + 0.5) * 200.0 + 55.0;
            let indicator_color =
                egui::Color32::from_rgba_premultiplied(180, 200, 255, alpha as u8);
            let indicator_pos = egui::pos2(
                mw_rect.right() - MSG_TEXT_PADDING_X,
                mw_rect.bottom() - 24.0,
            );
            let tri = [
                egui::pos2(
                    indicator_pos.x - CLICK_INDICATOR_SIZE / 2.0,
                    indicator_pos.y - CLICK_INDICATOR_SIZE,
                ),
                egui::pos2(
                    indicator_pos.x + CLICK_INDICATOR_SIZE / 2.0,
                    indicator_pos.y - CLICK_INDICATOR_SIZE,
                ),
                egui::pos2(indicator_pos.x, indicator_pos.y),
            ];
            ui.painter().add(egui::Shape::convex_polygon(
                tri.to_vec(),
                indicator_color,
                egui::Stroke::NONE,
            ));
        }
    }

    fn draw_selections(&mut self, ui: &mut egui::Ui) {
        if self.pending_options.is_empty() {
            return;
        }

        let screen = ui.max_rect();

        ui.painter().rect_filled(
            screen,
            0.0,
            egui::Color32::from_rgba_premultiplied(0, 0, 0, 120),
        );

        let total_height = self.pending_options.len() as f32
            * (SEL_BUTTON_HEIGHT + SEL_BUTTON_SPACING)
            - SEL_BUTTON_SPACING;
        let start_y = screen.center().y - total_height / 2.0;

        let mut clicked_index: Option<usize> = None;

        for (i, option) in self.pending_options.iter().enumerate() {
            let btn_rect = egui::Rect::from_min_size(
                egui::pos2(
                    screen.center().x - SEL_BUTTON_WIDTH / 2.0,
                    start_y + i as f32 * (SEL_BUTTON_HEIGHT + SEL_BUTTON_SPACING),
                ),
                egui::vec2(SEL_BUTTON_WIDTH, SEL_BUTTON_HEIGHT),
            );

            let resp = ui.allocate_rect(btn_rect, egui::Sense::click());
            let is_hovered = resp.hovered();
            let bg = if is_hovered {
                SEL_BUTTON_HOVER_BG
            } else {
                SEL_BUTTON_BG
            };

            ui.painter().rect_filled(btn_rect, SEL_BUTTON_ROUNDING, bg);
            ui.painter().rect_stroke(
                btn_rect,
                SEL_BUTTON_ROUNDING,
                egui::Stroke::new(
                    if is_hovered { 1.5 } else { 0.5 },
                    egui::Color32::from_rgba_premultiplied(
                        if is_hovered { 140 } else { 80 },
                        if is_hovered { 180 } else { 100 },
                        255,
                        if is_hovered { 200 } else { 80 },
                    ),
                ),
                egui::StrokeKind::Outside,
            );

            let font = egui::FontId::proportional(17.0);
            let galley = ui.painter().layout_no_wrap(
                option.clone(),
                font,
                egui::Color32::from_rgb(220, 230, 250),
            );
            ui.painter().galley(
                egui::pos2(
                    btn_rect.center().x - galley.size().x / 2.0,
                    btn_rect.center().y - galley.size().y / 2.0,
                ),
                galley,
                egui::Color32::WHITE,
            );

            if resp.clicked() {
                clicked_index = Some(i);
            }
        }

        if let Some(idx) = clicked_index {
            let _ = self.selection_tx.send(idx as i32);
            self.pending_options.clear();
        }
    }

    fn draw_backlog(&mut self, ui: &mut egui::Ui) {
        if !self.show_backlog {
            return;
        }

        let screen = ui.max_rect();

        ui.painter().rect_filled(
            screen,
            0.0,
            egui::Color32::from_rgba_premultiplied(5, 8, 18, 240),
        );

        // Title
        let title_font = egui::FontId::proportional(20.0);
        let title_galley = ui.painter().layout_no_wrap(
            "テキスト履歴".to_string(),
            title_font,
            egui::Color32::from_rgb(160, 180, 220),
        );
        ui.painter().galley(
            egui::pos2(screen.left() + 40.0, screen.top() + 20.0),
            title_galley,
            egui::Color32::WHITE,
        );

        // Close button
        let close_rect = egui::Rect::from_min_size(
            egui::pos2(screen.right() - 80.0, screen.top() + 14.0),
            egui::vec2(60.0, 32.0),
        );
        let close_resp = ui.allocate_rect(close_rect, egui::Sense::click());
        let close_bg = if close_resp.hovered() {
            egui::Color32::from_rgba_premultiplied(80, 40, 40, 180)
        } else {
            egui::Color32::from_rgba_premultiplied(50, 30, 30, 120)
        };
        ui.painter().rect_filled(close_rect, 4.0, close_bg);
        let close_font = egui::FontId::proportional(14.0);
        let close_galley = ui.painter().layout_no_wrap(
            "閉じる".to_string(),
            close_font,
            egui::Color32::from_rgb(200, 180, 180),
        );
        ui.painter().galley(
            egui::pos2(
                close_rect.center().x - close_galley.size().x / 2.0,
                close_rect.center().y - close_galley.size().y / 2.0,
            ),
            close_galley,
            egui::Color32::WHITE,
        );

        // Close backlog on button click, right-click, Space, or Escape
        let should_close = close_resp.clicked()
            || ui.input(|i| i.pointer.secondary_clicked())
            || ui.input(|i| i.key_pressed(egui::Key::Space))
            || ui.input(|i| i.key_pressed(egui::Key::Escape));
        if should_close {
            self.show_backlog = false;
            return;
        }

        // Scrollable text area
        let text_area = egui::Rect::from_min_max(
            egui::pos2(screen.left() + 40.0, screen.top() + 60.0),
            egui::pos2(screen.right() - 40.0, screen.bottom() - 20.0),
        );

        let scroll_id = egui::Id::new("backlog_scroll");
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(text_area)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );
        egui::ScrollArea::vertical()
            .id_salt(scroll_id)
            .stick_to_bottom(true)
            .show(&mut child_ui, |ui| {
                let line_font = egui::FontId::proportional(15.0);
                for line in &self.backlog {
                    ui.label(
                        egui::RichText::new(line)
                            .font(line_font.clone())
                            .color(egui::Color32::from_rgb(190, 200, 215)),
                    );
                    ui.add_space(4.0);
                }
            });
    }

    fn draw_return_to_menu_warning(&mut self, ui: &mut egui::Ui) {
        if !self.show_return_to_menu_warning {
            return;
        }
        let screen = ui.max_rect();
        ui.painter().rect_filled(
            screen,
            0.0,
            egui::Color32::from_rgba_premultiplied(0, 0, 0, 170),
        );

        let panel = egui::Rect::from_center_size(screen.center(), egui::vec2(480.0, 180.0));
        ui.painter().rect_filled(
            panel,
            10.0,
            egui::Color32::from_rgba_premultiplied(30, 35, 50, 245),
        );
        ui.painter().rect_stroke(
            panel,
            10.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(140, 160, 210, 220)),
            egui::StrokeKind::Outside,
        );

        let title_font = egui::FontId::proportional(20.0);
        let text_font = egui::FontId::proportional(16.0);
        let title = ui.painter().layout_no_wrap(
            "RETURN TO MENU".to_string(),
            title_font,
            egui::Color32::from_rgb(230, 235, 245),
        );
        ui.painter().galley(
            egui::pos2(panel.center().x - title.size().x / 2.0, panel.top() + 20.0),
            title,
            egui::Color32::WHITE,
        );
        let msg = ui.painter().layout_no_wrap(
            "タイトルメニューに戻りますか？".to_string(),
            text_font.clone(),
            egui::Color32::from_rgb(215, 220, 230),
        );
        ui.painter().galley(
            egui::pos2(panel.center().x - msg.size().x / 2.0, panel.top() + 64.0),
            msg,
            egui::Color32::WHITE,
        );

        let yes_rect = egui::Rect::from_min_size(
            egui::pos2(panel.center().x - 170.0, panel.bottom() - 58.0),
            egui::vec2(140.0, 36.0),
        );
        let no_rect = egui::Rect::from_min_size(
            egui::pos2(panel.center().x + 30.0, panel.bottom() - 58.0),
            egui::vec2(140.0, 36.0),
        );
        let yes = ui.allocate_rect(yes_rect, egui::Sense::click());
        let no = ui.allocate_rect(no_rect, egui::Sense::click());

        for (rect, resp, text, active) in [
            (yes_rect, yes, "YES", true),
            (no_rect, no, "NO", false),
        ] {
            let bg = if resp.hovered() {
                egui::Color32::from_rgba_premultiplied(80, 110, 180, 240)
            } else {
                egui::Color32::from_rgba_premultiplied(50, 70, 120, 220)
            };
            ui.painter().rect_filled(rect, 6.0, bg);
            let fg = if active {
                egui::Color32::from_rgb(250, 250, 255)
            } else {
                egui::Color32::from_rgb(235, 235, 245)
            };
            let g = ui.painter().layout_no_wrap(text.to_string(), text_font.clone(), fg);
            ui.painter().galley(
                egui::pos2(
                    rect.center().x - g.size().x / 2.0,
                    rect.center().y - g.size().y / 2.0,
                ),
                g,
                egui::Color32::WHITE,
            );
            if resp.clicked() {
                let _ = self.return_to_menu_warning_tx.send(active);
                self.show_return_to_menu_warning = false;
            }
        }
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui) {
        let screen = ui.max_rect();

        // LOG button at top-left
        let btn_rect = egui::Rect::from_min_size(
            egui::pos2(screen.left() + 12.0, screen.top() + 12.0),
            egui::vec2(40.0, 28.0),
        );
        let resp = ui.allocate_rect(btn_rect, egui::Sense::click());
        let btn_bg = if resp.hovered() {
            egui::Color32::from_rgba_premultiplied(60, 80, 120, 160)
        } else {
            egui::Color32::from_rgba_premultiplied(30, 40, 60, 100)
        };
        ui.painter().rect_filled(btn_rect, 4.0, btn_bg);
        let icon_font = egui::FontId::proportional(13.0);
        let icon_galley = ui.painter().layout_no_wrap(
            "LOG".to_string(),
            icon_font,
            egui::Color32::from_rgba_premultiplied(180, 190, 210, 180),
        );
        ui.painter().galley(
            egui::pos2(
                btn_rect.center().x - icon_galley.size().x / 2.0,
                btn_rect.center().y - icon_galley.size().y / 2.0,
            ),
            icon_galley,
            egui::Color32::WHITE,
        );
        if resp.clicked() && self.msg_back_display_enabled {
            self.show_backlog = !self.show_backlog;
        }

        // Skip indicator (shows when Ctrl is held)
        if self.skip_mode.load(Ordering::Relaxed) {
            let skip_font = egui::FontId::proportional(13.0);
            let skip_galley = ui.painter().layout_no_wrap(
                "SKIP ▶▶".to_string(),
                skip_font,
                egui::Color32::from_rgba_premultiplied(255, 200, 100, 220),
            );
            ui.painter().galley(
                egui::pos2(screen.left() + 60.0, screen.top() + 16.0),
                skip_galley,
                egui::Color32::WHITE,
            );
        }

        // Error display removed as per request. Errors are now logged to terminal.
    }


    fn draw_wipe_overlay(&self, ui: &mut egui::Ui) {
        let Some(started) = self.wipe_started_at else {
            return;
        };
        let duration = self.wipe_duration_ms.max(1) as f32;
        let elapsed_ms = (Instant::now() - started).as_secs_f32() * 1000.0;
        if elapsed_ms >= duration {
            return;
        }
        let p = (elapsed_ms / duration).clamp(0.0, 1.0);
        let ramp = if p < 0.5 { p * 2.0 } else { (1.0 - p) * 2.0 };
        let alpha = (ramp * 220.0).clamp(0.0, 255.0) as u8;
        if alpha == 0 {
            return;
        }
        let color = match self.wipe_type.rem_euclid(4) {
            // Approximate C++ wipe families with distinct observable overlays.
            0 => egui::Color32::from_rgba_premultiplied(0, 0, 0, alpha),
            1 => egui::Color32::from_rgba_premultiplied(255, 255, 255, alpha),
            2 => egui::Color32::from_rgba_premultiplied(0, 0, 64, alpha),
            _ => egui::Color32::from_rgba_premultiplied(32, 0, 0, alpha),
        };
        ui.painter().rect_filled(ui.max_rect(), 0.0, color);
    }
}
