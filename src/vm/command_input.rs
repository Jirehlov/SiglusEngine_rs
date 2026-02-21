// Input / Mouse / Key / Editbox command routing — aligns with C++ cmd_input.cpp
use super::*;

const KEY_MAX: i32 = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeyWaitTickResult {
    IdleOrCompleted,
    Pending,
}

impl Vm {
    /// Route `global.input.<sub>` commands. Returns `true` if handled.
    pub(super) fn try_command_input(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        match element[0] {
            x if x == crate::elm::input::ELM_INPUT_CLEAR => {
                host.on_input_clear();
                true
            }
            x if x == crate::elm::input::ELM_INPUT_NEXT => {
                host.on_input_next();
                true
            }
            x if x == crate::elm::input::ELM_INPUT_DECIDE => {
                if element.len() > 1 {
                    let state = host.on_input_get_decide_state();
                    self.try_command_key_sub_with_state(&element[1..], ret_form, state, host);
                }
                true
            }
            x if x == crate::elm::input::ELM_INPUT_CANCEL => {
                if element.len() > 1 {
                    let state = host.on_input_get_cancel_state();
                    self.try_command_key_sub_with_state(&element[1..], ret_form, state, host);
                }
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(input)");
                true
            }
        }
    }

    /// Route `global.mouse.<sub>` commands. Returns `true` if handled.
    pub(super) fn try_command_mouse(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        match element[0] {
            x if x == crate::elm::mouse::ELM_MOUSE_CLEAR => {
                host.on_input_mouse_clear();
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_NEXT => {
                host.on_input_mouse_next();
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_POS_X
                || x == crate::elm::mouse::ELM_MOUSE_GET_POS_X =>
            {
                self.stack.push_int(host.on_input_get_mouse_state().pos_x);
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_POS_Y
                || x == crate::elm::mouse::ELM_MOUSE_GET_POS_Y =>
            {
                self.stack.push_int(host.on_input_get_mouse_state().pos_y);
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_GET_POS => {
                let mouse = host.on_input_get_mouse_state();
                self.assign_command_arg_int(args.first(), mouse.pos_x, host);
                self.assign_command_arg_int(args.get(1), mouse.pos_y, host);
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_SET_POS => {
                let x = args
                    .first()
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                let y = args
                    .get(1)
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                host.on_input_set_mouse_pos(x, y);
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_WHEEL => {
                self.stack
                    .push_int(host.on_input_get_mouse_state().wheel_delta);
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_LEFT => {
                if element.len() > 1 {
                    let state = host.on_input_get_mouse_state().left;
                    self.try_command_key_sub_with_state(&element[1..], ret_form, state, host);
                }
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_RIGHT => {
                if element.len() > 1 {
                    let state = host.on_input_get_mouse_state().right;
                    self.try_command_key_sub_with_state(&element[1..], ret_form, state, host);
                }
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(mouse)");
                true
            }
        }
    }

    /// Route `global.key.<sub>` (key_list level). Returns `true` if handled.
    pub(super) fn try_command_key_list(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        match element[0] {
            x if x == crate::elm::ELM_ARRAY => {
                if element.len() > 2 {
                    let key_no = element[1];
                    if !(0..KEY_MAX).contains(&key_no) {
                        host.on_error(&format!("key[{}] にアクセスしました。", key_no));
                    } else {
                        let state = host.on_input_get_key_state(key_no);
                        self.try_command_key_sub_with_state(&element[2..], ret_form, state, host);
                    }
                }
                true
            }
            x if x == crate::elm::list::ELM_KEYLIST_WAIT => {
                self.enqueue_key_wait_proc(false, host);
                true
            }
            x if x == crate::elm::list::ELM_KEYLIST_WAIT_FORCE => {
                self.enqueue_key_wait_proc(true, host);
                true
            }
            x if x == crate::elm::list::ELM_KEYLIST_CLEAR => {
                host.on_input_keylist_clear();
                true
            }
            x if x == crate::elm::list::ELM_KEYLIST_NEXT => {
                host.on_input_keylist_next();
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(keylist)");
                true
            }
        }
    }

    fn enqueue_key_wait_proc(&mut self, force_skip_disable: bool, host: &mut dyn Host) {
        // C++ cmd_input.cpp: KEYLIST_WAIT/WAIT_FORCE enqueue TNM_PROC_TYPE_KEY_WAIT.
        self.key_wait_proc.active = true;
        self.key_wait_proc.force_skip_disable = force_skip_disable;
        host.on_input_key_wait(force_skip_disable);
    }

    pub(super) fn run_key_wait_proc(&mut self, host: &mut dyn Host) -> KeyWaitTickResult {
        if !self.key_wait_proc.active {
            return KeyWaitTickResult::IdleOrCompleted;
        }
        if host.should_interrupt() {
            self.key_wait_proc.active = false;
            return KeyWaitTickResult::IdleOrCompleted;
        }
        if !self.key_wait_proc.force_skip_disable && host.should_skip_wait() {
            self.key_wait_proc.active = false;
            return KeyWaitTickResult::IdleOrCompleted;
        }
        if host.on_input_key_wait_has_press_stock() {
            host.on_input_key_wait_consume_frame();
            self.key_wait_proc.active = false;
            return KeyWaitTickResult::IdleOrCompleted;
        }
        KeyWaitTickResult::Pending
    }

    fn assign_command_arg_int(&mut self, arg: Option<&Prop>, value: i32, host: &mut dyn Host) {
        let Some(arg) = arg else {
            return;
        };
        let PropValue::Element(raw_element) = &arg.value else {
            return;
        };

        let element = self.resolve_command_element_alias(raw_element);
        let rhs = Prop {
            id: 0,
            form: crate::elm::form::INT,
            value: PropValue::Int(value),
        };
        match self.try_assign_internal(&element, 1, &rhs) {
            Ok(true) => {}
            Ok(false) | Err(_) => host.on_assign(&element, 1, &rhs),
        }
    }

    fn try_command_key_sub_with_state(
        &mut self,
        element: &[i32],
        ret_form: i32,
        state: VmInputButtonState,
        host: &mut dyn Host,
    ) {
        if element.is_empty() {
            return;
        }
        match element[0] {
            x if x == crate::elm::list::ELM_KEY_ON_DOWN => {
                self.stack.push_int(state.on_down as i32)
            }
            x if x == crate::elm::list::ELM_KEY_ON_UP => self.stack.push_int(state.on_up as i32),
            x if x == crate::elm::list::ELM_KEY_ON_DOWN_UP => {
                self.stack.push_int(state.on_down_up as i32)
            }
            x if x == crate::elm::list::ELM_KEY_IS_DOWN => {
                self.stack.push_int(state.is_down as i32)
            }
            x if x == crate::elm::list::ELM_KEY_IS_UP => self.stack.push_int(state.is_up as i32),
            x if x == crate::elm::list::ELM_KEY_ON_FLICK => {
                self.stack.push_int(state.on_flick as i32)
            }
            x if x == crate::elm::list::ELM_KEY_ON_REPEAT => {
                self.stack.push_int(state.on_repeat as i32)
            }
            x if x == crate::elm::list::ELM_KEY_GET_FLICK_ANGLE => {
                self.stack.push_int(state.flick_angle)
            }
            x if x == crate::elm::list::ELM_KEY_GET_FLICK_PIXEL => {
                self.stack.push_int(state.flick_pixel)
            }
            x if x == crate::elm::list::ELM_KEY_GET_FLICK_MM => self.stack.push_int(state.flick_mm),
            _ => {
                host.on_error("無効なコマンドが指定されました。(key)");
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
            }
        }
    }

    /// Route `global.editbox.<sub>` commands. Returns `true` if handled.
    pub(super) fn try_command_editbox_list(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        match element[0] {
            x if x == crate::elm::ELM_ARRAY => {
                if element.len() > 2 {
                    self.try_command_editbox(&element[2..], ret_form);
                }
                true
            }
            x if x == crate::elm::editbox::ELM_EDITBOXLIST_CLEAR_INPUT => true,
            _ => {
                host.on_error("無効なコマンドが指定されました。(editboxlist)");
                true
            }
        }
    }

    fn try_command_editbox(&mut self, element: &[i32], ret_form: i32) {
        if element.is_empty() {
            return;
        }
        match element[0] {
            x if x == crate::elm::editbox::ELM_EDITBOX_CREATE
                || x == crate::elm::editbox::ELM_EDITBOX_DESTROY
                || x == crate::elm::editbox::ELM_EDITBOX_SET_TEXT
                || x == crate::elm::editbox::ELM_EDITBOX_SET_FOCUS
                || x == crate::elm::editbox::ELM_EDITBOX_CLEAR_INPUT => {}
            x if x == crate::elm::editbox::ELM_EDITBOX_GET_TEXT => {
                self.stack.push_str(String::new());
            }
            x if x == crate::elm::editbox::ELM_EDITBOX_CHECK_DECIDED
                || x == crate::elm::editbox::ELM_EDITBOX_CHECK_CANCELED =>
            {
                self.stack.push_int(0);
            }
            _ => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                } else if ret_form == crate::elm::form::STR {
                    self.stack.push_str(String::new());
                }
            }
        }
    }
}
