use super::*;

include!("props_assign_rhs.rs");
include!("props_assign_call_user_prop.rs");

impl Vm {
    fn resolve_assign_int_rhs(&mut self, rhs: &Prop, host: &mut dyn Host) -> Result<i32> {
        match &rhs.value {
            PropValue::Int(v) => Ok(*v),
            PropValue::Element(el) => {
                let alias = self.resolve_command_element_alias(el);
                if let Some((v, form)) = self.try_property_internal(&alias, host)? {
                    return match v {
                        PropValue::Int(n) => Ok(n),
                        _ => bail!(
                            "CD_ASSIGN: source element form mismatch (expected INT, got form={})",
                            form
                        ),
                    };
                }
                if let Some((ret, form)) = host.on_property_typed(&alias) {
                    if form != crate::elm::form::INT {
                        bail!(
                            "CD_ASSIGN: source element form mismatch (expected INT, got form={})",
                            form
                        );
                    }
                    return Ok(ret.int);
                }
                bail!("CD_ASSIGN: unresolved INT source element");
            }
            _ => bail!("CD_ASSIGN: rhs type mismatch INT"),
        }
    }

    fn resolve_assign_str_rhs(&mut self, rhs: &Prop, host: &mut dyn Host) -> Result<String> {
        match &rhs.value {
            PropValue::Str(v) => Ok(v.clone()),
            PropValue::Element(el) => {
                let alias = self.resolve_command_element_alias(el);
                if let Some((v, form)) = self.try_property_internal(&alias, host)? {
                    return match v {
                        PropValue::Str(text) => Ok(text),
                        _ => bail!(
                            "CD_ASSIGN: source element form mismatch (expected STR, got form={})",
                            form
                        ),
                    };
                }
                if let Some((ret, form)) = host.on_property_typed(&alias) {
                    if form != crate::elm::form::STR {
                        bail!(
                            "CD_ASSIGN: source element form mismatch (expected STR, got form={})",
                            form
                        );
                    }
                    return Ok(ret.str_);
                }
                bail!("CD_ASSIGN: unresolved STR source element");
            }
            _ => bail!("CD_ASSIGN: rhs type mismatch STR"),
        }
    }

    fn assign_target_is_object_frame_action_sub(sub: i32) -> bool {
        use crate::elm::objectlist::*;
        matches!(sub, ELM_OBJECT_FRAME_ACTION | ELM_OBJECT_FRAME_ACTION_CH)
    }

