use super::*;

impl Vm {
    pub(super) fn apply_syscom_option_defaults(&mut self) {
        self.syscom_cfg.global_extra_switch_onoff =
            self.options.default_global_extra_switch.clone();
        self.syscom_cfg.global_extra_mode_values = self.options.default_global_extra_mode.clone();
        self.syscom_cfg.object_disp_onoff = self.options.default_object_disp.clone();
        self.syscom_cfg.local_extra_mode_values =
            self.options.default_local_extra_mode_value.clone();
        self.syscom_cfg.local_extra_mode_enable_flags =
            self.options.default_local_extra_mode_enable.clone();
        self.syscom_cfg.local_extra_mode_exist_flags =
            self.options.default_local_extra_mode_exist.clone();
        self.syscom_cfg.local_extra_switch_onoff_flags =
            self.options.default_local_extra_switch_onoff.clone();
        self.syscom_cfg.local_extra_switch_enable_flags =
            self.options.default_local_extra_switch_enable.clone();
        self.syscom_cfg.local_extra_switch_exist_flags =
            self.options.default_local_extra_switch_exist.clone();
        self.syscom_cfg.charakoe_onoff = self.options.default_charakoe_onoff.clone();
        self.syscom_cfg.charakoe_volume = self.options.default_charakoe_volume.clone();
    }
}

impl Vm {
    fn idx_in_range(idx: i32, cnt: usize) -> bool {
        idx >= 0 && (idx as usize) < cnt
    }
}

