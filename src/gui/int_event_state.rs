#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntEventLoopType {
    OneShot,
    Loop,
    Turn,
}

#[derive(Debug, Clone)]
struct IntEventYureState {
    swing: i32,
}

#[derive(Debug, Clone)]
struct IntEventState {
    loop_type: IntEventLoopType,
    value: i32,
    cur_time: i32,
    end_time: i32,
    delay_time: i32,
    start_value: i32,
    cur_value: i32,
    end_value: i32,
    speed_type: i32,
    real_flag: bool,
    yure: Option<IntEventYureState>,
    active: bool,
}
