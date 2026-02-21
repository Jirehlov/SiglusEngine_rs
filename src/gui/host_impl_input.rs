fn on_input_clear(&mut self) {
    if let Ok(mut st) = self.input_state.lock() {
        st.clear_all();
    }
}
fn on_input_next(&mut self) {
    if let Ok(mut st) = self.input_state.lock() {
        let _ = st.consume_key_wait_press_stock();
    }
}
fn on_input_mouse_clear(&mut self) {
    if let Ok(mut st) = self.input_state.lock() {
        st.clear_mouse();
    }
}
fn on_input_mouse_next(&mut self) {
    if let Ok(mut st) = self.input_state.lock() {
        st.next_mouse();
    }
}
fn on_input_keylist_clear(&mut self) {
    if let Ok(mut st) = self.input_state.lock() {
        st.clear_keyboard();
    }
}
fn on_input_keylist_next(&mut self) {
    if let Ok(mut st) = self.input_state.lock() {
        st.next_keyboard();
    }
}
fn on_input_key_wait(&mut self, force_skip_disable: bool) {
    // Proc enqueue hook (C++ TNM_PROC_TYPE_KEY_WAIT).
    // Host keeps this as an observation point; polling/consume happens in VM proc ticks.
    let _ = force_skip_disable;
}
fn on_input_key_wait_has_press_stock(&mut self) -> bool {
    if self.shutdown.load(Ordering::Relaxed) {
        return false;
    }
    match self.input_state.lock() {
        Ok(st) => st.has_key_wait_press_stock(),
        Err(_) => false,
    }
}
fn on_input_key_wait_consume_frame(&mut self) {
    if let Ok(mut st) = self.input_state.lock() {
        let _ = st.consume_key_wait_press_stock();
    }
}
fn on_input_set_mouse_pos(&mut self, x: i32, y: i32) {
    if let Ok(mut st) = self.input_state.lock() {
        st.mouse_x = x;
        st.mouse_y = y;
    }
    let _ = self.event_tx.send(HostEvent::SetCursorPos { x, y });
}
fn on_input_get_mouse_state(&mut self) -> siglus::vm::VmInputMouseState {
    match self.input_state.lock() {
        Ok(mut st) => siglus::vm::VmInputMouseState {
            pos_x: st.mouse_x,
            pos_y: st.mouse_y,
            wheel_delta: st.wheel_delta,
            left: st.mouse_left.snapshot_and_consume(),
            right: st.mouse_right.snapshot_and_consume(),
        },
        Err(_) => siglus::vm::VmInputMouseState::default(),
    }
}
fn on_input_get_key_state(&mut self, key_no: i32) -> siglus::vm::VmInputButtonState {
    if !(0..256).contains(&key_no) {
        return siglus::vm::VmInputButtonState::default();
    }
    match self.input_state.lock() {
        Ok(mut st) => st.keyboard[key_no as usize].snapshot_and_consume(),
        Err(_) => siglus::vm::VmInputButtonState::default(),
    }
}
fn on_input_get_decide_state(&mut self) -> siglus::vm::VmInputButtonState {
    match self.input_state.lock() {
        Ok(mut st) => st.decide.snapshot_and_consume(),
        Err(_) => siglus::vm::VmInputButtonState::default(),
    }
}
fn on_input_get_cancel_state(&mut self) -> siglus::vm::VmInputButtonState {
    match self.input_state.lock() {
        Ok(mut st) => st.cancel.snapshot_and_consume(),
        Err(_) => siglus::vm::VmInputButtonState::default(),
    }
}
fn on_input_get_left_flick_state(&mut self) -> siglus::vm::VmFlickState {
    match self.input_state.lock() {
        Ok(st) => st.left_flick_state(),
        Err(_) => siglus::vm::VmFlickState::default(),
    }
}
fn on_input_consume_left_flick_stock(&mut self) -> bool {
    match self.input_state.lock() {
        Ok(mut st) => st.consume_left_flick_stock(),
        Err(_) => false,
    }
}
