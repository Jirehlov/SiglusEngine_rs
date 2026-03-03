macro_rules! impl_app_render_overlay {
    () => {
        fn draw_toolbar(&mut self, ui: &mut egui::Ui) {
            let screen = ui.max_rect();
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
            let err_rect = egui::Rect::from_min_size(
                egui::pos2(screen.left() + 58.0, screen.top() + 12.0),
                egui::vec2(52.0, 28.0),
            );
            let err_resp = ui.allocate_rect(err_rect, egui::Sense::click());
            let err_bg = if err_resp.hovered() {
                egui::Color32::from_rgba_premultiplied(90, 50, 50, 180)
            } else {
                egui::Color32::from_rgba_premultiplied(70, 35, 35, 140)
            };
            ui.painter().rect_filled(err_rect, 4.0, err_bg);
            ui.painter().text(
                err_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("ERR {}", self.vm_error_history.len()),
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(255, 210, 210),
            );
            if err_resp.clicked() {
                self.show_vm_error_panel = !self.show_vm_error_panel;
            }

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
        }

        fn draw_vm_error_overlay(&self, ui: &mut egui::Ui) {
            let Some((level, message, at, ctx)) = &self.latest_vm_error else {
                return;
            };
            if at.elapsed().as_secs_f32() > 5.0 {
                return;
            }
            let (bg, fg, title) = match level {
                VmErrorLevel::Fatal => (
                    egui::Color32::from_rgba_premultiplied(120, 20, 20, 220),
                    egui::Color32::from_rgb(255, 220, 220),
                    "VM FATAL",
                ),
                VmErrorLevel::FileNotFound => (
                    egui::Color32::from_rgba_premultiplied(120, 80, 20, 220),
                    egui::Color32::from_rgb(255, 240, 200),
                    "VM FILE_NOT_FOUND",
                ),
            };
            let screen = ui.max_rect();
            let rect = egui::Rect::from_min_size(
                egui::pos2(screen.left() + 12.0, screen.top() + 50.0),
                egui::vec2((screen.width() - 24.0).min(900.0), 58.0),
            );
            ui.painter().rect_filled(rect, 6.0, bg);
            ui.painter().text(
                egui::pos2(rect.left() + 10.0, rect.top() + 10.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "{}: {} [{}:{}:pc{} {:?}]",
                    title, message, ctx.scene, ctx.line_no, ctx.pc, ctx.element
                ),
                egui::FontId::proportional(15.0),
                fg,
            );
        }

        fn draw_vm_error_panel(&mut self, ui: &mut egui::Ui) {
            if !self.show_vm_error_panel {
                return;
            }
            let screen = ui.max_rect();
            let rect = egui::Rect::from_min_size(
                egui::pos2(screen.left() + 12.0, screen.top() + 86.0),
                egui::vec2(
                    (screen.width() - 24.0).min(900.0),
                    (screen.height() * 0.45).max(180.0),
                ),
            );
            ui.painter().rect_filled(
                rect,
                6.0,
                egui::Color32::from_rgba_premultiplied(20, 20, 28, 220),
            );
            let header = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 32.0));
            ui.painter().text(
                egui::pos2(header.left() + 10.0, header.center().y),
                egui::Align2::LEFT_CENTER,
                "VM Error History",
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(220, 220, 240),
            );
            if let Some(until) = self.vm_error_copy_notice_until {
                if Instant::now() <= until {
                    ui.painter().text(
                        egui::pos2(header.left() + 320.0, header.center().y),
                        egui::Align2::LEFT_CENTER,
                        "已复制错误上下文",
                        egui::FontId::proportional(12.0),
                        egui::Color32::from_rgb(160, 255, 180),
                    );
                }
            }

            let mut draw_row = |x: f32, label: &str, active: bool| -> egui::Response {
                let r = egui::Rect::from_min_size(
                    egui::pos2(x, header.top() + 6.0),
                    egui::vec2(90.0, 20.0),
                );
                let resp = ui.allocate_rect(r, egui::Sense::click());
                let bg = if active {
                    egui::Color32::from_rgba_premultiplied(70, 90, 140, 220)
                } else {
                    egui::Color32::from_rgba_premultiplied(50, 50, 70, 160)
                };
                ui.painter().rect_filled(r, 4.0, bg);
                ui.painter().text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(11.0),
                    egui::Color32::WHITE,
                );
                resp
            };

            if draw_row(
                header.right() - 290.0,
                "ALL",
                self.vm_error_filter == VmErrorFilter::All,
            )
            .clicked()
            {
                self.vm_error_filter = VmErrorFilter::All;
            }
            if draw_row(
                header.right() - 194.0,
                "FATAL",
                self.vm_error_filter == VmErrorFilter::FatalOnly,
            )
            .clicked()
            {
                self.vm_error_filter = VmErrorFilter::FatalOnly;
            }
            if draw_row(
                header.right() - 98.0,
                "FILE",
                self.vm_error_filter == VmErrorFilter::FileNotFoundOnly,
            )
            .clicked()
            {
                self.vm_error_filter = VmErrorFilter::FileNotFoundOnly;
            }
            if draw_row(
                header.left() + 150.0,
                "TIME",
                self.vm_error_sort == VmErrorSort::TimeDesc,
            )
            .clicked()
            {
                self.vm_error_sort = VmErrorSort::TimeDesc;
            }
            if draw_row(
                header.left() + 246.0,
                "SCENE",
                self.vm_error_sort == VmErrorSort::SceneLineAsc,
            )
            .clicked()
            {
                self.vm_error_sort = VmErrorSort::SceneLineAsc;
            }
            if draw_row(
                header.left() + 342.0,
                "COPIED",
                self.vm_error_filter_recent_copy,
            )
            .clicked()
            {
                self.vm_error_filter_recent_copy = !self.vm_error_filter_recent_copy;
            }
            if draw_row(header.left() + 438.0, "PIN", !self.vm_error_pinned.is_empty()).clicked() {
                self.vm_error_filter_recent_copy = !self.vm_error_filter_recent_copy;
            }
            if draw_row(header.left() + 502.0, "CPYSEL", false).clicked() {
                let mut rows: Vec<String> = if self.vm_error_copy_selected.is_empty() {
                    self.vm_error_pinned.iter().cloned().collect()
                } else {
                    self.vm_error_copy_selected.iter().cloned().collect()
                };
                rows.sort();
                if !rows.is_empty() {
                    let joined = rows.join("\n");
                    ui.ctx().copy_text(joined.clone());
                    self.vm_error_last_copied = Some(joined);
                    self.vm_error_copy_notice_until = Some(Instant::now() + std::time::Duration::from_secs(2));
                }
            }
            if draw_row(header.left() + 586.0, "CLEAR", false).clicked() {
                self.vm_error_last_copied = None;
                self.vm_error_copy_notice_until = None;
                self.vm_error_filter_recent_copy = false;
                self.vm_error_copy_selected.clear();
                self.vm_error_copy_history.clear();
                self.vm_error_pinned.clear();
            }

            let mut hx = rect.left() + 8.0;
            let history_y = header.bottom() + 6.0;
            for (idx, payload) in self.vm_error_copy_history.iter().take(6).enumerate() {
                let active = self.vm_error_copy_selected.contains(payload);
                let pinned = self.vm_error_pinned.contains(payload);
                let summary = {
                    let mut v = payload.clone();
                    if v.len() > 18 {
                        v.truncate(18);
                        v.push('…');
                    }
                    v
                };
                let label = format!("#{} {}", idx + 1, summary);
                let w = 130.0;
                let r = egui::Rect::from_min_size(egui::pos2(hx, history_y), egui::vec2(w, 20.0));
                let resp = ui.allocate_rect(r, egui::Sense::click());
                let bg = if pinned {
                    egui::Color32::from_rgba_premultiplied(120, 90, 140, 220)
                } else if active {
                    egui::Color32::from_rgba_premultiplied(90, 130, 90, 220)
                } else {
                    egui::Color32::from_rgba_premultiplied(55, 55, 72, 170)
                };
                ui.painter().rect_filled(r, 4.0, bg);
                ui.painter().text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(11.0),
                    egui::Color32::WHITE,
                );
                if resp.clicked() {
                    ui.ctx().copy_text(payload.clone());
                    self.vm_error_last_copied = Some(payload.clone());
                    self.vm_error_copy_notice_until =
                        Some(Instant::now() + std::time::Duration::from_secs(2));
                    if active {
                        self.vm_error_copy_selected.remove(payload);
                    } else {
                        self.vm_error_copy_selected.insert(payload.clone());
                    }
                }
                if resp.secondary_clicked() {
                    if pinned {
                        self.vm_error_pinned.remove(payload);
                    } else {
                        self.vm_error_pinned.insert(payload.clone());
                    }
                }
                if resp.hovered() {
                    resp.clone().on_hover_text(payload.clone());
                }
                hx += w + 4.0;
            }

            let mut rows: Vec<_> = self
                .vm_error_history
                .iter()
                .filter(|(level, message, _ts, ctx)| {
                    let show = match self.vm_error_filter {
                        VmErrorFilter::All => true,
                        VmErrorFilter::FatalOnly => *level == VmErrorLevel::Fatal,
                        VmErrorFilter::FileNotFoundOnly => *level == VmErrorLevel::FileNotFound,
                    };
                    if !show {
                        return false;
                    }
                    let copied_row = format!(
                        "scene={} line={} pc={} level={:?} element={:?} message={}",
                        ctx.scene, ctx.line_no, ctx.pc, level, ctx.element, message
                    );
                    if self.vm_error_filter_recent_copy {
                        let selected = !self.vm_error_copy_selected.is_empty()
                            && self.vm_error_copy_selected.contains(&copied_row);
                        let pinned = self.vm_error_pinned.contains(&copied_row);
                        let fallback_last = self.vm_error_copy_selected.is_empty()
                            && self.vm_error_last_copied.as_deref() == Some(copied_row.as_str());
                        if !selected && !fallback_last && !pinned {
                            return false;
                        }
                    }
                    if self.vm_error_search.is_empty() {
                        return true;
                    }
                    let row = format!(
                        "{} {} {} {} {:?}",
                        ctx.scene, ctx.line_no, ctx.pc, message, ctx.element
                    );
                    row.to_ascii_lowercase()
                        .contains(&self.vm_error_search.to_ascii_lowercase())
                })
                .collect();
            match self.vm_error_sort {
                VmErrorSort::TimeDesc => rows.reverse(),
                VmErrorSort::SceneLineAsc => rows.sort_by(|a, b| {
                    a.3.scene
                        .cmp(&b.3.scene)
                        .then(a.3.line_no.cmp(&b.3.line_no))
                        .then(a.3.pc.cmp(&b.3.pc))
                        .then(a.2.cmp(&b.2))
                }),
            }

            let mut y = header.bottom() + 54.0;
            let line_h = 18.0;
            for (level, message, ts, ctx) in rows {
                if y + line_h > rect.bottom() - 6.0 {
                    break;
                }
                let color = if *level == VmErrorLevel::Fatal {
                    egui::Color32::from_rgb(255, 180, 180)
                } else {
                    egui::Color32::from_rgb(255, 230, 180)
                };
                let text = format!(
                    "[{:.3}s][{:?}] [{}:{}:pc{}] {}",
                    ts.elapsed().as_secs_f32(),
                    level,
                    ctx.scene,
                    ctx.line_no,
                    ctx.pc,
                    message
                );
                let row_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left() + 8.0, y - 1.0),
                    egui::vec2(rect.width() - 16.0, line_h),
                );
                let resp = ui.allocate_rect(row_rect, egui::Sense::click());
                let copy_payload = format!(
                    "scene={} line={} pc={} level={:?} element={:?} message={}",
                    ctx.scene, ctx.line_no, ctx.pc, level, ctx.element, message
                );
                if self.vm_error_last_copied.as_deref() == Some(copy_payload.as_str())
                    || self.vm_error_copy_selected.contains(copy_payload.as_str())
                    || self.vm_error_pinned.contains(copy_payload.as_str())
                {
                    ui.painter().rect_filled(
                        row_rect,
                        2.0,
                        egui::Color32::from_rgba_premultiplied(70, 130, 90, 120),
                    );
                }
                if resp.hovered() {
                    ui.painter().rect_filled(
                        row_rect,
                        2.0,
                        egui::Color32::from_rgba_premultiplied(80, 80, 120, 80),
                    );
                }
                if resp.clicked() {
                    ui.ctx().copy_text(copy_payload.clone());
                    self.vm_error_last_copied = Some(copy_payload.clone());
                    self.vm_error_copy_notice_until =
                        Some(Instant::now() + std::time::Duration::from_secs(2));
                    self.vm_error_copy_selected.insert(copy_payload.clone());
                    self.vm_error_copy_history.retain(|x| x != &copy_payload);
                    self.vm_error_copy_history.insert(0, copy_payload);
                    if self.vm_error_copy_history.len() > 12 {
                        self.vm_error_copy_history.truncate(12);
                    }
                }
                if resp.secondary_clicked() {
                    if self.vm_error_pinned.contains(copy_payload.as_str()) {
                        self.vm_error_pinned.remove(copy_payload.as_str());
                    } else {
                        self.vm_error_pinned.insert(copy_payload.clone());
                    }
                }
                ui.painter().text(
                    egui::pos2(rect.left() + 10.0, y),
                    egui::Align2::LEFT_TOP,
                    text,
                    egui::FontId::proportional(12.0),
                    color,
                );
                y += line_h;
            }
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
            let ramp = match self.wipe_direction {
                WipeDirection::SystemIn => 1.0 - p,
                WipeDirection::SystemOut => p,
                WipeDirection::Normal => {
                    if p < 0.5 {
                        p * 2.0
                    } else {
                        (1.0 - p) * 2.0
                    }
                }
            };
            let alpha = (ramp * 255.0).clamp(0.0, 255.0) as u8;
            if alpha == 0 {
                return;
            }
            let color = match self.wipe_direction {
                WipeDirection::SystemIn | WipeDirection::SystemOut => {
                    egui::Color32::from_rgba_premultiplied(0, 0, 0, alpha)
                }
                WipeDirection::Normal => match self.wipe_type.rem_euclid(4) {
                    0 => egui::Color32::from_rgba_premultiplied(0, 0, 0, alpha),
                    1 => egui::Color32::from_rgba_premultiplied(255, 255, 255, alpha),
                    2 => egui::Color32::from_rgba_premultiplied(0, 0, 64, alpha),
                    _ => egui::Color32::from_rgba_premultiplied(32, 0, 0, alpha),
                },
            };
            ui.painter().rect_filled(ui.max_rect(), 0.0, color);
        }
    };
}
