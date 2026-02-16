use super::opcode::op;
use super::*;

impl Vm {
    const SCRIPT_ARG_CNT_MAX: usize = 1024;
    const SCRIPT_EXP_ARG_CNT_MAX: usize = 64;

    pub(super) fn push_host_ret(&mut self, ret: &HostReturn, ret_form: i32) {
        if ret_form == crate::elm::form::INT {
            self.stack.push_int(ret.int);
        } else if ret_form == crate::elm::form::STR {
            self.stack.push_str(ret.str_.clone());
        } else if ret_form == crate::elm::form::VOID {
            // no-op
        } else {
            // Element / other: best-effort stub.
            self.stack.push_element(&ret.element);
        }
    }

    pub(super) fn pop_single_arg(&mut self, form: i32) -> Result<Prop> {
        if form == crate::elm::form::INT {
            Ok(Prop {
                id: -1,
                form: crate::elm::form::INT,
                value: PropValue::Int(self.stack.pop_int()?),
            })
        } else if form == crate::elm::form::STR {
            Ok(Prop {
                id: -1,
                form: crate::elm::form::STR,
                value: PropValue::Str(self.stack.pop_str()?),
            })
        } else {
            Ok(Prop {
                id: -1,
                form,
                value: PropValue::Element(self.stack.pop_element()?),
            })
        }
    }

    fn pop_arg_list_with_cap(&mut self, cap: usize) -> Result<Vec<Prop>> {
        let arg_cnt = self.lexer.pop_i32()?;
        if arg_cnt <= 0 {
            return Ok(Vec::new());
        }

        let n = arg_cnt as usize;
        let keep = n.min(cap);
        let mut out: Vec<Option<Prop>> = vec![None; keep];

        for i in (0..n).rev() {
            let form_code = self.lexer.pop_i32()?;
            let prop = if form_code == crate::elm::form::INT {
                Prop {
                    id: -1,
                    form: crate::elm::form::INT,
                    value: PropValue::Int(self.stack.pop_int()?),
                }
            } else if form_code == crate::elm::form::STR {
                Prop {
                    id: -1,
                    form: crate::elm::form::STR,
                    value: PropValue::Str(self.stack.pop_str()?),
                }
            } else if form_code == crate::elm::form::LABEL {
                // stored as int
                Prop {
                    id: -1,
                    form: crate::elm::form::INT,
                    value: PropValue::Int(self.stack.pop_int()?),
                }
            } else if form_code == crate::elm::form::LIST {
                let sub = self.pop_arg_list_with_cap(Self::SCRIPT_EXP_ARG_CNT_MAX)?;
                Prop {
                    id: -1,
                    form: crate::elm::form::LIST,
                    value: PropValue::List(sub),
                }
            } else {
                Prop {
                    id: -1,
                    form: form_code,
                    value: PropValue::Element(self.stack.pop_element()?),
                }
            };

            if i < keep {
                out[i] = Some(prop);
            }
        }

        Ok(out.into_iter().flatten().collect())
    }

    pub(super) fn pop_arg_list(&mut self) -> Result<Vec<Prop>> {
        self.pop_arg_list_with_cap(Self::SCRIPT_ARG_CNT_MAX)
    }

