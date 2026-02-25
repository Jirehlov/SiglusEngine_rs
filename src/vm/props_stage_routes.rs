fn stage_index_from_property_root(root: i32) -> Option<i32> {
    if crate::elm::global::is_stage(root) {
        return None;
    }
    if crate::elm::global::is_back(root) {
        return Some(0);
    }
    if crate::elm::global::is_front(root) {
        return Some(1);
    }
    if crate::elm::global::is_next(root) {
        return Some(2);
    }
    None
}

fn group_query_is_int(sub: i32) -> bool {
    use crate::elm::group::*;
    matches!(
        sub,
        ELM_GROUP_GET_HIT_NO
            | ELM_GROUP_GET_PUSHED_NO
            | ELM_GROUP_GET_DECIDED_NO
            | ELM_GROUP_GET_RESULT
            | ELM_GROUP_GET_RESULT_BUTTON_NO
            | ELM_GROUP_ORDER
            | ELM_GROUP_LAYER
            | ELM_GROUP_CANCEL_PRIORITY
    )
}

fn object_query_is_str(sub: i32) -> bool {
    use crate::elm::objectlist::*;
    matches!(
        sub,
        ELM_OBJECT_GET_FILE_NAME | ELM_OBJECT_GET_STRING | ELM_OBJECT_GET_ELEMENT_NAME
    )
}

fn object_query_is_int(sub: i32) -> bool {
    Self::object_property_is_settable_int(sub) || Self::object_query_is_readonly_int(sub)
}

fn object_query_is_readonly_int(sub: i32) -> bool {
    use crate::elm::objectlist::*;
    matches!(
        sub,
        ELM_OBJECT_GET_PAT_CNT
            | ELM_OBJECT_GET_SIZE_X
            | ELM_OBJECT_GET_SIZE_Y
            | ELM_OBJECT_GET_SIZE_Z
            | ELM_OBJECT_GET_PIXEL_COLOR_R
            | ELM_OBJECT_GET_PIXEL_COLOR_G
            | ELM_OBJECT_GET_PIXEL_COLOR_B
            | ELM_OBJECT_GET_PIXEL_COLOR_A
            | ELM_OBJECT_EXIST_TYPE
            | ELM_OBJECT_GET_TYPE
            | ELM_OBJECT_GET_NUMBER
            | ELM_OBJECT_GET_MOVIE_SEEK_TIME
            | ELM_OBJECT_FRAME_ACTION
            | ELM_OBJECT_FRAME_ACTION_CH
            | ELM_OBJECT_X_REP
            | ELM_OBJECT_Y_REP
            | ELM_OBJECT_Z_REP
            | ELM_OBJECT_TR_REP
            | ELM_OBJECT_CHECK_MOVIE
            | ELM_OBJECT_GET_BUTTON_STATE
            | ELM_OBJECT_GET_BUTTON_HIT_STATE
            | ELM_OBJECT_GET_BUTTON_REAL_STATE
            | ELM_OBJECT_GET_BUTTON_PUSHKEEP
            | ELM_OBJECT_GET_BUTTON_ALPHA_TEST
            | ELM_OBJECT_GET_BUTTON_NO
            | ELM_OBJECT_GET_BUTTON_GROUP_NO
            | ELM_OBJECT_GET_BUTTON_ACTION_NO
            | ELM_OBJECT_GET_BUTTON_SE_NO
            | ELM_OBJECT_EMOTE_CHECK_PLAYING
            | ELM_OBJECT__IAPP_DUMMY
    )
}

fn object_query_is_frame_action_sub(sub: i32) -> bool {
    use crate::elm::objectlist::*;
    matches!(sub, ELM_OBJECT_FRAME_ACTION | ELM_OBJECT_FRAME_ACTION_CH)
}

