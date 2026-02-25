use super::*;

impl Vm {
    fn maybe_emit_excall_counter_aggregate_hint(&self, host: &mut dyn Host) {
        if let Some(line) = crate::vm::take_excall_counter_aggregate_hint("session") {
            host.on_trace(&line);
        }
    }

    pub(super) fn arg_str_others(args: &[Prop], idx: usize) -> String {
        match args.get(idx).map(|p| &p.value) {
            Some(PropValue::Str(v)) => v.clone(),
            Some(PropValue::Int(v)) => v.to_string(),
            _ => String::new(),
        }
    }

    fn counter_expected_argc(method: i32) -> Option<usize> {
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
            // C++ cmd_others.cpp reads al_begin[0].Int even for check_active.
            ELM_COUNTER_CHECK_ACTIVE => Some(1),
            _ => None,
        }
    }

    fn counter_arg_to_int(arg: &Prop) -> Option<i32> {
        match &arg.value {
            PropValue::Int(v) => Some(*v),
            PropValue::Str(s) => s.trim().parse::<i32>().ok(),
            _ => None,
        }
    }

    fn counter_collect_int_args(
        method: i32,
        args: &[Prop],
        host: &mut dyn Host,
    ) -> Option<Vec<i32>> {
        let Some(expected) = Self::counter_expected_argc(method) else {
            return None;
        };
        if args.len() != expected {
            host.on_error("無効なコマンドが指定されました。(counter)");
            return None;
        }

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

        let mut parsed = Vec::with_capacity(int_argc);
        for idx in 0..int_argc {
            let Some(v) = args.get(idx).and_then(Self::counter_arg_to_int) else {
                host.on_error("無効なコマンドが指定されました。(counter)");
                return None;
            };
            parsed.push(v);
        }
        Some(parsed)
    }

    fn counter_wait_observation_owner(&self) -> (i32, &'static str, i32, i32) {
        let (proc_depth, proc_top) = self.observe_proc_stack_tuple();
        let owner_id = if proc_depth > 1 {
            crate::vm::SYSCOM_WAIT_OWNER_PROC_BASE
        } else {
            0
        };
        let phase = crate::vm::classify_syscom_wait_owner(owner_id).as_str();
        (owner_id, phase, proc_depth, proc_top)
    }

    fn counter_emit_wait_observation(
        &self,
        counter_idx: usize,
        method: i32,
        option: i32,
        host: &mut dyn Host,
    ) {
        let kind = if method == crate::elm::counter::ELM_COUNTER_WAIT {
            "wait"
        } else {
            "wait_key"
        };
        let (owner_id, phase, proc_depth, proc_top) = self.counter_wait_observation_owner();
        host.on_trace(&format!(
            "vm: counter_observe {} owner={} phase={} depth={} top={} idx={} option={} value={} active={}",
            kind,
            owner_id,
            phase,
            proc_depth,
            proc_top,
            counter_idx,
            option,
            self.counter_values.get(counter_idx).copied().unwrap_or(0),
            self.counter_active
                .get(counter_idx)
                .copied()
                .unwrap_or(false) as i32,
        ));
    }
}