impl Vm {
    pub(super) fn try_command_syscom_misc_lowfreq(
        &mut self,
        x: i32,
        args: &[Prop],
        ret_form: i32,
    ) -> Option<bool> {
        let push_ok = |vm: &mut Vm| {
            if ret_form == crate::elm::form::INT {
                vm.stack.push_int(0);
            }
        };
        let push_int = |vm: &mut Vm, v: i32| {
            if ret_form == crate::elm::form::INT {
                vm.stack.push_int(v);
            }
        };
        let object_disp_idx =
            |idx: i32| Self::idx_in_range(idx, self.options.default_object_disp_cnt);
        let global_extra_mode_idx =
            |idx: i32| Self::idx_in_range(idx, self.options.default_global_extra_mode_cnt);
        let global_extra_switch_idx =
            |idx: i32| Self::idx_in_range(idx, self.options.default_global_extra_switch_cnt);
        let local_extra_mode_idx =
            |idx: i32| Self::idx_in_range(idx, self.options.default_local_extra_mode_cnt);
        let local_extra_switch_idx =
            |idx: i32| Self::idx_in_range(idx, self.options.default_local_extra_switch_cnt);
        let charakoe_idx = |idx: i32| Self::idx_in_range(idx, self.options.default_charakoe_cnt);

        match x {
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_MWND_BTN_ENABLE => {
                self.syscom_cfg.mwnd_btn_enabled = 1;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_MWND_BTN_DISABLE => {
                self.syscom_cfg.mwnd_btn_enabled = 0;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_MWND_BTN_TOUCH_ENABLE => {
                self.syscom_cfg.mwnd_btn_touch_enabled = 1;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_MWND_BTN_TOUCH_DISABLE => {
                self.syscom_cfg.mwnd_btn_touch_enabled = 0;
                push_ok(self);
                Some(true)
            }

            y if y == crate::elm::syscom::ELM_SYSCOM_SET_OBJECT_DISP_ONOFF => {
                let idx = Self::arg_int(args, 0);
                let on = if Self::arg_int(args, 1) != 0 { 1 } else { 0 };
                if object_disp_idx(idx) {
                    self.syscom_cfg.object_disp_onoff.insert(idx, on);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_OBJECT_DISP_ONOFF_DEFAULT => {
                let idx = Self::arg_int(args, 0);
                if object_disp_idx(idx) {
                    let dv = self
                        .options
                        .default_object_disp
                        .get(&idx)
                        .copied()
                        .unwrap_or(0);
                    self.syscom_cfg.object_disp_onoff.insert(idx, dv);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_OBJECT_DISP_ONOFF => {
                let idx = Self::arg_int(args, 0);
                let v = if object_disp_idx(idx) {
                    self.syscom_cfg
                        .object_disp_onoff
                        .get(&idx)
                        .copied()
                        .unwrap_or(0)
                } else {
                    0
                };
                push_int(self, v);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_GLOBAL_EXTRA_MODE_VALUE => {
                let idx = Self::arg_int(args, 0);
                let v = Self::arg_int(args, 1);
                if global_extra_mode_idx(idx) {
                    self.syscom_cfg.global_extra_mode_values.insert(idx, v);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_GLOBAL_EXTRA_MODE_VALUE_DEFAULT => {
                let idx = Self::arg_int(args, 0);
                if global_extra_mode_idx(idx) {
                    let dv = self
                        .options
                        .default_global_extra_mode
                        .get(&idx)
                        .copied()
                        .unwrap_or(0);
                    self.syscom_cfg.global_extra_mode_values.insert(idx, dv);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_GLOBAL_EXTRA_MODE_VALUE => {
                let idx = Self::arg_int(args, 0);
                let v = if global_extra_mode_idx(idx) {
                    self.syscom_cfg
                        .global_extra_mode_values
                        .get(&idx)
                        .copied()
                        .unwrap_or(0)
                } else {
                    0
                };
                push_int(self, v);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_GLOBAL_EXTRA_SWITCH_ONOFF => {
                let idx = Self::arg_int(args, 0);
                let on = if Self::arg_int(args, 1) != 0 { 1 } else { 0 };
                if global_extra_switch_idx(idx) {
                    self.syscom_cfg.global_extra_switch_onoff.insert(idx, on);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_GLOBAL_EXTRA_SWITCH_ONOFF_DEFAULT => {
                let idx = Self::arg_int(args, 0);
                if global_extra_switch_idx(idx) {
                    let dv = self
                        .options
                        .default_global_extra_switch
                        .get(&idx)
                        .copied()
                        .unwrap_or(0);
                    self.syscom_cfg.global_extra_switch_onoff.insert(idx, dv);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_GLOBAL_EXTRA_SWITCH_ONOFF => {
                let idx = Self::arg_int(args, 0);
                let v = if global_extra_switch_idx(idx) {
                    self.syscom_cfg
                        .global_extra_switch_onoff
                        .get(&idx)
                        .copied()
                        .unwrap_or(0)
                } else {
                    0
                };
                push_int(self, v);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_LOCAL_EXTRA_SWITCH_ONOFF_FLAG => {
                let idx = Self::arg_int(args, 0);
                let on = if Self::arg_int(args, 1) != 0 { 1 } else { 0 };
                if local_extra_switch_idx(idx) {
                    self.syscom_cfg
                        .local_extra_switch_onoff_flags
                        .insert(idx, on);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_LOCAL_EXTRA_SWITCH_ONOFF_FLAG => {
                let idx = Self::arg_int(args, 0);
                let on = if local_extra_switch_idx(idx) {
                    self.syscom_cfg
                        .local_extra_switch_onoff_flags
                        .get(&idx)
                        .copied()
                        .unwrap_or(0)
                } else {
                    0
                };
                push_int(self, on);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_LOCAL_EXTRA_SWITCH_ENABLE_FLAG => {
                let idx = Self::arg_int(args, 0);
                let on = if Self::arg_int(args, 1) != 0 { 1 } else { 0 };
                if local_extra_switch_idx(idx) {
                    self.syscom_cfg
                        .local_extra_switch_enable_flags
                        .insert(idx, on);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_LOCAL_EXTRA_SWITCH_ENABLE_FLAG => {
                let idx = Self::arg_int(args, 0);
                let on = if local_extra_switch_idx(idx) {
                    self.syscom_cfg
                        .local_extra_switch_enable_flags
                        .get(&idx)
                        .copied()
                        .or_else(|| {
                            self.options
                                .default_local_extra_switch_enable
                                .get(&idx)
                                .copied()
                        })
                        .unwrap_or(1)
                } else {
                    0
                };
                push_int(self, on);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_LOCAL_EXTRA_SWITCH_EXIST_FLAG => {
                let idx = Self::arg_int(args, 0);
                let on = if Self::arg_int(args, 1) != 0 { 1 } else { 0 };
                if local_extra_switch_idx(idx) {
                    self.syscom_cfg
                        .local_extra_switch_exist_flags
                        .insert(idx, on);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_LOCAL_EXTRA_SWITCH_EXIST_FLAG => {
                let idx = Self::arg_int(args, 0);
                let on = if local_extra_switch_idx(idx) {
                    self.syscom_cfg
                        .local_extra_switch_exist_flags
                        .get(&idx)
                        .copied()
                        .or_else(|| {
                            self.options
                                .default_local_extra_switch_exist
                                .get(&idx)
                                .copied()
                        })
                        .unwrap_or(1)
                } else {
                    0
                };
                push_int(self, on);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_CHECK_LOCAL_EXTRA_SWITCH_ENABLE => {
                let idx = Self::arg_int(args, 0);
                let out = if local_extra_switch_idx(idx) {
                    let enable = self
                        .syscom_cfg
                        .local_extra_switch_enable_flags
                        .get(&idx)
                        .copied()
                        .or_else(|| {
                            self.options
                                .default_local_extra_switch_enable
                                .get(&idx)
                                .copied()
                        })
                        .unwrap_or(1)
                        != 0;
                    let exist = self
                        .syscom_cfg
                        .local_extra_switch_exist_flags
                        .get(&idx)
                        .copied()
                        .or_else(|| {
                            self.options
                                .default_local_extra_switch_exist
                                .get(&idx)
                                .copied()
                        })
                        .unwrap_or(1)
                        != 0;
                    if enable && exist { 1 } else { 0 }
                } else {
                    0
                };
                push_int(self, out);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_LOCAL_EXTRA_MODE_VALUE => {
                let mode_no = Self::arg_int(args, 0);
                let value = Self::arg_int(args, 1);
                if local_extra_mode_idx(mode_no) {
                    self.syscom_cfg
                        .local_extra_mode_values
                        .insert(mode_no, value);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_LOCAL_EXTRA_MODE_VALUE => {
                let mode_no = Self::arg_int(args, 0);
                let value = if local_extra_mode_idx(mode_no) {
                    self.syscom_cfg
                        .local_extra_mode_values
                        .get(&mode_no)
                        .copied()
                        .or_else(|| {
                            self.options
                                .default_local_extra_mode_value
                                .get(&mode_no)
                                .copied()
                        })
                        .unwrap_or(0)
                } else {
                    0
                };
                push_int(self, value);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_LOCAL_EXTRA_MODE_ENABLE_FLAG => {
                let mode_no = Self::arg_int(args, 0);
                let on = if Self::arg_int(args, 1) != 0 { 1 } else { 0 };
                if local_extra_mode_idx(mode_no) {
                    self.syscom_cfg
                        .local_extra_mode_enable_flags
                        .insert(mode_no, on);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_LOCAL_EXTRA_MODE_ENABLE_FLAG => {
                let mode_no = Self::arg_int(args, 0);
                let on = if local_extra_mode_idx(mode_no) {
                    self.syscom_cfg
                        .local_extra_mode_enable_flags
                        .get(&mode_no)
                        .copied()
                        .or_else(|| {
                            self.options
                                .default_local_extra_mode_enable
                                .get(&mode_no)
                                .copied()
                        })
                        .unwrap_or(1)
                } else {
                    0
                };
                push_int(self, on);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_LOCAL_EXTRA_MODE_EXIST_FLAG => {
                let mode_no = Self::arg_int(args, 0);
                let on = if Self::arg_int(args, 1) != 0 { 1 } else { 0 };
                if local_extra_mode_idx(mode_no) {
                    self.syscom_cfg
                        .local_extra_mode_exist_flags
                        .insert(mode_no, on);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_LOCAL_EXTRA_MODE_EXIST_FLAG => {
                let mode_no = Self::arg_int(args, 0);
                let on = if local_extra_mode_idx(mode_no) {
                    self.syscom_cfg
                        .local_extra_mode_exist_flags
                        .get(&mode_no)
                        .copied()
                        .or_else(|| {
                            self.options
                                .default_local_extra_mode_exist
                                .get(&mode_no)
                                .copied()
                        })
                        .unwrap_or(1)
                } else {
                    0
                };
                push_int(self, on);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_CHECK_LOCAL_EXTRA_MODE_ENABLE => {
                let mode_no = Self::arg_int(args, 0);
                let out = if local_extra_mode_idx(mode_no) {
                    let enable = self
                        .syscom_cfg
                        .local_extra_mode_enable_flags
                        .get(&mode_no)
                        .copied()
                        .or_else(|| {
                            self.options
                                .default_local_extra_mode_enable
                                .get(&mode_no)
                                .copied()
                        })
                        .unwrap_or(1)
                        != 0;
                    let exist = self
                        .syscom_cfg
                        .local_extra_mode_exist_flags
                        .get(&mode_no)
                        .copied()
                        .or_else(|| {
                            self.options
                                .default_local_extra_mode_exist
                                .get(&mode_no)
                                .copied()
                        })
                        .unwrap_or(1)
                        != 0;
                    if enable && exist { 1 } else { 0 }
                } else {
                    0
                };
                push_int(self, out);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_BGMFADE_VOLUME => {
                self.syscom_cfg.bgmfade_volume = Self::arg_int(args, 0).clamp(0, 255);
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_BGMFADE_VOLUME_DEFAULT => {
                self.syscom_cfg.bgmfade_volume = 255;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_BGMFADE_VOLUME => {
                push_int(self, self.syscom_cfg.bgmfade_volume);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_BGMFADE_ONOFF => {
                self.syscom_cfg.bgmfade_onoff = if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_BGMFADE_ONOFF_DEFAULT => {
                self.syscom_cfg.bgmfade_onoff = 1;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_BGMFADE_ONOFF => {
                push_int(self, self.syscom_cfg.bgmfade_onoff);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_KOEMODE => {
                let v = Self::arg_int(args, 0);
                self.syscom_cfg.koemode = if (0..=2).contains(&v) { v } else { 0 };
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_KOEMODE_DEFAULT => {
                self.syscom_cfg.koemode = 0;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_KOEMODE => {
                push_int(self, self.syscom_cfg.koemode);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_CHARAKOE_ONOFF => {
                let idx = Self::arg_int(args, 0);
                let on = if Self::arg_int(args, 1) != 0 { 1 } else { 0 };
                if charakoe_idx(idx) {
                    self.syscom_cfg.charakoe_onoff.insert(idx, on);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_CHARAKOE_ONOFF_DEFAULT => {
                let idx = Self::arg_int(args, 0);
                if charakoe_idx(idx) {
                    let dv = self
                        .options
                        .default_charakoe_onoff
                        .get(&idx)
                        .copied()
                        .unwrap_or(0);
                    self.syscom_cfg.charakoe_onoff.insert(idx, dv);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_CHARAKOE_ONOFF => {
                let idx = Self::arg_int(args, 0);
                let v = if charakoe_idx(idx) {
                    self.syscom_cfg
                        .charakoe_onoff
                        .get(&idx)
                        .copied()
                        .or_else(|| self.options.default_charakoe_onoff.get(&idx).copied())
                        .unwrap_or(0)
                } else {
                    0
                };
                push_int(self, v);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_CHARAKOE_VOLUME => {
                let idx = Self::arg_int(args, 0);
                let vol = Self::arg_int(args, 1).clamp(0, 255);
                if charakoe_idx(idx) {
                    self.syscom_cfg.charakoe_volume.insert(idx, vol);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_CHARAKOE_VOLUME_DEFAULT => {
                let idx = Self::arg_int(args, 0);
                if charakoe_idx(idx) {
                    let dv = self
                        .options
                        .default_charakoe_volume
                        .get(&idx)
                        .copied()
                        .unwrap_or(255);
                    self.syscom_cfg.charakoe_volume.insert(idx, dv);
                }
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_CHARAKOE_VOLUME => {
                let idx = Self::arg_int(args, 0);
                let v = if charakoe_idx(idx) {
                    self.syscom_cfg
                        .charakoe_volume
                        .get(&idx)
                        .copied()
                        .or_else(|| self.options.default_charakoe_volume.get(&idx).copied())
                        .unwrap_or(0)
                } else {
                    0
                };
                push_int(self, v);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_JITAN_NORMAL_ONOFF => {
                self.syscom_cfg.jitan_normal_onoff =
                    if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_JITAN_NORMAL_ONOFF_DEFAULT => {
                self.syscom_cfg.jitan_normal_onoff = 0;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_JITAN_NORMAL_ONOFF => {
                push_int(self, self.syscom_cfg.jitan_normal_onoff);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_JITAN_AUTO_MODE_ONOFF => {
                self.syscom_cfg.jitan_auto_mode_onoff =
                    if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_JITAN_AUTO_MODE_ONOFF_DEFAULT => {
                self.syscom_cfg.jitan_auto_mode_onoff = 0;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_JITAN_AUTO_MODE_ONOFF => {
                push_int(self, self.syscom_cfg.jitan_auto_mode_onoff);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_JITAN_KOE_REPLAY_ONOFF => {
                self.syscom_cfg.jitan_koe_replay_onoff =
                    if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_JITAN_KOE_REPLAY_ONOFF_DEFAULT => {
                self.syscom_cfg.jitan_koe_replay_onoff = 0;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_JITAN_KOE_REPLAY_ONOFF => {
                push_int(self, self.syscom_cfg.jitan_koe_replay_onoff);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_JITAN_SPEED => {
                self.syscom_cfg.jitan_speed = Self::arg_int(args, 0).clamp(0, 1000);
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_JITAN_SPEED_DEFAULT => {
                self.syscom_cfg.jitan_speed = 0;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_JITAN_SPEED => {
                push_int(self, self.syscom_cfg.jitan_speed);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_SLEEP_ONOFF => {
                self.syscom_cfg.sleep_onoff = if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_SLEEP_ONOFF_DEFAULT => {
                self.syscom_cfg.sleep_onoff = 0;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_SLEEP_ONOFF => {
                push_int(self, self.syscom_cfg.sleep_onoff);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_WHEEL_NEXT_MESSAGE_ONOFF => {
                self.syscom_cfg.wheel_next_message_onoff =
                    if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_WHEEL_NEXT_MESSAGE_ONOFF_DEFAULT => {
                self.syscom_cfg.wheel_next_message_onoff = 0;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_WHEEL_NEXT_MESSAGE_ONOFF => {
                push_int(self, self.syscom_cfg.wheel_next_message_onoff);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_KOE_DONT_STOP_ONOFF => {
                self.syscom_cfg.koe_dont_stop_onoff =
                    if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_KOE_DONT_STOP_ONOFF_DEFAULT => {
                self.syscom_cfg.koe_dont_stop_onoff = 0;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_KOE_DONT_STOP_ONOFF => {
                push_int(self, self.syscom_cfg.koe_dont_stop_onoff);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_SKIP_UNREAD_MESSAGE_ONOFF => {
                self.syscom_cfg.skip_unread_message_onoff =
                    if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_SET_SKIP_UNREAD_MESSAGE_ONOFF_DEFAULT => {
                self.syscom_cfg.skip_unread_message_onoff = 0;
                push_ok(self);
                Some(true)
            }
            y if y == crate::elm::syscom::ELM_SYSCOM_GET_SKIP_UNREAD_MESSAGE_ONOFF => {
                push_int(self, self.syscom_cfg.skip_unread_message_onoff);
                Some(true)
            }
            _ => None,
        }
    }
}
