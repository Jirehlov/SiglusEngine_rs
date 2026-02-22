/// Effect / Screen / Quake command routing.
///
/// C++ reference: cmd_effect.cpp
///
/// Covers:
///   - `global.screen.*` — screen-level properties and sub-dispatchers
///   - `global.screen.effect[idx].*` — per-effect properties
///   - `global.screen.quake[idx].*` — quake start/end/wait/check
///
/// Approach: Property get commands push default 0 values.
/// Property set commands delegate to Host callbacks.
/// Quake commands are accepted as no-ops (no animation backend yet).
use super::*;

impl Vm {
    // ---------------------------------------------------------------
    // Top-level: global.screen
    // ---------------------------------------------------------------

    /// Route `global.screen.<sub>` commands matching C++ `tnm_command_proc_screen`.
    ///
    /// `element` starts AFTER the `ELM_GLOBAL_SCREEN` root, i.e. element[0] is
    /// the first screen sub-element.
    pub(super) fn try_command_screen(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            // Bare `screen` element → accept (C++ sets ret element).
            return true;
        }
        let sub = element[0];
        use crate::elm::screen::*;

        match sub {
            // --- Sub-dispatchers ---
            ELM_SCREEN_EFFECT => {
                self.try_command_effect_list(&element[1..], arg_list_id, args, ret_form, host)
            }
            ELM_SCREEN_QUAKE => {
                self.try_command_quake_list(&element[1..], arg_list_id, args, ret_form, host)
            }
            ELM_SCREEN_SHAKE => {
                // C++ p_screen->shake().start(arg0, true)
                // Accept as no-op.
                true
            }

            // --- Scalar properties on effect_list[0] ---
            ELM_SCREEN_X
            | ELM_SCREEN_Y
            | ELM_SCREEN_Z
            | ELM_SCREEN_MONO
            | ELM_SCREEN_REVERSE
            | ELM_SCREEN_BRIGHT
            | ELM_SCREEN_DARK
            | ELM_SCREEN_COLOR_R
            | ELM_SCREEN_COLOR_G
            | ELM_SCREEN_COLOR_B
            | ELM_SCREEN_COLOR_RATE
            | ELM_SCREEN_COLOR_ADD_R
            | ELM_SCREEN_COLOR_ADD_G
            | ELM_SCREEN_COLOR_ADD_B => {
                // C++ al_id==0 → push, al_id==1 → set
                if arg_list_id == 0 {
                    self.stack.push_int(0);
                } else {
                    host.on_screen_property(
                        sub,
                        args.first()
                            .and_then(|p| match p.value {
                                PropValue::Int(v) => Some(v),
                                _ => None,
                            })
                            .unwrap_or(0),
                    );
                }
                true
            }

            // --- Event properties on effect_list[0] ---
            ELM_SCREEN_X_EVE
            | ELM_SCREEN_Y_EVE
            | ELM_SCREEN_Z_EVE
            | ELM_SCREEN_MONO_EVE
            | ELM_SCREEN_REVERSE_EVE
            | ELM_SCREEN_BRIGHT_EVE
            | ELM_SCREEN_DARK_EVE
            | ELM_SCREEN_COLOR_R_EVE
            | ELM_SCREEN_COLOR_G_EVE
            | ELM_SCREEN_COLOR_B_EVE
            | ELM_SCREEN_COLOR_RATE_EVE
            | ELM_SCREEN_COLOR_ADD_R_EVE
            | ELM_SCREEN_COLOR_ADD_G_EVE
            | ELM_SCREEN_COLOR_ADD_B_EVE => {
                // C++ dispatches to tnm_command_proc_int_event.
                self.try_command_int_event(&element[1..], arg_list_id, args, ret_form, host, sub)
            }

            _ => {
                host.on_error("無効なコマンドが指定されました。(screen)");
                true
            }
        }
    }

    // ---------------------------------------------------------------
    // Effect list: global.screen.effect[idx]
    // ---------------------------------------------------------------

    /// Route `screen.effect.<sub>` commands matching C++ `tnm_command_proc_effect_list`.
    pub(super) fn try_command_effect_list(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            // Bare effect_list element → accept.
            return true;
        }
        use crate::elm::effectlist::*;

        if element[0] == crate::elm::ELM_ARRAY {
            // Indexed access: effect[idx].sub
            if element.len() >= 2 {
                let idx = element[1];
                let size = host.on_effect_list_get_size();
                if size >= 0 && (idx < 0 || idx >= size) {
                    if self.options.disp_out_of_range_error {
                        host.on_error("範囲外のエフェクト番号が指定されました。(effect_list)");
                    }
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(0);
                    } else if ret_form == crate::elm::form::STR {
                        self.stack.push_str(String::new());
                    }
                    return true;
                }
                let rest = if element.len() > 2 {
                    &element[2..]
                } else {
                    &[]
                };
                host.on_effect_list_resize((idx + 1).max(0));
                return self.try_command_effect(rest, arg_list_id, args, ret_form, host);
            }
            return true;
        }

        match element[0] {
            ELM_EFFECTLIST_RESIZE => {
                // C++ p_effect_list->resize(arg0)
                let n = args
                    .first()
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0)
                    .max(0);
                host.on_effect_list_resize(n);
                true
            }
            ELM_EFFECTLIST_GET_SIZE => {
                // C++ tnm_stack_push_int(p_effect_list->get_size())
                self.stack.push_int(host.on_effect_list_get_size().max(0));
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(effectlist)");
                true
            }
        }
    }

    // ---------------------------------------------------------------
    // Individual effect: global.screen.effect[idx].<prop>
    // ---------------------------------------------------------------

    /// Route per-effect commands matching C++ `tnm_command_proc_effect`.
    fn try_command_effect(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        let sub = element[0];
        use crate::elm::effect::*;

        match sub {
            ELM_EFFECT_INIT => {
                // C++ p_effect->reinit()
                host.on_effect_init();
                true
            }

            // --- Scalar properties ---
            ELM_EFFECT_X
            | ELM_EFFECT_Y
            | ELM_EFFECT_Z
            | ELM_EFFECT_MONO
            | ELM_EFFECT_REVERSE
            | ELM_EFFECT_BRIGHT
            | ELM_EFFECT_DARK
            | ELM_EFFECT_COLOR_R
            | ELM_EFFECT_COLOR_G
            | ELM_EFFECT_COLOR_B
            | ELM_EFFECT_COLOR_RATE
            | ELM_EFFECT_COLOR_ADD_R
            | ELM_EFFECT_COLOR_ADD_G
            | ELM_EFFECT_COLOR_ADD_B
            | ELM_EFFECT_WIPE_COPY
            | ELM_EFFECT_WIPE_ERASE
            | ELM_EFFECT_BEGIN_ORDER
            | ELM_EFFECT_END_ORDER
            | ELM_EFFECT_BEGIN_LAYER
            | ELM_EFFECT_END_LAYER => {
                if arg_list_id == 0 {
                    self.stack.push_int(0);
                } else {
                    host.on_effect_property(
                        sub,
                        args.first()
                            .and_then(|p| match p.value {
                                PropValue::Int(v) => Some(v),
                                _ => None,
                            })
                            .unwrap_or(0),
                    );
                }
                true
            }

            // --- Event properties ---
            ELM_EFFECT_X_EVE
            | ELM_EFFECT_Y_EVE
            | ELM_EFFECT_Z_EVE
            | ELM_EFFECT_MONO_EVE
            | ELM_EFFECT_REVERSE_EVE
            | ELM_EFFECT_BRIGHT_EVE
            | ELM_EFFECT_DARK_EVE
            | ELM_EFFECT_COLOR_R_EVE
            | ELM_EFFECT_COLOR_G_EVE
            | ELM_EFFECT_COLOR_B_EVE
            | ELM_EFFECT_COLOR_RATE_EVE
            | ELM_EFFECT_COLOR_ADD_R_EVE
            | ELM_EFFECT_COLOR_ADD_G_EVE
            | ELM_EFFECT_COLOR_ADD_B_EVE => {
                // C++ dispatches to tnm_command_proc_int_event.
                self.try_command_int_event(&element[1..], arg_list_id, args, _ret_form, host, sub)
            }

            _ => {
                host.on_error("無効なコマンドが指定されました。(effect)");
                true
            }
        }
    }

    // ---------------------------------------------------------------
    // Quake list: global.screen.quake[idx]
    // ---------------------------------------------------------------

    /// Route `screen.quake.<sub>` commands matching C++ `tnm_command_proc_quake_list`.
    pub(super) fn try_command_quake_list(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }

        if element[0] == crate::elm::ELM_ARRAY {
            if element.len() >= 2 {
                let idx = element[1];
                let size = host.on_quake_list_get_size();
                if size >= 0 && (idx < 0 || idx >= size) {
                    if self.options.disp_out_of_range_error {
                        host.on_error("範囲外のクェイク番号が指定されました。(quake_list)");
                    }
                    if ret_form == crate::elm::form::INT {
                        self.stack.push_int(0);
                    } else if ret_form == crate::elm::form::STR {
                        self.stack.push_str(String::new());
                    }
                    return true;
                }
                let rest = if element.len() > 2 {
                    &element[2..]
                } else {
                    &[]
                };
                host.on_quake_list_resize((idx + 1).max(0));
                return self.try_command_quake(rest, arg_list_id, args, ret_form, host);
            }
            return true;
        }
        host.on_error("無効なコマンドが指定されました。(quakelist)");
        true
    }

    // ---------------------------------------------------------------
    // Individual quake: global.screen.quake[idx].<action>
    // ---------------------------------------------------------------

    /// Route per-quake commands matching C++ `tnm_command_proc_quake`.
    ///
    /// C++ `cmd_effect.cpp` wait/check 时序对照表：
    ///
    /// | 分支 | 条件 | VM循环行为 | 返回值语义 |
    /// | --- | --- | --- | --- |
    /// | `quake.wait` / `quake.wait_key` | `on_quake_is_active()==true` | 每帧调用 `on_wait_frame()`，直到 inactive 或被 `interrupt/skip_wait` 打断 | 立即返回（void） |
    /// | `quake.wait` / `quake.wait_key` | `on_quake_is_active()==false` | 不进入循环，直接继续执行后续 opcode | 立即返回（void） |
    /// | `quake.check` | 任意时刻 | 不推进帧，不改变 quake 状态 | `1=active, 0=inactive` |
    ///
    /// 该表用于约束 VM 调度语义：
    /// - WAIT 是“阻塞轮询 + 帧推进”；
    /// - CHECK 是“无副作用快照读取”；
    /// - 二者共享 `on_quake_is_active()` 的判定基准，避免脚本层观察到不一致时序。
    fn try_command_quake(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            return true;
        }
        let sub = element[0];
        use crate::elm::quake::*;

        let int_arg = |idx: usize, default: i32| {
            args.get(idx)
                .and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v),
                    _ => None,
                })
                .unwrap_or(default)
        };

        match sub {
            ELM_QUAKE_START
            | ELM_QUAKE_START_WAIT
            | ELM_QUAKE_START_WAIT_KEY
            | ELM_QUAKE_START_NOWAIT
            | ELM_QUAKE_START_ALL
            | ELM_QUAKE_START_ALL_WAIT
            | ELM_QUAKE_START_ALL_WAIT_KEY
            | ELM_QUAKE_START_ALL_NOWAIT => {
                let is_all = matches!(
                    sub,
                    ELM_QUAKE_START_ALL
                        | ELM_QUAKE_START_ALL_WAIT
                        | ELM_QUAKE_START_ALL_WAIT_KEY
                        | ELM_QUAKE_START_ALL_NOWAIT
                );
                let wait_flag = matches!(sub, ELM_QUAKE_START_WAIT | ELM_QUAKE_START_ALL_WAIT)
                    || matches!(sub, ELM_QUAKE_START_WAIT_KEY | ELM_QUAKE_START_ALL_WAIT_KEY);
                let key_flag =
                    matches!(sub, ELM_QUAKE_START_WAIT_KEY | ELM_QUAKE_START_ALL_WAIT_KEY);
                let type_v = int_arg(0, 0);
                let time = int_arg(1, 1000).max(1);
                let cnt = int_arg(2, 0);
                let end_cnt = if arg_list_id >= 1 { int_arg(3, 0) } else { 0 };
                let begin_order = if is_all {
                    if arg_list_id >= 2 {
                        int_arg(4, i32::MIN)
                    } else {
                        i32::MIN
                    }
                } else if arg_list_id >= 2 {
                    int_arg(4, 0)
                } else {
                    0
                };
                let end_order = if is_all {
                    if arg_list_id >= 2 {
                        int_arg(5, i32::MAX)
                    } else {
                        i32::MAX
                    }
                } else if arg_list_id >= 2 {
                    int_arg(5, 0)
                } else {
                    0
                };

                let mut power = 0;
                let mut vec = 0;
                let mut center_x = 0;
                let mut center_y = 0;
                if let Some(last) = args.last() {
                    if let PropValue::List(opt) = &last.value {
                        power = opt
                            .first()
                            .and_then(|p| match p.value {
                                PropValue::Int(v) => Some(v),
                                _ => None,
                            })
                            .unwrap_or(0);
                        if type_v == 2 {
                            center_x = opt
                                .get(1)
                                .and_then(|p| match p.value {
                                    PropValue::Int(v) => Some(v),
                                    _ => None,
                                })
                                .unwrap_or(0);
                            center_y = opt
                                .get(2)
                                .and_then(|p| match p.value {
                                    PropValue::Int(v) => Some(v),
                                    _ => None,
                                })
                                .unwrap_or(0);
                        } else {
                            vec = opt
                                .get(1)
                                .and_then(|p| match p.value {
                                    PropValue::Int(v) => Some(v),
                                    _ => None,
                                })
                                .unwrap_or(0);
                        }
                    }
                }

                let kind = match type_v {
                    1 => VmQuakeKind::Dir,
                    2 => VmQuakeKind::Zoom,
                    _ => VmQuakeKind::Vec,
                };
                host.on_quake_start(VmQuakeRequest {
                    sub,
                    kind,
                    time_ms: time,
                    cnt,
                    end_cnt,
                    begin_order,
                    end_order,
                    wait_flag,
                    key_flag,
                    power,
                    vec,
                    center_x,
                    center_y,
                });
                true
            }
            ELM_QUAKE_END => {
                host.on_quake_end();
                true
            }
            ELM_QUAKE_WAIT | ELM_QUAKE_WAIT_KEY => {
                while host.on_quake_is_active() {
                    if host.should_interrupt() || host.should_skip_wait() {
                        break;
                    }
                    host.on_wait_frame();
                }
                true
            }
            ELM_QUAKE_CHECK => {
                self.stack.push_int(i32::from(host.on_quake_is_active()));
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(quake)");
                true
            }
        }
    }
}
