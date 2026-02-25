impl Vm {
    fn object_frame_action_push_default(stack: &mut IfcStack, ret_form: i32) {
        if ret_form == crate::elm::form::INT {
            stack.push_int(0);
        } else if ret_form == crate::elm::form::STR {
            stack.push_str(String::new());
        }
    }

    fn frame_counter_object_epoch_slot(
        list_id: i32,
        obj_idx: i32,
        stage_idx: Option<i32>,
    ) -> usize {
        let stage = stage_idx.unwrap_or(-1).max(-1) as usize + 1;
        let list = list_id.max(0) as usize;
        let obj = obj_idx.max(0) as usize;
        Self::FRAME_COUNTER_OBJECT_EPOCH_BASE + stage * 200_000 + list * 20_000 + obj
    }

    fn frame_counter_bind_object_context(
        &mut self,
        slot: usize,
        list_id: i32,
        obj_idx: i32,
        sub: i32,
        stage_idx: Option<i32>,
        ch_idx: Option<i32>,
    ) {
        let specs = [
            (Self::FRAME_COUNTER_CTX_LIST_ID, list_id),
            (Self::FRAME_COUNTER_CTX_OBJ_IDX, obj_idx),
            (Self::FRAME_COUNTER_CTX_SUB_ID, sub),
            (Self::FRAME_COUNTER_CTX_STAGE_IDX, stage_idx.unwrap_or(-1)),
            (Self::FRAME_COUNTER_CTX_CH_IDX, ch_idx.unwrap_or(-1)),
            (
                Self::FRAME_COUNTER_CTX_EPOCH,
                self.frame_counter_object_epoch(list_id, obj_idx, stage_idx),
            ),
            (Self::FRAME_COUNTER_CTX_BOUND, 1),
        ];
        for (kind, value) in specs {
            let mslot = Self::frame_counter_meta_slot(slot, kind);
            self.ensure_counter_slot(mslot);
            self.counter_values[mslot] = value;
            self.counter_active[mslot] = false;
        }
    }

    fn frame_counter_object_epoch(
        &self,
        list_id: i32,
        obj_idx: i32,
        stage_idx: Option<i32>,
    ) -> i32 {
        self.counter_values
            .get(Self::frame_counter_object_epoch_slot(
                list_id, obj_idx, stage_idx,
            ))
            .copied()
            .unwrap_or(0)
    }

    pub(super) fn frame_counter_begin_object_rebind_guard(
        &mut self,
        list_id: i32,
        obj_idx: i32,
        stage_idx: Option<i32>,
    ) {
        let epoch_slot = Self::frame_counter_object_epoch_slot(list_id, obj_idx, stage_idx);
        self.ensure_counter_slot(epoch_slot);
        self.counter_values[epoch_slot] = self.counter_values[epoch_slot].saturating_add(1);
        self.counter_active[epoch_slot] = false;
    }

    fn frame_counter_invalidate_object_context_inner(
        &mut self,
        list_id: i32,
        obj_idx: i32,
        stage_idx: Option<i32>,
    ) {
        let Some(meta_slots) = self
            .counter_values
            .len()
            .checked_sub(Self::FRAME_COUNTER_META_BASE)
        else {
            return;
        };
        if meta_slots < Self::FRAME_COUNTER_META_SPAN {
            return;
        }
        let target_stage = stage_idx.unwrap_or(-1);
        let max_slot = meta_slots / Self::FRAME_COUNTER_META_SPAN;
        for slot in 0..max_slot {
            let Some((ctx_list, ctx_obj, _ctx_sub, ctx_stage, _ctx_ch)) =
                self.frame_counter_object_context(slot)
            else {
                continue;
            };
            if ctx_list != list_id || ctx_obj != obj_idx {
                continue;
            }
            if ctx_stage.unwrap_or(-1) != target_stage {
                continue;
            }
            self.counter_active[slot] = false;
            self.counter_values[slot] = 0;
            for kind in 0..Self::FRAME_COUNTER_META_SPAN {
                let mslot = Self::frame_counter_meta_slot(slot, kind);
                self.ensure_counter_slot(mslot);
                self.counter_values[mslot] = 0;
                self.counter_active[mslot] = false;
            }
        }
        self.frame_counter_reclaim_meta_tail();
    }

    pub(super) fn frame_counter_invalidate_object_context(
        &mut self,
        list_id: i32,
        obj_idx: i32,
        stage_idx: Option<i32>,
    ) {
        self.frame_counter_begin_object_rebind_guard(list_id, obj_idx, stage_idx);
        self.frame_counter_invalidate_object_context_inner(list_id, obj_idx, stage_idx);
    }

    pub(super) fn frame_counter_invalidate_object_context_after_guard(
        &mut self,
        list_id: i32,
        obj_idx: i32,
        stage_idx: Option<i32>,
    ) {
        self.frame_counter_invalidate_object_context_inner(list_id, obj_idx, stage_idx);
    }

    pub(super) fn frame_counter_reset_stage_runtime_state(&mut self, stage_idxs: &[i32]) {
        if stage_idxs.is_empty() {
            return;
        }
        let Some(meta_slots) = self
            .counter_values
            .len()
            .checked_sub(Self::FRAME_COUNTER_META_BASE)
        else {
            return;
        };
        if meta_slots < Self::FRAME_COUNTER_META_SPAN {
            return;
        }
        let max_slot = meta_slots / Self::FRAME_COUNTER_META_SPAN;
        for slot in 0..max_slot {
            let Some((_ctx_list, _ctx_obj, _ctx_sub, ctx_stage, _ctx_ch)) =
                self.frame_counter_object_context(slot)
            else {
                continue;
            };
            let Some(ctx_stage) = ctx_stage else {
                continue;
            };
            if !stage_idxs.contains(&ctx_stage) {
                continue;
            }
            self.counter_active[slot] = false;
            self.counter_values[slot] = 0;
            for kind in 0..Self::FRAME_COUNTER_META_SPAN {
                let mslot = Self::frame_counter_meta_slot(slot, kind);
                self.ensure_counter_slot(mslot);
                self.counter_values[mslot] = 0;
                self.counter_active[mslot] = false;
            }
        }
        self.frame_counter_reclaim_meta_tail();
    }

    fn frame_counter_reclaim_meta_tail(&mut self) {
        if self.counter_values.len() <= Self::FRAME_COUNTER_META_BASE {
            return;
        }
        let mut trim_to = self.counter_values.len();
        while trim_to > Self::FRAME_COUNTER_META_BASE {
            let idx = trim_to - 1;
            if self.counter_values.get(idx).copied().unwrap_or(0) != 0 {
                break;
            }
            if self.counter_active.get(idx).copied().unwrap_or(false) {
                break;
            }
            trim_to -= 1;
        }
        if trim_to < self.counter_values.len() {
            let old_len = self.counter_values.len();
            self.counter_values.truncate(trim_to);
            self.counter_active.truncate(trim_to);
            if trim_to <= Self::EXCALL_COUNTER_BASE
                && old_len > Self::EXCALL_COUNTER_BASE
                && Self::excall_counter_trace_enabled()
            {
                let line = crate::vm::format_excall_counter_trace(
                    Self::EXCALL_COUNTER_BASE,
                    crate::vm::VmExcallCounterPhase::Reclaim,
                    trim_to as i32,
                    false,
                );
                log::debug!("{} old_len={} trim_to={}", line, old_len, trim_to);
            }
        }
    }

    fn frame_counter_object_context(
        &self,
        slot: usize,
    ) -> Option<(i32, i32, i32, Option<i32>, Option<i32>)> {
        let bound = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_CTX_BOUND,
            ))
            .copied()
            .unwrap_or(0)
            != 0;
        if !bound {
            return None;
        }
        let list_id = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_CTX_LIST_ID,
            ))
            .copied()
            .unwrap_or(0);
        let obj_idx = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_CTX_OBJ_IDX,
            ))
            .copied()
            .unwrap_or(0);
        let sub = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_CTX_SUB_ID,
            ))
            .copied()
            .unwrap_or(0);
        let stage_raw = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_CTX_STAGE_IDX,
            ))
            .copied()
            .unwrap_or(-1);
        let ch_raw = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_CTX_CH_IDX,
            ))
            .copied()
            .unwrap_or(-1);
        let stage_idx = if stage_raw >= 0 {
            Some(stage_raw)
        } else {
            None
        };
        let ch_idx = if ch_raw >= 0 { Some(ch_raw) } else { None };
        Some((list_id, obj_idx, sub, stage_idx, ch_idx))
    }

    fn frame_counter_context_slot_matches(
        &self,
        slot: usize,
        list_id: i32,
        obj_idx: i32,
        stage_idx: Option<i32>,
        ch_idx: Option<i32>,
    ) -> bool {
        if Self::frame_action_counter_slot(list_id, obj_idx, stage_idx, ch_idx) != slot {
            return false;
        }
        let bound_epoch = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_CTX_EPOCH,
            ))
            .copied()
            .unwrap_or(0);
        bound_epoch == self.frame_counter_object_epoch(list_id, obj_idx, stage_idx)
    }

    fn frame_counter_tick_by_mode(
        &mut self,
        slot: usize,
        host: &mut dyn Host,
        list_id: i32,
        obj_idx: i32,
        sub: i32,
        stage_idx: Option<i32>,
        ch_idx: Option<i32>,
    ) {
        let (past_game_time, past_real_time) = host
            .on_object_frame_action_counter_elapsed(list_id, obj_idx, sub, stage_idx, ch_idx)
            .unwrap_or_else(|| host.on_frame_counter_elapsed());
        let before = self.counter_values.get(slot).copied().unwrap_or(0);
        let real = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_MODE_REAL,
            ))
            .copied()
            .unwrap_or(0)
            != 0;
        if real {
            self.frame_counter_tick_real(slot, past_real_time);
        } else {
            self.frame_counter_tick_game(slot, past_game_time);
        }
        let after = self.counter_values.get(slot).copied().unwrap_or(0);
        let active = self.counter_active.get(slot).copied().unwrap_or(false);
        Self::trace_excall_counter_tick(
            slot, list_id, obj_idx, sub, stage_idx, ch_idx, before, after, active,
        );
    }
}
