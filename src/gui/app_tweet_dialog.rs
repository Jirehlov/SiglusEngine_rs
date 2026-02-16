impl GuiApp {
    fn draw_tweet_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.tweet_dialog_open {
            return;
        }

        let screen = ui.max_rect();
        ui.painter().rect_filled(
            screen,
            0.0,
            egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
        );

        let dialog_rect = egui::Rect::from_center_size(screen.center(), egui::vec2(620.0, 320.0));
        ui.painter().rect_filled(
            dialog_rect,
            10.0,
            egui::Color32::from_rgba_premultiplied(24, 28, 44, 245),
        );
        ui.painter().rect_stroke(
            dialog_rect,
            10.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(120, 140, 190, 180)),
            egui::StrokeKind::Outside,
        );

        let content_rect = dialog_rect.shrink2(egui::vec2(20.0, 18.0));
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );

        child_ui.label(
            egui::RichText::new("ツイートダイアログ（暫定）")
                .size(22.0)
                .color(egui::Color32::from_rgb(220, 228, 245)),
        );
        child_ui.add_space(6.0);

        let user_line = if self.tweet_authorized {
            format!("{} (@{})", self.tweet_user_name, self.tweet_screen_name)
        } else {
            "未認証（C++: 認証後に投稿可）".to_string()
        };
        child_ui.label(
            egui::RichText::new(user_line)
                .size(15.0)
                .color(egui::Color32::from_rgb(190, 204, 230)),
        );
        child_ui.add_space(8.0);

        child_ui.add_sized(
            [content_rect.width() - 8.0, 130.0],
            egui::TextEdit::multiline(&mut self.tweet_text)
                .hint_text("ここに投稿文を入力")
                .desired_rows(5),
        );
        child_ui.add_space(8.0);

        child_ui.label(
            egui::RichText::new(&self.tweet_status_line)
                .size(14.0)
                .color(egui::Color32::from_rgb(205, 190, 150)),
        );
        child_ui.add_space(8.0);

        let mut close_clicked = false;
        child_ui.horizontal(|ui| {
            if ui.button("認証").clicked() {
                self.tweet_authorized = true;
                if self.tweet_user_name.is_empty() {
                    self.tweet_user_name = "RustUser".to_string();
                }
                if self.tweet_screen_name.is_empty() {
                    self.tweet_screen_name = "siglus_stub".to_string();
                }
                self.tweet_status_line = "認証済み。投稿可能です。".to_string();
                self.tweet_confirm_empty = false;
            }

            let tweet_enabled = self.tweet_authorized;
            if ui
                .add_enabled(tweet_enabled, egui::Button::new("投稿"))
                .clicked()
            {
                if self.tweet_text.trim().is_empty() && !self.tweet_confirm_empty {
                    self.tweet_status_line =
                        "メッセージが空です。もう一度投稿を押すと空投稿（stub）として処理します。"
                            .to_string();
                    self.tweet_confirm_empty = true;
                } else {
                    self.tweet_status_line = "投稿成功（stub）".to_string();
                    self.tweet_text.clear();
                    self.tweet_confirm_empty = false;
                    self.tweet_dialog_open = false;
                }
            }

            if ui.button("閉じる").clicked() {
                close_clicked = true;
            }
        });

        if close_clicked || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.tweet_dialog_open = false;
            self.tweet_confirm_empty = false;
        }
    }
}
