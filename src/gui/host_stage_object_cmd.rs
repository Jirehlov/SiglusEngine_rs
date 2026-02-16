impl GuiHost {
    pub(super) fn apply_object_command(
        &mut self,
        plane: StagePlane,
        object_index: i32,
        cmd: i32,
        args: &[siglus::vm::Prop],
    ) {
        
        match cmd {
            x if x == siglus::elm::objectlist::ELM_OBJECT_CHANGE_FILE => {
                if let Some(siglus::vm::Prop {
                    value: siglus::vm::PropValue::Str(file_name),
                    ..
                }) = args.first()
                {
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.file_name = file_name.clone();
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
                    let state = self.get_or_create_object_state(plane, object_index);
                    state.file_name = file_name.clone();
                    state.visible = args.get(1).and_then(|p| p.as_int()).unwrap_or(1) != 0;
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
                let _ = self.event_tx.send(HostEvent::SetObjectVisible {
                    stage: plane,
                    index: object_index,
                    visible,
                });
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
            x if x == siglus::elm::objectlist::ELM_OBJECT_CREATE_CAPTURE
                || x == siglus::elm::objectlist::ELM_OBJECT_CREATE_FROM_CAPTURE_FILE =>
            {
                // Treat these creation commands as lifecycle reset points.
                let st = self.get_or_create_object_state(plane, object_index);
                reset_object_state_preserve_seq(st);
                let _ = self.event_tx.send(HostEvent::RemoveObject {
                    stage: plane,
                    index: object_index,
                });
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
                }
            }
            x if x == siglus::elm::objectlist::ELM_OBJECT_FREE || x == siglus::elm::objectlist::ELM_OBJECT_INIT => {
                let st = self.get_or_create_object_state(plane, object_index);
                reset_object_state_preserve_seq(st);
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
            _ => {
                if is_object_file_create_command(cmd) {
                    if let Some(siglus::vm::Prop {
                        value: siglus::vm::PropValue::Str(file_name),
                        ..
                    }) = args.first()
                    {
                        let state = self.get_or_create_object_state(plane, object_index);
                        state.file_name = file_name.clone();
                        state.visible = args.get(1).and_then(|p| p.as_int()).unwrap_or(1) != 0;
                        self.emit_object_image(plane, object_index);
                    }
                }
            }
        }
    }

}
