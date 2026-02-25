impl Vm {
    const FRAME_COUNTER_META_BASE: usize = 3_000_000;
    const FRAME_COUNTER_OBJECT_EPOCH_BASE: usize = 2_600_000;
    const FRAME_COUNTER_META_SPAN: usize = 14;
    const FRAME_COUNTER_MODE_FRAME: usize = 0;
    const FRAME_COUNTER_MODE_LOOP: usize = 1;
    const FRAME_COUNTER_MODE_REAL: usize = 2;
    const FRAME_COUNTER_START_VALUE: usize = 3;
    const FRAME_COUNTER_END_VALUE: usize = 4;
    const FRAME_COUNTER_FRAME_TIME: usize = 5;
    const FRAME_COUNTER_CUR_TIME: usize = 6;
    const FRAME_COUNTER_CTX_LIST_ID: usize = 7;
    const FRAME_COUNTER_CTX_OBJ_IDX: usize = 8;
    const FRAME_COUNTER_CTX_SUB_ID: usize = 9;
    const FRAME_COUNTER_CTX_STAGE_IDX: usize = 10;
    const FRAME_COUNTER_CTX_CH_IDX: usize = 11;
    const FRAME_COUNTER_CTX_BOUND: usize = 12;
    const FRAME_COUNTER_CTX_EPOCH: usize = 13;
    fn frame_action_counter_slot(
        list_id: i32,
        obj_idx: i32,
        stage_idx: Option<i32>,
        ch_idx: Option<i32>,
    ) -> usize {
        let stage = stage_idx.unwrap_or(-1).max(-1) as usize + 1;
        let list = list_id.max(0) as usize;
        let obj = obj_idx.max(0) as usize;
        let ch = ch_idx.unwrap_or(-1).max(-1) as usize + 1;
        1_000_000 + stage * 200_000 + list * 20_000 + obj * 64 + ch
    }
    fn frame_counter_meta_slot(slot: usize, kind: usize) -> usize {
        Self::FRAME_COUNTER_META_BASE + slot * Self::FRAME_COUNTER_META_SPAN + kind
    }
    fn frame_counter_set_mode(
        &mut self,
        slot: usize,
        frame_mode: bool,
        frame_loop: bool,
        frame_real: bool,
        start_value: i32,
        end_value: i32,
        frame_time: i32,
    ) {
        let specs = [
            (Self::FRAME_COUNTER_MODE_FRAME, frame_mode as i32),
            (Self::FRAME_COUNTER_MODE_LOOP, frame_loop as i32),
            (Self::FRAME_COUNTER_MODE_REAL, frame_real as i32),
            (Self::FRAME_COUNTER_START_VALUE, start_value),
            (Self::FRAME_COUNTER_END_VALUE, end_value),
            (Self::FRAME_COUNTER_FRAME_TIME, frame_time),
            (Self::FRAME_COUNTER_CUR_TIME, 0),
        ];
        for (kind, value) in specs {
            let mslot = Self::frame_counter_meta_slot(slot, kind);
            self.ensure_counter_slot(mslot);
            self.counter_values[mslot] = value;
            self.counter_active[mslot] = false;
        }
        self.counter_values[slot] = self.frame_counter_current_value(slot);
    }
    pub(super) fn frame_action_counter_tick_all(&mut self, host: &mut dyn Host) {
        if self.script_counter_time_stop_flag {
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
            if !self.counter_active.get(slot).copied().unwrap_or(false) {
                continue;
            }
            if let Some((list_id, obj_idx, sub, stage_idx, ch_idx)) =
                self.frame_counter_object_context(slot)
            {
                if !self
                    .frame_counter_context_slot_matches(slot, list_id, obj_idx, stage_idx, ch_idx)
                {
                    self.counter_active[slot] = false;
                    continue;
                }
                if sub == crate::elm::objectlist::ELM_OBJECT_FRAME_ACTION_CH {
                    if let Some(ch) = ch_idx {
                        if let Some(size) = Self::object_frame_action_ch_size(
                            host, list_id, obj_idx, sub, stage_idx,
                        ) {
                            if ch < 0 || ch >= size {
                                self.counter_active[slot] = false;
                                continue;
                            }
                        }
                    }
                }
                self.frame_counter_tick_by_mode(
                    slot, host, list_id, obj_idx, sub, stage_idx, ch_idx,
                );
            }
        }
    }
    fn frame_counter_stop_for_object(
        &mut self,
        list_id: i32,
        obj_idx: i32,
        stage_idx: Option<i32>,
        sub: i32,
        ch_idx: Option<i32>,
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
        let target_ch = ch_idx.unwrap_or(-1);
        let max_slot = meta_slots / Self::FRAME_COUNTER_META_SPAN;
        for slot in 0..max_slot {
            let Some((ctx_list, ctx_obj, ctx_sub, ctx_stage, ctx_ch)) =
                self.frame_counter_object_context(slot)
            else {
                continue;
            };
            if ctx_list != list_id || ctx_obj != obj_idx || ctx_sub != sub {
                continue;
            }
            if ctx_stage.unwrap_or(-1) != target_stage {
                continue;
            }
            if sub == crate::elm::objectlist::ELM_OBJECT_FRAME_ACTION_CH
                && ctx_ch.unwrap_or(-1) != target_ch
            {
                continue;
            }
            self.counter_active[slot] = false;
            let bound_slot = Self::frame_counter_meta_slot(slot, Self::FRAME_COUNTER_CTX_BOUND);
            self.ensure_counter_slot(bound_slot);
            self.counter_values[bound_slot] = 0;
            for kind in 0..Self::FRAME_COUNTER_META_SPAN {
                let mslot = Self::frame_counter_meta_slot(slot, kind);
                self.ensure_counter_slot(mslot);
                self.counter_values[mslot] = 0;
                self.counter_active[mslot] = false;
            }
            self.counter_values[slot] = 0;
        }
        self.frame_counter_reclaim_meta_tail();
    }
    fn frame_counter_current_value(&self, slot: usize) -> i32 {
        let frame_mode = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_MODE_FRAME,
            ))
            .copied()
            .unwrap_or(0)
            != 0;
        if !frame_mode {
            return self.counter_values.get(slot).copied().unwrap_or(0);
        }
        let frame_loop = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_MODE_LOOP,
            ))
            .copied()
            .unwrap_or(0)
            != 0;
        let start_value = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_START_VALUE,
            ))
            .copied()
            .unwrap_or(0);
        let end_value = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_END_VALUE,
            ))
            .copied()
            .unwrap_or(0);
        let frame_time = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_FRAME_TIME,
            ))
            .copied()
            .unwrap_or(0);
        let cur_time = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_CUR_TIME,
            ))
            .copied()
            .unwrap_or(0);
        if frame_time <= 0 || start_value == end_value {
            return end_value;
        }
        let delta = end_value - start_value;
        let mut value = (delta as i64 * cur_time as i64 / frame_time as i64) as i32;
        if frame_loop {
            if delta == 0 {
                return end_value;
            }
            value %= delta;
            value + start_value
        } else {
            value += start_value;
            if start_value > end_value {
                value.clamp(end_value, start_value)
            } else {
                value.clamp(start_value, end_value)
            }
        }
    }
    fn frame_counter_set_value(&mut self, slot: usize, value: i32) {
        let frame_mode = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_MODE_FRAME,
            ))
            .copied()
            .unwrap_or(0)
            != 0;
        if !frame_mode {
            self.counter_values[slot] = value;
            return;
        }
        let start_value = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_START_VALUE,
            ))
            .copied()
            .unwrap_or(0);
        let end_value = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_END_VALUE,
            ))
            .copied()
            .unwrap_or(0);
        let frame_time = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_FRAME_TIME,
            ))
            .copied()
            .unwrap_or(0);
        let loop_mode = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_MODE_LOOP,
            ))
            .copied()
            .unwrap_or(0)
            != 0;
        let cur_slot = Self::frame_counter_meta_slot(slot, Self::FRAME_COUNTER_CUR_TIME);
        self.ensure_counter_slot(cur_slot);
        if end_value == start_value {
            self.counter_values[cur_slot] = 0;
            self.counter_values[slot] = end_value;
            return;
        }
        let mut cur = (value - start_value) * frame_time / (end_value - start_value);
        if loop_mode {
            let upper = frame_time - 1;
            if upper < 0 {
                cur = 0;
            } else {
                cur = cur.clamp(0, upper);
            }
        } else {
            cur = cur.clamp(0, frame_time.max(0));
        }
        self.counter_values[cur_slot] = cur;
        self.counter_values[slot] = self.frame_counter_current_value(slot);
    }
    fn frame_counter_tick_inner(&mut self, slot: usize, elapsed: i32) {
        if !self.counter_active.get(slot).copied().unwrap_or(false) {
            return;
        }
        let step = elapsed.max(0);
        let frame_mode = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_MODE_FRAME,
            ))
            .copied()
            .unwrap_or(0)
            != 0;
        if !frame_mode {
            if let Some(v) = self.counter_values.get_mut(slot) {
                *v += step;
            }
            return;
        }
        let frame_loop = self
            .counter_values
            .get(Self::frame_counter_meta_slot(
                slot,
                Self::FRAME_COUNTER_MODE_LOOP,
            ))
            .copied()
            .unwrap_or(0)
            != 0;
        let frame_time_slot = Self::frame_counter_meta_slot(slot, Self::FRAME_COUNTER_FRAME_TIME);
        let cur_time_slot = Self::frame_counter_meta_slot(slot, Self::FRAME_COUNTER_CUR_TIME);
        self.ensure_counter_slot(frame_time_slot.max(cur_time_slot));
        self.counter_values[cur_time_slot] += step;
        let frame_time = self.counter_values[frame_time_slot];
        if !frame_loop && frame_time > 0 && self.counter_values[cur_time_slot] >= frame_time {
            self.counter_active[slot] = false;
        }
        self.counter_values[slot] = self.frame_counter_current_value(slot);
    }
    fn try_command_object_frame_action(
        &mut self,
        list_id: i32,
        obj_idx: i32,
        sub: i32,
        element: &[i32],
        args: &[Prop],
        ret_form: i32,
        stage_idx: Option<i32>,
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::frameaction::{
            is_frameaction_end, is_frameaction_is_end_action, is_frameaction_start,
            is_frameactionlist_get_size, is_frameactionlist_resize,
        };
        use crate::elm::objectlist::{ELM_OBJECT_FRAME_ACTION, ELM_OBJECT_FRAME_ACTION_CH};
        let tail = if element.len() > 1 {
            &element[1..]
        } else {
            &[]
        };
        if tail.is_empty() {
            host.on_error_fatal("無効なコマンドが指定されました。(frame_action)");
            return true;
        }
        if tail[0] == crate::elm::ELM_UP {
            host.on_error_fatal("無効なコマンドが指定されました。(frame_action)");
            return true;
        }

        let method = if sub == ELM_OBJECT_FRAME_ACTION_CH && tail[0] == crate::elm::ELM_ARRAY {
            if tail.len() < 2 {
                host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                return true;
            }
            let idx = tail[1];
            if idx < 0 {
                if self.options.disp_out_of_range_error {
                    host.on_error_fatal(
                        "範囲外のフレームアクション番号が指定されました。(frame_action_ch)",
                    );
                }
                Self::object_frame_action_push_default(&mut self.stack, ret_form);
                return true;
            }
            let ch_size = Self::object_frame_action_ch_size(host, list_id, obj_idx, sub, stage_idx);
            if let Some(size) = ch_size {
                if idx >= size {
                    if self.options.disp_out_of_range_error {
                        host.on_error_fatal(
                            "範囲外のフレームアクション番号が指定されました。(frame_action_ch)",
                        );
                    }
                    Self::object_frame_action_push_default(&mut self.stack, ret_form);
                    return true;
                }
            }
            if tail.len() < 3 {
                host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                return true;
            }
            if tail[2] == crate::elm::ELM_UP {
                if tail.len() < 4 {
                    host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                    Self::object_frame_action_push_default(&mut self.stack, ret_form);
                    return true;
                }
                if tail[3] == crate::elm::frameaction::ELM_FRAMEACTION_COUNTER {
                    if tail.len() < 5 {
                        host.on_error_fatal(
                            "無効なコマンドが指定されました。(frame_action.counter)",
                        );
                    } else {
                        host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                    }
                    Self::object_frame_action_push_default(&mut self.stack, ret_form);
                    return true;
                }
                if is_frameactionlist_get_size(tail[3]) {
                    if tail.len() > 4 {
                        host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                        Self::object_frame_action_push_default(&mut self.stack, ret_form);
                        return true;
                    }
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(ch_size.unwrap_or(0));
                    }
                    return true;
                }
                if is_frameactionlist_resize(tail[3]) {
                    if tail.len() > 4 {
                        host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                        Self::object_frame_action_push_default(&mut self.stack, ret_form);
                        return true;
                    }
                    self.frame_counter_begin_object_rebind_guard(list_id, obj_idx, stage_idx);
                    host.on_object_action(list_id, obj_idx, sub, args, stage_idx);
                    self.frame_counter_invalidate_object_context_after_guard(
                        list_id, obj_idx, stage_idx,
                    );
                    return true;
                }
                tail[3]
            } else {
                if is_frameactionlist_get_size(tail[2]) || is_frameactionlist_resize(tail[2]) {
                    host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                    Self::object_frame_action_push_default(&mut self.stack, ret_form);
                    return true;
                }
                tail[2]
            }
        } else {
            tail[0]
        };

        if sub == ELM_OBJECT_FRAME_ACTION_CH {
            if is_frameactionlist_get_size(method) {
                if ret_form == crate::elm::form::INT {
                    if let Some((v, _)) =
                        host.on_object_frame_action_property(list_id, obj_idx, sub, tail, stage_idx)
                    {
                        self.push_vm_value(crate::elm::form::INT, v);
                    } else {
                        self.stack.push_int(0);
                    }
                }
                return true;
            }
            if is_frameactionlist_resize(method) {
                self.frame_counter_begin_object_rebind_guard(list_id, obj_idx, stage_idx);
                host.on_object_action(list_id, obj_idx, sub, args, stage_idx);
                self.frame_counter_invalidate_object_context_after_guard(
                    list_id, obj_idx, stage_idx,
                );
                return true;
            }
        }

        if sub == ELM_OBJECT_FRAME_ACTION {
            use crate::elm::objectlist::{ELM_OBJECT_LOAD_GAN, ELM_OBJECT_START_GAN};
            if method == ELM_OBJECT_LOAD_GAN || method == ELM_OBJECT_START_GAN {
                let should_invalidate = if method == ELM_OBJECT_LOAD_GAN {
                    let gan_path = Self::object_arg_str(args, 0);
                    self.object_gan_track_load_changed(list_id, obj_idx, stage_idx, &gan_path)
                } else {
                    let set_no = Self::arg_int(args, 0);
                    self.object_gan_track_start_changed(list_id, obj_idx, stage_idx, set_no)
                };
                host.on_object_action(list_id, obj_idx, sub, args, stage_idx);
                if should_invalidate {
                    self.frame_counter_invalidate_object_context(list_id, obj_idx, stage_idx);
                }
                return true;
            }
        }

        if method == crate::elm::frameaction::ELM_FRAMEACTION_COUNTER {
            use crate::elm::counter::*;

            if sub == ELM_OBJECT_FRAME_ACTION_CH
                && Self::frame_action_ch_counter_tail_guard(tail, &mut self.stack, ret_form, host)
            {
                return true;
            }

            let counter_parse = if sub == ELM_OBJECT_FRAME_ACTION_CH {
                Self::frame_action_ch_counter_method_and_channel(tail)
            } else {
                Self::frame_action_counter_method_and_channel(sub, tail)
            };
            let Some((counter_method, ch_idx)) = counter_parse else {
                host.on_error_fatal("無効なコマンドが指定されました。(frame_action.counter)");
                return true;
            };

            if Self::frame_action_ch_up_counter_arg_guard(
                sub,
                tail,
                counter_method,
                args,
                &mut self.stack,
                ret_form,
                host,
            ) {
                return true;
            }

            let slot = Self::frame_action_counter_slot(list_id, obj_idx, stage_idx, ch_idx);
            self.ensure_counter_slot(slot);
            self.frame_counter_bind_object_context(slot, list_id, obj_idx, sub, stage_idx, ch_idx);

            let Some(int_args) = Self::frame_action_counter_int_args(
                counter_method,
                args,
                &mut self.stack,
                ret_form,
                host,
            ) else {
                return true;
            };

            match counter_method {
                ELM_COUNTER_SET => {
                    self.frame_counter_set_value(slot, int_args[0]);
                }
                ELM_COUNTER_GET => {
                    if ret_form == crate::elm::form::INT {
                        let value = self.frame_counter_current_value(slot);
                        self.counter_values[slot] = value;
                        self.stack.push_int(value);
                    }
                }
                ELM_COUNTER_RESET => {
                    self.counter_values[slot] = 0;
                    self.counter_active[slot] = false;
                    self.frame_counter_set_mode(slot, false, false, false, 0, 0, 0);
                    host.on_object_action(list_id, obj_idx, sub, args, stage_idx);
                }
                ELM_COUNTER_START | ELM_COUNTER_START_REAL | ELM_COUNTER_RESUME => {
                    self.counter_active[slot] = true;
                    host.on_object_action(list_id, obj_idx, sub, args, stage_idx);
                }
                ELM_COUNTER_START_FRAME | ELM_COUNTER_START_FRAME_REAL => {
                    let start_value = int_args[0];
                    let end_value = int_args[1];
                    let frame_time = int_args[2];
                    self.frame_counter_set_mode(
                        slot,
                        true,
                        false,
                        counter_method == ELM_COUNTER_START_FRAME_REAL,
                        start_value,
                        end_value,
                        frame_time,
                    );
                    self.counter_active[slot] = true;
                    host.on_object_action(list_id, obj_idx, sub, args, stage_idx);
                }
                ELM_COUNTER_START_FRAME_LOOP | ELM_COUNTER_START_FRAME_LOOP_REAL => {
                    let start_value = int_args[0];
                    let end_value = int_args[1];
                    let frame_time = int_args[2];
                    self.frame_counter_set_mode(
                        slot,
                        true,
                        true,
                        counter_method == ELM_COUNTER_START_FRAME_LOOP_REAL,
                        start_value,
                        end_value,
                        frame_time,
                    );
                    self.counter_active[slot] = true;
                    host.on_object_action(list_id, obj_idx, sub, args, stage_idx);
                }
                ELM_COUNTER_STOP => {
                    self.counter_active[slot] = false;
                    host.on_object_action(list_id, obj_idx, sub, args, stage_idx);
                }
                ELM_COUNTER_WAIT | ELM_COUNTER_WAIT_KEY => {
                    host.on_object_action(list_id, obj_idx, sub, args, stage_idx);
                }
                ELM_COUNTER_CHECK_VALUE => {
                    let time = int_args[0];
                    if ret_form == crate::elm::form::INT {
                        let value = self.frame_counter_current_value(slot);
                        self.counter_values[slot] = value;
                        self.stack.push_int(if value - time >= 0 { 1 } else { 0 });
                    }
                }
                ELM_COUNTER_CHECK_ACTIVE => {
                    if ret_form == crate::elm::form::INT {
                        self.stack
                            .push_int(if self.counter_active[slot] { 1 } else { 0 });
                    }
                }
                _ => {
                    host.on_error_fatal("無効なコマンドが指定されました。(frame_action.counter)");
                }
            }
            return true;
        }

        if is_frameaction_start(method) || is_frameaction_end(method) {
            if is_frameaction_start(method) {
                self.frame_counter_stop_for_object(list_id, obj_idx, stage_idx, sub, None);
            }
            if is_frameaction_end(method) {
                let ch_idx = if sub == ELM_OBJECT_FRAME_ACTION_CH
                    && tail.first().copied() == Some(crate::elm::ELM_ARRAY)
                {
                    tail.get(1).copied()
                } else {
                    None
                };
                self.frame_counter_stop_for_object(list_id, obj_idx, stage_idx, sub, ch_idx);
            }
            host.on_object_action(list_id, obj_idx, sub, args, stage_idx);
            return true;
        }

        if is_frameaction_is_end_action(method) {
            if ret_form == crate::elm::form::INT {
                if let Some((v, form)) =
                    host.on_object_frame_action_property(list_id, obj_idx, sub, tail, stage_idx)
                {
                    self.push_vm_value(form, v);
                } else {
                    self.stack.push_int(0);
                }
            }
            return true;
        }

        if sub == ELM_OBJECT_FRAME_ACTION_CH {
            host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
            return true;
        }
        if sub == ELM_OBJECT_FRAME_ACTION {
            host.on_error_fatal("無効なコマンドが指定されました。(frame_action)");
            return true;
        }
        false
    }
}
