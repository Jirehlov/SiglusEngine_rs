impl Vm {
    const EXCALL_COUNTER_BASE: usize = 5_000_000;
    const EXCALL_COUNTER_SCOPE_SPAN: usize = 100_000;
    const EXCALL_FRAME_ACTION_COUNTER_SCOPE: usize = 10_000;
    const EXCALL_FRAME_ACTION_CH_COUNTER_SCOPE: usize = 20_000;

    fn resolve_excall_scope(
        &self,
        excall_slot: Option<usize>,
        host: &mut dyn Host,
    ) -> Option<usize> {
        match excall_slot {
            None => Some(1),
            Some(0 | 1) => excall_slot,
            Some(_) => {
                host.on_error("excall[] のインデックスが範囲外です。");
                None
            }
        }
    }

    fn excall_scope_ready(&self, scope: usize) -> bool {
        scope == 0 || self.excall_allocated.get(scope).copied().unwrap_or(false)
    }

    fn excall_scope_supports_lifecycle(scope: usize) -> bool {
        scope != 0
    }

    fn excall_script_font_scope(scope: usize) -> usize {
        // C++ cmd_call.cpp routes both `excall[0].script` and `excall[1].script`
        // into cmd_script.cpp::tnm_command_proc_script_excall, which always reads/writes
        // Gp_excall->m_font_name / m_pod font fields.
        // Therefore script-excall font storage must be unified on scope1.
        if scope == 0 { 1 } else { scope }
    }

    fn excall_report_invalid_global_lifecycle(&self, host: &mut dyn Host, op: &str) {
        if self.options.disp_out_of_range_error {
            host.on_error(&format!("excall[0].{} は無効です。", op));
        }
    }

    fn excall_scope_requires_ready(element0: i32) -> bool {
        use crate::elm::excall::*;
        matches!(
            element0,
            ELM_EXCALL_F
                | ELM_EXCALL_COUNTER
                | ELM_EXCALL_FRAME_ACTION
                | ELM_EXCALL_FRAME_ACTION_CH
                | ELM_EXCALL_STAGE
                | ELM_EXCALL_BACK
                | ELM_EXCALL_FRONT
                | ELM_EXCALL_NEXT
                | ELM_EXCALL_SCRIPT
        )
    }

    fn excall_counter_slot(scope: usize, lane: usize, idx: usize) -> usize {
        Self::EXCALL_COUNTER_BASE + scope * Self::EXCALL_COUNTER_SCOPE_SPAN + lane + idx
    }

    fn reset_excall_scope_stage_runtime_state(&mut self, scope: usize) {
        if scope == 0 {
            return;
        }
        // C++ elm_excall.cpp::ready/free reinitialize m_stage_list, including back/front/next lanes.
        self.frame_counter_reset_stage_runtime_state(&[0, 1, 2]);
        self.object_gan_loaded_path
            .retain(|(_, _, stage_idx), _| *stage_idx < 0 || *stage_idx > 2);
        self.object_gan_started_set
            .retain(|(_, _, stage_idx), _| *stage_idx < 0 || *stage_idx > 2);
    }

    fn reset_excall_scope_runtime_state(&mut self, scope: usize, ready: bool) {
        if scope >= self.excall_counter_list_size.len() {
            return;
        }
        self.reset_excall_scope_stage_runtime_state(scope);
        if scope < self.excall_flags_f.len() {
            self.excall_flags_f[scope].fill(0);
            self.excall_script_font_name[scope].clear();
            self.excall_script_font_bold[scope] = -1;
            self.excall_script_font_shadow[scope] = -1;
        }
        if scope == 0 {
            self.frame_action = FrameAction::default();
            if ready {
                let n = self.options.preloaded_frame_action_ch_count;
                self.frame_action_ch.resize(n, FrameAction::default());
            } else {
                self.frame_action_ch.clear();
            }
        } else {
            self.excall_frame_action = FrameAction::default();
            if ready {
                let n = self.options.preloaded_frame_action_ch_count;
                self.excall_frame_action_ch
                    .resize(n, FrameAction::default());
            } else {
                self.excall_frame_action_ch.clear();
            }
        }
    }

    fn reset_excall_scope_counter_slots(&mut self, scope: usize) {
        if scope >= self.excall_counter_list_size.len() {
            return;
        }
        let begin = Self::EXCALL_COUNTER_BASE + scope * Self::EXCALL_COUNTER_SCOPE_SPAN;
        let end = begin + Self::EXCALL_COUNTER_SCOPE_SPAN;
        if begin >= self.counter_values.len() {
            return;
        }
        let end = end.min(self.counter_values.len());
        for idx in begin..end {
            self.counter_values[idx] = 0;
            if idx < self.counter_active.len() {
                self.counter_active[idx] = false;
            }
        }
    }

    fn push_ret_default(stack: &mut IfcStack, ret_form: i32) {
        if ret_form == crate::elm::form::INT {
            stack.push_int(0);
        } else if ret_form == crate::elm::form::STR {
            stack.push_str(String::new());
        }
    }

    fn excall_f_bit_get(list: &[i32], bit: i32, index: i32) -> i32 {
        let idx = index.max(0) as usize;
        if bit == 32 {
            return list.get(idx).copied().unwrap_or(0);
        }
        let unit = (32 / bit.max(1)).max(1) as usize;
        let word = list.get(idx / unit).copied().unwrap_or(0);
        let shift = (idx % unit) * bit as usize;
        (word >> shift) & ((1i32 << bit) - 1)
    }

    fn excall_f_bit_set(list: &mut Vec<i32>, bit: i32, index: i32, value: i32) {
        let idx = index.max(0) as usize;
        if bit == 32 {
            if list.len() <= idx {
                list.resize(idx + 1, 0);
            }
            list[idx] = value;
            return;
        }
        let unit = (32 / bit.max(1)).max(1) as usize;
        let word_idx = idx / unit;
        if list.len() <= word_idx {
            list.resize(word_idx + 1, 0);
        }
        let shift = (idx % unit) * bit as usize;
        let mask = ((1i32 << bit) - 1) << shift;
        let v = (value & ((1i32 << bit) - 1)) << shift;
        list[word_idx] = (list[word_idx] & !mask) | v;
    }

    fn excall_f_max_index(list: &[i32], bit: i32) -> usize {
        if bit == 32 {
            list.len()
        } else {
            let unit = (32 / bit.max(1)).max(1) as usize;
            list.len() * unit
        }
    }

    fn excall_f_resize_bits(list: &mut Vec<i32>, bit: i32, size: i32) {
        let n = size.max(0) as usize;
        if bit == 32 {
            list.resize(n, 0);
            return;
        }
        let unit = (32 / bit.max(1)).max(1) as usize;
        let words = if n == 0 { 0 } else { (n + unit - 1) / unit };
        list.resize(words, 0);
    }

    fn excall_report_not_ready(&self, host: &mut dyn Host) {
        if self.options.disp_out_of_range_error {
            host.on_error("システムコールが準備されていません！");
        }
    }

    fn try_command_excall_f_bits(
        &mut self,
        list: &mut Vec<i32>,
        bit: i32,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::list::*;
        if element.is_empty() {
            return true;
        }

        let method = match element[0] {
            ELM_INTLISTREF_ARRAY => ELM_INTLIST_ARRAY,
            ELM_INTLISTREF_BIT => ELM_INTLIST_BIT,
            ELM_INTLISTREF_BIT2 => ELM_INTLIST_BIT2,
            ELM_INTLISTREF_BIT4 => ELM_INTLIST_BIT4,
            ELM_INTLISTREF_BIT8 => ELM_INTLIST_BIT8,
            ELM_INTLISTREF_BIT16 => ELM_INTLIST_BIT16,
            ELM_INTLISTREF_GET_SIZE => ELM_INTLIST_GET_SIZE,
            ELM_INTLISTREF_RESIZE => ELM_INTLIST_RESIZE,
            ELM_INTLISTREF_CLEAR => ELM_INTLIST_CLEAR,
            ELM_INTLISTREF_SETS => ELM_INTLIST_SETS,
            v => v,
        };

        match method {
            ELM_INTLIST_ARRAY => {
                let idx = element.get(1).copied().unwrap_or(-1);
                let max_index = Self::excall_f_max_index(list, bit);
                if idx < 0 || (idx as usize) >= max_index {
                    if self.options.disp_out_of_range_error {
                        host.on_error("excall.F[] のインデックスが範囲外です。");
                    }
                    Self::push_ret_default(&mut self.stack, ret_form);
                    return true;
                }
                if arg_list_id == 0 {
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(Self::excall_f_bit_get(list, bit, idx));
                    }
                } else if arg_list_id == 1 {
                    Self::excall_f_bit_set(list, bit, idx, Self::arg_int(args, 0));
                }
                true
            }
            ELM_INTLIST_BIT => self.try_command_excall_f_bits(
                list,
                1,
                &element[1..],
                arg_list_id,
                args,
                ret_form,
                host,
            ),
            ELM_INTLIST_BIT2 => self.try_command_excall_f_bits(
                list,
                2,
                &element[1..],
                arg_list_id,
                args,
                ret_form,
                host,
            ),
            ELM_INTLIST_BIT4 => self.try_command_excall_f_bits(
                list,
                4,
                &element[1..],
                arg_list_id,
                args,
                ret_form,
                host,
            ),
            ELM_INTLIST_BIT8 => self.try_command_excall_f_bits(
                list,
                8,
                &element[1..],
                arg_list_id,
                args,
                ret_form,
                host,
            ),
            ELM_INTLIST_BIT16 => self.try_command_excall_f_bits(
                list,
                16,
                &element[1..],
                arg_list_id,
                args,
                ret_form,
                host,
            ),
            ELM_INTLIST_GET_SIZE => {
                if ret_form == crate::elm::form::INT {
                    self.stack
                        .push_int(Self::excall_f_max_index(list, bit) as i32);
                }
                true
            }
            ELM_INTLIST_INIT => {
                list.fill(0);
                true
            }
            ELM_INTLIST_RESIZE => {
                Self::excall_f_resize_bits(list, bit, Self::arg_int(args, 0));
                true
            }
            ELM_INTLIST_CLEAR => {
                let begin = Self::arg_int(args, 0);
                let end = Self::arg_int(args, 1);
                let fill = Self::arg_int(args, 2);
                let max_index = Self::excall_f_max_index(list, bit);
                for idx in begin..=end {
                    if idx < 0 || (idx as usize) >= max_index {
                        if self.options.disp_out_of_range_error {
                            host.on_error("excall.F clear のインデックスが範囲外です。");
                        }
                        continue;
                    }
                    Self::excall_f_bit_set(list, bit, idx, fill);
                }
                true
            }
            ELM_INTLIST_SETS => {
                let mut idx = Self::arg_int(args, 0);
                let max_index = Self::excall_f_max_index(list, bit);
                for arg in args.iter().skip(1) {
                    if idx < 0 {
                        idx += 1;
                        continue;
                    }
                    if (idx as usize) >= max_index {
                        if self.options.disp_out_of_range_error {
                            host.on_error("excall.F sets のインデックスが範囲外です。");
                        }
                        break;
                    }
                    let v = match &arg.value {
                        PropValue::Int(n) => *n,
                        PropValue::Str(s) => s.parse::<i32>().unwrap_or(0),
                        _ => 0,
                    };
                    Self::excall_f_bit_set(list, bit, idx, v);
                    idx += 1;
                }
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(excall.F)");
                true
            }
        }
    }

    fn try_command_excall_f_scoped(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
        scope: usize,
    ) -> bool {
        if scope > 1 {
            host.on_error("excall[] のインデックスが範囲外です。");
            Self::push_ret_default(&mut self.stack, ret_form);
            return true;
        }
        if element.is_empty() {
            return true;
        }

        let mut local = if scope == 0 {
            std::mem::take(&mut self.flags_f)
        } else {
            std::mem::take(&mut self.excall_flags_f[scope])
        };
        let handled = self.try_command_excall_f_bits(
            &mut local,
            32,
            element,
            arg_list_id,
            args,
            ret_form,
            host,
        );
        if scope == 0 {
            self.flags_f = local;
        } else {
            self.excall_flags_f[scope] = local;
        }
        handled
    }

    fn try_command_excall_counter_scoped(
        &mut self,
        element: &[i32],
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
        scope: usize,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        let method = Self::normalize_excall_counter_method(element[0]);
        let mut chain = element.to_vec();
        chain[0] = method;
        if scope == 0 {
            return self.try_command_counter_list_with_options(
                &chain,
                args,
                ret_form,
                host,
                self.counter_list_size,
                self.options.disp_out_of_range_error,
                "counter",
            );
        }
        let list_size = self
            .excall_counter_list_size
            .get(scope)
            .copied()
            .unwrap_or(0);
        self.try_command_counter_list_with_options(
            &chain,
            args,
            ret_form,
            host,
            list_size,
            self.options.disp_out_of_range_error,
            "excall.counter",
        )
    }

    fn normalize_excall_counter_method(method: i32) -> i32 {
        use crate::elm::counterlist::ELM_COUNTERLIST_GET_SIZE;
        use crate::elm::list::*;

        match method {
            ELM_INTLISTREF_ARRAY => crate::elm::ELM_ARRAY,
            ELM_INTLISTREF_GET_SIZE => ELM_COUNTERLIST_GET_SIZE,
            v => v,
        }
    }

    fn try_command_excall_frame_action_counter(
        &mut self,
        scope: usize,
        ch_index: Option<i32>,
        chain: &[i32],
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) {
        if chain.is_empty() {
            host.on_error_fatal("無効なコマンドが指定されました。(frame_action.counter)");
            return;
        }
        let idx = ch_index.unwrap_or(0).max(0) as usize;
        let lane = if ch_index.is_some() {
            Self::EXCALL_FRAME_ACTION_CH_COUNTER_SCOPE
        } else {
            Self::EXCALL_FRAME_ACTION_COUNTER_SCOPE
        };
        let slot = Self::excall_counter_slot(scope, lane, idx);
        self.try_command_counter(slot, chain, args, ret_form, host);
    }

    fn try_command_excall_frame_action_scoped(
        &mut self,
        chain: &[i32],
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
        scope: usize,
    ) -> bool {
        if !self.excall_scope_ready(scope) {
            self.excall_report_not_ready(host);
            return true;
        }

        let fa = if scope == 0 {
            &mut self.frame_action
        } else {
            &mut self.excall_frame_action
        };

        if chain.first().copied() == Some(crate::elm::frameaction::ELM_FRAMEACTION_COUNTER) {
            self.try_command_excall_frame_action_counter(
                scope,
                None,
                &chain[1..],
                args,
                ret_form,
                host,
            );
            return true;
        }

        let scene = self.scene.clone();
        match Self::command_proc_frame_action(fa, chain, args, ret_form, &scene, &mut self.stack) {
            Ok(true) => true,
            Ok(false) => {
                host.on_error_fatal("無効なコマンドが指定されました。(frame_action)");
                true
            }
            Err(e) => {
                host.on_error(&format!("frame_action 処理に失敗しました: {e}"));
                true
            }
        }
    }

    fn try_command_excall_frame_action_ch_scoped(
        &mut self,
        chain: &[i32],
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
        scope: usize,
    ) -> bool {
        use crate::elm::frameaction::{
            ELM_FRAMEACTIONLIST_GET_SIZE, ELM_FRAMEACTIONLIST_RESIZE, ELM_FRAMEACTION_COUNTER,
        };

        if !self.excall_scope_ready(scope) {
            self.excall_report_not_ready(host);
            return true;
        }

        let list = if scope == 0 {
            &mut self.frame_action_ch
        } else {
            &mut self.excall_frame_action_ch
        };

        if chain.is_empty() {
            return true;
        }

        if chain[0] == crate::elm::ELM_ARRAY {
            let idx = chain.get(1).copied().unwrap_or(-1);
            if idx < 0 {
                if self.options.disp_out_of_range_error {
                    host.on_error_fatal(
                        "範囲外のフレームアクション番号が指定されました。(frame_action_ch)",
                    );
                }
                Self::push_ret_default(&mut self.stack, ret_form);
                return true;
            }
            let idx = idx as usize;
            if idx >= list.len() {
                if self.options.disp_out_of_range_error {
                    host.on_error_fatal(
                        "範囲外のフレームアクション番号が指定されました。(frame_action_ch)",
                    );
                }
                Self::push_ret_default(&mut self.stack, ret_form);
                return true;
            }
            let sub = &chain[2..];
            if sub.first().copied() == Some(crate::elm::ELM_UP) {
                let up = &sub[1..];
                if up.is_empty() {
                    host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                    Self::push_ret_default(&mut self.stack, ret_form);
                    return true;
                }
                if up[0] == ELM_FRAMEACTION_COUNTER {
                    if up.len() < 2 {
                        host.on_error_fatal(
                            "無効なコマンドが指定されました。(frame_action.counter)",
                        );
                    } else {
                        host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                    }
                    Self::push_ret_default(&mut self.stack, ret_form);
                    return true;
                }
                if up[0] == ELM_FRAMEACTIONLIST_GET_SIZE {
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(list.len() as i32);
                    }
                    return true;
                }
                if up[0] == ELM_FRAMEACTIONLIST_RESIZE {
                    let size = Self::arg_int(args, 0).max(0) as usize;
                    list.resize(size, FrameAction::default());
                    return true;
                }
                host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
                Self::push_ret_default(&mut self.stack, ret_form);
                return true;
            }
            if sub.first().copied() == Some(ELM_FRAMEACTION_COUNTER) {
                self.try_command_excall_frame_action_counter(
                    scope,
                    Some(idx as i32),
                    &sub[1..],
                    args,
                    ret_form,
                    host,
                );
                return true;
            }
            let scene = self.scene.clone();
            match Self::command_proc_frame_action(
                &mut list[idx],
                sub,
                args,
                ret_form,
                &scene,
                &mut self.stack,
            ) {
                Ok(true) => return true,
                Ok(false) => {
                    host.on_error_fatal("無効なコマンドが指定されました。(frame_action)");
                    return true;
                }
                Err(e) => {
                    host.on_error(&format!("frame_action_ch 処理に失敗しました: {e}"));
                    return true;
                }
            }
        }

        if chain[0] == ELM_FRAMEACTIONLIST_GET_SIZE {
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(list.len() as i32);
            }
            return true;
        }

        if chain[0] == ELM_FRAMEACTIONLIST_RESIZE {
            let size = Self::arg_int(args, 0).max(0) as usize;
            list.resize(size, FrameAction::default());
            return true;
        }

        host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
        true
    }
}