    fn validate_object_frame_action_assign_tail(
        &self,
        sub: i32,
        tail: &[i32],
        host: &mut dyn Host,
    ) -> bool {
        use crate::elm::frameaction::{is_frameactionlist_get_size, is_frameactionlist_resize};
        use crate::elm::objectlist::ELM_OBJECT_FRAME_ACTION_CH;

        if tail.is_empty() {
            host.on_error_fatal("CD_ASSIGN stage.object.frame_action: missing tail element");
            return false;
        }

        let head = tail[0];
        if head == crate::elm::ELM_UP {
            host.on_error_fatal("CD_ASSIGN stage.object.frame_action: ELM_UP is not supported");
            return false;
        }

        if sub == ELM_OBJECT_FRAME_ACTION_CH {
            if head == crate::elm::ELM_ARRAY {
                if tail.len() < 2 {
                    host.on_error_fatal(
                        "CD_ASSIGN stage.object.frame_action_ch: missing array index",
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
                                "CD_ASSIGN stage.object.frame_action_ch[idx].up: missing method",
                            );
                            return false;
                        }
                    } else if is_frameactionlist_get_size(nested)
                        || is_frameactionlist_resize(nested)
                    {
                        host.on_error_fatal(
                            "CD_ASSIGN stage.object.frame_action_ch[idx]: unexpected list method",
                        );
                        return false;
                    }
                }
                return true;
            }
            if is_frameactionlist_get_size(head) || is_frameactionlist_resize(head) {
                if tail.len() > 1 {
                    host.on_error_fatal(
                        "CD_ASSIGN stage.object.frame_action_ch.list: unexpected nested tail",
                    );
                    return false;
                }
                return true;
            }
        }

        true
    }

    fn stage_assign_root(element: &[i32]) -> Option<(i32, usize)> {
        if element.is_empty() {
            return None;
        }
        if crate::elm::global::is_stage(element[0]) {
            if element.len() >= 3 && element[1] == crate::elm::ELM_ARRAY {
                return Some((element[2], 3));
            }
            return None;
        }
        if crate::elm::global::is_back(element[0]) {
            return Some((0, 1));
        }
        if crate::elm::global::is_front(element[0]) {
            return Some((1, 1));
        }
        if crate::elm::global::is_next(element[0]) {
            return Some((2, 1));
        }
        None
    }

    pub(super) fn try_assign_internal(
        &mut self,
        element: &[i32],
        al_id: i32,
        rhs: &Prop,
        host: &mut dyn Host,
    ) -> Result<bool> {
        // In the original engine, al_id==1 is the common "set" path for properties.
        if al_id != 1 {
            return Ok(false);
        }

        // ----- Stage object/group property writes (C++ cmd_stage/cmd_object alignment) -----
        if let Some((stage_idx, mut pos)) = Self::stage_assign_root(element) {
            use crate::elm::group::{ELM_GROUP_CANCEL_PRIORITY, ELM_GROUP_LAYER, ELM_GROUP_ORDER};
            use crate::elm::objectlist::{ELM_STAGE_OBJBTNGROUP, ELM_STAGE_OBJECT};

            let stage_size = host.on_stage_list_get_size();
            if stage_size >= 0 && (stage_idx < 0 || stage_idx >= stage_size) {
                if self.options.disp_out_of_range_error {
                    host.on_error_fatal("範囲外のステージ番号が指定されました。(stage_list)");
                }
                return Ok(true);
            }

            if pos < element.len() && element[pos] == ELM_STAGE_OBJBTNGROUP {
                pos += 1;
                if pos + 2 < element.len() && element[pos] == crate::elm::ELM_ARRAY {
                    let group_idx = element[pos + 1];
                    let sub = element[pos + 2];
                    let group_size = host.on_group_list_get_size(stage_idx);
                    if group_size >= 0 && (group_idx < 0 || group_idx >= group_size) {
                        if self.options.disp_out_of_range_error {
                            host.on_error_fatal(
                                "範囲外のグループ番号が指定されました。(group_list)",
                            );
                        }
                        return Ok(true);
                    }
                    if matches!(
                        sub,
                        ELM_GROUP_ORDER | ELM_GROUP_LAYER | ELM_GROUP_CANCEL_PRIORITY
                    ) {
                        let v = match &rhs.value {
                            PropValue::Int(x) => *x,
                            _ => bail!("CD_ASSIGN group: rhs type mismatch"),
                        };
                        host.on_group_property(stage_idx, group_idx, sub, v);
                        return Ok(true);
                    }
                }
            }

            if pos < element.len() && element[pos] == ELM_STAGE_OBJECT {
                pos += 1;
                if pos + 2 < element.len() && element[pos] == crate::elm::ELM_ARRAY {
                    let obj_idx = element[pos + 1];
                    let sub = element[pos + 2];
                    let obj_size = host.on_object_list_get_size(ELM_STAGE_OBJECT, Some(stage_idx));
                    if obj_size >= 0 && (obj_idx < 0 || obj_idx >= obj_size) {
                        if self.options.disp_out_of_range_error {
                            host.on_error_fatal(
                                "範囲外のオブジェクト番号が指定されました。(object_list)",
                            );
                        }
                        return Ok(true);
                    }
                    if !host.on_object_is_use(ELM_STAGE_OBJECT, obj_idx, Some(stage_idx)) {
                        return Ok(true);
                    }
                    if element.len() != pos + 3 {
                        let tail = &element[pos + 3..];
                        if Self::assign_target_is_object_frame_action_sub(sub) {
                            if !self.validate_object_frame_action_assign_tail(sub, tail, host) {
                                return Ok(true);
                            }
                            host.on_error_fatal(
                                "CD_ASSIGN stage.object.frame_action: composite target is command/property-only",
                            );
                            return Ok(true);
                        }
                        host.on_error_fatal("CD_ASSIGN stage.object: invalid composite target");
                        return Ok(true);
                    }
                    if !Self::object_property_is_settable_int(sub) {
                        if Self::object_query_is_int(sub) || Self::object_query_is_str(sub) {
                            host.on_error_fatal("CD_ASSIGN stage.object: target is read-only");
                            return Ok(true);
                        }
                        host.on_error_fatal("CD_ASSIGN stage.object: unknown target");
                        return Ok(true);
                    }
                    let v = self
                        .resolve_assign_int_rhs(rhs, host)
                        .map_err(|e| anyhow::anyhow!("CD_ASSIGN stage.object: {}", e))?;
                    host.on_object_property(ELM_STAGE_OBJECT, obj_idx, sub, v, Some(stage_idx));
                    return Ok(true);
                }
            }
        }

        // ----- Flag int-list writes: A[idx] = int -----
        if element.len() >= 3 && Self::is_intflag(element[0]) && element[1] == crate::elm::ELM_ARRAY
        {
            let idx = element[2] as isize;
            if idx < 0 {
                bail!("CD_ASSIGN intflag: negative index {}", idx);
            }
            let v = match &rhs.value {
                PropValue::Int(x) => *x,
                _ => bail!("CD_ASSIGN intflag: rhs type mismatch"),
            };
            if let Some(list) = self.get_intflag_mut(element[0]) {
                let idx = idx as usize;
                if idx >= list.len() {
                    bail!(
                        "CD_ASSIGN intflag: index {} out of range {}",
                        idx,
                        list.len()
                    );
                }
                list[idx] = v;
            }
            return Ok(true);
        }

        // ----- Flag str-list writes: S[idx] = str -----
        if element.len() >= 3 && Self::is_strflag(element[0]) && element[1] == crate::elm::ELM_ARRAY
        {
            let idx = element[2] as isize;
            if idx < 0 {
                bail!("CD_ASSIGN strflag: negative index {}", idx);
            }
            let v = match &rhs.value {
                PropValue::Str(s) => s.clone(),
                _ => bail!("CD_ASSIGN strflag: rhs type mismatch"),
            };
            if let Some(list) = self.get_strflag_mut(element[0]) {
                let idx = idx as usize;
                if idx >= list.len() {
                    bail!(
                        "CD_ASSIGN strflag: index {} out of range {}",
                        idx,
                        list.len()
                    );
                }
                list[idx] = v;
            }
            return Ok(true);
        }

        // ELM_GLOBAL_NAMAE[idx] = "str"
        if element.len() >= 3
            && crate::elm::global::is_namae_access(element[0])
            && element[1] == crate::elm::ELM_ARRAY
        {
            let idx = element[2] as isize;
            if idx < 0 {
                bail!("CD_ASSIGN namae: negative index {}", idx);
            }
            let v = match &rhs.value {
                PropValue::Str(s) => s.clone(),
                _ => bail!("CD_ASSIGN namae: rhs type mismatch"),
            };
            let list = &mut self.global_namae;
            let idx = idx as usize;
            if idx >= list.len() {
                bail!("CD_ASSIGN namae: index {} out of range {}", idx, list.len());
            }
            list[idx] = v;
            return Ok(true);
        }

        if self.try_assign_user_or_call_props(element, rhs, host)? {
            return Ok(true);
        }

        Ok(false)
    }
}
