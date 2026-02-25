impl GuiHost {
    fn group_hit_candidate_button(
        &mut self,
        plane: StagePlane,
        group_idx: i32,
        cursor: (f32, f32),
    ) -> Option<i32> {
        self.objects
            .iter()
            .filter_map(|((p, idx), obj)| {
                if *p != plane || !obj.visible {
                    return None;
                }
                let btn = self.get_object_button_state(*p, *idx);
                if btn.group_no != group_idx || btn.button_no < 0 {
                    return None;
                }
                if btn.state == 2 || btn.real_state == 2 {
                    return None;
                }
                let local = self.screen_to_object_local(obj, cursor)?;
                if !self.local_in_object_clip(obj, local) {
                    return None;
                }
                if btn.alpha_test != 0 && !self.object_alpha_test_hit(obj, local) {
                    return None;
                }
                let dx = obj.x - cursor.0;
                let dy = obj.y - cursor.1;
                let dist2 = dx * dx + dy * dy;
                Some((btn.button_no, obj.layer, obj.order, obj.seq, dist2))
            })
            .max_by(|a, b| {
                // C++ draw-order parity: higher layer/order/seq on top; tie uses nearer cursor.
                (a.1, a.2, a.3, -(a.4 as i64)).cmp(&(b.1, b.2, b.3, -(b.4 as i64)))
            })
            .map(|(button_no, _, _, _, _)| button_no)
    }

    fn screen_to_object_local(&self, obj: &HostObjectState, cursor: (f32, f32)) -> Option<(f32, f32)> {
        // Approximate C++ object transform inverse path:
        // translate -> center shift -> rotate -> scale. Invert in reverse order.
        let sx = obj.scale_x;
        let sy = obj.scale_y;
        if sx.abs() < f32::EPSILON || sy.abs() < f32::EPSILON {
            return None;
        }

        let mut x = cursor.0 - obj.x;
        let mut y = cursor.1 - obj.y;

        x -= obj.center_x;
        y -= obj.center_y;

        let rad = obj.rotate_z_deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        let rx = x * cos + y * sin;
        let ry = -x * sin + y * cos;

        let lx = rx / sx + obj.center_x;
        let ly = ry / sy + obj.center_y;
        Some((lx, ly))
    }

    fn local_in_object_clip(&self, obj: &HostObjectState, local: (f32, f32)) -> bool {
        if obj.dst_clip_use {
            let (left, right) = ordered_clip_bounds(obj.dst_clip_left, obj.dst_clip_right);
            let (top, bottom) = ordered_clip_bounds(obj.dst_clip_top, obj.dst_clip_bottom);
            if local.0 < left || local.0 >= right || local.1 < top || local.1 >= bottom {
                return false;
            }
        }
        true
    }

    fn object_alpha_test_hit(&self, obj: &HostObjectState, local: (f32, f32)) -> bool {
        if obj.alpha <= 0.0 || obj.file_name.is_empty() {
            return false;
        }

        let Ok(img) = load_stage_like_cpp(
            &self.base_dir,
            &self.append_dirs,
            &obj.file_name,
            obj.pat_no,
        ) else {
            return false;
        };
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        if w == 0 || h == 0 {
            return false;
        }

        let (sample_x, sample_y) = self.local_to_source_sample(obj, local, w as f32, h as f32);
        let px = sample_to_pixel_index(sample_x, w);
        let py = sample_to_pixel_index(sample_y, h);
        rgba.get_pixel(px, py)[3] > 0
    }

    fn local_to_source_sample(
        &self,
        obj: &HostObjectState,
        local: (f32, f32),
        img_w: f32,
        img_h: f32,
    ) -> (f32, f32) {
        if !obj.src_clip_use {
            return local;
        }

        let (dst_left, dst_right) = if obj.dst_clip_use {
            (obj.dst_clip_left, obj.dst_clip_right)
        } else {
            (0.0, img_w)
        };
        let (dst_top, dst_bottom) = if obj.dst_clip_use {
            (obj.dst_clip_top, obj.dst_clip_bottom)
        } else {
            (0.0, img_h)
        };

        let mut u = normalized_signed_axis(local.0, dst_left, dst_right);
        let mut v = normalized_signed_axis(local.1, dst_top, dst_bottom);

        if obj.scale_x < 0.0 {
            u = 1.0 - u;
        }
        if obj.scale_y < 0.0 {
            v = 1.0 - v;
        }

        let sx = obj.src_clip_left + (obj.src_clip_right - obj.src_clip_left) * u;
        let sy = obj.src_clip_top + (obj.src_clip_bottom - obj.src_clip_top) * v;
        (sx, sy)
    }
}

fn normalized_signed_axis(value: f32, start: f32, end: f32) -> f32 {
    let span = end - start;
    if span.abs() < f32::EPSILON {
        return 0.0;
    }
    ((value - start) / span).clamp(0.0, 1.0)
}

fn ordered_clip_bounds(a: f32, b: f32) -> (f32, f32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn sample_to_pixel_index(sample: f32, size: u32) -> u32 {
    if size <= 1 {
        return 0;
    }
    let max_edge = size as f32 - f32::EPSILON;
    sample.clamp(0.0, max_edge) as u32
}
