use super::*;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
impl Vm {
    pub(super) fn arg_int(args: &[Prop], idx: usize) -> i32 {
        match args.get(idx).map(|p| &p.value) {
            Some(PropValue::Int(v)) => *v,
            _ => 0,
        }
    }
    fn unix_ms_to_stamp(unix_ms: i64) -> LocalSaveStamp {
        fn civil_from_days(z: i64) -> (i32, i32, i32) {
            let z = z + 719_468;
            let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
            let doe = z - era * 146_097;
            let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
            let y = yoe + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let d = doy - (153 * mp + 2) / 5 + 1;
            let m = mp + if mp < 10 { 3 } else { -9 };
            let y = y + if m <= 2 { 1 } else { 0 };
            (y as i32, m as i32, d as i32)
        }
        let unix_sec = unix_ms.div_euclid(1000);
        let ms = unix_ms.rem_euclid(1000) as i32;
        let days = unix_sec.div_euclid(86_400);
        let secs_of_day = unix_sec.rem_euclid(86_400);
        let hour = (secs_of_day / 3600) as i32;
        let minute = ((secs_of_day % 3600) / 60) as i32;
        let second = (secs_of_day % 60) as i32;
        let (year, month, day) = civil_from_days(days);
        let weekday = ((days + 4).rem_euclid(7)) as i32;
        LocalSaveStamp {
            year,
            month,
            day,
            weekday,
            hour,
            minute,
            second,
            millisecond: ms,
        }
    }
    pub(super) fn make_local_slot(&self) -> LocalSaveSlot {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let stamp =
            Self::unix_ms_to_stamp((now.as_secs() as i64) * 1000 + now.subsec_millis() as i64);
        LocalSaveSlot {
            stamp,
            scene_title: self.scene_title.clone(),
            message: self.last_sel_msg.clone(),
            state: self.snapshot_local_state(),
        }
    }
    fn first_empty_slot(map: &BTreeMap<i32, LocalSaveSlot>, cnt: i32) -> i32 {
        for i in 0..cnt.max(0) {
            if !map.contains_key(&i) {
                return i;
            }
        }
        -1
    }
    fn slot_arg(args: &[Prop], idx: usize) -> Option<i32> {
        let slot_no = Self::arg_int(args, idx);
        if slot_no < 0 {
            return None;
        }
        Some(slot_no)
    }
    pub(super) fn try_command_syscom(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        provider: &mut dyn SceneProvider,
        host: &mut dyn Host,
    ) -> Result<Option<bool>> {
        let x = element[0];
        match x {
            x if crate::elm::syscom::is_set_hide_mwnd_flag(x) => {
                let on = if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                match x {
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_HIDE_MWND_ONOFF_FLAG => {
                        self.hide_mwnd_onoff_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_HIDE_MWND_ENABLE_FLAG => {
                        self.hide_mwnd_enable_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_HIDE_MWND_EXIST_FLAG => {
                        self.hide_mwnd_exist_flag = on
                    }
                    _ => {}
                }
                if ret_form == crate::elm::form::INT { self.stack.push_int(0); }
                Ok(Some(true))
            }
            x if crate::elm::syscom::is_set_se_volume(x) => {
                if ret_form == crate::elm::form::INT { self.stack.push_int(0); }
                Ok(Some(true))
            }
            x if crate::elm::syscom::is_set_wipe_anime_onoff(x) => {
                match x {
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_NO_WIPE_ANIME_ONOFF => {
                        self.no_wipe_anime_onoff_flag =
                            if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                        self.options.no_wipe_anime = self.no_wipe_anime_onoff_flag != 0;
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_NO_WIPE_ANIME_ONOFF_DEFAULT => {
                        self.no_wipe_anime_onoff_flag = if self.options.no_wipe_anime_default {
                            1
                        } else {
                            0
                        };
                        self.options.no_wipe_anime = self.options.no_wipe_anime_default;
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_SKIP_WIPE_ANIME_ONOFF => {
                        self.skip_wipe_anime_onoff_flag =
                            if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                        self.options.skip_wipe_anime = self.skip_wipe_anime_onoff_flag != 0;
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_SKIP_WIPE_ANIME_ONOFF_DEFAULT => {
                        self.skip_wipe_anime_onoff_flag = if self.options.skip_wipe_anime_default {
                            1
                        } else {
                            0
                        };
                        self.options.skip_wipe_anime = self.options.skip_wipe_anime_default;
                    }
                    _ => {}
                }
                if ret_form == crate::elm::form::INT { self.stack.push_int(0); }
                Ok(Some(true))
            }
            x if x == crate::elm::syscom::ELM_SYSCOM_INIT_SYSCOM_FLAG => {
                self.hide_mwnd_enable_flag = 1;
                self.hide_mwnd_exist_flag = 1;
                self.read_skip_onoff_flag = 0;
                self.read_skip_enable_flag = 1;
                self.read_skip_exist_flag = 1;
                self.auto_mode_onoff_flag = 0;
                self.auto_mode_enable_flag = 1;
                self.auto_mode_exist_flag = 1;
                self.msg_back_enable_flag = 1;
                self.msg_back_exist_flag = 1;
                self.msg_back_open_flag = 0;
                self.return_to_sel_enable_flag = 1;
                self.return_to_sel_exist_flag = 1;
                self.return_to_menu_enable_flag = 1;
                self.return_to_menu_exist_flag = 1;
                self.save_enable_flag = 1;
                self.save_exist_flag = 1;
                self.load_enable_flag = 1;
                self.load_exist_flag = 1;
                self.end_game_enable_flag = 1;
                self.end_game_exist_flag = 1;
                self.game_end_flag = 0;
                self.game_end_no_warning_flag = 0;
                self.game_end_save_done_flag = 0;
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            x if crate::elm::syscom::is_msg_back_dialog_control(x) => {
                match x {
                    y if y == crate::elm::syscom::ELM_SYSCOM_OPEN_MSG_BACK => {
                        self.msg_back_open_flag = if self.msg_back_enable_flag != 0
                            && self.msg_back_exist_flag != 0
                            && self.msg_back_has_message != 0
                            && self.msg_back_disable_flag == 0
                        {
                            self.read_skip_onoff_flag = 0;
                            1
                        } else {
                            0
                        };
                        host.on_msg_back_state(
                            self.msg_back_open_flag != 0 && self.msg_back_disp_off_flag == 0,
                        );
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_CLOSE_MSG_BACK => {
                        self.msg_back_open_flag = 0;
                        host.on_msg_back_state(false);
                    }
                    _ => {}
                }
                if ret_form == crate::elm::form::INT { self.stack.push_int(0); }
                Ok(Some(true))
            }
            x if crate::elm::syscom::is_set_enable_flag(x) => {
                let on = if Self::arg_int(args, 0) != 0 { 1 } else { 0 };
                match x {
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_READ_SKIP_ONOFF_FLAG => {
                        self.read_skip_onoff_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_READ_SKIP_ENABLE_FLAG => {
                        self.read_skip_enable_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_READ_SKIP_EXIST_FLAG => {
                        self.read_skip_exist_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_AUTO_MODE_ONOFF_FLAG => {
                        self.auto_mode_onoff_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_AUTO_MODE_ENABLE_FLAG => {
                        self.auto_mode_enable_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_AUTO_MODE_EXIST_FLAG => {
                        self.auto_mode_exist_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_MSG_BACK_ENABLE_FLAG => {
                        self.msg_back_enable_flag = on;
                        if on == 0 {
                            self.msg_back_open_flag = 0;
                            host.on_msg_back_state(false);
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_MSG_BACK_EXIST_FLAG => {
                        self.msg_back_exist_flag = on;
                        if on == 0 {
                            self.msg_back_open_flag = 0;
                            host.on_msg_back_state(false);
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_RETURN_TO_SEL_ENABLE_FLAG => {
                        self.return_to_sel_enable_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_RETURN_TO_SEL_EXIST_FLAG => {
                        self.return_to_sel_exist_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_RETURN_TO_MENU_ENABLE_FLAG => {
                        self.return_to_menu_enable_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_RETURN_TO_MENU_EXIST_FLAG => {
                        self.return_to_menu_exist_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_SAVE_ENABLE_FLAG => {
                        self.save_enable_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_SAVE_EXIST_FLAG => {
                        self.save_exist_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_LOAD_ENABLE_FLAG => {
                        self.load_enable_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_LOAD_EXIST_FLAG => {
                        self.load_exist_flag = on
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_END_GAME_ENABLE_FLAG => self.end_game_enable_flag = on,
                    y if y == crate::elm::syscom::ELM_SYSCOM_SET_END_GAME_EXIST_FLAG => self.end_game_exist_flag = on,
                    _ => {}
                }
                if ret_form == crate::elm::form::INT { self.stack.push_int(0); }
                Ok(Some(true))
            }
            x if crate::elm::syscom::is_save_or_load(x) => {
                let slot_no = Self::slot_arg(args, 0);
                let ok = match x {
                    y if y == crate::elm::syscom::ELM_SYSCOM_SAVE => {
                        if let Some(slot_no) = slot_no {
                            self.local_save_slots
                                .insert(slot_no, self.make_local_slot());
                            true
                        } else {
                            false
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_QUICK_SAVE => {
                        if let Some(slot_no) = slot_no {
                            self.quick_save_slots
                                .insert(slot_no, self.make_local_slot());
                            true
                        } else {
                            false
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_INNER_SAVE => {
                        if let Some(slot_no) = slot_no {
                            self.inner_save_slots
                                .insert(slot_no, self.make_local_slot());
                            true
                        } else {
                            false
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_LOAD => {
                        if let Some(slot) =
                            slot_no.and_then(|slot_no| self.local_save_slots.get(&slot_no).cloned())
                        {
                            self.apply_local_state(&slot.state);
                            true
                        } else {
                            false
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_QUICK_LOAD => {
                        if let Some(slot) =
                            slot_no.and_then(|slot_no| self.quick_save_slots.get(&slot_no).cloned())
                        {
                            self.apply_local_state(&slot.state);
                            true
                        } else {
                            false
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_INNER_LOAD => {
                        if let Some(slot) =
                            slot_no.and_then(|slot_no| self.inner_save_slots.get(&slot_no).cloned())
                        {
                            self.apply_local_state(&slot.state);
                            true
                        } else {
                            false
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_CLEAR_INNER_SAVE => slot_no
                        .map(|slot_no| self.inner_save_slots.remove(&slot_no).is_some())
                        .unwrap_or(false),
                    y if y == crate::elm::syscom::ELM_SYSCOM_COPY_INNER_SAVE => {
                        let dst = Self::slot_arg(args, 1);
                        if let Some((dst, v)) = dst.zip(
                            slot_no
                                .and_then(|slot_no| self.inner_save_slots.get(&slot_no).cloned()),
                        ) {
                            self.inner_save_slots.insert(dst, v);
                            true
                        } else {
                            false
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_COPY_SAVE => {
                        let dst = Self::slot_arg(args, 1);
                        if let Some((dst, v)) = dst.zip(
                            slot_no
                                .and_then(|slot_no| self.local_save_slots.get(&slot_no).cloned()),
                        ) {
                            self.local_save_slots.insert(dst, v);
                            true
                        } else {
                            false
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_COPY_QUICK_SAVE => {
                        let dst = Self::slot_arg(args, 1);
                        if let Some((dst, v)) = dst.zip(
                            slot_no
                                .and_then(|slot_no| self.quick_save_slots.get(&slot_no).cloned()),
                        ) {
                            self.quick_save_slots.insert(dst, v);
                            true
                        } else {
                            false
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_CHANGE_SAVE => {
                        let dst = Self::slot_arg(args, 1);
                        if let Some((dst, v)) = dst
                            .zip(slot_no.and_then(|slot_no| self.local_save_slots.remove(&slot_no)))
                        {
                            self.local_save_slots.insert(dst, v);
                            true
                        } else {
                            false
                        }
                    }
                    y if y == crate::elm::syscom::ELM_SYSCOM_CHANGE_QUICK_SAVE => {
                        let dst = Self::slot_arg(args, 1);
                        if let Some((dst, v)) = dst
                            .zip(slot_no.and_then(|slot_no| self.quick_save_slots.remove(&slot_no)))
                        {
                            self.quick_save_slots.insert(dst, v);
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(if ok { 1 } else { 0 });
                }
                Ok(Some(true))
            }
            x if crate::elm::syscom::is_query_int(x) => {
                if ret_form == crate::elm::form::INT {
                    let slot_no = Self::slot_arg(args, 0);
                    let val = if crate::elm::syscom::is_hide_mwnd_query(x) {
                        if x == crate::elm::syscom::ELM_SYSCOM_GET_HIDE_MWND_ONOFF_FLAG {
                            self.hide_mwnd_onoff_flag
                        } else if x == crate::elm::syscom::ELM_SYSCOM_GET_HIDE_MWND_ENABLE_FLAG {
                            self.hide_mwnd_enable_flag
                        } else if x == crate::elm::syscom::ELM_SYSCOM_GET_HIDE_MWND_EXIST_FLAG {
                            self.hide_mwnd_exist_flag
                        } else {
                            if self.hide_mwnd_enable_flag != 0 && self.hide_mwnd_exist_flag != 0 {
                                1
                            } else {
                                0
                            }
                        }
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_READ_SKIP_ONOFF_FLAG {
                        self.read_skip_onoff_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_READ_SKIP_ENABLE_FLAG {
                        self.read_skip_enable_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_READ_SKIP_EXIST_FLAG {
                        self.read_skip_exist_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_AUTO_MODE_ONOFF_FLAG {
                        self.auto_mode_onoff_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_AUTO_MODE_ENABLE_FLAG {
                        self.auto_mode_enable_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_AUTO_MODE_EXIST_FLAG {
                        self.auto_mode_exist_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_MSG_BACK_ENABLE_FLAG {
                        self.msg_back_enable_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_MSG_BACK_EXIST_FLAG {
                        self.msg_back_exist_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_CHECK_MSG_BACK_OPEN {
                        self.msg_back_open_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_RETURN_TO_SEL_ENABLE_FLAG {
                        self.return_to_sel_enable_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_RETURN_TO_SEL_EXIST_FLAG {
                        self.return_to_sel_exist_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_RETURN_TO_MENU_ENABLE_FLAG {
                        self.return_to_menu_enable_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_RETURN_TO_MENU_EXIST_FLAG {
                        self.return_to_menu_exist_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_ENABLE_FLAG {
                        self.save_enable_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_EXIST_FLAG {
                        self.save_exist_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_LOAD_ENABLE_FLAG {
                        self.load_enable_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_LOAD_EXIST_FLAG {
                        self.load_exist_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_END_GAME_ENABLE_FLAG { self.end_game_enable_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_END_GAME_EXIST_FLAG { self.end_game_exist_flag
                    } else if crate::elm::syscom::is_feature_check_query(x) {
                        if x == crate::elm::syscom::ELM_SYSCOM_CHECK_READ_SKIP_ENABLE {
                            (self.read_skip_enable_flag != 0 && self.read_skip_exist_flag != 0)
                                as i32
                        } else if x == crate::elm::syscom::ELM_SYSCOM_CHECK_AUTO_MODE_ENABLE {
                            (self.auto_mode_enable_flag != 0 && self.auto_mode_exist_flag != 0)
                                as i32
                        } else if x == crate::elm::syscom::ELM_SYSCOM_CHECK_MSG_BACK_ENABLE {
                            (self.msg_back_enable_flag != 0
                                && self.msg_back_exist_flag != 0
                                && self.msg_back_has_message != 0
                                && self.msg_back_disable_flag == 0)
                                as i32
                        } else if x == crate::elm::syscom::ELM_SYSCOM_CHECK_RETURN_TO_SEL_ENABLE {
                            (self.return_to_sel_enable_flag != 0
                                && self.return_to_sel_exist_flag != 0)
                                as i32
                        } else if x == crate::elm::syscom::ELM_SYSCOM_CHECK_RETURN_TO_MENU_ENABLE {
                            (self.return_to_menu_enable_flag != 0
                                && self.return_to_menu_exist_flag != 0)
                                as i32
                        } else if x == crate::elm::syscom::ELM_SYSCOM_CHECK_SAVE_ENABLE {
                            (self.save_enable_flag != 0 && self.save_exist_flag != 0) as i32
                        } else if x == crate::elm::syscom::ELM_SYSCOM_CHECK_END_GAME_ENABLE {
                            (self.end_game_enable_flag != 0 && self.end_game_exist_flag != 0) as i32
                        } else {
                            (self.load_enable_flag != 0 && self.load_exist_flag != 0) as i32
                        }
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_END_SAVE_EXIST {
                        slot_no
                            .map(|slot_no| host.on_syscom_end_save_exist(slot_no).unwrap_or(self.end_save_slots.contains_key(&slot_no)) as i32)
                            .unwrap_or(0)
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_NO_WIPE_ANIME_ONOFF {
                        self.no_wipe_anime_onoff_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SKIP_WIPE_ANIME_ONOFF {
                        self.skip_wipe_anime_onoff_flag
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_CNT {
                        100
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_CNT {
                        10
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_EXIST {
                        slot_no
                            .map(|slot_no| self.local_save_slots.contains_key(&slot_no) as i32)
                            .unwrap_or(0)
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_EXIST {
                        slot_no
                            .map(|slot_no| self.quick_save_slots.contains_key(&slot_no) as i32)
                            .unwrap_or(0)
                    } else if x == crate::elm::syscom::ELM_SYSCOM_CHECK_INNER_SAVE {
                        slot_no
                            .map(|slot_no| self.inner_save_slots.contains_key(&slot_no) as i32)
                            .unwrap_or(0)
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_NEW_NO {
                        Self::first_empty_slot(&self.local_save_slots, 100)
                    } else if x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_NEW_NO {
                        Self::first_empty_slot(&self.quick_save_slots, 10)
                    } else if crate::elm::syscom::is_slot_time_query(x) {
                        let slot = if x >= crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_YEAR
                            && x <= crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_MILLISECOND
                        {
                            slot_no.and_then(|slot_no| self.quick_save_slots.get(&slot_no))
                        } else {
                            slot_no.and_then(|slot_no| self.local_save_slots.get(&slot_no))
                        };
                        if let Some(slot) = slot {
                            let s = &slot.stamp;
                            if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_YEAR
                                || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_YEAR
                            {
                                s.year
                            } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_MONTH
                                || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_MONTH
                            {
                                s.month
                            } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_DAY
                                || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_DAY
                            {
                                s.day
                            } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_WEEKDAY
                                || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_WEEKDAY
                            {
                                s.weekday
                            } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_HOUR
                                || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_HOUR
                            {
                                s.hour
                            } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_MINUTE
                                || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_MINUTE
                            {
                                s.minute
                            } else if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_SECOND
                                || x == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_SECOND
                            {
                                s.second
                            } else {
                                s.millisecond
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    };
                    self.stack.push_int(val);
                }
                Ok(Some(true))
            }
            x if crate::elm::syscom::is_slot_text_query(x) => {
                if ret_form == crate::elm::form::STR {
                    let slot_no = Self::slot_arg(args, 0);
                    let text = match x {
                        y if y == crate::elm::syscom::ELM_SYSCOM_GET_CURRENT_SAVE_SCENE_TITLE => {
                            self.scene_title.clone()
                        }
                        y if y == crate::elm::syscom::ELM_SYSCOM_GET_CURRENT_SAVE_MESSAGE => {
                            self.last_sel_msg.clone()
                        }
                        y if y == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_TITLE => slot_no
                            .and_then(|slot_no| self.local_save_slots.get(&slot_no))
                            .map(|s| s.scene_title.clone())
                            .unwrap_or_default(),
                        y if y == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_MESSAGE => slot_no
                            .and_then(|slot_no| self.local_save_slots.get(&slot_no))
                            .map(|s| s.message.clone())
                            .unwrap_or_default(),
                        y if y == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_TITLE => slot_no
                            .and_then(|slot_no| self.quick_save_slots.get(&slot_no))
                            .map(|s| s.scene_title.clone())
                            .unwrap_or_default(),
                        y if y == crate::elm::syscom::ELM_SYSCOM_GET_QUICK_SAVE_MESSAGE => slot_no
                            .and_then(|slot_no| self.quick_save_slots.get(&slot_no))
                            .map(|s| s.message.clone())
                            .unwrap_or_default(),
                        _ => String::new(),
                    };
                    self.stack.push_str(text);
                }
                Ok(Some(true))
            }
            x if crate::elm::syscom::is_get_new_no(x) => {
                if ret_form == crate::elm::form::INT {
                    let v = if x == crate::elm::syscom::ELM_SYSCOM_GET_SAVE_NEW_NO {
                        Self::first_empty_slot(&self.local_save_slots, 100)
                    } else {
                        Self::first_empty_slot(&self.quick_save_slots, 10)
                    };
                    self.stack.push_int(v);
                }
                Ok(Some(true))
            }
            x if crate::elm::syscom::is_delete_save(x) => {
                let slot_no = Self::slot_arg(args, 0);
                let ok = if x == crate::elm::syscom::ELM_SYSCOM_DELETE_QUICK_SAVE {
                    slot_no
                        .map(|slot_no| self.quick_save_slots.remove(&slot_no).is_some())
                        .unwrap_or(false)
                } else {
                    slot_no
                        .map(|slot_no| self.local_save_slots.remove(&slot_no).is_some())
                        .unwrap_or(false)
                };
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(if ok { 1 } else { 0 });
                }
                Ok(Some(true))
            }
            x if x == crate::elm::syscom::ELM_SYSCOM_RETURN_TO_SEL => {
                self.handle_syscom_return_to_sel(args, ret_form, provider, host)
            }
            x if x == crate::elm::syscom::ELM_SYSCOM_RETURN_TO_MENU => {
                self.handle_syscom_return_to_menu(args, ret_form, provider, host)
            }
            x if x == crate::elm::syscom::ELM_SYSCOM_END_GAME => {
                self.handle_syscom_end_game(args, ret_form, provider, host)
            }
            x if x == crate::elm::syscom::ELM_SYSCOM_END_SAVE => {
                self.handle_syscom_end_save(args, ret_form, host)
            }
            x if x == crate::elm::syscom::ELM_SYSCOM_END_LOAD => {
                self.handle_syscom_end_load(args, ret_form, provider, host)
            }
            x if x == crate::elm::syscom::ELM_SYSCOM_SET_RETURN_SCENE_ONCE => {
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
            x if x == crate::elm::syscom::ELM_SYSCOM_GET_SYSTEM_EXTRA_INT_VALUE => {
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
            x if x == crate::elm::syscom::ELM_SYSCOM_GET_SYSTEM_EXTRA_STR_VALUE => {
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
            x if crate::elm::syscom::is_open_dialog(x) => {
                host.on_open_tweet_dialog();
                if ret_form == crate::elm::form::INT {
                    self.stack.push_int(0);
                }
                Ok(Some(true))
            }
            x if crate::elm::syscom::is_call_ex(x) => {
                let scene = match args.get(0).map(|p| &p.value) {
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
                        provider,
                    )?;
                }
                Ok(Some(true))
            }
            _ => Ok(None),
        }
    }
}
