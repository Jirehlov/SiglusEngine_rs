fn int_event_update_time(state: &mut IntEventState, game_past_time: i32, real_past_time: i32) {
    if state.real_flag {
        state.cur_time = state.cur_time.saturating_add(real_past_time);
    } else {
        state.cur_time = state.cur_time.saturating_add(game_past_time);
    }
}

fn int_event_frame_sub(state: &mut IntEventState) -> bool {
    let end_time = state.end_time;
    if end_time <= 0 {
        return false;
    }

    let start_value = state.start_value;
    let end_value = state.end_value;
    let mut cur_time = state.cur_time - state.delay_time;

    if state.loop_type == IntEventLoopType::OneShot && cur_time - end_time >= 0 {
        return false;
    }

    if cur_time <= 0 {
        state.cur_value = start_value;
        return true;
    }

    if state.loop_type == IntEventLoopType::Loop {
        cur_time %= end_time;
    }

    if state.loop_type == IntEventLoopType::Turn {
        cur_time %= end_time.saturating_mul(2);
        if cur_time - end_time > 0 {
            cur_time = end_time - (cur_time - end_time);
        }
    }

    let delta = end_value - start_value;
    state.cur_value = match state.speed_type {
        1 => {
            ((delta as f64) * (cur_time as f64) * (cur_time as f64)
                / (end_time as f64)
                / (end_time as f64)
                + start_value as f64) as i32
        }
        2 => {
            (-(delta as f64) * ((cur_time - end_time) as f64) * ((cur_time - end_time) as f64)
                / (end_time as f64)
                / (end_time as f64)
                + end_value as f64) as i32
        }
        _ => ((delta as f64) * (cur_time as f64) / (end_time as f64) + start_value as f64) as i32,
    };

    true
}

fn int_event_tick_preview(state: &IntEventState) -> Option<IntEventState> {
    let mut next = state.clone();
    next.cur_value = next.value;
    int_event_update_time(&mut next, 8, 8);
    if !int_event_frame_sub(&mut next) {
        next.active = false;
        next.cur_value = next.value;
        return Some(next);
    }
    Some(next)
}

fn make_int_event_state(
    &self,
    loop_type: IntEventLoopType,
    start_value: i32,
    end_value: i32,
    total_time: i32,
    delay_time: i32,
    speed_type: i32,
    realtime: i32,
) -> Option<IntEventState> {
    if total_time <= 0 {
        return None;
    }
    Some(IntEventState {
        loop_type,
        value: end_value,
        cur_time: 0,
        end_time: total_time,
        delay_time,
        start_value,
        cur_value: start_value,
        end_value,
        speed_type,
        real_flag: realtime > 0,
        yure: None,
        active: true,
    })
}

fn on_int_event_set(
    &mut self,
    owner_id: i32,
    start: i32,
    end: i32,
    time: i32,
    delay: i32,
    realtime: i32,
    value_override: Option<i32>,
) {
    let start_value = value_override.unwrap_or(start);
    if let Some(state) = self.make_int_event_state(
        IntEventLoopType::OneShot,
        start_value,
        end,
        time,
        delay,
        0,
        realtime,
    ) {
        self.int_events.insert(owner_id, state);
    } else {
        self.int_events.remove(&owner_id);
    }
}

fn on_int_event_loop(
    &mut self,
    owner_id: i32,
    start: i32,
    end: i32,
    time: i32,
    delay: i32,
    speed_type: i32,
    realtime: i32,
) {
    if let Some(state) = self.make_int_event_state(
        IntEventLoopType::Loop,
        start,
        end,
        time,
        delay,
        speed_type,
        realtime,
    ) {
        self.int_events.insert(owner_id, state);
    } else {
        self.int_events.remove(&owner_id);
    }
}

fn on_int_event_turn(
    &mut self,
    owner_id: i32,
    start: i32,
    end: i32,
    time: i32,
    delay: i32,
    speed_type: i32,
    realtime: i32,
) {
    if let Some(state) = self.make_int_event_state(
        IntEventLoopType::Turn,
        start,
        end,
        time,
        delay,
        speed_type,
        realtime,
    ) {
        self.int_events.insert(owner_id, state);
    } else {
        self.int_events.remove(&owner_id);
    }
}

fn on_int_event_yure(
    &mut self,
    owner_id: i32,
    center: i32,
    swing: i32,
    time: i32,
    delay: i32,
    speed_type: i32,
    realtime: bool,
) {
    let realtime_flag = if realtime { 1 } else { 0 };
    let amp = swing.abs();
    if let Some(mut state) = self.make_int_event_state(
        IntEventLoopType::Turn,
        center - amp,
        center + amp,
        time,
        delay,
        speed_type,
        realtime_flag,
    ) {
        state.value = center;
        state.yure = Some(IntEventYureState { swing: amp });
        self.int_events.insert(owner_id, state);
    } else {
        self.int_events.remove(&owner_id);
    }
}

fn on_int_event_end(&mut self, owner_id: i32) {
    if let Some(state) = self.int_events.get_mut(&owner_id) {
        state.active = false;
        state.cur_value = state.value;
    }
}

fn on_int_event_wait(&mut self, owner_id: i32, key_skip: bool) {
    loop {
        if !self.on_int_event_check(owner_id) {
            break;
        }
        if key_skip && self.should_skip_wait() {
            self.int_events.remove(&owner_id);
            break;
        }
        if self.shutdown.load(Ordering::Relaxed) {
            break;
        }
        self.on_wait_frame();
    }
}

fn on_int_event_check(&mut self, owner_id: i32) -> bool {
    let Some(state) = self.int_events.get(&owner_id) else {
        return false;
    };
    if !state.active {
        return false;
    }
    let Some(next) = Self::int_event_tick_preview(state) else {
        return false;
    };
    let active = next.active;
    self.int_events.insert(owner_id, next);
    active
}

fn on_int_event_get_value(&mut self, owner_id: i32) -> i32 {
    self.int_events
        .get(&owner_id)
        .map(|state| {
            if let Some(yure) = &state.yure {
                let center = state.value;
                let delta = (state.cur_value - center).clamp(-yure.swing, yure.swing);
                center + delta
            } else {
                state.cur_value
            }
        })
        .unwrap_or(0)
}
