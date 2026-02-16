impl GuiHost {
    fn get_or_create_object_state(
        &mut self,
        plane: StagePlane,
        object_index: i32,
    ) -> &mut HostObjectState {
        use std::collections::btree_map::Entry;
        let key = (plane, object_index);
        match self.objects.entry(key) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(v) => {
                let mut st = default_host_object_state();
                st.seq = self.next_object_seq;
                self.next_object_seq = self.next_object_seq.saturating_add(1);
                v.insert(st)
            }
        }
    }

    fn emit_object_render_state(&mut self, plane: StagePlane, object_index: i32) {
        if let Some(state) = self.objects.get(&(plane, object_index)) {
            let _ = self.event_tx.send(HostEvent::SetObjectRenderState {
                stage: plane,
                index: object_index,
                center_x: state.center_x,
                center_y: state.center_y,
                scale_x: state.scale_x,
                scale_y: state.scale_y,
                rotate_z_deg: state.rotate_z_deg,
                alpha: state.alpha,
                dst_clip_use: state.dst_clip_use,
                dst_clip_left: state.dst_clip_left,
                dst_clip_top: state.dst_clip_top,
                dst_clip_right: state.dst_clip_right,
                dst_clip_bottom: state.dst_clip_bottom,
                src_clip_use: state.src_clip_use,
                src_clip_left: state.src_clip_left,
                src_clip_top: state.src_clip_top,
                src_clip_right: state.src_clip_right,
                src_clip_bottom: state.src_clip_bottom,
            });
        }
    }

    fn refresh_object_image(&mut self, plane: StagePlane, object_index: i32) {
        if self
            .objects
            .get(&(plane, object_index))
            .map(|s| !s.file_name.is_empty())
            .unwrap_or(false)
        {
            self.emit_object_image(plane, object_index);
        }
    }

    pub(super) fn apply_stage_plane_command(
        &mut self,
        plane: StagePlane,
        cmd: i32,
        args: &[siglus::vm::Prop],
    ) {
                if cmd == siglus::elm::objectlist::ELM_STAGE_CREATE_OBJECT {
            let requested = args.first().and_then(|p| p.as_int()).unwrap_or(0).max(0) as i32;
            for idx in 0..requested {
                let _ = self.get_or_create_object_state(plane, idx);
            }
            let mut removed_keys = Vec::new();
            for (&(p, idx), _) in &self.objects {
                if p == plane && idx >= requested {
                    removed_keys.push((p, idx));
                }
            }
            for key in removed_keys {
                self.objects.remove(&key);
                let _ = self.event_tx.send(HostEvent::RemoveObject {
                    stage: key.0,
                    index: key.1,
                });
            }
            if requested == 0 {
                let _ = self
                    .event_tx
                    .send(HostEvent::ClearPlaneObjects { stage: plane });
            }
        }
    }

    fn emit_object_image(&mut self, plane: StagePlane, object_index: i32) {
        let Some(state) = self.objects.get(&(plane, object_index)).cloned() else {
            return;
        };
        if state.file_name.is_empty() {
            return;
        }

        match load_stage_like_cpp(
            &self.base_dir,
            &self.append_dirs,
            &state.file_name,
            state.pat_no,
        ) {
            Ok(img) => {
                let img = apply_color_semantics(&img, &state);
                let _ = self.event_tx.send(HostEvent::UpsertObjectImage {
                    stage: plane,
                    index: object_index,
                    image: Arc::new(img),
                });
                let _ = self.event_tx.send(HostEvent::SetObjectPos {
                    stage: plane,
                    index: object_index,
                    x: state.x,
                    y: state.y,
                });
                let _ = self.event_tx.send(HostEvent::SetObjectVisible {
                    stage: plane,
                    index: object_index,
                    visible: state.visible,
                });
                let _ = self.event_tx.send(HostEvent::SetObjectSort {
                    stage: plane,
                    index: object_index,
                    order: state.order,
                    layer: state.layer,
                    seq: state.seq,
                });
                self.emit_object_render_state(plane, object_index);
            }
            Err(e) => {
                let detail = format!("{e:#}");
                if detail.contains("image not found") {
                    error!(
                        "Failed to load object image {} (stage={:?}, idx={}): [NOT_FOUND] {}",
                        state.file_name, plane, object_index, detail
                    );
                } else {
                    error!(
                        "Failed to load object image {} (stage={:?}, idx={}): [DECODE_OR_PARSE] {}",
                        state.file_name, plane, object_index, detail
                    );
                }
                let _ = self.event_tx.send(HostEvent::MissingObjectImage {
                    stage: plane,
                    index: object_index,
                    name: state.file_name.clone(),
                });
            }
        }
    }
}