    pub(super) fn proc_gosub(&mut self, ret_form: i32) -> Result<()> {
        let label_no = self.lexer.pop_i32()?;
        let _args = self.pop_arg_list()?;

        // store expected return type in caller
        if let Some(top) = self.frames.last_mut() {
            top.expect_ret_form = ret_form;
        }

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

        // Populate call.L / call.K from args (like tnm_command_proc_gosub).
        {
            let call = &mut self.frames.last_mut().unwrap().call;
            let mut li = 0usize;
            let mut ki = 0usize;
            for a in &_args {
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

        self.lexer.jump_to_label(label_no)?;
        Ok(())
    }

    pub(super) fn proc_return(&mut self, host: &mut dyn Host) -> Result<bool> {
        let args = self.pop_arg_list()?;

        if self.frames.len() <= 1 {
            // no caller -> end script
            return Ok(false);
        }

        // pop callee
        let callee = self.frames.pop().unwrap();
        let frame_action_flag = callee.frame_action_flag;
        self.scene = callee.return_scene;
        self.lexer.set_scene(callee.return_dat);
        self.reload_user_props_from_current_scene();
        self.lexer.pc = callee.return_pc;
        self.lexer.cur_line_no = callee.return_line_no;

        // now we're back in caller
        let ret_form = self.frames.last().unwrap().expect_ret_form;

        if ret_form == crate::elm::form::INT {
            if args.len() == 1 {
                if let PropValue::Int(v) = args[0].value {
                    self.stack.push_int(v);
                } else {
                    self.stack.push_int(0);
                }
            } else {
                self.stack.push_int(0);
            }
        } else if ret_form == crate::elm::form::STR {
            if args.len() == 1 {
                if let PropValue::Str(s) = &args[0].value {
                    self.stack.push_str(s.clone());
                } else {
                    self.stack.push_str(String::new());
                }
            } else {
                self.stack.push_str(String::new());
            }
        }

        // If this was invoked as a frame action, stop after returning to the caller.
        // (Matches tnm_proc_script() behaviour when frame_action_flag is set.)
        let _ = host;
        Ok(!frame_action_flag)
    }

    pub(super) fn calculate_1(&mut self, form: i32, opr: u8, _host: &mut dyn Host) -> Result<()> {
        if form != crate::elm::form::INT {
            return Ok(());
        }

        let rhs = self.stack.pop_int()?;
        match opr {
            x if x == op::PLUS => self.stack.push_int(rhs),
            x if x == op::MINUS => self.stack.push_int(-rhs),
            x if x == op::TILDE => self.stack.push_int(!rhs),
            // C++ tnm_calculate_1 has no default branch: unknown operator pushes nothing.
            _ => {}
        }
        Ok(())
    }

    pub(super) fn calculate_2(
        &mut self,
        form_l: i32,
        form_r: i32,
        opr: u8,
        host: &mut dyn Host,
    ) -> Result<()> {
        if form_l == crate::elm::form::INT && form_r == crate::elm::form::INT {
            let rhs = self.stack.pop_int()?;
            let lhs = self.stack.pop_int()?;
            match opr {
                x if x == op::PLUS => self.stack.push_int(lhs.wrapping_add(rhs)),
                x if x == op::MINUS => self.stack.push_int(lhs.wrapping_sub(rhs)),
                x if x == op::MULTIPLE => self.stack.push_int(lhs.wrapping_mul(rhs)),
                x if x == op::EQUAL => self.stack.push_int((lhs == rhs) as i32),
                x if x == op::NOT_EQUAL => self.stack.push_int((lhs != rhs) as i32),
                x if x == op::GREATER => self.stack.push_int((lhs > rhs) as i32),
                x if x == op::GREATER_EQUAL => self.stack.push_int((lhs >= rhs) as i32),
                x if x == op::LESS => self.stack.push_int((lhs < rhs) as i32),
                x if x == op::LESS_EQUAL => self.stack.push_int((lhs <= rhs) as i32),
                x if x == op::LOGICAL_AND => self.stack.push_int(((lhs != 0) && (rhs != 0)) as i32),
                x if x == op::LOGICAL_OR => self.stack.push_int(((lhs != 0) || (rhs != 0)) as i32),
                x if x == op::AND => self.stack.push_int(lhs & rhs),
                x if x == op::OR => self.stack.push_int(lhs | rhs),
                x if x == op::HAT => self.stack.push_int(lhs ^ rhs),
                x if x == op::SL => self.stack.push_int(lhs.wrapping_shl(rhs as u32)),
                x if x == op::SR => self.stack.push_int(lhs.wrapping_shr(rhs as u32)),
                x if x == op::SR3 => self
                    .stack
                    .push_int(((lhs as u32).wrapping_shr(rhs as u32)) as i32),
                x if x == op::DIVIDE => {
                    if rhs == 0 {
                        host.on_error("0 除算を行いました！'/'");
                        self.stack.push_int(0);
                    } else {
                        self.stack.push_int(lhs / rhs);
                    }
                }
                x if x == op::AMARI => {
                    if rhs == 0 {
                        host.on_error("0 除算を行いました！'%'");
                        self.stack.push_int(0);
                    } else {
                        self.stack.push_int(lhs % rhs);
                    }
                }
                // C++ tnm_calculate_2 has no default branch for INT/INT: unknown operator pushes nothing.
                _ => {}
            }
            return Ok(());
        }

        if form_l == crate::elm::form::STR && form_r == crate::elm::form::INT {
            let rhs = self.stack.pop_int()?;
            let lhs = self.stack.pop_str()?;
            match opr {
                x if x == op::MULTIPLE => {
                    let out = if rhs <= 0 {
                        String::new()
                    } else {
                        lhs.repeat(rhs as usize)
                    };
                    self.stack.push_str(out);
                }
                _ => {}
            }
            return Ok(());
        }

        if form_l == crate::elm::form::STR && form_r == crate::elm::form::STR {
            let rhs = self.stack.pop_str()?;
            let lhs = self.stack.pop_str()?;
            match opr {
                x if x == op::PLUS => {
                    self.stack.push_str(format!("{}{}", lhs, rhs));
                }
                x if x == op::EQUAL => {
                    self.stack
                        .push_int((lhs.to_lowercase() == rhs.to_lowercase()) as i32);
                }
                x if x == op::NOT_EQUAL => {
                    self.stack
                        .push_int((lhs.to_lowercase() != rhs.to_lowercase()) as i32);
                }
                x if x == op::GREATER => {
                    self.stack
                        .push_int((lhs.to_lowercase() > rhs.to_lowercase()) as i32);
                }
                x if x == op::GREATER_EQUAL => {
                    self.stack
                        .push_int((lhs.to_lowercase() >= rhs.to_lowercase()) as i32);
                }
                x if x == op::LESS => {
                    self.stack
                        .push_int((lhs.to_lowercase() < rhs.to_lowercase()) as i32);
                }
                x if x == op::LESS_EQUAL => {
                    self.stack
                        .push_int((lhs.to_lowercase() <= rhs.to_lowercase()) as i32);
                }
                _ => {}
            }
            return Ok(());
        }

        Ok(())
    }
}
