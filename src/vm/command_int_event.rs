/// int_event / int_event_list command routing.
///
/// C++ reference: cmd_others.cpp — `tnm_command_proc_int_event` / `tnm_command_proc_int_event_list`
///
/// Covers:
///   - Per-event: set/set_real/loop/loop_real/turn/turn_real/end/wait/wait_key/check
///   - Event list: indexed access, resize
///
/// All actual animation/interpolation state lives host-side.
/// The VM routes commands, parses args, and delegates via Host callbacks.
use super::*;

impl Vm {
    // ---------------------------------------------------------------
    // int_event: single event property dispatcher
    // ---------------------------------------------------------------

    /// Route int_event sub-commands matching C++ `tnm_command_proc_int_event`.
    ///
    /// `element` starts AFTER the `*_EVE` root, i.e. element[0] is the event
    /// sub-command (SET, LOOP, END, WAIT, CHECK, etc.).
    ///
    /// `owner_id` is the elm id of the parent property (e.g. ELM_OBJECT_X_EVE)
    /// so the host can identify which event is being targeted.
    pub(super) fn try_command_int_event(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
        owner_id: i32,
    ) -> bool {
        if element.is_empty() {
            // bare element reference — accept (C++ sets ret element)
            return true;
        }

        let sub = element[0];
        use crate::elm::intevent::*;

        match sub {
            // --- set / set_real ---
            ELM_INTEVENT_SET | ELM_INTEVENT_SET_REAL => {
                // C++ tnm_command_proc_int_event: SET/SET_REAL
                // positional: start(0), end(1), time(2), delay(3)
                // named-arg id=0 → value override
                // realtime_flag: SET=0, SET_REAL=1
                let start = Self::int_arg(args, 0);
                let end = Self::int_arg(args, 1);
                let time = Self::int_arg(args, 2);
                let delay = Self::int_arg(args, 3);
                let realtime = if sub == ELM_INTEVENT_SET_REAL { 1 } else { 0 };

                // named-arg value override
                let mut value_override: Option<i32> = None;
                for arg in args.iter() {
                    if arg.id == 0 {
                        if let PropValue::Int(v) = arg.value {
                            value_override = Some(v);
                        }
                    }
                }

                host.on_int_event_set(owner_id, start, end, time, delay, realtime, value_override);
                true
            }

            // --- loop / loop_real ---
            ELM_INTEVENT_LOOP | ELM_INTEVENT_LOOP_REAL => {
                // C++ p_int_event->loop_event(start, end, time, delay, count, realtime)
                let start = Self::int_arg(args, 0);
                let end = Self::int_arg(args, 1);
                let time = Self::int_arg(args, 2);
                let delay = Self::int_arg(args, 3);
                let count = Self::int_arg(args, 4);
                let realtime = if sub == ELM_INTEVENT_LOOP_REAL { 1 } else { 0 };
                host.on_int_event_loop(owner_id, start, end, time, delay, count, realtime);
                true
            }

            // --- turn / turn_real ---
            ELM_INTEVENT_TURN | ELM_INTEVENT_TURN_REAL => {
                // C++ p_int_event->turn_event(start, end, time, delay, count, realtime)
                let start = Self::int_arg(args, 0);
                let end = Self::int_arg(args, 1);
                let time = Self::int_arg(args, 2);
                let delay = Self::int_arg(args, 3);
                let count = Self::int_arg(args, 4);
                let realtime = if sub == ELM_INTEVENT_TURN_REAL { 1 } else { 0 };
                host.on_int_event_turn(owner_id, start, end, time, delay, count, realtime);
                true
            }

            // --- end ---
            ELM_INTEVENT_END => {
                // C++ p_int_event->end_event()
                host.on_int_event_end(owner_id);
                true
            }

            // --- wait / wait_key ---
            ELM_INTEVENT_WAIT | ELM_INTEVENT_WAIT_KEY => {
                // C++ pushes a TNM_PROC_TYPE_EVENT_WAIT proc.
                // In headless VM we accept and let host handle the wait semantics.
                let key_skip = sub == ELM_INTEVENT_WAIT_KEY;
                host.on_int_event_wait(owner_id, key_skip);
                true
            }

            // --- check ---
            ELM_INTEVENT_CHECK => {
                // C++ tnm_stack_push_int(p_int_event->check_event() ? 1 : 0)
                // Without actual event state, push 0 (not active).
                self.stack.push_int(0);
                true
            }

            // --- get_event_value ---
            ELM_INTEVENT_GET_EVENT_VALUE => {
                // C++ tnm_stack_push_int(p_int_event->get_event_value())
                self.stack.push_int(0);
                true
            }

            // --- yure / yure_real (shake) ---
            ELM_INTEVENT_YURE | ELM_INTEVENT_YURE_REAL => {
                // accept — host handles shake animation
                true
            }

            // --- __set (internal) ---
            ELM_INTEVENT__SET => {
                // accept — internal use
                true
            }

            _ => {
                host.on_error("無効なコマンドが指定されました。(intevent)");
                true
            }
        }
    }

    // ---------------------------------------------------------------
    // int_event_list: indexed event list dispatcher
    // ---------------------------------------------------------------

    /// Route int_event_list sub-commands matching C++ `tnm_command_proc_int_event_list`.
    ///
    /// `element` starts AFTER the `*_REP_EVE` root.
    pub(super) fn try_command_int_event_list(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
        owner_id: i32,
    ) -> bool {
        if element.is_empty() {
            // bare list reference — accept
            return true;
        }

        if element[0] == crate::elm::ELM_ARRAY {
            // Indexed: event_list[idx].sub
            if element.len() >= 2 {
                let _idx = element[1];
                let rest = if element.len() > 2 { &element[2..] } else { &[] };
                return self.try_command_int_event(rest, arg_list_id, args, ret_form, host, owner_id);
            }
            // bare indexed access — get/set value
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(0);
            }
            return true;
        }

        if element[0] == crate::elm::intevent::ELM_INTEVENTLIST_RESIZE {
            // C++ p_int_event_list->resize(arg0)
            // accept — host manages event list storage
            true
        } else {
            host.on_error("無効なコマンドが指定されました。(inteventlist)");
            true
        }
    }

    // ---------------------------------------------------------------
    // Utility: extract positional int arg
    // ---------------------------------------------------------------

    pub(super) fn int_arg(args: &[Prop], index: usize) -> i32 {
        args.get(index)
            .and_then(|p| match p.value {
                PropValue::Int(v) => Some(v),
                _ => None,
            })
            .unwrap_or(0)
    }
}
