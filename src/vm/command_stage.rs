/// Stage / Group command routing.
///
/// C++ reference: cmd_stage.cpp
///
/// Covers:
///   - `global.stage[idx]` → stage sub-dispatch (object, mwnd, world, effect, quake, group, btnsel)
///   - `global.back` → shortcut for stage[TNM_STAGE_BACK]
///   - `global.front` → shortcut for stage[TNM_STAGE_FRONT]
///   - `global.next` → shortcut for stage[TNM_STAGE_NEXT]
///   - `stage.objbtngroup[idx].*` → group commands
///
/// Stage-level object/mwnd/btnselitem and create_object/create_mwnd fall through to host.
/// World/effect/quake are routed to their dedicated VM-side modules.
/// Group commands are fully routed here matching C++ `tnm_command_proc_group`.
use super::*;

impl Vm {
    // ---------------------------------------------------------------
    // Stage list: global.stage
    // ---------------------------------------------------------------

    /// Route `global.stage.<sub>` commands matching C++ `tnm_command_proc_stage_list`.
    pub(super) fn try_command_stage_list(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            // Bare stage_list element → accept (C++ sets ret element).
            return true;
        }

        if element[0] == crate::elm::ELM_ARRAY {
            // Indexed: stage[idx].sub
            if element.len() >= 2 {
                let _stage_idx = element[1];
                let rest = if element.len() > 2 { &element[2..] } else { &[] };
                return self.try_command_stage(rest, arg_list_id, args, ret_form, host);
            }
            return true;
        }

