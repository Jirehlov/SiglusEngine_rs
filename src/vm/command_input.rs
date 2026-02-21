// Input / Mouse / Key / Editbox command routing — aligns with C++ cmd_input.cpp
use super::*;

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
            x if x == crate::elm::input::ELM_INPUT_CLEAR => true,
            x if x == crate::elm::input::ELM_INPUT_NEXT => true,
            x if x == crate::elm::input::ELM_INPUT_DECIDE => {
                // Delegate to key(VK_EX_DECIDE)
                if element.len() > 1 {
                    self.try_command_key_sub(&element[1..], ret_form);
                }
                true
            }
            x if x == crate::elm::input::ELM_INPUT_CANCEL => {
                // Delegate to key(VK_EX_CANCEL)
                if element.len() > 1 {
                    self.try_command_key_sub(&element[1..], ret_form);
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
        _args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        match element[0] {
            x if x == crate::elm::mouse::ELM_MOUSE_CLEAR
                || x == crate::elm::mouse::ELM_MOUSE_NEXT =>
            {
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_POS_X
                || x == crate::elm::mouse::ELM_MOUSE_GET_POS_X =>
            {
                self.stack.push_int(0);
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_POS_Y
                || x == crate::elm::mouse::ELM_MOUSE_GET_POS_Y =>
            {
                self.stack.push_int(0);
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_GET_POS
                || x == crate::elm::mouse::ELM_MOUSE_SET_POS =>
            {
                // Headless → no-op
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_WHEEL => {
                self.stack.push_int(0);
                true
            }
            x if x == crate::elm::mouse::ELM_MOUSE_LEFT
                || x == crate::elm::mouse::ELM_MOUSE_RIGHT =>
            {
                if element.len() > 1 {
                    self.try_command_key_sub(&element[1..], ret_form);
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
                // key[key_no].<sub>
                if element.len() > 2 {
                    self.try_command_key_sub(&element[2..], ret_form);
                }
                true
            }
            x if x == crate::elm::list::ELM_KEYLIST_WAIT
                || x == crate::elm::list::ELM_KEYLIST_WAIT_FORCE =>
            {
                // C++ pushes proc; headless → accept
                true
            }
            x if x == crate::elm::list::ELM_KEYLIST_CLEAR
                || x == crate::elm::list::ELM_KEYLIST_NEXT =>
            {
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(keylist)");
                true
            }
        }
    }

    /// Handle individual key queries (ON_DOWN, IS_DOWN, etc.). Headless returns 0.
    fn try_command_key_sub(&mut self, element: &[i32], ret_form: i32) {
        if element.is_empty() {
            return;
        }
        match element[0] {
            x if x == crate::elm::list::ELM_KEY_ON_DOWN
                || x == crate::elm::list::ELM_KEY_ON_UP
                || x == crate::elm::list::ELM_KEY_ON_DOWN_UP
                || x == crate::elm::list::ELM_KEY_IS_DOWN
                || x == crate::elm::list::ELM_KEY_IS_UP
                || x == crate::elm::list::ELM_KEY_ON_FLICK
                || x == crate::elm::list::ELM_KEY_ON_REPEAT =>
            {
                self.stack.push_int(0);
            }
            x if x == crate::elm::list::ELM_KEY_GET_FLICK_ANGLE
                || x == crate::elm::list::ELM_KEY_GET_FLICK_PIXEL
                || x == crate::elm::list::ELM_KEY_GET_FLICK_MM =>
            {
                self.stack.push_int(0);
            }
            _ => {
                // Unknown key sub-command — push 0 defensively
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
                // editbox[idx].<sub>
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
                || x == crate::elm::editbox::ELM_EDITBOX_CLEAR_INPUT =>
            {
                // accept
            }
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
