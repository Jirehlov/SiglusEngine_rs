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

    pub(super) fn first_empty_slot(map: &BTreeMap<i32, LocalSaveSlot>, cnt: i32) -> i32 {
        for i in 0..cnt.max(0) {
            if !map.contains_key(&i) {
                return i;
            }
        }
        -1
    }

    pub(super) fn slot_arg(args: &[Prop], idx: usize) -> Option<i32> {
        let slot_no = Self::arg_int(args, idx);
        if slot_no < 0 {
            return None;
        }
        Some(slot_no)
    }
}