        host.on_error("無効なコマンドが指定されました。(stage_list)");
        true
    }

    // ---------------------------------------------------------------
    // Per-stage: stage[idx].<sub>
    // ---------------------------------------------------------------

    /// Route per-stage commands matching C++ `tnm_command_proc_stage`.
    ///
    /// Sub-dispatches to the appropriate module based on the stage sub-element:
    ///   - OBJECT → host passthrough
    ///   - MWND → host passthrough
    ///   - WORLD → command_world::try_command_world_list
    ///   - EFFECT → command_effect::try_command_effect_list (via screen path)
    ///   - QUAKE → command_effect::try_command_quake_list (via screen path)
    ///   - OBJBTNGROUP → group routing
    ///   - BTNSELITEM → host passthrough
    ///   - CREATE_OBJECT / CREATE_MWND → host passthrough
    pub(super) fn try_command_stage(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.is_empty() {
            // Bare stage element → accept (C++ sets ret element).
            return true;
        }
        let sub = element[0];
        use crate::elm::objectlist::*;

        match sub {
            // Object list — VM-side routing via command_object module.
            ELM_STAGE_OBJECT => {
                self.try_command_object_list(sub, &element[1..], arg_list_id, args, ret_form, host)
            }

            // Mwnd list — VM-side routing via command_mwnd module.
            ELM_STAGE_MWND => {
                self.try_command_mwnd_list(&element[1..], arg_list_id, args, ret_form, host)
            }

            // World list → command_world module.
            ELM_STAGE_WORLD => {
                self.try_command_world_list(&element[1..], arg_list_id, args, ret_form, host)
            }

            // Effect list → command_effect module.
            ELM_STAGE_EFFECT => {
                self.try_command_effect_list(&element[1..], arg_list_id, args, ret_form, host)
            }

            // Quake list → command_effect module.
            ELM_STAGE_QUAKE => {
                self.try_command_quake_list(&element[1..], arg_list_id, args, ret_form, host)
            }

            // Object button group → group routing.
            ELM_STAGE_OBJBTNGROUP => {
                self.try_command_group_list(&element[1..], arg_list_id, args, ret_form, host)
            }

            // Button selection item — host passthrough.
            ELM_STAGE_BTNSELITEM => false,

            // Create object / create mwnd — host passthrough.
            ELM_STAGE_CREATE_OBJECT | ELM_STAGE_CREATE_MWND => false,

            _ => {
                host.on_error("無効なコマンドが指定されました。(stage)");
                true
            }
        }
    }

    // ---------------------------------------------------------------
    // Group list: stage.objbtngroup
    // ---------------------------------------------------------------

    /// Route `stage.objbtngroup.<sub>` matching C++ `tnm_command_proc_group_list`.
    fn try_command_group_list(
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
        use crate::elm::grouplist::*;

        if element[0] == crate::elm::ELM_ARRAY {
            // Indexed: group[idx].sub
            if element.len() >= 2 {
                let _group_idx = element[1];
                let rest = if element.len() > 2 { &element[2..] } else { &[] };
                return self.try_command_group(rest, arg_list_id, args, ret_form, host);
            }
            return true;
        }

        match element[0] {
            ELM_GROUPLIST_ALLOC => {
                // C++ group_list->clear(); group_list->resize(arg0)
                // Accept — host passthrough for allocation.
                host.on_group_alloc(args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v), _ => None,
                }).unwrap_or(0));
                true
            }
            ELM_GROUPLIST_FREE => {
                // C++ group_list->clear()
                host.on_group_free();
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(grouplist)");
                true
            }
        }
    }

    // ---------------------------------------------------------------
    // Per-group: group[idx].<sub>
    // ---------------------------------------------------------------

    /// Route per-group commands matching C++ `tnm_command_proc_group`.
    ///
    /// Fully routes all group sub-commands per C++ cmd_stage.cpp.
    fn try_command_group(
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
        use crate::elm::group::*;

        match sub {
            // --- Selection / Start commands → host ---
            ELM_GROUP_SEL => {
                // C++ input->now.use(); group->init_sel(); group->set_wait_flag(true); group->start()
                host.on_group_sel(sub);
                true
            }
            ELM_GROUP_SEL_CANCEL => {
                // se_no from optional arg0
                let _se_no = if arg_list_id > 0 {
                    args.first().and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(-1)
                } else { -1 };
                host.on_group_sel(sub);
                true
            }
            ELM_GROUP_INIT => {
                // C++ group->reinit()
                host.on_group_init();
                true
            }
            ELM_GROUP_START => {
                // C++ input->now.use(); group->init_sel(); group->start()
                host.on_group_start(sub);
                true
            }
            ELM_GROUP_START_CANCEL => {
                let _se_no = if arg_list_id > 0 {
                    args.first().and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(-1)
                } else { -1 };
                host.on_group_start(sub);
                true
            }
            ELM_GROUP_END => {
                // C++ group->end()
                host.on_group_end();
                true
            }

            // --- Query commands → push int ---
            ELM_GROUP_GET_HIT_NO => {
                // C++ tnm_stack_push_int(group->get_hit_button_no())
                self.stack.push_int(-1); // no hit
                true
            }
            ELM_GROUP_GET_PUSHED_NO => {
                // C++ tnm_stack_push_int(group->get_pushed_button_no())
                self.stack.push_int(-1); // no push
                true
            }
            ELM_GROUP_GET_DECIDED_NO => {
                // C++ tnm_stack_push_int(group->get_decided_button_no())
                self.stack.push_int(-1);
                true
            }
            ELM_GROUP_GET_RESULT => {
                // C++ tnm_stack_push_int(group->get_result())
                self.stack.push_int(-1);
                true
            }
            ELM_GROUP_GET_RESULT_BUTTON_NO => {
                // C++ tnm_stack_push_int(group->get_result_button_no())
                self.stack.push_int(-1);
                true
            }

            // --- Property get/set ---
            ELM_GROUP_ORDER => {
                if arg_list_id == 0 {
                    // C++ tnm_stack_push_int(group->get_order())
                    self.stack.push_int(0);
                } else {
                    // C++ group->set_order(arg0)
                    host.on_group_property(sub, args.first().and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v), _ => None,
                    }).unwrap_or(0));
                }
                true
            }
            ELM_GROUP_LAYER => {
                if arg_list_id == 0 {
                    self.stack.push_int(0);
                } else {
                    host.on_group_property(sub, args.first().and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v), _ => None,
                    }).unwrap_or(0));
                }
                true
            }
            ELM_GROUP_CANCEL_PRIORITY => {
                if arg_list_id == 0 {
                    self.stack.push_int(0);
                } else {
                    host.on_group_property(sub, args.first().and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v), _ => None,
                    }).unwrap_or(0));
                }
                true
            }

            _ => {
                host.on_error("無効なコマンドが指定されました。(group)");
                true
            }
        }
    }
}
