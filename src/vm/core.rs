use super::opcode::cd;
use super::*;
use std::collections::BTreeMap;
impl Vm {
    pub(super) fn resolve_command_element_alias(&self, element: &[i32]) -> Vec<i32> {
        let mut cur = element.to_vec();
        for _ in 0..4 {
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
                let cp_idx = elm_code(cur[1]) as usize;
                if let Some(frame) = self.frames.last() {
                    if let Some(cp) = frame.call.user_props.get(cp_idx) {
                        if let PropValue::Element(base) = &cp.value {
                            let mut next = base.clone();
                            next.extend_from_slice(&cur[2..]);
                            replaced = Some(next);
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

            flags_a: vec![0i32; FLAG_LIST_SIZE],
            flags_b: vec![0i32; FLAG_LIST_SIZE],
            flags_c: vec![0i32; FLAG_LIST_SIZE],
            flags_d: vec![0i32; FLAG_LIST_SIZE],
            flags_e: vec![0i32; FLAG_LIST_SIZE],
            flags_f: vec![0i32; FLAG_LIST_SIZE],
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
            no_wipe_anime_onoff_flag: if options.no_wipe_anime { 1 } else { 0 },
            skip_wipe_anime_onoff_flag: if options.skip_wipe_anime { 1 } else { 0 },
            script_skip_unread_message_flag: 0,
            script_stage_time_stop_flag: 0,
            system_extra_int_values: options.system_extra_int_values.clone(),
            system_extra_str_values: options.system_extra_str_values.clone(),
            return_scene_once: None,
            wipe_end_at: None,
            last_pc: 0,
            last_line_no: 0,
            last_scene: String::new(),

            local_save_slots: BTreeMap::new(),
            quick_save_slots: BTreeMap::new(),
            inner_save_slots: BTreeMap::new(),
        }
    }

    pub(super) fn snapshot_local_state(&self) -> VmLocalState {
        VmLocalState {
            scene: self.scene.clone(),
            lexer: self.lexer.clone(),
            stack: self.stack.clone(),
            frames: self.frames.clone(),
            scene_title: self.scene_title.clone(),
            user_prop_forms: self.user_prop_forms.clone(),
            user_prop_values: self.user_prop_values.clone(),
            frame_action: self.frame_action.clone(),
            frame_action_ch: self.frame_action_ch.clone(),
            flags_a: self.flags_a.clone(),
            flags_b: self.flags_b.clone(),
            flags_c: self.flags_c.clone(),
            flags_d: self.flags_d.clone(),
            flags_e: self.flags_e.clone(),
            flags_f: self.flags_f.clone(),
            flags_x: self.flags_x.clone(),
            flags_g: self.flags_g.clone(),
            flags_z: self.flags_z.clone(),
            flags_s: self.flags_s.clone(),
            flags_m: self.flags_m.clone(),
            global_namae: self.global_namae.clone(),
            local_namae: self.local_namae.clone(),
            save_point_set: self.save_point_set,
            sel_point_set: self.sel_point_set,
            save_point_snapshot: self.save_point_snapshot.clone(),
            sel_point_snapshot: self.sel_point_snapshot.clone(),
            sel_point_stock: self.sel_point_stock.clone(),
            cur_mwnd_element: self.cur_mwnd_element.clone(),
            cur_sel_mwnd_element: self.cur_sel_mwnd_element.clone(),
            last_sel_msg: self.last_sel_msg.clone(),
            hide_mwnd_onoff_flag: self.hide_mwnd_onoff_flag,
            hide_mwnd_enable_flag: self.hide_mwnd_enable_flag,
            hide_mwnd_exist_flag: self.hide_mwnd_exist_flag,
            read_skip_onoff_flag: self.read_skip_onoff_flag,
            read_skip_enable_flag: self.read_skip_enable_flag,
            read_skip_exist_flag: self.read_skip_exist_flag,
            auto_mode_onoff_flag: self.auto_mode_onoff_flag,
            auto_mode_enable_flag: self.auto_mode_enable_flag,
            auto_mode_exist_flag: self.auto_mode_exist_flag,
            msg_back_enable_flag: self.msg_back_enable_flag,
            msg_back_exist_flag: self.msg_back_exist_flag,
            msg_back_open_flag: self.msg_back_open_flag,
            msg_back_has_message: self.msg_back_has_message,
            msg_back_disable_flag: self.msg_back_disable_flag,
            msg_back_off_flag: self.msg_back_off_flag,
            msg_back_disp_off_flag: self.msg_back_disp_off_flag,
            msg_back_proc_off_flag: self.msg_back_proc_off_flag,
            return_to_sel_enable_flag: self.return_to_sel_enable_flag,
            return_to_sel_exist_flag: self.return_to_sel_exist_flag,
            return_to_menu_enable_flag: self.return_to_menu_enable_flag,
            return_to_menu_exist_flag: self.return_to_menu_exist_flag,
            save_enable_flag: self.save_enable_flag,
            save_exist_flag: self.save_exist_flag,
            load_enable_flag: self.load_enable_flag,
            load_exist_flag: self.load_exist_flag,
            no_wipe_anime_onoff_flag: self.no_wipe_anime_onoff_flag,
            skip_wipe_anime_onoff_flag: self.skip_wipe_anime_onoff_flag,
            script_skip_unread_message_flag: self.script_skip_unread_message_flag,
            script_stage_time_stop_flag: self.script_stage_time_stop_flag,
            last_pc: self.last_pc,
            last_line_no: self.last_line_no,
            last_scene: self.last_scene.clone(),
        }
    }

    pub(super) fn apply_local_state(&mut self, st: &VmLocalState) {
        self.scene = st.scene.clone();
        self.lexer = st.lexer.clone();
        self.stack = st.stack.clone();
        self.frames = st.frames.clone();
        self.scene_title = st.scene_title.clone();
        self.user_prop_forms = st.user_prop_forms.clone();
        self.user_prop_values = st.user_prop_values.clone();
        self.frame_action = st.frame_action.clone();
        self.frame_action_ch = st.frame_action_ch.clone();
        self.flags_a = st.flags_a.clone();
        self.flags_b = st.flags_b.clone();
        self.flags_c = st.flags_c.clone();
        self.flags_d = st.flags_d.clone();
        self.flags_e = st.flags_e.clone();
        self.flags_f = st.flags_f.clone();
        self.flags_x = st.flags_x.clone();
        self.flags_g = st.flags_g.clone();
        self.flags_z = st.flags_z.clone();
        self.flags_s = st.flags_s.clone();
        self.flags_m = st.flags_m.clone();
        self.global_namae = st.global_namae.clone();
        self.local_namae = st.local_namae.clone();
        self.save_point_set = st.save_point_set;
        self.sel_point_set = st.sel_point_set;
        self.cur_mwnd_element = st.cur_mwnd_element.clone();
        self.cur_sel_mwnd_element = st.cur_sel_mwnd_element.clone();
        self.last_sel_msg = st.last_sel_msg.clone();
        self.hide_mwnd_onoff_flag = st.hide_mwnd_onoff_flag;
        self.hide_mwnd_enable_flag = st.hide_mwnd_enable_flag;
        self.hide_mwnd_exist_flag = st.hide_mwnd_exist_flag;
        self.read_skip_onoff_flag = st.read_skip_onoff_flag;
        self.read_skip_enable_flag = st.read_skip_enable_flag;
        self.read_skip_exist_flag = st.read_skip_exist_flag;
        self.auto_mode_onoff_flag = st.auto_mode_onoff_flag;
        self.auto_mode_enable_flag = st.auto_mode_enable_flag;
        self.auto_mode_exist_flag = st.auto_mode_exist_flag;
        self.msg_back_enable_flag = st.msg_back_enable_flag;
        self.msg_back_exist_flag = st.msg_back_exist_flag;
        self.msg_back_open_flag = st.msg_back_open_flag;
        self.msg_back_has_message = st.msg_back_has_message;
        self.msg_back_disable_flag = st.msg_back_disable_flag;
        self.msg_back_off_flag = st.msg_back_off_flag;
        self.msg_back_disp_off_flag = st.msg_back_disp_off_flag;
        self.msg_back_proc_off_flag = st.msg_back_proc_off_flag;
        self.return_to_sel_enable_flag = st.return_to_sel_enable_flag;
        self.return_to_sel_exist_flag = st.return_to_sel_exist_flag;
        self.return_to_menu_enable_flag = st.return_to_menu_enable_flag;
        self.return_to_menu_exist_flag = st.return_to_menu_exist_flag;
        self.save_enable_flag = st.save_enable_flag;
        self.save_exist_flag = st.save_exist_flag;
        self.load_enable_flag = st.load_enable_flag;
        self.load_exist_flag = st.load_exist_flag;
        self.no_wipe_anime_onoff_flag = st.no_wipe_anime_onoff_flag;
        self.skip_wipe_anime_onoff_flag = st.skip_wipe_anime_onoff_flag;
        self.options.no_wipe_anime = self.no_wipe_anime_onoff_flag != 0;
        self.options.skip_wipe_anime = self.skip_wipe_anime_onoff_flag != 0;
        self.script_skip_unread_message_flag = st.script_skip_unread_message_flag;
        self.script_stage_time_stop_flag = st.script_stage_time_stop_flag;
        self.last_pc = st.last_pc;
        self.last_line_no = st.last_line_no;
        self.last_scene = st.last_scene.clone();

        self.save_point_snapshot = st.save_point_snapshot.clone();
        self.sel_point_snapshot = st.sel_point_snapshot.clone();
        self.sel_point_stock = st.sel_point_stock.clone();
        self.wipe_end_at = None;
        self.halted = false;
    }
    pub fn set_options(&mut self, options: VmOptions) {
        self.options = options;
        self.no_wipe_anime_onoff_flag = if self.options.no_wipe_anime { 1 } else { 0 };
        self.skip_wipe_anime_onoff_flag = if self.options.skip_wipe_anime { 1 } else { 0 };
        self.system_extra_int_values = self.options.system_extra_int_values.clone();
        self.system_extra_str_values = self.options.system_extra_str_values.clone();
    }

    pub fn snapshot_persistent_state(&self) -> VmPersistentState {
        VmPersistentState {
            flags_a: self.flags_a.clone(),
            flags_b: self.flags_b.clone(),
            flags_c: self.flags_c.clone(),
            flags_d: self.flags_d.clone(),
            flags_e: self.flags_e.clone(),
            flags_f: self.flags_f.clone(),
            flags_x: self.flags_x.clone(),
            flags_g: self.flags_g.clone(),
            flags_z: self.flags_z.clone(),
            flags_s: self.flags_s.clone(),
            flags_m: self.flags_m.clone(),
            global_namae: self.global_namae.clone(),
            local_namae: self.local_namae.clone(),
            save_point_set: self.save_point_set,
            sel_point_set: self.sel_point_set,
        }
    }

    pub fn apply_persistent_state(&mut self, st: &VmPersistentState) {
        fn restore_fixed_i32(dst: &mut Vec<i32>, src: &[i32]) {
            if src.len() == FLAG_LIST_SIZE {
                *dst = src.to_vec();
            }
        }

        fn restore_fixed_string(dst: &mut Vec<String>, src: &[String]) {
            if src.len() == FLAG_LIST_SIZE {
                *dst = src.to_vec();
            }
        }

        restore_fixed_i32(&mut self.flags_a, &st.flags_a);
        restore_fixed_i32(&mut self.flags_b, &st.flags_b);
        restore_fixed_i32(&mut self.flags_c, &st.flags_c);
        restore_fixed_i32(&mut self.flags_d, &st.flags_d);
        restore_fixed_i32(&mut self.flags_e, &st.flags_e);
        restore_fixed_i32(&mut self.flags_f, &st.flags_f);
        restore_fixed_i32(&mut self.flags_x, &st.flags_x);
        restore_fixed_i32(&mut self.flags_g, &st.flags_g);
        restore_fixed_i32(&mut self.flags_z, &st.flags_z);
        restore_fixed_string(&mut self.flags_s, &st.flags_s);
        restore_fixed_string(&mut self.flags_m, &st.flags_m);
        restore_fixed_string(&mut self.global_namae, &st.global_namae);
        restore_fixed_string(&mut self.local_namae, &st.local_namae);
        self.save_point_set = st.save_point_set;
        self.sel_point_set = st.sel_point_set;
        self.save_point_snapshot = if self.save_point_set {
            Some(self.snapshot_persistent_state())
        } else {
            None
        };
        self.sel_point_snapshot = if self.sel_point_set {
            Some(self.snapshot_persistent_state())
        } else {
            None
        };
        self.sel_point_stock = None;
    }

    pub(super) fn clear_transient_flow_state(&mut self) {
        self.stack = IfcStack::default();
        self.last_sel_msg.clear();
        self.sel_point_stock = None;
        self.return_scene_once = None;
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

    pub fn run(&mut self, host: &mut dyn Host, provider: &mut dyn SceneProvider) -> Result<()> {
        self.run_inner(host, provider).with_context(|| {
            format!(
                "vm: error at pc={} line={} scene={}",
                self.last_pc, self.last_line_no, self.last_scene
            )
        })
    }

    pub(super) fn run_inner(
        &mut self,
        host: &mut dyn Host,
        provider: &mut dyn SceneProvider,
    ) -> Result<()> {
        while !self.lexer.is_eof() && !self.halted {
            self.last_pc = self.lexer.pc;
            self.last_line_no = self.lexer.cur_line_no;
            self.last_scene = self.scene.clone();
            host.on_location(&self.scene_title, &self.scene, self.lexer.cur_line_no);

            if self.steps >= self.max_steps {
                self.halted = true;
                break;
            }
            if host.should_interrupt() {
                self.halted = true;
                break;
            }
            self.steps += 1;

            let code = self.lexer.pop_u8()?;

            self.stats.opcode_hits[code as usize] += 1;

            if code == cd::SEL_BLOCK_START || code == cd::SEL_BLOCK_END {
                // Selection blocks are UI-driven; ignore in the headless VM for now.
                continue;
            }

            if code == cd::PUSH {
                let form_code = self.lexer.pop_i32()?;
                if form_code == crate::elm::form::INT {
                    let v = self.lexer.pop_i32()?;
                    self.stack.push_int(v);
                } else if form_code == crate::elm::form::STR {
                    let s = self.lexer.pop_str_ret()?;
                    self.stack.push_str(s);
                }
                continue;
            }

            if code == cd::PROPERTY {
                let element_raw = self.stack.pop_element()?;
                let element = self.resolve_command_element_alias(&element_raw);

                if let Some((v, form)) = self.try_property_internal(&element) {
                    self.push_vm_value(form, v);
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
                let form_l = self.lexer.pop_i32()?;
                let form_r = self.lexer.pop_i32()?;
                let opr = self.lexer.pop_u8()?;
                self.calculate_2(form_l, form_r, opr, host)?;
                continue;
            }

            if code == cd::ELM_POINT {
                self.stack.elm_point();
                continue;
            }

            if code == cd::ASSIGN {
                let left_form = self.lexer.pop_i32()?;
                let right_form = self.lexer.pop_i32()?;
                let al_id = self.lexer.pop_i32()?;

                // fixed-form rhs
                let rhs = self.pop_single_arg(right_form)?;
                let element = self.stack.pop_element()?;

                if !self.try_assign_internal(&element, al_id, &rhs)? {
                    host.on_assign(&element, al_id, &rhs);
                }

                // left_form currently unused (host decides)
                let _ = left_form;
                continue;
            }

            if code == cd::NL {
                let old_line_no = self.lexer.cur_line_no;
                let line_no = self.lexer.pop_i32()?;
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
                let arg_list_id = self.lexer.pop_i32()?;

                let mut args = self.pop_arg_list()?;

                let element_raw = self.stack.pop_element()?;
                let element = self.resolve_command_element_alias(&element_raw);

                let named_arg_cnt = self.lexer.pop_i32()?;
                if named_arg_cnt > 0 {
                    for a in 0..named_arg_cnt as usize {
                        let id = self.lexer.pop_i32()?;
                        if let Some(idx) = args.len().checked_sub(a + 1) {
                            args[idx].id = id;
                        }
                    }
                }
                let ret_form = self.lexer.pop_i32()?;
                let mut read_flag_no = None;
                if Self::command_needs_read_flag_tail(&element) {
                    read_flag_no = Some(self.lexer.pop_i32()?);
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
                    let label_no = self.lexer.pop_i32()?;
                    self.lexer.jump_to_label(label_no)?;
                }
                x if x == cd::GOTO_FALSE => {
                    let cond = self.stack.pop_int()?;
                    let label_no = self.lexer.pop_i32()?;
                    if cond == 0 {
                        self.lexer.jump_to_label(label_no)?;
                    }
                }
                x if x == cd::GOTO_TRUE => {
                    let cond = self.stack.pop_int()?;
                    let label_no = self.lexer.pop_i32()?;
                    if cond != 0 {
                        self.lexer.jump_to_label(label_no)?;
                    }
                }
                x if x == cd::GOSUB => {
                    self.proc_gosub(crate::elm::form::INT)?;
                }
                x if x == cd::GOSUBSTR => {
                    self.proc_gosub(crate::elm::form::STR)?;
                }
                x if x == cd::RETURN => {
                    if !self.proc_return(host)? {
                        return Ok(());
                    }
                }
                x if x == cd::POP => {
                    let form_code = self.lexer.pop_i32()?;
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
                    let form_code = self.lexer.pop_i32()?;
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
                    let form_code = self.lexer.pop_i32()?;
                    let prop_id = self.lexer.pop_i32()?;
                    let mut size = 0;
                    if form_code == crate::elm::form::INTLIST
                        || form_code == crate::elm::form::STRLIST
                    {
                        size = self.stack.pop_int()?;
                    }
                    self.proc_dec_prop(form_code, prop_id, size);
                }
                x if x == cd::ARG => {
                    self.proc_arg()?;
                }
                x if x == cd::OPERATE_1 => {
                    let form = self.lexer.pop_i32()?;
                    let opr = self.lexer.pop_u8()?;
                    self.calculate_1(form, opr, host)?;
                }
                x if x == cd::NAME => {
                    let s = self.stack.pop_str()?;
                    host.on_name(&s);
                }
                x if x == cd::TEXT => {
                    let read_flag_no = self.lexer.pop_i32()?;
                    let msg = self.stack.pop_str()?;
                    host.on_text(&msg, read_flag_no);
                }
                x if x == cd::NONE => {
                    host.on_script_fatal("スクリプトの解析に失敗しました。");
                    self.halted = true;
                    return Ok(());
                }
                x if x == cd::EOF => {
                    host.on_script_fatal("ファイルの終端に到達しました。");
                    self.halted = true;
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
