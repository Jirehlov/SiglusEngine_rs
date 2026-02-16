use super::*;


impl Vm {
    pub(super) fn command_proc_frame_action(
        fa: &mut FrameAction,
        chain: &[i32],
        args: &[Prop],
        ret_form: i32,
        scene: &str,
        stack: &mut IfcStack,
    ) -> Result<bool> {
        if chain.is_empty() {
            return Ok(true);
        }

        let method = chain[0];
        if crate::elm::frameaction::is_frameaction_start(method) {
            let end_time = match args.get(0).map(|p| &p.value) {
                Some(PropValue::Int(v)) => *v,
                _ => 0,
            };
            let cmd_name = match args.get(1).map(|p| &p.value) {
                Some(PropValue::Str(s)) => s.clone(),
                _ => String::new(),
            };
            let extra = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            let real_flag = if crate::elm::frameaction::is_frameaction_start_real(method) {
                1
            } else {
                0
            };
            fa.set_param(end_time, real_flag, scene.to_string(), cmd_name, extra);
            return Ok(true);
        }

        if crate::elm::frameaction::is_frameaction_end(method) {
            // In the original engine, end triggers an end action callback.
            // The headless VM does not run the engine frame loop, so we just clear params.
            fa.reinit();
            return Ok(true);
        }

        if crate::elm::frameaction::is_frameaction_is_end_action(method) {
            if ret_form == crate::elm::form::INT {
                stack.push_int(if fa.end_action_flag { 1 } else { 0 });
            }
            return Ok(true);
        }

        Ok(false)
    }

    /// Handle `math.*` sub-commands (ELM_GLOBAL_MATH + chain).
    pub(super) fn proc_math_command(&mut self, chain: &[i32], args: &[Prop], ret_form: i32) {
        if chain.is_empty() {
            return;
        }
        let method = chain[0];
        let arg_int = |idx: usize| -> i32 {
            match args.get(idx).map(|p| &p.value) {
                Some(PropValue::Int(v)) => *v,
                _ => 0,
            }
        };

        let result: Option<i32> = if crate::elm::math::is_abs(method) {
            Some(arg_int(0).abs())
        } else if crate::elm::math::is_min(method) {
            Some(arg_int(0).min(arg_int(1)))
        } else if crate::elm::math::is_max(method) {
            Some(arg_int(0).max(arg_int(1)))
        } else if crate::elm::math::is_rand(method) {
            // Return a pseudo-random value in [0, arg0). Use wrapping for determinism.
            let upper = arg_int(0);
            if upper <= 0 {
                Some(0)
            } else {
                // Simple LCG-style random for headless reproducibility
                let seed = (self.steps as i32)
                    .wrapping_mul(1103515245)
                    .wrapping_add(12345);
                Some(((seed as u32) % (upper as u32)) as i32)
            }
        } else if crate::elm::math::is_limit(method) {
            let val = arg_int(0);
            let lo = arg_int(1);
            let hi = arg_int(2);
            Some(val.clamp(lo, hi))
        } else if crate::elm::math::is_sqrt(method) {
            let v = arg_int(0);
            if v < 0 {
                Some(0)
            } else {
                Some((v as f64).sqrt() as i32)
            }
        } else if crate::elm::math::is_sin(method) {
            let deg = arg_int(0);
            Some(((deg as f64).to_radians().sin() * 1000.0) as i32)
        } else if crate::elm::math::is_cos(method) {
            let deg = arg_int(0);
            Some(((deg as f64).to_radians().cos() * 1000.0) as i32)
        } else if crate::elm::math::is_arctan(method) {
            let y = arg_int(0);
            let x = arg_int(1);
            Some((y as f64).atan2(x as f64).to_degrees() as i32)
        } else {
            // Unknown math sub-command; return 0 as fallback.
            Some(0)
        };

        if let Some(v) = result {
            if ret_form == crate::elm::form::INT {
                self.stack.push_int(v);
            }
        }
    }

