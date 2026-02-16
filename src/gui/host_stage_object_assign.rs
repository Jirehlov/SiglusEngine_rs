impl GuiHost {
    pub(super) fn apply_object_assign(
        &mut self,
        plane: StagePlane,
        object_index: i32,
        prop: i32,
        rhs: &siglus::vm::Prop,
    ) {
                let Some(v) = rhs.as_int() else {
            return;
        };

        match prop {
            x if x == siglus::elm::objectlist::ELM_OBJECT_DISP => {
                let visible_v = {
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.visible = v != 0;
                    state.visible
                };
                let _ = self.event_tx.send(HostEvent::SetObjectVisible {
                    stage: plane,
                    index: object_index,
                    visible: visible_v,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_PATNO => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.pat_no = v.max(0) as usize;
                self.emit_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_ORDER => {
                let (order_v, layer_v, seq_v) = {
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.order = v;
                    (state.order, state.layer, state.seq)
                };
                let _ = self.event_tx.send(HostEvent::SetObjectSort {
                    stage: plane,
                    index: object_index,
                    order: order_v,
                    layer: layer_v,
                    seq: seq_v,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_LAYER => {
                let (order_v, layer_v, seq_v) = {
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.layer = v;
                    (state.order, state.layer, state.seq)
                };
                let _ = self.event_tx.send(HostEvent::SetObjectSort {
                    stage: plane,
                    index: object_index,
                    order: order_v,
                    layer: layer_v,
                    seq: seq_v,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_X => {
                let y = {
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.x = v as f32;
                    state.y
                };
                let _ = self.event_tx.send(HostEvent::SetObjectPos {
                    stage: plane,
                    index: object_index,
                    x: v as f32,
                    y,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_Y => {
                let x = {
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.y = v as f32;
                    state.x
                };
                let _ = self.event_tx.send(HostEvent::SetObjectPos {
                    stage: plane,
                    index: object_index,
                    x,
                    y: v as f32,
                });
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CENTER_X => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.center_x = v as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CENTER_Y => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.center_y = v as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SCALE_X => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.scale_x = v as f32 / 1000.0;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SCALE_Y => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.scale_y = v as f32 / 1000.0;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_ROTATE_Z => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.rotate_z_deg = v as f32 / 10.0;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_ALPHA_BLEND => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.alpha_blend = v != 0;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_TR => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.alpha = (v as f32 / 255.0).clamp(0.0, 1.0);
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CLIP_USE => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.dst_clip_use = v != 0;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CLIP_LEFT => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.dst_clip_left = v as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CLIP_TOP => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.dst_clip_top = v as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CLIP_RIGHT => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.dst_clip_right = v as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_CLIP_BOTTOM => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.dst_clip_bottom = v as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SRC_CLIP_USE => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.src_clip_use = v != 0;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SRC_CLIP_LEFT => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.src_clip_left = v as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SRC_CLIP_TOP => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.src_clip_top = v as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SRC_CLIP_RIGHT => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.src_clip_right = v as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_SRC_CLIP_BOTTOM => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.src_clip_bottom = v as f32;
                self.emit_object_render_state(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_RATE => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.color_rate = (v as f32 / 1000.0).clamp(0.0, 4.0);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_R => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.color_r = (v as f32 / 255.0).clamp(0.0, 4.0);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_G => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.color_g = (v as f32 / 255.0).clamp(0.0, 4.0);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_B => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.color_b = (v as f32 / 255.0).clamp(0.0, 4.0);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_ADD_R => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.color_add_r = v as f32;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_ADD_G => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.color_add_g = v as f32;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_COLOR_ADD_B => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.color_add_b = v as f32;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_BRIGHT => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.bright = v as f32;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_DARK => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.dark = v as f32;
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_MONO => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.mono = (v as f32 / 255.0).clamp(0.0, 1.0);
                self.refresh_object_image(plane, object_index);
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_REVERSE => {
                let state = self.get_or_create_object_state(plane, object_index);
                state.reverse = v != 0;
                self.refresh_object_image(plane, object_index);
            }
            _ => {}
        }
    }

}
