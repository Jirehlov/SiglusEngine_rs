/// Sound / BGM / KOE / PCM / PCMCH / SE / MOV / PCM_EVENT command routing.
///
/// C++ reference: cmd_sound.cpp, cmd_koe.cpp
///
/// Approach: Parse arguments per C++ source and delegate to Host callbacks.
/// Query-type commands (check, get_volume, get_regist_name) push default
/// return values directly. Play/stop/pause/resume/volume commands call
/// Host trait methods so the GUI host can dispatch to audio backends.
use super::*;

impl Vm {
    /// C++ TNM_BGM_START_POS_INI = -1
    const TNM_BGM_START_POS_INI: i32 = -1;

    /// Route `global.bgm.<sub>` commands matching C++ `tnm_command_proc_bgm`.
    ///
    /// Named-arg parsing fully aligned with C++ cmd_sound.cpp:
    ///   BGM PLAY:  id 0→regist_name, 1→loop_flag, 2→wait_flag, 3→start_pos, 4→fade_in_time, 5→fade_out_time
    ///   BGM READY: id 0→regist_name, 1→loop_flag, 3→start_pos, 5→fade_out_time
    ///   BGM RESUME: named id 0→delay_time
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
            // -----------------------------------------------------------
            // BGM PLAY — full named-arg override (C++ lines 21-50)
            // -----------------------------------------------------------
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
                        if let Some(PropValue::Int(v)) = args.get(2).map(|p| &p.value) { fade_out_time = *v; }
                        if let Some(PropValue::Int(v)) = args.get(1).map(|p| &p.value) { fade_in_time = *v; }
                        if let Some(PropValue::Str(s)) = args.get(0).map(|p| &p.value) { regist_name = s.clone(); }
                    }
                    1 => {
                        if let Some(PropValue::Int(v)) = args.get(1).map(|p| &p.value) { fade_in_time = *v; }
                        if let Some(PropValue::Str(s)) = args.get(0).map(|p| &p.value) { regist_name = s.clone(); }
                    }
                    0 => {
                        if let Some(PropValue::Str(s)) = args.get(0).map(|p| &p.value) { regist_name = s.clone(); }
                    }
                    _ => {}
                }

                // Named-arg overrides
                for arg in args.iter() {
                    match arg.id {
                        0 => { if let PropValue::Str(s) = &arg.value { regist_name = s.clone(); } }
                        1 => { if let PropValue::Int(v) = arg.value { loop_flag = v != 0; } }
                        2 => { if let PropValue::Int(v) = arg.value { wait_flag = v != 0; } }
                        3 => { if let PropValue::Int(v) = arg.value { start_pos = v; } }
                        4 => { if let PropValue::Int(v) = arg.value { fade_in_time = v; } }
                        5 => { if let PropValue::Int(v) = arg.value { fade_out_time = v; } }
                        _ => {}
                    }
                }

                // C++ dispatches: wait_flag → play_wait, loop_flag → play, else → play_oneshot
                host.on_bgm_play(&regist_name, loop_flag, wait_flag, fade_in_time, fade_out_time, start_pos, false);
                true
            }
            // -----------------------------------------------------------
            // BGM PLAY_ONESHOT — positional only (C++ lines 51-56)
            // -----------------------------------------------------------
            ELM_BGM_PLAY_ONESHOT => {
                let name = args.first().and_then(|p| match &p.value {
                    PropValue::Str(s) => Some(s.clone()), _ => None,
                }).unwrap_or_default();
                let fade_in = args.get(1).and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                let fade_out = args.get(2).and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                host.on_bgm_play(&name, false, false, fade_in, fade_out, Self::TNM_BGM_START_POS_INI, false);
                true
            }
            // -----------------------------------------------------------
            // BGM PLAY_WAIT — positional only (C++ lines 57-62)
            // -----------------------------------------------------------
            ELM_BGM_PLAY_WAIT => {
                let name = args.first().and_then(|p| match &p.value {
                    PropValue::Str(s) => Some(s.clone()), _ => None,
                }).unwrap_or_default();
                let fade_in = args.get(1).and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                let fade_out = args.get(2).and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                host.on_bgm_play(&name, true, true, fade_in, fade_out, Self::TNM_BGM_START_POS_INI, false);
                true
            }
            // -----------------------------------------------------------
            // BGM READY — full named-arg override (C++ lines 63-88)
            // -----------------------------------------------------------
            ELM_BGM_READY => {
                let mut regist_name = String::new();
                let mut loop_flag = true;
                let mut fade_out_time = 100i32; // C++ default for READY
                let mut start_pos = Self::TNM_BGM_START_POS_INI;

                // Positional args (C++ fallthrough switch)
                match arg_list_id {
                    n if n >= 2 => {
                        if let Some(PropValue::Int(v)) = args.get(1).map(|p| &p.value) { fade_out_time = *v; }
                        if let Some(PropValue::Str(s)) = args.get(0).map(|p| &p.value) { regist_name = s.clone(); }
                    }
                    0 => {
                        if let Some(PropValue::Str(s)) = args.get(0).map(|p| &p.value) { regist_name = s.clone(); }
                    }
                    _ => {}
                }

                // Named-arg overrides (id 2 / wait_flag not used for READY)
                for arg in args.iter() {
                    match arg.id {
                        0 => { if let PropValue::Str(s) = &arg.value { regist_name = s.clone(); } }
                        1 => { if let PropValue::Int(v) = arg.value { loop_flag = v != 0; } }
                        3 => { if let PropValue::Int(v) = arg.value { start_pos = v; } }
                        5 => { if let PropValue::Int(v) = arg.value { fade_out_time = v; } }
                        _ => {}
                    }
                }

                // C++: loop_flag → play, else → play_oneshot; fade_in=0, ready=true
                host.on_bgm_play(&regist_name, loop_flag, false, 0, fade_out_time, start_pos, true);
                true
            }
            // -----------------------------------------------------------
            // BGM READY_ONESHOT — positional only (C++ lines 89-94)
            // -----------------------------------------------------------
            ELM_BGM_READY_ONESHOT => {
                let name = args.first().and_then(|p| match &p.value {
                    PropValue::Str(s) => Some(s.clone()), _ => None,
                }).unwrap_or_default();
                let fade_in = args.get(1).and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                let fade_out = args.get(2).and_then(|p| match p.value { PropValue::Int(v) => Some(v), _ => None }).unwrap_or(0);
                host.on_bgm_play(&name, false, false, fade_in, fade_out, Self::TNM_BGM_START_POS_INI, true);
                true
            }
            // -----------------------------------------------------------
            // BGM STOP (C++ lines 95-99)
            // -----------------------------------------------------------
            ELM_BGM_STOP => {
                let fade_out = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v), _ => None,
                }).unwrap_or(0);
                host.on_bgm_stop(fade_out);
                true
            }
            // -----------------------------------------------------------
            // BGM PAUSE (C++ lines 100-104)
            // -----------------------------------------------------------
            ELM_BGM_PAUSE => {
                let fade = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v), _ => None,
                }).unwrap_or(0);
                host.on_bgm_pause(fade);
                true
            }
            // -----------------------------------------------------------
            // BGM RESUME / RESUME_WAIT — named-arg delay_time (C++ lines 105-122)
            // -----------------------------------------------------------
            ELM_BGM_RESUME => {
                let fade = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v), _ => None,
                }).unwrap_or(0);
                // Named-arg id 0 → delay_time
                let mut delay_time = 0i32;
                for arg in args.iter() {
                    if arg.id == 0 {
                        if let PropValue::Int(v) = arg.value { delay_time = v; }
                    }
                }
                host.on_bgm_resume(fade, false, delay_time);
                true
            }
            ELM_BGM_RESUME_WAIT => {
                let fade = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v), _ => None,
                }).unwrap_or(0);
                // C++ RESUME_WAIT: delay_time always 0
                host.on_bgm_resume(fade, true, 0);
                true
            }
            // -----------------------------------------------------------
            // Wait / Check / Volume / Query (unchanged from before)
            // -----------------------------------------------------------
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
                let vol = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v), _ => None,
                }).unwrap_or(100);
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
                host.on_error("無効なコマンドが指定されました。(bgm)");
                true
            }
        }
    }

    /// Route `global.pcm.<sub>` commands matching C++ `tnm_command_proc_pcm`.
    pub(super) fn try_command_pcm(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.len() < 2 {
            return true;
        }
        let sub = element[1];
        use crate::elm::pcm::*;

        match sub {
            ELM_PCM_PLAY => {
                let name = args.first().and_then(|p| match &p.value {
                    PropValue::Str(s) => Some(s.clone()),
                    _ => None,
                }).unwrap_or_default();
                host.on_pcm_play(&name);
                true
            }
            ELM_PCM_STOP => {
                host.on_pcm_stop();
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(pcm)");
                true
            }
        }
    }

    /// Route `global.pcmch[idx].<sub>` commands matching C++ `tnm_command_proc_pcmch`.
    ///
    /// Named-arg parsing aligned with C++ cmd_sound.cpp:
    ///   id 0 → loop_flag, id 1 → wait_flag, id 2 → fade_in_time,
    ///   id 3 → volume_type, id 4 → bgm_fade_target, id 5 → bgm_fade2_target,
    ///   id 6 → chara_no, id 7 → pcm_name (str), id 8 → koe_no,
    ///   id 9 → se_no, id 10 → bgm_name (str), id 11 → bgm_fade_source
    pub(super) fn try_command_pcmch(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        // Extract channel index from element path.
        // Path shapes: [ELM_ARRAY, idx, sub] or just [sub] (bare pcmch).
        let (ch_idx, sub) = if element.len() >= 3
            && element[0] == crate::elm::ELM_ARRAY
        {
            (element[1], element[2])
        } else {
            (-1, *element.last().unwrap_or(&-1))
        };
        if sub == -1 {
            return true; // bare pcmch element
        }
        use crate::elm::pcmch::*;

        match sub {
            ELM_PCMCH_PLAY | ELM_PCMCH_PLAY_LOOP | ELM_PCMCH_PLAY_WAIT
            | ELM_PCMCH_READY | ELM_PCMCH_READY_LOOP => {
                // Parse positional args
                let mut pcm_name = args.first().and_then(|p| match &p.value {
                    PropValue::Str(s) => Some(s.clone()),
                    _ => None,
                }).unwrap_or_default();
                let mut fade_in_time = if arg_list_id >= 1 {
                    args.get(1).and_then(|p| match p.value {
                        PropValue::Int(v) => Some(v), _ => None,
                    }).unwrap_or(0)
                } else { 0 };

                // Defaults per C++
                let mut loop_flag = sub == ELM_PCMCH_PLAY_LOOP || sub == ELM_PCMCH_READY_LOOP;
                let mut wait_flag = sub == ELM_PCMCH_PLAY_WAIT;
                let mut volume_type = 0; // TNM_VOLUME_TYPE_PCM
                let mut chara_no = -1i32;
                let mut _bgm_fade_target = false;
                let mut _bgm_fade2_target = false;
                let mut _bgm_fade_source = false;
                let mut _koe_no = -1i32;
                let mut _se_no = -1i32;
                let mut _bgm_name = String::new();
                let ready = sub == ELM_PCMCH_READY || sub == ELM_PCMCH_READY_LOOP;

                // Named-arg overrides (C++ iterates nal_end..named_al_end)
                for arg in args.iter() {
                    match arg.id {
                        0 => { if let PropValue::Int(v) = arg.value { loop_flag = v != 0; } }
                        1 => { if let PropValue::Int(v) = arg.value { wait_flag = v != 0; } }
                        2 => { if let PropValue::Int(v) = arg.value { fade_in_time = v; } }
                        3 => { if let PropValue::Int(v) = arg.value { volume_type = v; } }
                        4 => { if let PropValue::Int(v) = arg.value { _bgm_fade_target = v != 0; } }
                        5 => { if let PropValue::Int(v) = arg.value { _bgm_fade2_target = v != 0; } }
                        6 => { if let PropValue::Int(v) = arg.value { chara_no = v; } }
                        7 => { if let PropValue::Str(s) = &arg.value { pcm_name = s.clone(); } }
                        8 => { if let PropValue::Int(v) = arg.value { _koe_no = v; } }
                        9 => { if let PropValue::Int(v) = arg.value { _se_no = v; } }
                        10 => { if let PropValue::Str(s) = &arg.value { _bgm_name = s.clone(); } }
                        11 => { if let PropValue::Int(v) = arg.value { _bgm_fade_source = v != 0; } }
                        _ => {}
                    }
                }

                host.on_pcmch_play(ch_idx, &pcm_name, loop_flag, wait_flag, fade_in_time, volume_type, chara_no, ready);
                true
            }
            ELM_PCMCH_STOP => {
                let fade = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v), _ => None,
                }).unwrap_or(0);
                host.on_pcmch_stop(ch_idx, fade);
                true
            }
            ELM_PCMCH_PAUSE => {
                let fade = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v), _ => None,
                }).unwrap_or(0);
                host.on_pcmch_pause(ch_idx, fade);
                true
            }
            ELM_PCMCH_RESUME | ELM_PCMCH_RESUME_WAIT => {
                let fade = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v), _ => None,
                }).unwrap_or(0);
                let wait = sub == ELM_PCMCH_RESUME_WAIT;
                // C++ also processes named-arg id 0 → delay_time.
                host.on_pcmch_resume(ch_idx, fade, wait);
                true
            }
            ELM_PCMCH_WAIT | ELM_PCMCH_WAIT_KEY | ELM_PCMCH_WAIT_FADE | ELM_PCMCH_WAIT_FADE_KEY => {
                true
            }
            ELM_PCMCH_CHECK => {
                self.stack.push_int(0);
                true
            }
            ELM_PCMCH_SET_VOLUME | ELM_PCMCH_SET_VOLUME_MAX | ELM_PCMCH_SET_VOLUME_MIN => {
                let vol = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v), _ => None,
                }).unwrap_or(100);
                host.on_pcmch_set_volume(ch_idx, sub, vol);
                true
            }
            ELM_PCMCH_GET_VOLUME => {
                self.stack.push_int(100);
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(pcmch)");
                true
            }
        }
    }

    /// Route `global.se.<sub>` commands matching C++ `tnm_command_proc_se`.
    pub(super) fn try_command_se(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.len() < 2 {
            return true;
        }
        let sub = element[1];
        use crate::elm::se::*;

        match sub {
            ELM_SE_PLAY | ELM_SE_PLAY_BY_FILE_NAME | ELM_SE_PLAY_BY_KOE_NO | ELM_SE_PLAY_BY_SE_NO => {
                let id = args.first().and_then(|p| match &p.value {
                    PropValue::Int(v) => Some(*v),
                    _ => None,
                }).unwrap_or(0);
                let name = args.first().and_then(|p| match &p.value {
                    PropValue::Str(s) => Some(s.clone()),
                    _ => None,
                }).unwrap_or_default();
                host.on_se_play(id, &name);
                true
            }
            ELM_SE_STOP => {
                let fade = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v),
                    _ => None,
                }).unwrap_or(0);
                host.on_se_stop(fade);
                true
            }
            ELM_SE_WAIT | ELM_SE_WAIT_KEY => {
                true
            }
            ELM_SE_CHECK => {
                self.stack.push_int(0);
                true
            }
            ELM_SE_SET_VOLUME | ELM_SE_SET_VOLUME_MAX | ELM_SE_SET_VOLUME_MIN => {
                true
            }
            ELM_SE_GET_VOLUME => {
                self.stack.push_int(100);
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(se)");
                true
            }
        }
    }

    /// Route `global.mov.<sub>` commands matching C++ `tnm_command_proc_mov`.
    pub(super) fn try_command_mov(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        if element.len() < 2 {
            return true;
        }
        let sub = element[1];
        use crate::elm::mov::*;

        match sub {
            ELM_MOV_PLAY | ELM_MOV_PLAY_WAIT | ELM_MOV_PLAY_WAIT_KEY => {
                let name = args.first().and_then(|p| match &p.value {
                    PropValue::Str(s) => Some(s.clone()),
                    _ => None,
                }).unwrap_or_default();
                host.on_mov_play(&name);
                true
            }
            ELM_MOV_STOP => {
                host.on_mov_stop();
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(mov)");
                true
            }
        }
    }

    /// Route `global.pcmevent[idx].<sub>` commands.
    /// C++ reference: cmd_sound.cpp::tnm_command_proc_pcm_event.
    pub(super) fn try_command_pcmevent(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
        _args: &[Prop],
        _ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        let sub = match element.last() {
            Some(v) => *v,
            None => return true,
        };
        use crate::elm::pcmevent::*;

        match sub {
            ELM_PCMEVENT_START_ONESHOT | ELM_PCMEVENT_START_LOOP | ELM_PCMEVENT_START_RANDOM => {
                // Accept – no audio backend yet.
                true
            }
            ELM_PCMEVENT_STOP => {
                true
            }
            ELM_PCMEVENT_CHECK => {
                self.stack.push_int(0);
                true
            }
            ELM_PCMEVENT_WAIT | ELM_PCMEVENT_WAIT_KEY => {
                true
            }
            _ => {
                host.on_error("無効なコマンドが指定されました。(pcmevent)");
                true
            }
        }
    }

    /// Top-level dispatch for all sound-family global elements.
    /// Returns `true` if the command was handled (accepted or errored),
    /// `false` if it should fall through to the generic `on_command`.
    pub(super) fn try_command_sound(
        &mut self,
        element: &[i32],
        arg_list_id: i32,
        args: &[Prop],
        ret_form: i32,
        host: &mut dyn Host,
    ) -> bool {
        let root = element[0];
        use crate::elm::global::*;

        match root {
            ELM_GLOBAL_BGM => self.try_command_bgm(&element[1..], arg_list_id, args, ret_form, host),
            ELM_GLOBAL_PCM => self.try_command_pcm(&element[1..], arg_list_id, args, ret_form, host),
            ELM_GLOBAL_PCMCH => self.try_command_pcmch(&element[1..], arg_list_id, args, ret_form, host),
            ELM_GLOBAL_SE => self.try_command_se(&element[1..], arg_list_id, args, ret_form, host),
            ELM_GLOBAL_MOV => self.try_command_mov(&element[1..], arg_list_id, args, ret_form, host),
            ELM_GLOBAL_PCMEVENT => self.try_command_pcmevent(&element[1..], arg_list_id, args, ret_form, host),
            // KOE root element — pass through to host for now (bare element ref).
            ELM_GLOBAL_KOE | ELM_GLOBAL_KOE_ST | ELM_GLOBAL_EXKOE => true,
            // KOE play/wait — accept as no-op (audio not implemented).
            ELM_GLOBAL_KOE_PLAY_WAIT | ELM_GLOBAL_KOE_PLAY_WAIT_KEY
            | ELM_GLOBAL_EXKOE_PLAY_WAIT | ELM_GLOBAL_EXKOE_PLAY_WAIT_KEY => true,
            // KOE stop/wait — accept as no-op.
            ELM_GLOBAL_KOE_STOP | ELM_GLOBAL_KOE_WAIT | ELM_GLOBAL_KOE_WAIT_KEY => true,
            // KOE volume is already handled via dedicated koe_get_volume / koe_check arms.
            ELM_GLOBAL_KOE_SET_VOLUME | ELM_GLOBAL_KOE_SET_VOLUME_MAX | ELM_GLOBAL_KOE_SET_VOLUME_MIN => true,
            // BGMTABLE — accept silently.
            ELM_GLOBAL_BGMTABLE => true,
            _ => false,
        }
    }
}
