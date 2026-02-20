/// World command routing.
///
/// C++ reference: cmd_world.cpp
///
/// Covers:
///   - World list: create_world / destroy_world / array access
///   - Per-world: camera eye/pint/up xyz get/set, calc_camera_eye, calc_camera_pint,
///     set_camera_eye/pint/up, camera_view_angle, mono, order, layer, wipe_copy/erase,
///     camera event dispatchers, set_camera_eve_xz_rotate
///
/// Property get commands push default 0 values.
/// Property set / calc commands delegate to Host callbacks.
use super::*;

#[allow(dead_code)]
impl Vm {
    // ---------------------------------------------------------------
    // World list: global.screen.world_list
    // ---------------------------------------------------------------

    /// Route world list commands matching C++ `tnm_command_proc_world_list`.
    /// `element` starts after the world_list root element.
    pub(super) fn try_command_world_list(
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
        use crate::elm::worldlist::*;

        if element[0] == crate::elm::ELM_ARRAY {
            // Indexed: world_list[idx].sub
            if element.len() >= 2 {
                let _idx = element[1];
                let rest = if element.len() > 2 { &element[2..] } else { &[] };
                return self.try_command_world(rest, arg_list_id, args, ret_form, host);
            }
            return true;
        }

        match element[0] {
            ELM_WORLDLIST_CREATE_WORLD => {
                // C++ creates a new world, pushes its index.
                host.on_world_create();
                self.stack.push_int(0);
                true
            }
            ELM_WORLDLIST_DESTROY_WORLD => {
                // C++ destroys the last world.
                host.on_world_destroy();
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(world_list)");
                true
            }
        }
    }

    // ---------------------------------------------------------------
    // Per-world: world_list[idx].<prop>
    // ---------------------------------------------------------------

    /// Route per-world commands matching C++ `tnm_command_proc_world`.
    fn try_command_world(
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
        use crate::elm::world::*;

        match sub {
            ELM_WORLD_INIT => {
                // C++ p_world->reinit()
                host.on_world_init();
                true
            }
            ELM_WORLD_GET_NO => {
                // C++ tnm_stack_push_int(p_world->get_world_no())
                self.stack.push_int(0);
                true
            }

            // --- Camera scalar properties (get/set) ---
            ELM_WORLD_CAMERA_EYE_X | ELM_WORLD_CAMERA_EYE_Y | ELM_WORLD_CAMERA_EYE_Z
            | ELM_WORLD_CAMERA_PINT_X | ELM_WORLD_CAMERA_PINT_Y | ELM_WORLD_CAMERA_PINT_Z
            | ELM_WORLD_CAMERA_UP_X | ELM_WORLD_CAMERA_UP_Y | ELM_WORLD_CAMERA_UP_Z
            | ELM_WORLD_CAMERA_VIEW_ANGLE
            | ELM_WORLD_MONO
            | ELM_WORLD_ORDER | ELM_WORLD_LAYER
            | ELM_WORLD_WIPE_COPY | ELM_WORLD_WIPE_ERASE => {
                if arg_list_id == 0 {
                    self.stack.push_int(0);
                } else {
                    host.on_world_property(sub, args.first().and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    }).unwrap_or(0));
                }
                true
            }

            // --- Camera set helpers (3 args) ---
            ELM_WORLD_SET_CAMERA_EYE | ELM_WORLD_SET_CAMERA_PINT | ELM_WORLD_SET_CAMERA_UP => {
                let x = args.first().and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                let y = args.get(1).and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                let z = args.get(2).and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                host.on_world_set_camera(sub, x, y, z);
                true
            }

            // --- Camera calc helpers (3 args: distance, rotate_h, rotate_v) ---
            ELM_WORLD_CALC_CAMERA_EYE | ELM_WORLD_CALC_CAMERA_PINT => {
                // C++ uses trig: sin/cos on distance/rotate params to compute camera pos.
                // For now, accept — the actual calculation is done host-side or in a future pass.
                let distance = args.first().and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                let rotate_h = args.get(1).and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                let rotate_v = args.get(2).and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                host.on_world_calc_camera(sub, distance, rotate_h, rotate_v);
                true
            }

            // --- set_camera_eve_xz_rotate (5 args) ---
            ELM_WORLD_SET_CAMERA_EVE_XZ_ROTATE => {
                // C++ 5 args: effectively animation params.
                // Accept as no-op.
                true
            }

            // --- Camera event dispatchers ---
            ELM_WORLD_CAMERA_EYE_X_EVE | ELM_WORLD_CAMERA_EYE_Y_EVE | ELM_WORLD_CAMERA_EYE_Z_EVE
            | ELM_WORLD_CAMERA_PINT_X_EVE | ELM_WORLD_CAMERA_PINT_Y_EVE | ELM_WORLD_CAMERA_PINT_Z_EVE
            | ELM_WORLD_CAMERA_UP_X_EVE | ELM_WORLD_CAMERA_UP_Y_EVE | ELM_WORLD_CAMERA_UP_Z_EVE => {
                // C++ tnm_command_proc_int_event — accept for now.
                true
            }

            _ => {
                host.on_error("無効なコマンドが指定されました。(world)");
                true
            }
        }
    }
}