fn try_property_object_all_eve(
    list_id: i32,
    obj_idx: i32,
    tail: &[i32],
    stage_idx: i32,
    host: &mut dyn Host,
) -> Option<(PropValue, i32)> {
    use crate::elm::allevent::{ELM_ALLEVENT_CHECK, ELM_ALLEVENT_END, ELM_ALLEVENT_WAIT};

    if tail.is_empty() {
        host.on_error_fatal("CD_PROPERTY stage.object.all_eve: missing method");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }
    if tail[0] == crate::elm::ELM_UP {
        host.on_error_fatal("CD_PROPERTY stage.object.all_eve: ELM_UP is not supported");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }
    if tail[0] == crate::elm::ELM_ARRAY {
        host.on_error_fatal("CD_PROPERTY stage.object.all_eve: array access is invalid");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }
    if tail.len() > 1 {
        host.on_error_fatal("CD_PROPERTY stage.object.all_eve: unexpected nested tail");
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }

    match tail[0] {
        ELM_ALLEVENT_CHECK => Some((
            PropValue::Int(host.on_object_get(
                list_id,
                obj_idx,
                crate::elm::objectlist::ELM_OBJECT_ALL_EVE,
                Some(stage_idx),
            )),
            crate::elm::form::INT,
        )),
        ELM_ALLEVENT_END | ELM_ALLEVENT_WAIT => {
            host.on_error_fatal(
                "CD_PROPERTY stage.object.all_eve: mutating method requires command lane",
            );
            Some((PropValue::Int(0), crate::elm::form::INT))
        }
        _ => {
            host.on_error_fatal("無効なコマンドが指定されました。(allevent)");
            Some((PropValue::Int(0), crate::elm::form::INT))
        }
    }
}


include!("props_stage_composite_routes.rs");

fn object_query_is_known_composite_sub(sub: i32) -> bool {
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
            | ELM_OBJECT_X_REP_EVE
            | ELM_OBJECT_Y_REP_EVE
            | ELM_OBJECT_Z_REP_EVE
            | ELM_OBJECT_TR_REP_EVE
            | ELM_OBJECT_SRC_CLIP_LEFT_EVE
            | ELM_OBJECT_SRC_CLIP_TOP_EVE
            | ELM_OBJECT_SRC_CLIP_RIGHT_EVE
            | ELM_OBJECT_SRC_CLIP_BOTTOM_EVE
            | ELM_OBJECT_ALL_EVE
            | ELM_OBJECT_CHILD
    )
}

fn validate_object_frame_action_tail(&self, sub: i32, tail: &[i32], host: &mut dyn Host) -> bool {
    use crate::elm::frameaction::{is_frameactionlist_get_size, is_frameactionlist_resize};
    use crate::elm::objectlist::ELM_OBJECT_FRAME_ACTION_CH;

    if tail.is_empty() {
        // C++ frame_action/frame_action_list: empty tail returns element ptr lane;
        // property lane degrades to typed default without fatal.
        return true;
    }

    let head = tail[0];
    if head == crate::elm::ELM_UP {
        host.on_error_fatal("無効なコマンドが指定されました。(frame_action)");
        return false;
    }

    if sub == ELM_OBJECT_FRAME_ACTION_CH {
        if head == crate::elm::ELM_ARRAY {
            if tail.len() < 2 {
                host.on_error_fatal(
                    "無効なコマンドが指定されました。(frame_action_ch)",
                );
                return false;
            }
            let idx = tail[1];
            if idx < 0 && self.options.disp_out_of_range_error {
                host.on_error_fatal(
                    "範囲外のフレームアクション番号が指定されました。(frame_action_ch)",
                );
            }
            if idx < 0 {
                return false;
            }
            if tail.len() >= 3 {
                let nested = tail[2];
                if nested == crate::elm::ELM_UP {
                    if tail.len() == 3 {
                        host.on_error_fatal(
                            "無効なコマンドが指定されました。(frame_action_ch)",
                        );
                        return false;
                    }
                    if tail.len() >= 5
                        && (is_frameactionlist_get_size(tail[3]) || is_frameactionlist_resize(tail[3]))
                    {
                        host.on_error_fatal(
                            "無効なコマンドが指定されました。(frame_action_ch)",
                        );
                        return false;
                    }
                    if tail.len() > 5
                        && tail[3] == crate::elm::frameaction::ELM_FRAMEACTION_COUNTER
                    {
                        host.on_error_fatal(
                            "無効なコマンドが指定されました。(frame_action_ch)",
                        );
                        return false;
                    }
                } else if is_frameactionlist_get_size(nested) || is_frameactionlist_resize(nested) {
                    host.on_error_fatal(
                        "無効なコマンドが指定されました。(frame_action_ch)",
                    );
                    return false;
                } else if tail.len() > 4
                    && nested == crate::elm::frameaction::ELM_FRAMEACTION_COUNTER
                {
                    host.on_error_fatal(
                        "無効なコマンドが指定されました。(frame_action_ch)",
                    );
                    return false;
                }
            }
            return true;
        }

        if is_frameactionlist_get_size(head) || is_frameactionlist_resize(head) {
            if tail.len() > 1 {
                host.on_error_fatal(
                    "無効なコマンドが指定されました。(frame_action_ch)",
                );
                return false;
            }
            return true;
        }
    }

    true
}

