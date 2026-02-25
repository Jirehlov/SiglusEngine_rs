use super::command_input::KeyWaitTickResult;
use super::opcode::cd;
use super::*;
use std::collections::BTreeMap;
impl Vm {
    pub(super) fn resolve_command_element_alias(&self, element: &[i32]) -> Vec<i32> {
        let mut cur = element.to_vec();
        for _ in 0..8 {
            if cur.is_empty() {
                break;
            }
            let mut replaced = None;
            if crate::elm::owner::is_user_prop(cur[0]) {
                let up_idx = elm_code(cur[0]) as usize;
                if let Some(PropValue::Element(base)) = self.user_prop_values.get(up_idx) {
                    let mut next = base.clone();
                    next.extend_from_slice(&cur[1..]);
                    replaced = Some(next);
                }
            } else if cur.len() >= 2
                && crate::elm::call::is_cur_call(cur[0])
                && crate::elm::owner::is_call_prop(cur[1])
            {
                let cp_code = elm_code(cur[1]) as usize;
                if let Some(frame) = self.frames.last() {
                    if let Some(slot) = Self::resolve_call_prop_slot(&frame.call, cp_code) {
                        if let Some(cp) = frame.call.user_props.get(slot) {
                            if let PropValue::Element(base) = &cp.value {
                                let mut next = base.clone();
                                next.extend_from_slice(&cur[2..]);
                                replaced = Some(next);
                            }
                        }
                    }
                }
            }
            if let Some(next) = replaced {
                cur = next;
            } else {
                break;
            }
        }
        cur
    }
    pub fn new(scene: String, dat: Arc<SceneDat>) -> Self {
        let lexer = SceneLexer::new(dat.clone());
        let (user_prop_forms, user_prop_values) = make_user_props(&dat);
        let options = VmOptions::default();
        Self {
            scene: scene.clone(),
            lexer,
            stack: IfcStack::default(),
            frames: vec![Frame {
                return_pc: 0,
                return_scene: scene,
                return_dat: dat,
                return_line_no: 0,
                expect_ret_form: crate::elm::form::VOID,
                call_type: VmCallType::None,
                excall_flag: false,
                frame_action_flag: false,
                arg_cnt: 0,
                call: CallContext::new(DEFAULT_CALL_FLAG_CNT),
            }],
            max_steps: 1_000_000_000,
            steps: 0,
            halted: false,
            scene_title: String::new(),
            options: options.clone(),
            stats: VmStats::default(),
            user_prop_forms,
            user_prop_values,
            frame_action: FrameAction::default(),
            frame_action_ch: Vec::new(),
            excall_frame_action: FrameAction::default(),
            excall_frame_action_ch: Vec::new(),
            key_wait_proc: KeyWaitProc::default(),
            group_wait_proc: GroupWaitProc::default(),
            excall_allocated: [false; 2],
            flags_a: vec![0i32; FLAG_LIST_SIZE],
            flags_b: vec![0i32; FLAG_LIST_SIZE],
            flags_c: vec![0i32; FLAG_LIST_SIZE],
            flags_d: vec![0i32; FLAG_LIST_SIZE],
            flags_e: vec![0i32; FLAG_LIST_SIZE],
            flags_f: vec![0i32; FLAG_LIST_SIZE],
            excall_flags_f: [vec![0i32; FLAG_LIST_SIZE], vec![0i32; FLAG_LIST_SIZE]],
            flags_x: vec![0i32; FLAG_LIST_SIZE],
            flags_g: vec![0i32; FLAG_LIST_SIZE],
            flags_z: vec![0i32; FLAG_LIST_SIZE],
            flags_s: vec![String::new(); FLAG_LIST_SIZE],
            flags_m: vec![String::new(); FLAG_LIST_SIZE],
            global_namae: vec![String::new(); FLAG_LIST_SIZE],
            local_namae: vec![String::new(); FLAG_LIST_SIZE],
            save_point_set: false,
            sel_point_set: false,
            save_point_snapshot: None,
            sel_point_snapshot: None,
            sel_point_stock: None,
            cur_mwnd_element: vec![
                crate::elm::global::ELM_GLOBAL_FRONT,
                crate::elm::objectlist::ELM_STAGE_MWND,
                crate::elm::ELM_ARRAY,
                -1,
            ],
            cur_sel_mwnd_element: vec![
                crate::elm::global::ELM_GLOBAL_FRONT,
                crate::elm::objectlist::ELM_STAGE_MWND,
                crate::elm::ELM_ARRAY,
                -1,
            ],
            last_sel_msg: String::new(),
            hide_mwnd_onoff_flag: 0,
            hide_mwnd_enable_flag: 1,
            hide_mwnd_exist_flag: 1,
            read_skip_onoff_flag: 0,
            read_skip_enable_flag: 1,
            read_skip_exist_flag: 1,
            auto_mode_onoff_flag: 0,
            auto_mode_enable_flag: 1,
            auto_mode_exist_flag: 1,
            msg_back_enable_flag: 1,
            msg_back_exist_flag: 1,
            msg_back_open_flag: 0,
            msg_back_has_message: 0,
            msg_back_disable_flag: 0,
            msg_back_off_flag: 0,
            msg_back_disp_off_flag: 0,
            msg_back_proc_off_flag: 0,
            return_to_sel_enable_flag: 1,
            return_to_sel_exist_flag: 1,
            return_to_menu_enable_flag: 1,
            return_to_menu_exist_flag: 1,
            save_enable_flag: 1,
            save_exist_flag: 1,
            load_enable_flag: 1,
            load_exist_flag: 1,
            end_game_enable_flag: 1,
            end_game_exist_flag: 1,
            game_end_flag: 0,
            game_end_no_warning_flag: 0,
            game_end_save_done_flag: 0,
            syscom_cfg: VmSyscomConfigState::default(),
            no_wipe_anime_onoff_flag: if options.no_wipe_anime { 1 } else { 0 },
            skip_wipe_anime_onoff_flag: if options.skip_wipe_anime { 1 } else { 0 },
            script_skip_unread_message_flag: 0,
            script_stage_time_stop_flag: 0,
            system_wipe_flag: 0,
            do_frame_action_flag: 0,
            do_load_after_call_flag: 0,
            game_timer_move_flag: 1,
            syscom_menu_disable_flag: 0,
            system_extra_int_values: options.system_extra_int_values.clone(),
            system_extra_str_values: options.system_extra_str_values.clone(),
            return_scene_once: None,
            wipe_end_at: None,
            last_pc: 0,
            last_line_no: 0,
            last_scene: String::new(),
            proc_stack: vec![VmProcType::Script],
            script_dont_set_save_point: false,
            script_skip_disable: false,
            script_ctrl_disable: false,
            script_not_stop_skip_by_click: false,
            script_not_skip_msg_by_click: false,
            script_auto_mode_flag: false,
            script_auto_mode_moji_wait: -1,
            script_auto_mode_min_wait: -1,
            script_auto_mode_moji_cnt: 0,
            script_mouse_cursor_hide_onoff: -1,
            script_mouse_cursor_hide_time: -1,
            script_msg_speed: -1,
            script_msg_nowait: false,
            script_async_msg_mode: false,
            script_async_msg_mode_once: false,
            script_hide_mwnd_disable: false,
            script_cursor_disp_off: false,
            script_cursor_move_by_key_disable: false,
            script_key_disable: [false; 256],
            script_mwnd_anime_off_flag: false,
            script_mwnd_anime_on_flag: false,
            script_mwnd_disp_off_flag: false,
            script_koe_dont_stop_on_flag: false,
            script_koe_dont_stop_off_flag: false,
            script_shortcut_disable: false,
            script_quake_stop_flag: false,
            script_emote_mouth_stop_flag: false,
            script_bgmfade_flag: false,
            script_vsync_wait_off_flag: false,
            script_skip_trigger: false,
            script_ignore_r_flag: false,
            script_cursor_no: 0,
            script_time_stop_flag: false,
            script_counter_time_stop_flag: false,
            script_frame_action_time_stop_flag: false,
            script_font_name: String::new(),
            script_font_bold: -1,
            script_font_shadow: -1,
            script_allow_joypad_mode_onoff: -1,
            excall_script_font_name: [String::new(), String::new()],
            excall_script_font_bold: [-1, -1],
            excall_script_font_shadow: [-1, -1],
            counter_list_size: options.preloaded_counter_count.max(1),
            excall_counter_list_size: [
                options.preloaded_counter_count.max(1),
                options.preloaded_counter_count.max(1),
            ],
            counter_values: Vec::new(),
            counter_active: Vec::new(),
            database_tables: Vec::new(),
            database_row_calls: Vec::new(),
            database_col_calls: Vec::new(),
            database_col_types: Vec::new(),
            cg_table_off_flag: false,
            cg_flags: Vec::new(),
            cg_name_to_flag: BTreeMap::new(),
            cg_group_codes: Vec::new(),
            cg_code_exist_cnt: Vec::new(),
            bgm_name_listened: BTreeMap::new(),
            g00buf_loaded: Vec::new(),
            mask_slots: Vec::new(),
            object_gan_loaded_path: BTreeMap::new(),
            object_gan_started_set: BTreeMap::new(),
            local_save_slots: BTreeMap::new(),
            quick_save_slots: BTreeMap::new(),
            inner_save_slots: BTreeMap::new(),
            end_save_slots: BTreeMap::new(),
        }
    }
    pub(super) fn command_needs_read_flag_tail(element: &[i32]) -> bool {
        crate::elm::global::command_needs_read_flag_tail(element)
    }
    pub(super) fn command_text_arg(arg_list_id: i32, args: &[Prop]) -> Option<String> {
        let arg = args.get(0)?;
        match arg_list_id {
            0 => match &arg.value {
                PropValue::Int(v) => Some(v.to_string()),
                PropValue::Str(s) => Some(s.clone()),
                _ => None,
            },
            1 => match &arg.value {
                PropValue::Str(s) => Some(s.clone()),
                PropValue::Int(v) => Some(v.to_string()),
                _ => None,
            },
            _ => None,
        }
    }
    pub(super) fn dispatch_message_command(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        read_flag_no: Option<i32>,
        host: &mut dyn Host,
    ) {
        if element.len() != 1 {
            return;
        }
        match element[0] {
            x if crate::elm::global::is_print(x) => {
                if let Some(text) = Self::command_text_arg(arg_list_id, args) {
                    if self.msg_back_off_flag == 0 {
                        self.msg_back_has_message = 1;
                    }
                    host.on_text(&text, read_flag_no.unwrap_or(0));
                }
            }
            x if crate::elm::global::is_set_namae_cmd(x) => {
                if let Some(name) = args.get(0).and_then(|arg| match &arg.value {
                    PropValue::Str(s) => Some(s.clone()),
                    PropValue::Int(v) => Some(v.to_string()),
                    _ => None,
                }) {
                    host.on_name(&name);
                }
            }
            _ => {}
        }
    }
    pub(super) fn is_selection_command(elm: i32) -> bool {
        crate::elm::global::is_selection_command(elm)
    }
    pub(super) fn capture_last_selection_message(
        &mut self,
        element: &[i32],
        args: &[Prop],
        ret: &HostReturn,
    ) {
        if element.len() != 1 || !Self::is_selection_command(element[0]) {
            return;
        }
        let opts = extract_selection_options(args);
        let idx = ret.int.max(0) as usize;
        if let Some(msg) = opts.get(idx) {
            self.last_sel_msg = msg.clone();
        }
    }

