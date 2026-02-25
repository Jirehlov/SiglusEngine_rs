fn object_composite_is_int_event_list_sub(sub: i32) -> bool {
    use crate::elm::objectlist::*;
    matches!(
        sub,
        ELM_OBJECT_X_REP_EVE | ELM_OBJECT_Y_REP_EVE | ELM_OBJECT_Z_REP_EVE | ELM_OBJECT_TR_REP_EVE
    )
}

fn object_composite_is_int_event_sub(sub: i32) -> bool {
    use crate::elm::objectlist::*;
    matches!(
        sub,
        ELM_OBJECT_X_EVE
            | ELM_OBJECT_Y_EVE
            | ELM_OBJECT_Z_EVE
            | ELM_OBJECT_SCALE_Z_EVE
            | ELM_OBJECT_SCALE_X_EVE
            | ELM_OBJECT_SCALE_Y_EVE
            | ELM_OBJECT_ROTATE_X_EVE
            | ELM_OBJECT_ROTATE_Y_EVE
            | ELM_OBJECT_ROTATE_Z_EVE
            | ELM_OBJECT_TR_EVE
            | ELM_OBJECT_MONO_EVE
            | ELM_OBJECT_REVERSE_EVE
            | ELM_OBJECT_BRIGHT_EVE
            | ELM_OBJECT_DARK_EVE
            | ELM_OBJECT_CENTER_X_EVE
            | ELM_OBJECT_CENTER_Y_EVE
            | ELM_OBJECT_CENTER_Z_EVE
            | ELM_OBJECT_CENTER_REP_X_EVE
            | ELM_OBJECT_CENTER_REP_Y_EVE
            | ELM_OBJECT_CENTER_REP_Z_EVE
            | ELM_OBJECT_COLOR_RATE_EVE
            | ELM_OBJECT_COLOR_ADD_R_EVE
            | ELM_OBJECT_COLOR_ADD_G_EVE
            | ELM_OBJECT_COLOR_ADD_B_EVE
            | ELM_OBJECT_COLOR_R_EVE
            | ELM_OBJECT_COLOR_G_EVE
            | ELM_OBJECT_COLOR_B_EVE
            | ELM_OBJECT_PATNO_EVE
            | ELM_OBJECT_CLIP_LEFT_EVE
            | ELM_OBJECT_CLIP_TOP_EVE
            | ELM_OBJECT_CLIP_RIGHT_EVE
            | ELM_OBJECT_CLIP_BOTTOM_EVE
            | ELM_OBJECT_SRC_CLIP_LEFT_EVE
            | ELM_OBJECT_SRC_CLIP_TOP_EVE
            | ELM_OBJECT_SRC_CLIP_RIGHT_EVE
            | ELM_OBJECT_SRC_CLIP_BOTTOM_EVE
    )
}

fn int_event_method_is_query(method: i32) -> bool {
    use crate::elm::intevent::{ELM_INTEVENT_CHECK, ELM_INTEVENT_GET_EVENT_VALUE};
    method == ELM_INTEVENT_CHECK || method == ELM_INTEVENT_GET_EVENT_VALUE
}

fn int_event_method_is_known(method: i32) -> bool {
    use crate::elm::intevent::*;
    matches!(
        method,
        ELM_INTEVENT_SET
            | ELM_INTEVENT_LOOP
            | ELM_INTEVENT_TURN
            | ELM_INTEVENT_END
            | ELM_INTEVENT_WAIT
            | ELM_INTEVENT_CHECK
            | ELM_INTEVENT__SET
            | ELM_INTEVENT_SET_REAL
            | ELM_INTEVENT_LOOP_REAL
            | ELM_INTEVENT_TURN_REAL
            | ELM_INTEVENT_WAIT_KEY
            | ELM_INTEVENT_YURE
            | ELM_INTEVENT_YURE_REAL
            | ELM_INTEVENT_GET_EVENT_VALUE
    )
}