fn default_frame_action_property(
    &self,
    sub: i32,
    tail: &[i32],
    host: &mut dyn Host,
) -> Option<(PropValue, i32)> {
    use crate::elm::frameaction::{
        is_frameaction_end, is_frameaction_is_end_action, is_frameaction_start,
        is_frameactionlist_get_size, is_frameactionlist_resize,
    };
    use crate::elm::objectlist::{ELM_OBJECT_FRAME_ACTION, ELM_OBJECT_FRAME_ACTION_CH};

    let err = |host: &mut dyn Host, msg: &str| {
        host.on_error_fatal(msg);
        (PropValue::Int(0), crate::elm::form::INT)
    };

    if tail.is_empty() {
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }

    if sub == ELM_OBJECT_FRAME_ACTION_CH {
        if tail[0] == crate::elm::ELM_ARRAY {
            if tail.len() < 3 {
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if tail[2] == crate::elm::ELM_UP {
                if tail.len() == 3 {
                    return Some(err(host, "無効なコマンドが指定されました。(frame_action_ch)"));
                }
                if is_frameactionlist_get_size(tail[3]) {
                    if tail.len() > 4 {
                        return Some(err(host, "無効なコマンドが指定されました。(frame_action_ch)"));
                    }
                    return Some((PropValue::Int(0), crate::elm::form::INT));
                }
                if is_frameactionlist_resize(tail[3]) {
                    return Some(err(host, "無効なコマンドが指定されました。(frame_action_ch)"));
                }
                if tail[3] == crate::elm::frameaction::ELM_FRAMEACTION_COUNTER {
                    if tail.len() == 4 {
                        return Some(err(host, "無効なコマンドが指定されました。(frame_action.counter)"));
                    }
                    if tail.len() > 5 {
                        return Some(err(host, "無効なコマンドが指定されました。(frame_action_ch)"));
                    }
                    return Some(err(host, "無効なコマンドが指定されました。(frame_action_ch)"));
                }
            }
        }
        if (is_frameactionlist_get_size(tail[0]) || is_frameactionlist_resize(tail[0]))
            && tail.len() > 1
        {
            return Some(err(host, "無効なコマンドが指定されました。(frame_action_ch)"));
        }
    }

    let method = if sub == ELM_OBJECT_FRAME_ACTION_CH && tail[0] == crate::elm::ELM_ARRAY {
        if tail.len() >= 4 && tail[2] == crate::elm::ELM_UP {
            tail[3]
        } else if tail.len() >= 3 {
            tail[2]
        } else {
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
    } else {
        tail[0]
    };

    if sub == ELM_OBJECT_FRAME_ACTION_CH
        && tail.first().copied() == Some(crate::elm::ELM_ARRAY)
        && tail.get(2).copied() == Some(crate::elm::frameaction::ELM_FRAMEACTION_COUNTER)
        && tail.len() > 4
    {
        return Some(err(host, "無効なコマンドが指定されました。(frame_action_ch)"));
    }

    if is_frameaction_is_end_action(method) {
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }

    if is_frameaction_start(method) || is_frameaction_end(method) {
        return Some(err(
            host,
            "無効なコマンドが指定されました。(frame_action)",
        ));
    }

    if sub == ELM_OBJECT_FRAME_ACTION_CH {
        if is_frameactionlist_get_size(tail[0]) {
            return Some((PropValue::Int(0), crate::elm::form::INT));
        }
        if is_frameactionlist_resize(tail[0]) {
            return Some(err(
                host,
                "無効なコマンドが指定されました。(frame_action_ch)",
            ));
        }
    }

    if sub == ELM_OBJECT_FRAME_ACTION || sub == ELM_OBJECT_FRAME_ACTION_CH {
        return Some(err(host, "無効なコマンドが指定されました。(frame_action)"));
    }

    None
}

fn object_query_uses_query_api(sub: i32) -> bool {
    use crate::elm::objectlist::*;
    matches!(
        sub,
        ELM_OBJECT_GET_MOVIE_SEEK_TIME
            | ELM_OBJECT_CHECK_MOVIE
            | ELM_OBJECT_FRAME_ACTION
            | ELM_OBJECT_FRAME_ACTION_CH
            | ELM_OBJECT__IAPP_DUMMY
    )
}

pub(super) fn object_property_is_settable_int(sub: i32) -> bool {
    use crate::elm::objectlist::*;
    matches!(
        sub,
        ELM_OBJECT_WIPE_COPY
            | ELM_OBJECT_WIPE_ERASE
            | ELM_OBJECT_CLICK_DISABLE
            | ELM_OBJECT_DISP
            | ELM_OBJECT_PATNO
            | ELM_OBJECT_WORLD
            | ELM_OBJECT_ORDER
            | ELM_OBJECT_LAYER
            | ELM_OBJECT_X
            | ELM_OBJECT_Y
            | ELM_OBJECT_Z
            | ELM_OBJECT_CENTER_X
            | ELM_OBJECT_CENTER_Y
            | ELM_OBJECT_CENTER_Z
            | ELM_OBJECT_CENTER_REP_X
            | ELM_OBJECT_CENTER_REP_Y
            | ELM_OBJECT_CENTER_REP_Z
            | ELM_OBJECT_SCALE_X
            | ELM_OBJECT_SCALE_Y
            | ELM_OBJECT_SCALE_Z
            | ELM_OBJECT_ROTATE_X
            | ELM_OBJECT_ROTATE_Y
            | ELM_OBJECT_ROTATE_Z
            | ELM_OBJECT_CLIP_USE
            | ELM_OBJECT_CLIP_LEFT
            | ELM_OBJECT_CLIP_TOP
            | ELM_OBJECT_CLIP_RIGHT
            | ELM_OBJECT_CLIP_BOTTOM
            | ELM_OBJECT_SRC_CLIP_USE
            | ELM_OBJECT_SRC_CLIP_LEFT
            | ELM_OBJECT_SRC_CLIP_TOP
            | ELM_OBJECT_SRC_CLIP_RIGHT
            | ELM_OBJECT_SRC_CLIP_BOTTOM
            | ELM_OBJECT_TR
            | ELM_OBJECT_MONO
            | ELM_OBJECT_REVERSE
            | ELM_OBJECT_BRIGHT
            | ELM_OBJECT_DARK
            | ELM_OBJECT_COLOR_R
            | ELM_OBJECT_COLOR_G
            | ELM_OBJECT_COLOR_B
            | ELM_OBJECT_COLOR_RATE
            | ELM_OBJECT_COLOR_ADD_R
            | ELM_OBJECT_COLOR_ADD_G
            | ELM_OBJECT_COLOR_ADD_B
            | ELM_OBJECT_MASK_NO
            | ELM_OBJECT_TONECURVE_NO
            | ELM_OBJECT_CULLING
            | ELM_OBJECT_ALPHA_TEST
            | ELM_OBJECT_ALPHA_BLEND
            | ELM_OBJECT_BLEND
            | ELM_OBJECT_LIGHT_NO
            | ELM_OBJECT_FOG_USE
            | ELM_OBJECT_EMOTE_KOE_CHARA_NO
            | ELM_OBJECT_EMOTE_MOUTH_VOLUME
    )
}

fn try_property_stage_object_group(
    &mut self,
    element: &[i32],
    host: &mut dyn Host,
) -> Option<(PropValue, i32)> {
    use crate::elm::objectlist::{
        ELM_OBJECTLIST_GET_SIZE, ELM_STAGE_OBJBTNGROUP, ELM_STAGE_OBJECT,
    };

    if element.is_empty() {
        return None;
    }

    let (stage_idx, mut pos) = if crate::elm::global::is_stage(element[0]) {
        if element.len() < 3 || element[1] != crate::elm::ELM_ARRAY {
            return None;
        }
        (element[2], 3)
    } else if let Some(idx) = Self::stage_index_from_property_root(element[0]) {
        (idx, 1)
    } else {
        return None;
    };

    let stage_size = host.on_stage_list_get_size();
    if stage_size >= 0 && (stage_idx < 0 || stage_idx >= stage_size) {
        if self.options.disp_out_of_range_error {
            host.on_error_fatal("範囲外のステージ番号が指定されました。(stage_list)");
        }
        return Some((PropValue::Int(0), crate::elm::form::INT));
    }

    if pos >= element.len() {
        return None;
    }

    if element[pos] == ELM_STAGE_OBJECT {
        pos += 1;
        if pos < element.len() && element[pos] == ELM_OBJECTLIST_GET_SIZE {
            let n = host.on_object_list_get_size(ELM_STAGE_OBJECT, Some(stage_idx));
            return Some((PropValue::Int(n.max(0)), crate::elm::form::INT));
        }
        if pos + 3 <= element.len() && element[pos] == crate::elm::ELM_ARRAY {
            let obj_idx = element[pos + 1];
            let obj_size = host.on_object_list_get_size(ELM_STAGE_OBJECT, Some(stage_idx));
            if obj_size >= 0 && (obj_idx < 0 || obj_idx >= obj_size) {
                if self.options.disp_out_of_range_error {
                    host.on_error_fatal("範囲外のオブジェクト番号が指定されました。(object_list)");
                }
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if !host.on_object_is_use(ELM_STAGE_OBJECT, obj_idx, Some(stage_idx)) {
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            let sub = element[pos + 2];
            if element.len() != pos + 3 {
                let tail = &element[pos + 3..];
                if Self::object_query_is_frame_action_sub(sub) {
                    if !self.validate_object_frame_action_tail(sub, tail, host) {
                        return Some((PropValue::Int(0), crate::elm::form::INT));
                    }
                    if let Some((v, form)) = self.default_frame_action_property(sub, tail, host) {
                        return Some((v, form));
                    }
                    if let Some((v, form)) = host.on_object_frame_action_property(
                        ELM_STAGE_OBJECT,
                        obj_idx,
                        sub,
                        tail,
                        Some(stage_idx),
                    ) {
                        return Some((v, form));
                    }
                    host.on_error_fatal(
                        "無効なコマンドが指定されました。(frame_action)",
                    );
                    return Some((PropValue::Int(0), crate::elm::form::INT));
                }
                if Self::object_query_is_int(sub) || Self::object_query_is_str(sub) {
                    host.on_error_fatal("CD_PROPERTY stage.object: invalid composite target");
                    return Some((PropValue::Int(0), crate::elm::form::INT));
                }
                if sub == crate::elm::objectlist::ELM_OBJECT_ALL_EVE {
                    return Self::try_property_object_all_eve(
                        ELM_STAGE_OBJECT,
                        obj_idx,
                        tail,
                        stage_idx,
                        host,
                    );
                }
                if Self::object_query_is_known_composite_sub(sub) {
                    let (proc_depth, proc_top) = self.observe_proc_stack_tuple();
                    return Self::try_property_object_known_command_only_composite(
                        ELM_STAGE_OBJECT,
                        obj_idx,
                        sub,
                        tail,
                        stage_idx,
                        self.options.disp_out_of_range_error,
                        proc_depth,
                        proc_top,
                        host,
                    );
                }
                return None;
            }
            if Self::object_query_is_str(sub) {
                return Some((
                    PropValue::Str(host.on_object_get_str(
                        ELM_STAGE_OBJECT,
                        obj_idx,
                        sub,
                        Some(stage_idx),
                    )),
                    crate::elm::form::STR,
                ));
            }
            if Self::object_query_is_int(sub) {
                let v = if Self::object_query_uses_query_api(sub) {
                    host.on_object_query(ELM_STAGE_OBJECT, obj_idx, sub, &[], Some(stage_idx))
                } else {
                    host.on_object_get(ELM_STAGE_OBJECT, obj_idx, sub, Some(stage_idx))
                };
                return Some((PropValue::Int(v), crate::elm::form::INT));
            }
        }
        return None;
    }

    if element[pos] == ELM_STAGE_OBJBTNGROUP {
        pos += 1;
        if pos < element.len() && element[pos] == ELM_OBJECTLIST_GET_SIZE {
            let n = host.on_group_list_get_size(stage_idx);
            return Some((PropValue::Int(n.max(0)), crate::elm::form::INT));
        }

        if pos + 3 <= element.len()
            && element[pos] == crate::elm::ELM_ARRAY
            && pos + 2 < element.len()
        {
            let group_idx = element[pos + 1];
            let sub = element[pos + 2];
            let group_size = host.on_group_list_get_size(stage_idx);
            if group_size >= 0 && (group_idx < 0 || group_idx >= group_size) {
                if self.options.disp_out_of_range_error {
                    host.on_error_fatal("範囲外のグループ番号が指定されました。(group_list)");
                }
                return Some((PropValue::Int(0), crate::elm::form::INT));
            }
            if Self::group_query_is_int(sub) {
                return Some((
                    PropValue::Int(host.on_group_get(stage_idx, group_idx, sub)),
                    crate::elm::form::INT,
                ));
            }
        }
        return None;
    }

    None
}