    fn flick_angle_matches(angle_type: i32, angle: f64) -> bool {
        let pi = std::f64::consts::PI;
        match angle_type {
            1 => angle < -pi / 2.0 || pi / 2.0 <= angle,
            2 => -pi / 2.0 <= angle && angle < pi / 2.0,
            3 => angle >= 0.0,
            4 => angle < 0.0,
            5 => angle < -pi * 3.0 / 4.0 || pi * 3.0 / 4.0 <= angle,
            6 => pi / 4.0 <= angle && angle < pi * 3.0 / 4.0,
            7 => -pi / 4.0 <= angle && angle < pi / 4.0,
            8 => -pi * 3.0 / 4.0 <= angle && angle < -pi / 4.0,
            _ => false,
        }
    }

    fn run_flick_scene_proc(
        &mut self,
        host: &mut dyn Host,
        provider: &mut dyn SceneProvider,
    ) -> Result<()> {
        if !self.is_flick_scene_allowed(host) {
            return Ok(());
        }
        if self.options.flick_scene_routes.is_empty() {
            return Ok(());
        }
        let flick = host.on_input_get_left_flick_state();
        if !flick.has_flick_stock {
            return Ok(());
        }

        let routes = self.options.flick_scene_routes.clone();
        for route in routes {
            if !Self::flick_angle_matches(route.angle_type, flick.angle_radian) {
                continue;
            }
            if !host.on_input_consume_left_flick_stock() {
                break;
            }
            self.read_skip_onoff_flag = 0;
            self.proc_farcall_like(
                &route.scene,
                route.z_no,
                crate::elm::form::VOID,
                &[],
                false,
                provider,
            )?;
            self.run_inner(host, provider)?;
            break;
        }

        Ok(())
    }