    pub(super) fn proc_user_cmd_call(
        &mut self,
        user_cmd_id: i32,
        args: &[Prop],
        ret_form: i32,
        frame_action_flag: bool,
        provider: &mut dyn SceneProvider,
    ) -> Result<()> {
        // Set return type expectation on the caller.
        if let Some(frame) = self.frames.last_mut() {
            frame.expect_ret_form = ret_form;
        }

        // New call frame.
        let return_pc = self.lexer.pc;
        let return_scene = self.scene.clone();
        let return_dat = self.lexer.dat.clone();
        let return_line_no = self.lexer.cur_line_no;
        self.frames.push(Frame {
            return_pc,
            return_scene,
            return_dat,
            return_line_no,
            expect_ret_form: crate::elm::form::VOID,
            frame_action_flag,
            arg_cnt: args.len(),
            call: CallContext::new(DEFAULT_CALL_FLAG_CNT),
        });

        // Push args in forward order (C++ behaviour).
        for a in args {
            match &a.value {
                PropValue::Int(v) => self.stack.push_int(*v),
                PropValue::Str(s) => self.stack.push_str(s.clone()),
                PropValue::Element(el) => self.stack.push_element(el),
                _ => self.stack.push_element(&[]),
            }
        }

        // Jump to command body.
        let inc_cnt = provider.inc_cmd_count();
        if user_cmd_id < inc_cnt {
            if let Some((scene, offset)) = provider.get_inc_cmd_target(user_cmd_id)? {
                let dat = provider.get_scene(&scene)?;
                self.scene = scene.to_string();
                self.lexer.set_scene(dat);
                self.reload_user_props_from_current_scene();
                if offset < 0 {
                    bail!(
                        "lexer: user_cmd {} has negative offset {}",
                        user_cmd_id,
                        offset
                    );
                }
                self.lexer.pc = offset as usize;
            } else {
                bail!("lexer: user_cmd {} not found", user_cmd_id);
            }
        } else {
            let scn_cmd_no = user_cmd_id - inc_cnt;
            self.lexer.jump_to_scn_cmd_index(scn_cmd_no)?;
        }
        Ok(())
    }

    pub(super) fn proc_jump(
        &mut self,
        scene: &str,
        z_no: i32,
        provider: &mut dyn SceneProvider,
    ) -> Result<()> {
        let dat = provider.get_scene(scene)?;
        self.scene = scene.to_string();
        self.lexer.set_scene(dat);
        self.reload_user_props_from_current_scene();
        self.lexer.jump_to_z_label(z_no)?;
        Ok(())
    }

    pub(super) fn proc_farcall_like(
        &mut self,
        scene: &str,
        z_no: i32,
        expect_ret_form: i32,
        call_args: &[Prop],
        provider: &mut dyn SceneProvider,
    ) -> Result<()> {
        // Set return type expectation on the caller.
        if let Some(frame) = self.frames.last_mut() {
            frame.expect_ret_form = expect_ret_form;
        }

        // New call frame (saves current scene state).
        let return_pc = self.lexer.pc;
        let return_scene = self.scene.clone();
        let return_dat = self.lexer.dat.clone();
        let return_line_no = self.lexer.cur_line_no;
        self.frames.push(Frame {
            return_pc,
            return_scene,
            return_dat,
            return_line_no,
            expect_ret_form: crate::elm::form::VOID,
            frame_action_flag: false,
            arg_cnt: 0,
            call: CallContext::new(DEFAULT_CALL_FLAG_CNT),
        });

        // Populate call.L / call.K (best-effort; matches C++ farcall arg expansion).
        {
            let call = &mut self.frames.last_mut().unwrap().call;
            let mut li = 0usize;
            let mut ki = 0usize;
            for a in call_args {
                match &a.value {
                    PropValue::Int(v) => {
                        if li < call.l.len() {
                            call.l[li] = *v;
                        }
                        li += 1;
                    }
                    PropValue::Str(s) => {
                        if ki < call.k.len() {
                            call.k[ki] = s.clone();
                        }
                        ki += 1;
                    }
                    _ => {}
                }
            }
        }

        // Switch scene & jump to z-label.
        let dat = provider.get_scene(scene)?;
        self.scene = scene.to_string();
        self.lexer.set_scene(dat);
        self.reload_user_props_from_current_scene();
        self.lexer.jump_to_z_label(z_no)?;
        Ok(())
    }

}