fn child_property_requires_runtime_args(sub: i32) -> bool {
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

fn composite_int_event_owner_id(sub: i32, list_index: Option<i32>) -> i32 {
    // Keep owner-id stable for host hooks, and encode list index for *_rep_eve paths.
    match list_index {
        Some(idx) => ((sub & 0xFFFF) << 16) | (idx & 0xFFFF),
        None => sub,
    }
}

fn try_property_object_child_composite(
    list_id: i32,
    obj_idx: i32,
    tail: &[i32],
    stage_idx: i32,
    disp_out_of_range_error: bool,
    host: &mut dyn Host,
) -> Option<(PropValue, i32)> {
    use crate::elm::objectlist::ELM_OBJECTLIST_GET_SIZE;

    if tail.is_empty() {
        host.on_error_fatal("CD_PROPERTY stage.object.child: missing method");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }
    if tail[0] == crate::elm::ELM_UP {
        host.on_error_fatal("CD_PROPERTY stage.object.child: ELM_UP is not supported");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }
    if tail[0] == ELM_OBJECTLIST_GET_SIZE {
        if tail.len() > 1 {
            host.on_error_fatal("CD_PROPERTY stage.object.child.get_size: unexpected nested tail");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        let size = host.on_object_child_list_get_size(list_id, obj_idx, Some(stage_idx));
        return Some((PropValue::Int(size.max(0)), crate::elm::form::INT));
    }
    if tail[0] == crate::elm::ELM_ARRAY {
        if tail.len() < 2 {
            host.on_error_fatal("CD_PROPERTY stage.object.child: missing array index");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        let idx = tail[1];
        let size = host.on_object_child_list_get_size(list_id, obj_idx, Some(stage_idx));
        if idx < 0 || (size >= 0 && idx >= size) {
            if disp_out_of_range_error {
                host.on_error_fatal(
                    "範囲外のオブジェクト番号が指定されました。(object_list child)",
                );
            }
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if !host.on_object_child_is_use(list_id, obj_idx, idx, Some(stage_idx)) {
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail.len() == 2 {
            return Some((PropValue::Int(1), crate::elm::form::INT));
        }

        let sub = tail[2];
        if tail.len() > 3 {
            host.on_error_fatal("CD_PROPERTY stage.object.child[idx]: unexpected nested tail");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if Self::object_query_is_str(sub) {
            let v = host.on_object_child_get_str(list_id, obj_idx, idx, sub, Some(stage_idx));
            return Some((PropValue::Str(v), crate::elm::form::STR));
        }
        if Self::object_query_is_int(sub) {
            if Self::child_property_requires_runtime_args(sub) {
                host.on_error_fatal(
                    "CD_PROPERTY stage.object.child[idx]: property requires runtime args",
                );
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            let v = if Self::object_query_uses_query_api(sub) {
                host.on_object_child_query(list_id, obj_idx, idx, sub, &[], Some(stage_idx))
            } else {
                host.on_object_child_get(list_id, obj_idx, idx, sub, Some(stage_idx))
            };
            return Some((PropValue::Int(v), crate::elm::form::INT));
        }

        host.on_error_fatal("CD_PROPERTY stage.object.child[idx]: command-only route");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }

    host.on_error_fatal("CD_PROPERTY stage.object.child: command-only route");
    Some((PropValue::Int(0), crate::elm::form::INT))
}

fn try_property_object_int_event_composite(
    sub: i32,
    tail: &[i32],
    disp_out_of_range_error: bool,
    proc_depth: i32,
    proc_top: i32,
    host: &mut dyn Host,
) -> Option<(PropValue, i32)> {
    if tail.is_empty() {
        host.on_error_fatal("CD_PROPERTY stage.object.*_eve: missing method");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }
    if tail[0] == crate::elm::ELM_UP {
        host.on_error_fatal("CD_PROPERTY stage.object.*_eve: ELM_UP is not supported");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }

    let (method, list_idx) = if Self::object_composite_is_int_event_list_sub(sub) {
        if tail[0] != crate::elm::ELM_ARRAY {
            host.on_error_fatal("CD_PROPERTY stage.object.*_rep_eve: array index required");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail.len() < 2 {
            host.on_error_fatal("CD_PROPERTY stage.object.*_rep_eve: missing array index");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail[1] < 0 && disp_out_of_range_error {
            host.on_error_fatal("範囲外のイベント番号が指定されました。(int_event_list)");
        }
        if tail.len() < 3 {
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail[2] == crate::elm::ELM_ARRAY || tail[2] == crate::elm::ELM_UP {
            host.on_error_fatal("CD_PROPERTY stage.object.*_rep_eve[idx]: invalid nested tail");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail.len() > 3 {
            host.on_error_fatal("CD_PROPERTY stage.object.*_rep_eve[idx]: unexpected nested tail");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        (tail[2], Some(tail[1]))
    } else {
        if tail[0] == crate::elm::ELM_ARRAY {
            host.on_error_fatal("CD_PROPERTY stage.object.*_eve: array access is invalid");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if tail.len() > 1 {
            host.on_error_fatal("CD_PROPERTY stage.object.*_eve: unexpected nested tail");
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        (tail[0], None)
    };

    let owner_id = Self::composite_int_event_owner_id(sub, list_idx);
    if Self::int_event_method_is_query(method) {
        let v = if method == crate::elm::intevent::ELM_INTEVENT_CHECK {
            if host.on_int_event_check(owner_id) {
                1
            } else {
                0
            }
        } else {
            host.on_int_event_get_value(owner_id)
        };
        return Some((PropValue::Int(v), crate::elm::form::INT));
    }
    if method == crate::elm::intevent::ELM_INTEVENT_WAIT
        || method == crate::elm::intevent::ELM_INTEVENT_WAIT_KEY
    {
        // flow_proc.cpp alignment: WAIT is proc-driven, so property lane exposes
        // a poll-style observable state instead of mutating command completion.
        // Consumer example: script can poll until `EVE_WAIT_DONE` / `EVE_WAIT_KEY_SKIPPED`
        // and keep stepping while `EVE_WAIT_PENDING` is returned.
        let key_skip = method == crate::elm::intevent::ELM_INTEVENT_WAIT_KEY;
        let before_wait = host.on_int_event_check(owner_id);
        if !before_wait {
            host.on_int_event_wait_status(owner_id, key_skip, crate::vm::EVE_WAIT_DONE);
            host.on_int_event_wait_status_with_proc(
                owner_id,
                key_skip,
                crate::vm::EVE_WAIT_DONE,
                proc_depth,
                proc_top,
            );
            return Some((
                PropValue::Int(crate::vm::EVE_WAIT_DONE),
                crate::elm::form::INT,
            ));
        }
        if key_skip && host.should_skip_wait() {
            host.on_int_event_wait_status(owner_id, key_skip, crate::vm::EVE_WAIT_KEY_SKIPPED);
            host.on_int_event_wait_status_with_proc(
                owner_id,
                key_skip,
                crate::vm::EVE_WAIT_KEY_SKIPPED,
                proc_depth,
                proc_top,
            );
            return Some((
                PropValue::Int(crate::vm::EVE_WAIT_KEY_SKIPPED),
                crate::elm::form::INT,
            ));
        }
        host.on_int_event_wait(owner_id, key_skip);
        host.on_wait_frame();
        let still_waiting = host.on_int_event_check(owner_id);
        let status = if still_waiting {
            crate::vm::EVE_WAIT_PENDING
        } else {
            crate::vm::EVE_WAIT_DONE
        };
        host.on_int_event_wait_status(owner_id, key_skip, status);
        host.on_int_event_wait_status_with_proc(
            owner_id,
            key_skip,
            status,
            proc_depth,
            proc_top,
        );
        return Some((PropValue::Int(status), crate::elm::form::INT));
    }
    if Self::int_event_method_is_known(method) {
        host.on_error_fatal(
            "CD_PROPERTY stage.object.*_eve: mutating method requires command lane",
        );
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }

    host.on_error_fatal("無効なコマンドが指定されました。(intevent)");
    Some((PropValue::Int(0), crate::elm::form::INT))
}

fn try_property_object_known_command_only_composite(
    list_id: i32,
    obj_idx: i32,
    sub: i32,
    tail: &[i32],
    stage_idx: i32,
    disp_out_of_range_error: bool,
    proc_depth: i32,
    proc_top: i32,
    host: &mut dyn Host,
) -> Option<(PropValue, i32)> {
    if sub == crate::elm::objectlist::ELM_OBJECT_CHILD {
        return Self::try_property_object_child_composite(
            list_id,
            obj_idx,
            tail,
            stage_idx,
            disp_out_of_range_error,
            host,
        );
    }
    if Self::object_composite_is_int_event_sub(sub)
        || Self::object_composite_is_int_event_list_sub(sub)
    {
        return Self::try_property_object_int_event_composite(
            sub,
            tail,
            disp_out_of_range_error,
            proc_depth,
            proc_top,
            host,
        );
    }

    if tail.is_empty() {
        host.on_error_fatal("CD_PROPERTY stage.object composite: missing method");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }
    if tail[0] == crate::elm::ELM_UP {
        host.on_error_fatal("CD_PROPERTY stage.object composite: ELM_UP is not supported");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }
    if tail[0] == crate::elm::ELM_ARRAY {
        host.on_error_fatal("CD_PROPERTY stage.object composite: array access is invalid");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }

    host.on_error_fatal("CD_PROPERTY stage.object composite: command-only route");
    Some((PropValue::Int(0), crate::elm::form::INT))
}
