use super::*;

impl Vm {
    fn to_runtime_prop_value(v: &PropValue) -> VmEndSaveRuntimePropValue {
        match v {
            PropValue::Int(i) => VmEndSaveRuntimePropValue::Int(*i),
            PropValue::Str(s) => VmEndSaveRuntimePropValue::Str(s.clone()),
            PropValue::List(items) => VmEndSaveRuntimePropValue::List(
                items
                    .iter()
                    .map(|p| VmEndSaveRuntimeProp {
                        id: p.id,
                        form: p.form,
                        value: Self::to_runtime_prop_value(&p.value),
                    })
                    .collect(),
            ),
            PropValue::Element(vals) => VmEndSaveRuntimePropValue::Element(vals.clone()),
            PropValue::IntList(vals) => VmEndSaveRuntimePropValue::IntList(vals.clone()),
            PropValue::StrList(vals) => VmEndSaveRuntimePropValue::StrList(vals.clone()),
        }
    }

    fn from_runtime_prop_value(v: &VmEndSaveRuntimePropValue) -> PropValue {
        match v {
            VmEndSaveRuntimePropValue::Int(i) => PropValue::Int(*i),
            VmEndSaveRuntimePropValue::Str(s) => PropValue::Str(s.clone()),
            VmEndSaveRuntimePropValue::List(items) => PropValue::List(
                items
                    .iter()
                    .map(|p| Prop {
                        id: p.id,
                        form: p.form,
                        value: Self::from_runtime_prop_value(&p.value),
                    })
                    .collect(),
            ),
            VmEndSaveRuntimePropValue::Element(vals) => PropValue::Element(vals.clone()),
            VmEndSaveRuntimePropValue::IntList(vals) => PropValue::IntList(vals.clone()),
            VmEndSaveRuntimePropValue::StrList(vals) => PropValue::StrList(vals.clone()),
        }
    }

    fn snapshot_runtime_frame_action(&self, fa: &FrameAction) -> VmEndSaveRuntimeFrameActionState {
        VmEndSaveRuntimeFrameActionState {
            end_time: fa.end_time,
            real_flag: fa.real_flag,
            scn_name: fa.scn_name.clone(),
            cmd_name: fa.cmd_name.clone(),
            args: fa
                .args
                .iter()
                .map(|arg| VmEndSaveRuntimeProp {
                    id: arg.id,
                    form: arg.form,
                    value: Self::to_runtime_prop_value(&arg.value),
                })
                .collect(),
            end_action_flag: fa.end_action_flag,
        }
    }

    fn apply_runtime_frame_action(fa: &VmEndSaveRuntimeFrameActionState) -> FrameAction {
        FrameAction {
            end_time: fa.end_time,
            real_flag: fa.real_flag,
            scn_name: fa.scn_name.clone(),
            cmd_name: fa.cmd_name.clone(),
            args: fa
                .args
                .iter()
                .map(|arg| Prop {
                    id: arg.id,
                    form: arg.form,
                    value: Self::from_runtime_prop_value(&arg.value),
                })
                .collect(),
            end_action_flag: fa.end_action_flag,
        }
    }

    pub(super) fn snapshot_end_save_runtime_state(&self) -> VmEndSaveRuntimeState {
        let frames = self
            .frames
            .iter()
            .map(|f| VmEndSaveRuntimeFrameState {
                return_pc: f.return_pc,
                return_scene: f.return_scene.clone(),
                return_line_no: f.return_line_no,
                expect_ret_form: f.expect_ret_form,
                frame_action_flag: f.frame_action_flag,
                arg_cnt: f.arg_cnt,
                call_l: f.call.l.clone(),
                call_k: f.call.k.clone(),
                call_user_props: f
                    .call
                    .user_props
                    .iter()
                    .map(|cp| VmEndSaveRuntimeCallPropState {
                        prop_id: cp.prop_id,
                        form: cp.form,
                        value: Self::to_runtime_prop_value(&cp.value),
                    })
                    .collect(),
            })
            .collect();
        VmEndSaveRuntimeState {
            scene: self.scene.clone(),
            lexer_scene: self.scene.clone(),
            lexer_pc: self.lexer.pc,
            lexer_line_no: self.lexer.cur_line_no,
            stack_ints: self.stack.ints.clone(),
            stack_strs: self.stack.strs.clone(),
            stack_points: self.stack.points.clone(),
            frames,
            user_prop_forms: self.user_prop_forms.clone(),
            user_prop_values: self
                .user_prop_values
                .iter()
                .map(Self::to_runtime_prop_value)
                .collect(),
            frame_action: self.snapshot_runtime_frame_action(&self.frame_action),
            frame_action_ch: self
                .frame_action_ch
                .iter()
                .map(|fa| self.snapshot_runtime_frame_action(fa))
                .collect(),
            save_point_snapshot: self.save_point_snapshot.clone(),
            sel_point_snapshot: self.sel_point_snapshot.clone(),
            sel_point_stock: self.sel_point_stock.clone(),
            cur_mwnd_element: self.cur_mwnd_element.clone(),
            cur_sel_mwnd_element: self.cur_sel_mwnd_element.clone(),
            hide_mwnd_onoff_flag: self.hide_mwnd_onoff_flag,
            msg_back_open_flag: self.msg_back_open_flag,
            msg_back_has_message: self.msg_back_has_message,
            msg_back_disable_flag: self.msg_back_disable_flag,
            msg_back_off_flag: self.msg_back_off_flag,
            msg_back_disp_off_flag: self.msg_back_disp_off_flag,
            msg_back_proc_off_flag: self.msg_back_proc_off_flag,
            system_wipe_flag: self.system_wipe_flag,
            do_frame_action_flag: self.do_frame_action_flag,
            do_load_after_call_flag: self.do_load_after_call_flag,
            last_pc: self.last_pc,
            last_line_no: self.last_line_no,
            last_scene: self.last_scene.clone(),
        }
    }

