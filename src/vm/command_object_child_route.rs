impl Vm {
    const ERR_OBJECT_CHILD_INVALID: &'static str = "無効なコマンドが指定されました。(object_list child)";
    const ERR_OBJECT_CHILD_OOR: &'static str = "範囲外のオブジェクト番号が指定されました。(object_list child)";

    fn object_child_emit_invalid(host: &mut dyn Host) {
        host.on_error_fatal(Self::ERR_OBJECT_CHILD_INVALID);
    }

    fn child_getter_lane_mismatch(
        host: &mut dyn Host,
        ret_form: i32,
        stack: &mut IfcStack,
    ) {
        // C++ cmd_object.cpp::tnm_command_proc_object_list keeps this lane in
        // `無効なコマンド... + get_element_name()`; Rust normalizes to child suffix.
        Self::object_child_emit_invalid(host);
        if ret_form != crate::elm::form::VOID {
            Self::object_frame_action_push_default(stack, ret_form);
        }
    }

    fn object_child_property_requires_runtime_args(sub: i32) -> bool {
        use crate::elm::objectlist::*;
        matches!(
            sub,
            ELM_OBJECT_GET_SIZE_X
                | ELM_OBJECT_GET_SIZE_Y
                | ELM_OBJECT_GET_SIZE_Z
                | ELM_OBJECT_GET_PIXEL_COLOR_R
                | ELM_OBJECT_GET_PIXEL_COLOR_G
                | ELM_OBJECT_GET_PIXEL_COLOR_B
                | ELM_OBJECT_GET_PIXEL_COLOR_A
        )
    }

    fn try_command_object_child(
        &mut self,
        list_id: i32,
        stage_idx: Option<i32>,
        obj_idx: i32,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::objectlist::{ELM_OBJECTLIST_GET_SIZE, ELM_OBJECTLIST_RESIZE};

        if element.is_empty() {
            return true;
        }
        if element[0] == crate::elm::ELM_UP {
            Self::object_child_emit_invalid(host);
            return true;
        }

        if element[0] == ELM_OBJECTLIST_GET_SIZE {
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(
                    host.on_object_child_list_get_size(list_id, obj_idx, stage_idx)
                        .max(0),
                );
            }
            return true;
        }
        if element[0] == ELM_OBJECTLIST_RESIZE {
            host.on_object_child_list_resize(
                list_id,
                obj_idx,
                Self::int_arg(args, 0).max(0),
                stage_idx,
            );
            return true;
        }

        if element[0] != crate::elm::ELM_ARRAY {
            Self::object_child_emit_invalid(host);
            return true;
        }
        if element.len() < 2 {
            Self::object_child_emit_invalid(host);
            return true;
        }

        let child_idx = element[1];
        let size = host.on_object_child_list_get_size(list_id, obj_idx, stage_idx);
        if size >= 0 && (child_idx < 0 || child_idx >= size) {
            if self.options.disp_out_of_range_error {
                host.on_error_fatal(Self::ERR_OBJECT_CHILD_OOR);
            }
            Self::object_frame_action_push_default(&mut self.stack, ret_form);
            return true;
        }
        if !host.on_object_child_is_use(list_id, obj_idx, child_idx, stage_idx) {
            Self::object_frame_action_push_default(&mut self.stack, ret_form);
            return true;
        }

        if element.len() == 2 {
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(1);
            }
            return true;
        }

        let sub = element[2];
        if element.len() > 3 {
            Self::object_child_emit_invalid(host);
            Self::object_frame_action_push_default(&mut self.stack, ret_form);
            return true;
        }

        if arg_list_id != 0 && arg_list_id != 1 {
            if Self::object_query_is_int(sub) || Self::object_query_is_str(sub) {
                Self::child_getter_lane_mismatch(
                    host,
                    ret_form,
                    &mut self.stack,
                );
                return true;
            }
            if Self::object_property_is_settable_int(sub) {
                Self::object_child_emit_invalid(host);
                return true;
            }
        }

        if arg_list_id == 0 {
            if Self::object_query_is_str(sub) {
                if ret_form != crate::elm::form::STR {
                    Self::child_getter_lane_mismatch(
                        host,
                        ret_form,
                        &mut self.stack,
                    );
                    return true;
                }
                self.stack.push_str(
                    host.on_object_child_get_str(list_id, obj_idx, child_idx, sub, stage_idx),
                );
                return true;
            }
            if Self::object_query_is_int(sub) {
                if ret_form != crate::elm::form::INT {
                    Self::child_getter_lane_mismatch(
                        host,
                        ret_form,
                        &mut self.stack,
                    );
                    return true;
                }
                if Self::object_child_property_requires_runtime_args(sub) && args.is_empty() {
                    Self::object_child_emit_invalid(host);
                    self.stack.push_int(0);
                } else {
                    let v = if args.is_empty() {
                        host.on_object_child_get(list_id, obj_idx, child_idx, sub, stage_idx)
                    } else {
                        host.on_object_child_query(list_id, obj_idx, child_idx, sub, args, stage_idx)
                    };
                    self.stack.push_int(v);
                }
                return true;
            }

            Self::child_getter_lane_mismatch(
                host,
                ret_form,
                &mut self.stack,
            );
            return true;
        }

        if arg_list_id == 1 {
            if Self::object_property_is_settable_int(sub) {
                if args.is_empty() {
                    Self::object_child_emit_invalid(host);
                    return true;
                }
                host.on_object_child_property(
                    list_id,
                    obj_idx,
                    child_idx,
                    sub,
                    Self::int_arg(args, 0),
                    stage_idx,
                );
                return true;
            }
            if Self::object_query_is_int(sub) || Self::object_query_is_str(sub) {
                Self::object_child_emit_invalid(host);
                return true;
            }
        }

        host.on_object_child_action(list_id, obj_idx, child_idx, sub, args, stage_idx);
        true
    }
}