    pub(super) fn run_inner(
        &mut self,
        host: &mut dyn Host,
        provider: &mut dyn SceneProvider,
    ) -> Result<()> {
        while !self.lexer.is_eof() && !self.halted {
            self.run_flick_scene_proc(host, provider)?;
            if self.run_key_wait_proc(host) == KeyWaitTickResult::Pending {
                self.frame_action_counter_tick_all(host);
                host.on_wait_frame();
                continue;
            }
            if self.run_group_wait_proc(host) {
                self.frame_action_counter_tick_all(host);
                host.on_wait_frame();
                continue;
            }
            // eng_frame.cpp alignment: frame_action counters advance on frame tick,
            // not only when scripts explicitly call counter.wait/check_value.
            self.frame_action_counter_tick_all(host);
            self.last_pc = self.lexer.pc;
            self.last_line_no = self.lexer.cur_line_no;
            self.last_scene = self.scene.clone();
            host.on_location(
                &self.scene_title,
                &self.scene,
                self.lexer.cur_line_no,
                self.lexer.pc,
            );
            if self.steps >= self.max_steps {
                self.halted = true;
                break;
            }
            if host.should_interrupt() {
                self.halted = true;
                break;
            }
            self.steps += 1;
            let code = self.vm_read_u8(host, "VM", "opcode")?;
            self.stats.opcode_hits[code as usize] += 1;
            if code == cd::SEL_BLOCK_START || code == cd::SEL_BLOCK_END {
                // Selection blocks are UI-driven; ignore in the headless VM for now.
                continue;
            }
            if code == cd::PUSH {
                let form_code = self.vm_read_i32(host, "CD_PUSH", "form")?;
                if form_code == crate::elm::form::INT {
                    let v = self.vm_read_i32(host, "CD_PUSH", "int payload")?;
                    self.stack.push_int(v);
                } else if form_code == crate::elm::form::STR {
                    let s = self.vm_read_str(host, "CD_PUSH", "str payload")?;
                    self.stack.push_str(s);
                }
                continue;
            }
            if code == cd::PROPERTY {
                let element_raw = self.stack.pop_element().map_err(|e| {
                    self.report_vm_fatal_with_context(
                        host,
                        &format!("CD_PROPERTY: pop target element failed: {}", e),
                    );
                    e
                })?;
                let element = self.resolve_command_element_alias(&element_raw);
                if let Some((v, form)) =
                    self.try_property_internal(&element, host).map_err(|e| {
                        self.report_vm_fatal_with_context(
                            host,
                            &format!(
                                "CD_PROPERTY internal route failed: target={:?} err={}",
                                element, e
                            ),
                        );
                        e
                    })?
                {
                    self.push_vm_value(form, v);
                } else if Vm::is_known_internal_property_target(&element) {
                    host.on_error_fatal(
                        "CD_PROPERTY: known internal target was not handled by VM route",
                    );
                    self.stack.push_int(0);
                } else if let Some((ret, ret_form)) = host.on_property_typed(&element) {
                    self.push_host_ret(&ret, ret_form);
                } else {
                    let ret = host.on_property(&element);
                    // Still conservative for unknown properties.
                    self.push_host_ret(&ret, crate::elm::form::INT);
                }
                continue;
            }
            if code == cd::OPERATE_2 {
                let form_l = self.vm_read_i32(host, "CD_OPERATE_2", "lhs form")?;
                let form_r = self.vm_read_i32(host, "CD_OPERATE_2", "rhs form")?;
                let opr = self.vm_read_u8(host, "CD_OPERATE_2", "operator")?;
                self.calculate_2(form_l, form_r, opr, host)?;
                continue;
            }
            if code == cd::ELM_POINT {
                self.stack.elm_point();
                continue;
            }
            if code == cd::ASSIGN {
                let left_form = self.vm_read_i32(host, "CD_ASSIGN", "lhs form")?;
                let right_form = self.vm_read_i32(host, "CD_ASSIGN", "rhs form")?;
                let al_id = self.vm_read_i32(host, "CD_ASSIGN", "arg_list_id")?;
                // fixed-form rhs
                let rhs = self.pop_single_arg(right_form).map_err(|e| {
                    self.report_vm_fatal_with_context(
                        host,
                        &format!("CD_ASSIGN: pop rhs failed: {}", e),
                    );
                    e
                })?;
                let element_raw = self.stack.pop_element().map_err(|e| {
                    self.report_vm_fatal_with_context(
                        host,
                        &format!("CD_ASSIGN: pop target element failed: {}", e),
                    );
                    e
                })?;
                let element = self.resolve_command_element_alias(&element_raw);
                if element.is_empty() {
                    self.report_vm_fatal_with_context(
                        host,
                        "CD_ASSIGN: empty target element after alias resolution",
                    );
                    bail!("CD_ASSIGN: empty target element after alias resolution");
                }
                match self.try_assign_internal(&element, al_id, &rhs, host) {
                    Ok(true) => {}
                    Ok(false) => {
                        if Vm::is_known_internal_property_target(&element) {
                            host.on_error_fatal(
                                "CD_ASSIGN: known internal target was not handled by VM route",
                            );
                        } else {
                            host.on_assign(&element, al_id, &rhs)
                        }
                    }
                    Err(e) => {
                        self.report_vm_fatal_with_context(
                            host,
                            &format!(
                                "CD_ASSIGN writeback failed: target={:?} al_id={} rhs_form={} err={}",
                                element, al_id, rhs.form, e
                            ),
                        );
                        return Err(e);
                    }
                }
                // left_form currently unused (host decides)
                let _ = left_form;
                continue;
            }
            if code == cd::NL {
                let old_line_no = self.lexer.cur_line_no;
                let line_no = self.vm_read_i32(host, "CD_NL", "line no")?;
                self.lexer.cur_line_no = line_no;
                if host.is_breaking()
                    && host.break_step_flag()
                    && self.lexer.cur_line_no != old_line_no
                {
                    host.on_break_step_line_advanced();
                    return Ok(());
                }
                continue;
            }

            if code == cd::COMMAND {
                let arg_list_id = self.vm_read_i32(host, "CD_COMMAND", "arg_list_id")?;

                let mut args = self.pop_arg_list(host)?;

                let element_raw = self.stack.pop_element().map_err(|e| {
                    self.report_vm_fatal_with_context(
                        host,
                        &format!("CD_COMMAND: pop target element failed: {}", e),
                    );
                    e
                })?;
                let element = self.resolve_command_element_alias(&element_raw);

                let named_arg_cnt = self.vm_read_i32(host, "CD_COMMAND", "named_arg_cnt")?;
                if named_arg_cnt > 0 {
                    for a in 0..named_arg_cnt as usize {
                        let id = self.vm_read_i32(host, "CD_COMMAND", "named arg id")?;
                        if let Some(idx) = args.len().checked_sub(a + 1) {
                            args[idx].id = id;
                        }
                    }
                }
                let ret_form = self.vm_read_i32(host, "CD_COMMAND", "ret form")?;
                let mut read_flag_no = None;
                if Self::command_needs_read_flag_tail(&element) {
                    read_flag_no = Some(self.vm_read_i32(host, "CD_COMMAND", "read flag no")?);
                }
                self.dispatch_message_command(&element, arg_list_id, &args, read_flag_no, host);

                if self.try_command(
                    &element,
                    arg_list_id,
                    &args,
                    named_arg_cnt,
                    ret_form,
                    provider,
                    host,
                )? {
                    continue;
                }

                let ret = host.on_command(&element, arg_list_id, &args, named_arg_cnt, ret_form);
                self.capture_last_selection_message(&element, &args, &ret);
                self.push_host_ret(&ret, ret_form);

                continue;
            }

            match code {
                x if x == cd::GOTO => {
                    let label_no = self.vm_read_i32(host, "CD_GOTO", "label")?;
                    self.proc_jump_to_label(label_no, "CD_GOTO", host)?;
                }
                x if x == cd::GOTO_FALSE => {
                    let cond = self.stack.pop_int()?;
                    let label_no = self.vm_read_i32(host, "CD_GOTO_FALSE", "label")?;
                    if cond == 0 {
                        self.proc_jump_to_label(label_no, "CD_GOTO_FALSE", host)?;
                    }
                }
                x if x == cd::GOTO_TRUE => {
                    let cond = self.stack.pop_int()?;
                    let label_no = self.vm_read_i32(host, "CD_GOTO_TRUE", "label")?;
                    if cond != 0 {
                        self.proc_jump_to_label(label_no, "CD_GOTO_TRUE", host)?;
                    }
                }
                x if x == cd::GOSUB => {
                    if let Err(e) = self.proc_gosub(crate::elm::form::INT, host) {
                        return Err(e);
                    }
                }
                x if x == cd::GOSUBSTR => {
                    if let Err(e) = self.proc_gosub(crate::elm::form::STR, host) {
                        return Err(e);
                    }
                }
                x if x == cd::RETURN => {
                    let should_continue = match self.proc_return(host) {
                        Ok(v) => v,
                        Err(e) => {
                            return Err(e);
                        }
                    };
                    if !should_continue {
                        return Ok(());
                    }
                }
                x if x == cd::POP => {
                    let form_code = self.vm_read_i32(host, "CD_POP", "form")?;
                    match form_code {
                        f if f == crate::elm::form::INT => {
                            let _ = self.stack.pop_int()?;
                        }
                        f if f == crate::elm::form::STR => {
                            let _ = self.stack.pop_str()?;
                        }
                        _ => {}
                    }
                }
                x if x == cd::COPY => {
                    let form_code = self.vm_read_i32(host, "CD_COPY", "form")?;
                    match form_code {
                        f if f == crate::elm::form::INT => {
                            let v = self.stack.back_int()?;
                            self.stack.push_int(v);
                        }
                        f if f == crate::elm::form::STR => {
                            let s = self.stack.back_str()?;
                            self.stack.push_str(s);
                        }
                        _ => {}
                    }
                }
                x if x == cd::COPY_ELM => {
                    self.stack.copy_element()?;
                }
                x if x == cd::DEC_PROP => {
                    let form_code = self.vm_read_i32(host, "CD_DEC_PROP", "form")?;
                    let prop_id = self.vm_read_i32(host, "CD_DEC_PROP", "prop id")?;
                    let mut size = 0;
                    if form_code == crate::elm::form::INTLIST
                        || form_code == crate::elm::form::STRLIST
                    {
                        size = self.stack.pop_int()?;
                    }
                    self.proc_dec_prop(form_code, prop_id, size);
                }
                x if x == cd::ARG => {
                    self.proc_arg(host)?;
                }
                x if x == cd::OPERATE_1 => {
                    let form = self.vm_read_i32(host, "CD_OPERATE_1", "form")?;
                    let opr = self.vm_read_u8(host, "CD_OPERATE_1", "operator")?;
                    self.calculate_1(form, opr, host)?;
                }
                x if x == cd::NAME => {
                    let s = self.stack.pop_str()?;
                    host.on_name(&s);
                }
                x if x == cd::TEXT => {
                    let read_flag_no = self.vm_read_i32(host, "CD_TEXT", "read flag no")?;
                    let msg = self.stack.pop_str()?;
                    host.on_text(&msg, read_flag_no);
                }
                x if x == cd::NONE => {
                    host.on_script_fatal("スクリプトの解析に失敗しました。");
                    self.halted = true;
                    self.proc_stack.clear();
                    self.proc_stack.push(VmProcType::None);
                    return Ok(());
                }
                x if x == cd::EOF => {
                    host.on_script_fatal("ファイルの終端に到達しました。");
                    self.halted = true;
                    self.proc_stack.clear();
                    self.proc_stack.push(VmProcType::None);
                    return Ok(());
                }
                other => {
                    host.on_error(&format!(
                        "unhandled opcode {} at pc={}",
                        other, self.lexer.pc
                    ));
                    bail!("vm: unhandled opcode {}", other);
                }
            }
        }
        Ok(())
    }
}
