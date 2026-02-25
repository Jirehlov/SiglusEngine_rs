// int_event Host callbacks (cmd_others.cpp alignment)

/// C++ cmd_others.cpp: int_event SET/SET_REAL.
fn on_int_event_set(
    &mut self,
    _owner_id: i32,
    _start: i32,
    _end: i32,
    _time: i32,
    _delay: i32,
    _realtime: i32,
    _value_override: Option<i32>,
) {
}

/// C++ cmd_others.cpp: int_event LOOP/LOOP_REAL.
fn on_int_event_loop(
    &mut self,
    _owner_id: i32,
    _start: i32,
    _end: i32,
    _time: i32,
    _delay: i32,
    _speed_type: i32,
    _realtime: i32,
) {
}

/// C++ cmd_others.cpp: int_event TURN/TURN_REAL.
fn on_int_event_turn(
    &mut self,
    _owner_id: i32,
    _start: i32,
    _end: i32,
    _time: i32,
    _delay: i32,
    _speed_type: i32,
    _realtime: i32,
) {
}

/// C++ cmd_others.cpp: int_event END.
fn on_int_event_end(&mut self, _owner_id: i32) {}

/// C++ cmd_others.cpp: int_event WAIT/WAIT_KEY.
fn on_int_event_wait(&mut self, _owner_id: i32, _key_skip: bool) {}

/// VM unified wait status callback for proc-level consumers.
///
/// Status is one of `crate::vm::EVE_WAIT_*` constants.
fn on_int_event_wait_status(&mut self, _owner_id: i32, _key_skip: bool, _status: i32) {}

/// VM wait status callback including proc-stack observation tuple.
///
/// Host usage hint: syscom flow points use shared owner-id constants:
/// `SYSCOM_WAIT_OWNER_PROC_*` (phase-specific) and
/// `SYSCOM_WAIT_OWNER_END_LOAD_{PRE_QUEUE,POST_QUEUE}`.
///
/// Recommended default consumer template (flow_proc.cpp alignment):
/// - phase bucket:
///   - `return_to_menu` => `SYSCOM_WAIT_OWNER_PROC_RETURN_TO_MENU`
///   - `return_to_sel`  => `SYSCOM_WAIT_OWNER_PROC_RETURN_TO_SEL`
///   - `end_game`       => `SYSCOM_WAIT_OWNER_PROC_END_GAME`
///   - `end_load_pre`   => `SYSCOM_WAIT_OWNER_END_LOAD_PRE_QUEUE`
///   - `end_load_post`  => `SYSCOM_WAIT_OWNER_END_LOAD_POST_QUEUE`
/// - fallback: unknown proc owner ids can be grouped as `proc_other`.
///
/// Recommended stats fields:
/// - `owner_id`, `proc_depth`, `proc_top`, `status`, `key_skip`.
///
/// Minimal log format example:
/// `vm.wait owner={owner_id} phase={phase} status={status} key_skip={key_skip} depth={proc_depth} top={proc_top}`
///
/// - `proc_depth`: current VM proc stack depth.
/// - `proc_top`: top proc type code (0=None,1=Script; aligned with runtime save encoding).
fn on_int_event_wait_status_with_proc(
    &mut self,
    _owner_id: i32,
    _key_skip: bool,
    _status: i32,
    _proc_depth: i32,
    _proc_top: i32,
) {
}
/// C++ cmd_others.cpp: int_event CHECK.
fn on_int_event_check(&mut self, _owner_id: i32) -> bool {
    false
}

/// C++ cmd_others.cpp: int_event GET_EVENT_VALUE.
fn on_int_event_get_value(&mut self, _owner_id: i32) -> i32 {
    0
}

/// C++ cmd_others.cpp: int_event YURE / YURE_REAL.
fn on_int_event_yure(
    &mut self,
    _owner_id: i32,
    _center: i32,
    _swing: i32,
    _time: i32,
    _delay: i32,
    _speed_type: i32,
    _realtime: bool,
) {
}
