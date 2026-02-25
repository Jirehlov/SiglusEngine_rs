use super::*;

impl Vm {
    pub(super) fn try_command_syscom_misc(
        &mut self,
        x: i32,
        args: &[Prop],
        ret_form: i32,
        provider: &mut dyn SceneProvider,
        host: &mut dyn Host,
    ) -> Result<Option<bool>> {
        if let Some(handled) = self.try_command_syscom_misc_lowfreq(x, args, ret_form) {
            return Ok(Some(handled));
        }
        match x {
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_RETURN_SCENE_ONCE => {
                let scene_name = match args.first().map(|p| &p.value) {
                    Some(PropValue::Str(s)) => s.clone(),
                    _ => String::new(),
                };
                let scene_z_no = match args.get(1).map(|p| &p.value) {
                    Some(PropValue::Int(v)) => *v,
                    _ => 0,
                };
                self.return_scene_once = Some((scene_name, scene_z_no));
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_WINDOW_MODE => {
                self.syscom_cfg.window_mode = Self::arg_int(args, 0);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_WINDOW_MODE_DEFAULT => {
                self.syscom_cfg.window_mode = 0;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_WINDOW_MODE => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.window_mode);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_WINDOW_MODE_SIZE => {
                self.syscom_cfg.window_mode_size = Self::arg_int(args, 0);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_WINDOW_MODE_SIZE_DEFAULT => {
                self.syscom_cfg.window_mode_size = 0;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_WINDOW_MODE_SIZE => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.window_mode_size);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_CHECK_WINDOW_MODE_SIZE_ENABLE => {
                if ret_form == crate::elm::form::INT {
                    self.stack
                        .push_int(if self.syscom_cfg.window_mode_size >= 0 {
                            1
                        } else {
                            0
                        });
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_SAVELOAD_ALERT_ONOFF => {
                self.syscom_cfg.saveload_alert_onoff = Self::arg_int(args, 0);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_SAVELOAD_ALERT_ONOFF_DEFAULT => {
                self.syscom_cfg.saveload_alert_onoff = 0;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_SAVELOAD_ALERT_ONOFF => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.saveload_alert_onoff);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_NO_MWND_ANIME_ONOFF => {
                self.syscom_cfg.no_mwnd_anime_onoff = Self::arg_int(args, 0);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_NO_MWND_ANIME_ONOFF_DEFAULT => {
                self.syscom_cfg.no_mwnd_anime_onoff = 0;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_NO_MWND_ANIME_ONOFF => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.no_mwnd_anime_onoff);
                }
                Ok(Some(true))
            }
            y if (crate::elm::syscom::ELM_SYSCOM_SET_FILTER_COLOR_R
                ..=crate::elm::syscom::ELM_SYSCOM_SET_FILTER_COLOR_A)
                .contains(&y) =>
            {
                let idx = (y - crate::elm::syscom::ELM_SYSCOM_SET_FILTER_COLOR_R) as usize;
                self.syscom_cfg.filter_color[idx] = Self::arg_int(args, 0);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if (crate::elm::syscom::ELM_SYSCOM_SET_FILTER_COLOR_R_DEFAULT
                ..=crate::elm::syscom::ELM_SYSCOM_SET_FILTER_COLOR_A_DEFAULT)
                .contains(&y) =>
            {
                let idx = (y - crate::elm::syscom::ELM_SYSCOM_SET_FILTER_COLOR_R_DEFAULT) as usize;
                self.syscom_cfg.filter_color[idx] = 0;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if (crate::elm::syscom::ELM_SYSCOM_GET_FILTER_COLOR_R
                ..=crate::elm::syscom::ELM_SYSCOM_GET_FILTER_COLOR_A)
                .contains(&y) =>
            {
                let idx = (y - crate::elm::syscom::ELM_SYSCOM_GET_FILTER_COLOR_R) as usize;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.filter_color[idx]);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_FONT_NAME => {
                self.syscom_cfg.font_name = match args.first().map(|p| &p.value) {
                    Some(PropValue::Str(v)) => v.clone(),
                    _ => String::new(),
                };
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_FONT_NAME_DEFAULT => {
                self.syscom_cfg.font_name.clear();
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_FONT_NAME => {
                if ret_form == crate::elm::form::STR {
                    self.stack.push_str(self.syscom_cfg.font_name.clone());
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_IS_FONT_EXIST => {
                let target_name = match args.first().map(|p| &p.value) {
                    Some(PropValue::Str(v)) => v.as_str(),
                    _ => "",
                };
                let exists = if target_name.is_empty() {
                    !self.syscom_cfg.font_name.is_empty()
                } else {
                    true
                };
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(if exists { 1 } else { 0 });
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_FONT_BOLD => {
                self.syscom_cfg.font_bold = Self::arg_int(args, 0);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_FONT_BOLD_DEFAULT => {
                self.syscom_cfg.font_bold = -1;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_FONT_BOLD => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.font_bold);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_FONT_DECORATION => {
                self.syscom_cfg.font_decoration = Self::arg_int(args, 0);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_FONT_DECORATION_DEFAULT => {
                self.syscom_cfg.font_decoration = -1;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_FONT_DECORATION => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.font_decoration);
                }
                Ok(Some(true))
            }

            y if y == crate::elm::syscom::ELM_SYSCOM_SET_BGM_VOLUME => {
                self.syscom_cfg.bgm_volume = Self::arg_int(args, 0).clamp(0, 100);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_BGM_VOLUME_DEFAULT => {
                self.syscom_cfg.bgm_volume = 100;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_BGM_VOLUME => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.bgm_volume);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_KOE_VOLUME => {
                self.syscom_cfg.koe_volume = Self::arg_int(args, 0).clamp(0, 100);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_KOE_VOLUME_DEFAULT => {
                self.syscom_cfg.koe_volume = 100;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_KOE_VOLUME => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.koe_volume);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_PCM_VOLUME => {
                self.syscom_cfg.pcm_volume = Self::arg_int(args, 0).clamp(0, 100);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_PCM_VOLUME_DEFAULT => {
                self.syscom_cfg.pcm_volume = 100;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_PCM_VOLUME => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.pcm_volume);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_SE_VOLUME => {
                self.syscom_cfg.se_volume = Self::arg_int(args, 0).clamp(0, 100);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_SE_VOLUME_DEFAULT => {
                self.syscom_cfg.se_volume = 100;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_SE_VOLUME => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.se_volume);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_ALL_VOLUME => {
                self.syscom_cfg.all_volume = Self::arg_int(args, 0).clamp(0, 100);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_ALL_VOLUME_DEFAULT => {
                self.syscom_cfg.all_volume = 100;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_ALL_VOLUME => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.all_volume);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_BGM_ONOFF => {
                self.syscom_cfg.bgm_onoff = if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_BGM_ONOFF => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.bgm_onoff);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_KOE_ONOFF => {
                self.syscom_cfg.koe_onoff = if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_KOE_ONOFF => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.koe_onoff);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_PCM_ONOFF => {
                self.syscom_cfg.pcm_onoff = if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_PCM_ONOFF => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.pcm_onoff);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_SE_ONOFF => {
                self.syscom_cfg.se_onoff = if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_SE_ONOFF => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.se_onoff);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_ALL_ONOFF => {
                self.syscom_cfg.all_onoff = if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_ALL_ONOFF => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.all_onoff);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_MESSAGE_SPEED => {
                self.syscom_cfg.message_speed = Self::arg_int(args, 0);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_MESSAGE_SPEED_DEFAULT => {
                self.syscom_cfg.message_speed = -1;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_MESSAGE_SPEED => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.message_speed);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_MESSAGE_NOWAIT => {
                self.syscom_cfg.message_nowait = if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_MESSAGE_NOWAIT => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.message_nowait);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_AUTO_MODE_MOJI_WAIT => {
                self.syscom_cfg.auto_mode_moji_wait = Self::arg_int(args, 0);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_AUTO_MODE_MOJI_WAIT_DEFAULT => {
                self.syscom_cfg.auto_mode_moji_wait = -1;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_AUTO_MODE_MOJI_WAIT => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.auto_mode_moji_wait);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_AUTO_MODE_MIN_WAIT => {
                self.syscom_cfg.auto_mode_min_wait = Self::arg_int(args, 0);
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_AUTO_MODE_MIN_WAIT_DEFAULT => {
                self.syscom_cfg.auto_mode_min_wait = -1;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_AUTO_MODE_MIN_WAIT => {
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(self.syscom_cfg.auto_mode_min_wait);
                }
                Ok(Some(true))
            }

            y if y == crate::elm::syscom::ELM_SYSCOM_GET_SYSTEM_EXTRA_INT_VALUE => {
                if ret_form == crate::elm::form::INT {
                    let idx = Self::arg_int(args, 0);
                    let v = if idx >= 0 {
                        self.system_extra_int_values
                            .get(idx as usize)
                            .copied()
                            .unwrap_or(0)
                    } else {
                        0
                    };
                    self.stack.push_int(v);
                }
                Ok(Some(true))
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_SYSTEM_EXTRA_STR_VALUE => {
                let idx = Self::arg_int(args, 0);
                if ret_form == crate::elm::form::STR {
                    let s = if idx >= 0 && (idx as usize) < self.system_extra_int_values.len() {
                        self.system_extra_str_values
                            .get(idx as usize)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    self.stack.push_str(s);
                } else if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if crate::elm::syscom::is_open_dialog(y) => {
                host.on_open_tweet_dialog();
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            y if crate::elm::syscom::is_call_ex(y) => {
                let scene = match args.first().map(|p| &p.value) {
                    Some(PropValue::Str(s)) => s.as_str(),
                    _ => "",
                };
                let z_no = match args.get(1).map(|p| &p.value) {
                    Some(PropValue::Int(v)) => *v,
                    _ => 0,
                };
                if !scene.is_empty() {
                    let call_args = if args.len() >= 2 { &args[2..] } else { &[] };
                    self.proc_farcall_like(
                        scene,
                        z_no,
                        crate::elm::form::VOID,
                        call_args,
                        true,
                        provider,
                    )?;
                }
                Ok(Some(true))
            }
            _ => Ok(None),
        }
    }
}