    pub(super) fn apply_end_save_state_with_provider(
        &mut self,
        st: &VmEndSaveState,
        provider: &mut dyn SceneProvider,
    ) -> Result<bool> {
        let Some(rt) = &st.runtime else {
            self.scene_title = st.scene_title.clone();
            self.last_sel_msg = st.message.clone();
            self.apply_persistent_state(&st.persistent);
            return Ok(false);
        };

        let lexer_scene = match provider.get_scene(&rt.lexer_scene) {
            Ok(scene) => scene,
            Err(_) => return Ok(false),
        };

        let mut frames = Vec::with_capacity(rt.frames.len());
        for f in &rt.frames {
            let return_dat = match provider.get_scene(&f.return_scene) {
                Ok(scene) => scene,
                Err(_) => return Ok(false),
            };
            let call_l = if f.call_l.is_empty() {
                vec![0; DEFAULT_CALL_FLAG_CNT]
            } else {
                f.call_l.clone()
            };
            let call_k = if f.call_k.is_empty() {
                vec![String::new(); DEFAULT_CALL_FLAG_CNT]
            } else {
                f.call_k.clone()
            };
            frames.push(Frame {
                return_pc: f.return_pc.min(return_dat.scn_bytes.len()),
                return_scene: f.return_scene.clone(),
                return_dat,
                return_line_no: f.return_line_no,
                expect_ret_form: f.expect_ret_form,
                frame_action_flag: f.frame_action_flag,
                arg_cnt: f.arg_cnt,
                call: CallContext {
                    l: call_l,
                    k: call_k,
                    user_props: f
                        .call_user_props
                        .iter()
                        .map(|cp| CallProp {
                            prop_id: cp.prop_id,
                            form: cp.form,
                            value: Self::from_runtime_prop_value(&cp.value),
                        })
                        .collect(),
                },
            });
        }
        if frames.is_empty() {
            let dat = match provider.get_scene(&rt.scene) {
                Ok(scene) => scene,
                Err(_) => return Ok(false),
            };
            frames.push(Frame {
                return_pc: 0,
                return_scene: rt.scene.clone(),
                return_dat: dat,
                return_line_no: 0,
                expect_ret_form: crate::elm::form::VOID,
                frame_action_flag: false,
                arg_cnt: 0,
                call: CallContext::new(DEFAULT_CALL_FLAG_CNT),
            });
        }

        self.scene_title = st.scene_title.clone();
        self.last_sel_msg = st.message.clone();
        self.apply_persistent_state(&st.persistent);
        self.scene = rt.scene.clone();
        self.lexer.set_scene(lexer_scene);
        self.lexer.pc = rt.lexer_pc.min(self.lexer.dat.scn_bytes.len());
        self.lexer.cur_line_no = rt.lexer_line_no;

        self.stack.ints = rt.stack_ints.clone();
        self.stack.strs = rt.stack_strs.clone();
        self.stack.points = rt
            .stack_points
            .iter()
            .copied()
            .filter(|p| *p <= self.stack.ints.len())
            .collect();

        self.frames = frames;

        self.user_prop_forms = rt.user_prop_forms.clone();
        self.user_prop_values = rt
            .user_prop_values
            .iter()
            .map(Self::from_runtime_prop_value)
            .collect();
        self.frame_action = Self::apply_runtime_frame_action(&rt.frame_action);
        self.frame_action_ch = rt
            .frame_action_ch
            .iter()
            .map(Self::apply_runtime_frame_action)
            .collect();
        self.save_point_snapshot = rt.save_point_snapshot.clone();
        self.sel_point_snapshot = rt.sel_point_snapshot.clone();
        self.sel_point_stock = rt.sel_point_stock.clone();
        self.cur_mwnd_element = rt.cur_mwnd_element.clone();
        self.cur_sel_mwnd_element = rt.cur_sel_mwnd_element.clone();
        self.hide_mwnd_onoff_flag = rt.hide_mwnd_onoff_flag;
        self.msg_back_open_flag = rt.msg_back_open_flag;
        self.msg_back_has_message = rt.msg_back_has_message;
        self.msg_back_disable_flag = rt.msg_back_disable_flag;
        self.msg_back_off_flag = rt.msg_back_off_flag;
        self.msg_back_disp_off_flag = rt.msg_back_disp_off_flag;
        self.msg_back_proc_off_flag = rt.msg_back_proc_off_flag;
        self.system_wipe_flag = rt.system_wipe_flag;
        self.do_frame_action_flag = rt.do_frame_action_flag;
        self.do_load_after_call_flag = rt.do_load_after_call_flag;

        self.last_pc = rt.last_pc;
        self.last_line_no = rt.last_line_no;
        self.last_scene = rt.last_scene.clone();
        Ok(true)
    }
}
