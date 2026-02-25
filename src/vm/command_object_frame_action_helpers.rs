impl Vm {
    fn frame_action_counter_arg_to_int(arg: &Prop) -> Option<i32> {
        match &arg.value {
            PropValue::Int(v) => Some(*v),
            PropValue::Str(s) => s.trim().parse::<i32>().ok(),
            _ => None,
        }
    }

    fn frame_action_counter_int_args(
        method: i32,
        args: &[Prop],
        stack: &mut IfcStack,
        ret_form: i32,
        host: &mut dyn Host,
    ) -> Option<Vec<i32>> {
        let int_argc = match method {
            crate::elm::counter::ELM_COUNTER_SET
            | crate::elm::counter::ELM_COUNTER_WAIT
            | crate::elm::counter::ELM_COUNTER_WAIT_KEY
            | crate::elm::counter::ELM_COUNTER_CHECK_VALUE
            | crate::elm::counter::ELM_COUNTER_CHECK_ACTIVE => 1,
            crate::elm::counter::ELM_COUNTER_START_FRAME
            | crate::elm::counter::ELM_COUNTER_START_FRAME_REAL
            | crate::elm::counter::ELM_COUNTER_START_FRAME_LOOP
            | crate::elm::counter::ELM_COUNTER_START_FRAME_LOOP_REAL => 3,
            _ => 0,
        };
        if int_argc == 0 {
            return Some(Vec::new());
        }
        let mut parsed = Vec::with_capacity(int_argc);
        for idx in 0..int_argc {
            let Some(v) = args
                .get(idx)
                .and_then(Self::frame_action_counter_arg_to_int)
            else {
                host.on_error_fatal("無効なコマンドが指定されました。(frame_action.counter)");
                Self::object_frame_action_push_default(stack, ret_form);
                return None;
            };
            parsed.push(v);
        }
        Some(parsed)
    }

    fn object_frame_action_ch_size(
        host: &mut dyn Host,
        list_id: i32,
        obj_idx: i32,
        sub: i32,
        stage_idx: Option<i32>,
    ) -> Option<i32> {
        let tail = [crate::elm::frameaction::ELM_FRAMEACTIONLIST_GET_SIZE];
        let (v, form) =
            host.on_object_frame_action_property(list_id, obj_idx, sub, &tail, stage_idx)?;
        if form != crate::elm::form::INT {
            return None;
        }
        match v {
            PropValue::Int(n) => Some(n.max(0)),
            _ => None,
        }
    }

    fn excall_counter_trace_enabled() -> bool {
        std::env::var("SIGLUS_EXCALL_COUNTER_TRACE")
            .map(|v| v != "0")
            .unwrap_or(false)
    }

    fn trace_excall_counter_tick(
        slot: usize,
        list_id: i32,
        obj_idx: i32,
        sub: i32,
        stage_idx: Option<i32>,
        ch_idx: Option<i32>,
        before: i32,
        after: i32,
        active: bool,
    ) {
        if slot < Self::EXCALL_COUNTER_BASE || !Self::excall_counter_trace_enabled() {
            return;
        }
        let line = crate::vm::format_excall_counter_trace(
            slot,
            crate::vm::VmExcallCounterPhase::Tick,
            after,
            active,
        );
        log::debug!(
            "{} list={} obj={} sub={} stage={:?} ch={:?} before={} after={}",
            line,
            list_id,
            obj_idx,
            sub,
            stage_idx,
            ch_idx,
            before,
            after,
        );
    }

    fn frame_action_counter_method_and_channel(
        sub: i32,
        tail: &[i32],
    ) -> Option<(i32, Option<i32>)> {
        if tail.len() < 2 || tail[0] != crate::elm::frameaction::ELM_FRAMEACTION_COUNTER {
            return None;
        }
        if sub == crate::elm::objectlist::ELM_OBJECT_FRAME_ACTION_CH
            && tail.first().copied() == Some(crate::elm::ELM_ARRAY)
        {
            return None;
        }
        Some((tail[1], None))
    }

    fn frame_action_ch_counter_tail_kind(tail: &[i32]) -> Option<bool> {
        // true: up.counter.<method>, false: counter.<method>
        if tail.len() < 3 || tail[0] != crate::elm::ELM_ARRAY {
            return None;
        }
        if tail[2] == crate::elm::ELM_UP {
            if tail.len() >= 4 && tail[3] == crate::elm::frameaction::ELM_FRAMEACTION_COUNTER {
                return Some(true);
            }
            return None;
        }
        if tail[2] == crate::elm::frameaction::ELM_FRAMEACTION_COUNTER {
            return Some(false);
        }
        None
    }

    fn frame_action_ch_counter_tail_has_extra(tail: &[i32]) -> bool {
        match Self::frame_action_ch_counter_tail_kind(tail) {
            Some(true) => tail.len() > 5,
            Some(false) => tail.len() > 4,
            None => false,
        }
    }

    fn frame_action_ch_counter_tail_guard(
        tail: &[i32],
        stack: &mut IfcStack,
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if Self::frame_action_ch_counter_tail_has_extra(tail) {
            host.on_error_fatal("無効なコマンドが指定されました。(frame_action_ch)");
            Self::object_frame_action_push_default(stack, ret_form);
            return true;
        }
        false
    }

    fn frame_action_counter_expected_argc(method: i32) -> Option<usize> {
        use crate::elm::counter::*;
        match method {
            ELM_COUNTER_SET => Some(1),
            ELM_COUNTER_GET => Some(0),
            ELM_COUNTER_RESET => Some(0),
            ELM_COUNTER_START | ELM_COUNTER_START_REAL | ELM_COUNTER_RESUME => Some(0),
            ELM_COUNTER_START_FRAME
            | ELM_COUNTER_START_FRAME_REAL
            | ELM_COUNTER_START_FRAME_LOOP
            | ELM_COUNTER_START_FRAME_LOOP_REAL => Some(3),
            ELM_COUNTER_STOP => Some(0),
            ELM_COUNTER_WAIT | ELM_COUNTER_WAIT_KEY => Some(1),
            ELM_COUNTER_CHECK_VALUE => Some(1),
            ELM_COUNTER_CHECK_ACTIVE => Some(1),
            _ => None,
        }
    }

    fn frame_action_ch_up_counter_arg_guard(
        sub: i32,
        tail: &[i32],
        method: i32,
        args: &[Prop],
        stack: &mut IfcStack,
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if sub != crate::elm::objectlist::ELM_OBJECT_FRAME_ACTION_CH {
            return false;
        }
        if !matches!(Self::frame_action_ch_counter_tail_kind(tail), Some(true)) {
            return false;
        }
        Self::frame_action_counter_arg_guard(method, args, stack, ret_form, host)
    }

    fn frame_action_counter_arg_guard(
        method: i32,
        args: &[Prop],
        stack: &mut IfcStack,
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        let Some(expect) = Self::frame_action_counter_expected_argc(method) else {
            return false;
        };
        if args.len() == expect {
            return false;
        }
        host.on_error_fatal("無効なコマンドが指定されました。(frame_action.counter)");
        Self::object_frame_action_push_default(stack, ret_form);
        true
    }

    fn frame_action_ch_counter_method_and_channel(tail: &[i32]) -> Option<(i32, Option<i32>)> {
        if tail.len() < 4 || tail[0] != crate::elm::ELM_ARRAY {
            return None;
        }
        let ch_idx = tail[1];
        if tail[2] == crate::elm::ELM_UP {
            if tail.len() < 5 || tail[3] != crate::elm::frameaction::ELM_FRAMEACTION_COUNTER {
                return None;
            }
            return Some((tail[4], Some(ch_idx)));
        }
        if tail[2] == crate::elm::frameaction::ELM_FRAMEACTION_COUNTER {
            return Some((tail[3], Some(ch_idx)));
        }
        None
    }
}
