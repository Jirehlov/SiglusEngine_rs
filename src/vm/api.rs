#[derive(Debug, Clone)]
pub enum PropValue {
    Int(i32),
    Str(String),
    /// Argument list (FM_LIST)
    List(Vec<Prop>),
    /// Raw element chain (fallback / refs)
    Element(Vec<i32>),
    /// Call/local intlist storage (best-effort)
    IntList(Vec<i32>),
    /// Call/local strlist storage (best-effort)
    StrList(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct Prop {
    pub id: i32,
    pub form: i32,
    pub value: PropValue,
}

/// Flatten command arguments into selectable string options.
///
/// Siglus selection commands may provide options either as direct string props
/// or nested lists of string props.
pub fn extract_selection_options(args: &[Prop]) -> Vec<String> {
    let mut options = Vec::new();
    for arg in args {
        match &arg.value {
            PropValue::Str(s) => options.push(s.clone()),
            PropValue::List(list) => {
                for item in list {
                    if let PropValue::Str(s) = &item.value {
                        options.push(s.clone());
                    }
                }
            }
            _ => {}
        }
    }
    options
}

#[derive(Debug, Default, Clone)]
pub struct HostReturn {
    pub int: i32,
    pub str_: String,
    pub element: Vec<i32>,
}

pub trait Host {
    fn on_name(&mut self, _name: &str) {}
    fn on_text(&mut self, _text: &str, _read_flag_no: i32) {}

    fn on_command(
        &mut self,
        _element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        _named_arg_cnt: i32,
        _ret_form: i32,
    ) -> HostReturn {
        HostReturn::default()
    }

    /// Called when the VM evaluates a property.
    ///
    /// If you need to return a non-INT value, prefer implementing `on_property_typed`.
    fn on_property(&mut self, _element: &[i32]) -> HostReturn {
        HostReturn::default()
    }

    /// Optional typed property hook. If it returns `Some((ret, form))`, the VM will
    /// push `ret` using `form`.
    fn on_property_typed(&mut self, _element: &[i32]) -> Option<(HostReturn, i32)> {
        None
    }

    fn on_assign(&mut self, _element: &[i32], _al_id: i32, _rhs: &Prop) {}

    /// Optional verbose trace lines from the VM.
    fn on_trace(&mut self, _msg: &str) {}

    fn on_error(&mut self, _msg: &str) {}

    /// Called when VM location/title context changes (for window caption, overlays, etc.).
    fn on_location(&mut self, _scene_title: &str, _scene: &str, _line_no: i32) {}

    /// Notify host that msg_back open-state changed.
    fn on_msg_back_state(&mut self, _open: bool) {}

    /// Notify host that msg_back display availability changed (script disp_off/on).
    fn on_msg_back_display(&mut self, _enabled: bool) {}


    /// Notify host that syscom requested opening the tweet dialog.
    fn on_open_tweet_dialog(&mut self) {}

    /// Called periodically by the VM to check if execution should be aborted.
    fn should_interrupt(&self) -> bool {
        false
    }

    /// C++ break/step emulation flags used by CD_NL flow control.
    fn is_breaking(&self) -> bool {
        false
    }

    fn break_step_flag(&self) -> bool {
        false
    }

    fn on_break_step_line_advanced(&mut self) {}

    /// C++ flow_script.cpp fatal parse/eof path hook.
    ///
    /// Expected host-side side effect: switch process to NONE-equivalent.
    fn on_script_fatal(&mut self, msg: &str) {
        self.on_error(msg);
    }

    /// Called by wait-related commands to allow host-level skip/fast-forward.
    fn should_skip_wait(&self) -> bool {
        false
    }
}
