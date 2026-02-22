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
                    if p < 0.5 { p * 2.0 } else { (1.0 - p) * 2.0 }
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
