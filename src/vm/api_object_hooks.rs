// Object Host callbacks (cmd_object.cpp alignment)

/// C++ cmd_object.cpp: object property set (int value).
fn on_object_property(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _property_id: i32,
    _value: i32,
    _stage_idx: Option<i32>,
) {
}
/// C++ cmd_object.cpp: object_list->get_sub(index, disp_out_of_range_error).
///
/// Return negative when the host cannot provide a concrete size yet.
fn on_object_list_get_size(&mut self, _list_id: i32, _stage_idx: Option<i32>) -> i32 {
    -1
}

/// C++ cmd_object.cpp: `object.child` list sizing (optional host capability).
///
/// Return negative when host cannot provide child-list size for this object.
fn on_object_child_list_get_size(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _stage_idx: Option<i32>,
) -> i32 {
    -1
}

/// C++ cmd_object.cpp: child object `is_use()` check under `object.child[idx]`.
fn on_object_child_is_use(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _child_index: i32,
    _stage_idx: Option<i32>,
) -> bool {
    true
}

/// C++ cmd_object.cpp: child object property get.
fn on_object_child_get(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _child_index: i32,
    _sub_id: i32,
    _stage_idx: Option<i32>,
) -> i32 {
    0
}

/// C++ cmd_object.cpp: child object string property get.
fn on_object_child_get_str(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _child_index: i32,
    _sub_id: i32,
    _stage_idx: Option<i32>,
) -> String {
    String::new()
}

/// C++ cmd_object.cpp: child object int-return query lane.
fn on_object_child_query(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _child_index: i32,
    _sub_id: i32,
    _args: &[Prop],
    _stage_idx: Option<i32>,
) -> i32 {
    0
}

/// C++ cmd_object.cpp: child list resize (`object.child.resize`).
fn on_object_child_list_resize(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _size: i32,
    _stage_idx: Option<i32>,
) {
}

/// C++ cmd_object.cpp: child object property set (int setter lane).
fn on_object_child_property(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _child_index: i32,
    _sub_id: i32,
    _value: i32,
    _stage_idx: Option<i32>,
) {
}

/// C++ cmd_object.cpp: child object action/lifecycle command.
fn on_object_child_action(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _child_index: i32,
    _sub_id: i32,
    _args: &[Prop],
    _stage_idx: Option<i32>,
) {
}

/// C++ cmd_object.cpp: `if (!p_obj->is_use()) {}` early-return branch.
fn on_object_is_use(&mut self, _list_id: i32, _obj_index: i32, _stage_idx: Option<i32>) -> bool {
    true
}

/// C++ cmd_object.cpp: object action/lifecycle command (sub_id identifies the command).
fn on_object_action(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _sub_id: i32,
    _args: &[Prop],
    _stage_idx: Option<i32>,
) {
}

/// C++ cmd_object.cpp: object property get.
fn on_object_get(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _sub_id: i32,
    _stage_idx: Option<i32>,
) -> i32 {
    0
}

/// C++ cmd_object.cpp: string property get (`get_file_name/get_string/get_element_name`).
fn on_object_get_str(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _sub_id: i32,
    _stage_idx: Option<i32>,
) -> String {
    String::new()
}

/// C++ cmd_object.cpp: int-returning action lane with argument payload (e.g. `__iapp_dummy`).
fn on_object_query(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _sub_id: i32,
    _args: &[Prop],
    _stage_idx: Option<i32>,
) -> i32 {
    0
}

/// C++ eng_frame.cpp / elm_counter.cpp: frame_action counter elapsed source.
///
/// Return per-object `(past_game_time, past_real_time)` when host can provide
/// a more precise time source; `None` falls back to global `on_frame_counter_elapsed`.
fn on_object_frame_action_counter_elapsed(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _sub_id: i32,
    _stage_idx: Option<i32>,
    _ch_index: Option<i32>,
) -> Option<(i32, i32)> {
    None
}

/// C++ cmd_object.cpp: frame_action/frame_action_ch composite property lane.
///
/// For element shapes like `stage.object[idx].frame_action.<...>` and
/// `stage.object[idx].frame_action_ch.<...>`, the VM can delegate typed
/// property access to host when internal flattening does not apply.
fn on_object_frame_action_property(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _sub_id: i32,
    _tail: &[i32],
    _stage_idx: Option<i32>,
) -> Option<(PropValue, i32)> {
    None
}

/// C++ cmd_object.cpp: frame_action/frame_action_ch composite assign lane.
///
/// Return `true` when host handled assignment for composite frame_action path.
fn on_object_frame_action_assign(
    &mut self,
    _list_id: i32,
    _obj_index: i32,
    _sub_id: i32,
    _tail: &[i32],
    _rhs: &Prop,
    _stage_idx: Option<i32>,
) -> bool {
    false
}
