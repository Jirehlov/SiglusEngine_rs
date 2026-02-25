use super::*;

impl Vm {
    pub(super) fn try_command_bgm(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.len() < 2 {
            // Bare `bgm` element → accept silently (C++ sets ret element)
            return true;
        }
        let sub = element[1];
        use crate::elm::bgm::*;

        match sub {
            // BGM PLAY — full named-arg override (C++ lines 21-50)
            ELM_BGM_PLAY => {
                let mut regist_name = String::new();
                let mut loop_flag = true;
                let mut wait_flag = false;
                let mut fade_in_time = 0i32;
                let mut fade_out_time = 0i32;
                let mut start_pos = Self::TNM_BGM_START_POS_INI;

                // Positional args (C++ fallthrough switch)
                #[allow(clippy::manual_unwrap_or_default)]
                match arg_list_id {
                    n if n >= 2 => {
                        if let Some(PropValue::Int(v)) = args.get(2).map(|p| &p.value) {
                            fade_out_time = *v;
                        }
                        if let Some(PropValue::Int(v)) = args.get(1).map(|p| &p.value) {
                            fade_in_time = *v;
                        }
                        if let Some(PropValue::Str(s)) = args.get(0).map(|p| &p.value) {
                            regist_name = s.clone();
                        }
                    }
                    1 => {
                        if let Some(PropValue::Int(v)) = args.get(1).map(|p| &p.value) {
                            fade_in_time = *v;
                        }
                        if let Some(PropValue::Str(s)) = args.get(0).map(|p| &p.value) {
                            regist_name = s.clone();
                        }
                    }
                    0 => {
                        if let Some(PropValue::Str(s)) = args.get(0).map(|p| &p.value) {
                            regist_name = s.clone();
                        }
                    }
                    _ => {}
                }

                // Named-arg overrides
                for arg in args.iter() {
                    match arg.id {
                        0 => {
                            if let PropValue::Str(s) = &arg.value {
                                regist_name = s.clone();
                            }
                        }
                        1 => {
                            if let PropValue::Int(v) = arg.value {
                                loop_flag = v != 0;
                            }
                        }
                        2 => {
                            if let PropValue::Int(v) = arg.value {
                                wait_flag = v != 0;
                            }
                        }
                        3 => {
                            if let PropValue::Int(v) = arg.value {
                                start_pos = v;
                            }
                        }
                        4 => {
                            if let PropValue::Int(v) = arg.value {
                                fade_in_time = v;
                            }
                        }
                        5 => {
                            if let PropValue::Int(v) = arg.value {
                                fade_out_time = v;
                            }
                        }
                        _ => {}
                    }
                }

                // C++ dispatches: wait_flag → play_wait, loop_flag → play, else → play_oneshot
                Self::sound_report_file_not_found(
                    host,
                    &regist_name,
                    "bgm.play",
                    VmResourceKind::Movie,
                );
                host.on_bgm_play(
                    &regist_name,
                    loop_flag,
                    wait_flag,
                    fade_in_time,
                    fade_out_time,
                    start_pos,
                    false,
                );
                true
            }
            // BGM PLAY_ONESHOT — positional only (C++ lines 51-56)
            ELM_BGM_PLAY_ONESHOT => {
                let name = Self::sound_arg_str(args, 0);
                let fade_in = args
                    .get(1)
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                let fade_out = args
                    .get(2)
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                Self::sound_report_file_not_found(
                    host,
                    &name,
                    "bgm.play_oneshot",
                    VmResourceKind::Movie,
                );
                host.on_bgm_play(
                    &name,
                    false,
                    false,
                    fade_in,
                    fade_out,
                    Self::TNM_BGM_START_POS_INI,
                    false,
                );
                true
            }
            // BGM PLAY_WAIT — positional only (C++ lines 57-62)
            ELM_BGM_PLAY_WAIT => {
                let name = Self::sound_arg_str(args, 0);
                let fade_in = args
                    .get(1)
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                let fade_out = args
                    .get(2)
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                Self::sound_report_file_not_found(
                    host,
                    &name,
                    "bgm.play_wait",
                    VmResourceKind::Movie,
                );
                host.on_bgm_play(
                    &name,
                    true,
                    true,
                    fade_in,
                    fade_out,
                    Self::TNM_BGM_START_POS_INI,
                    false,
                );
                true
            }
            // BGM READY — full named-arg override (C++ lines 63-88)
            ELM_BGM_READY => {
                let mut regist_name = String::new();
                let mut loop_flag = true;
                let mut fade_out_time = 100i32; // C++ default for READY
                let mut start_pos = Self::TNM_BGM_START_POS_INI;

                // Positional args (C++ fallthrough switch)
                match arg_list_id {
                    n if n >= 2 => {
                        if let Some(PropValue::Int(v)) = args.get(1).map(|p| &p.value) {
                            fade_out_time = *v;
                        }
                        if let Some(PropValue::Str(s)) = args.get(0).map(|p| &p.value) {
                            regist_name = s.clone();
                        }
                    }
                    0 => {
                        if let Some(PropValue::Str(s)) = args.get(0).map(|p| &p.value) {
                            regist_name = s.clone();
                        }
                    }
                    _ => {}
                }

                // Named-arg overrides (id 2 / wait_flag not used for READY)
                for arg in args.iter() {
                    match arg.id {
                        0 => {
                            if let PropValue::Str(s) = &arg.value {
                                regist_name = s.clone();
                            }
                        }
                        1 => {
                            if let PropValue::Int(v) = arg.value {
                                loop_flag = v != 0;
                            }
                        }
                        3 => {
                            if let PropValue::Int(v) = arg.value {
                                start_pos = v;
                            }
                        }
                        5 => {
                            if let PropValue::Int(v) = arg.value {
                                fade_out_time = v;
                            }
                        }
                        _ => {}
                    }
                }

                // C++: loop_flag → play, else → play_oneshot; fade_in=0, ready=true
                Self::sound_report_file_not_found(
                    host,
                    &regist_name,
                    "bgm.ready",
                    VmResourceKind::Movie,
                );
                host.on_bgm_play(
                    &regist_name,
                    loop_flag,
                    false,
                    0,
                    fade_out_time,
                    start_pos,
                    true,
                );
                true
            }
            // BGM READY_ONESHOT — positional only (C++ lines 89-94)
            ELM_BGM_READY_ONESHOT => {
                let name = Self::sound_arg_str(args, 0);
                let fade_in = args
                    .get(1)
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                let fade_out = args
                    .get(2)
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                Self::sound_report_file_not_found(
                    host,
                    &name,
                    "bgm.ready_oneshot",
                    VmResourceKind::Movie,
                );
                host.on_bgm_play(
                    &name,
                    false,
                    false,
                    fade_in,
                    fade_out,
                    Self::TNM_BGM_START_POS_INI,
                    true,
                );
                true
            }
            // BGM STOP (C++ lines 95-99)
            ELM_BGM_STOP => {
                let fade_out = args
                    .first()
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                host.on_bgm_stop(fade_out);
                true
            }
            // BGM PAUSE (C++ lines 100-104)
            ELM_BGM_PAUSE => {
                let fade = args
                    .first()
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                host.on_bgm_pause(fade);
                true
            }
            // BGM RESUME / RESUME_WAIT — named-arg delay_time (C++ lines 105-122)
            ELM_BGM_RESUME => {
                let fade = args
                    .first()
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                // Named-arg id 0 → delay_time
                let mut delay_time = 0i32;
                for arg in args.iter() {
                    if arg.id == 0 {
                        if let PropValue::Int(v) = arg.value {
                            delay_time = v;
                        }
                    }
                }
                host.on_bgm_resume(fade, false, delay_time);
                true
            }
            ELM_BGM_RESUME_WAIT => {
                let fade = args
                    .first()
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                // C++ RESUME_WAIT: delay_time always 0
                host.on_bgm_resume(fade, true, 0);
                true
            }
            // Wait / Check / Volume / Query (unchanged from before)
            ELM_BGM_WAIT | ELM_BGM_WAIT_KEY => {
                // C++ p_bgm->wait(key_skip_flag, key_skip_flag).
                // No return value.
                true
            }
            ELM_BGM_WAIT_FADE | ELM_BGM_WAIT_FADE_KEY => {
                // C++ p_bgm->wait_fade(). No return value.
                true
            }
            ELM_BGM_CHECK => {
                // C++ tnm_stack_push_int(p_bgm->check()) — 0 = not playing.
                self.stack.push_int(0);
                true
            }
            ELM_BGM_SET_VOLUME | ELM_BGM_SET_VOLUME_MAX | ELM_BGM_SET_VOLUME_MIN => {
                let vol = args
                    .first()
                    .and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(100);
                host.on_bgm_set_volume(sub, vol);
                true
            }
            ELM_BGM_GET_VOLUME => {
                self.stack.push_int(100);
                true
            }
            ELM_BGM_GET_REGIST_NAME => {
                self.stack.push_str(String::new());
                true
            }
            ELM_BGM_GET_PLAY_POS => {
                self.stack.push_int(0);
                true
            }
            _ => {
                Self::sound_report_invalid_command(host, "bgm", sub);
                true
            }
        }
    }
}
