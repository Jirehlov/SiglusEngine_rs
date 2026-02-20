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
    /// Route `global.bgm.<sub>` commands matching C++ `tnm_command_proc_bgm`.
    pub(super) fn try_command_bgm(
        &mut self,
        element: &[i32],
        _arg_list_id: i32,
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
            ELM_BGM_PLAY | ELM_BGM_PLAY_ONESHOT | ELM_BGM_PLAY_WAIT
            | ELM_BGM_READY | ELM_BGM_READY_ONESHOT => {
                let name = args.first().and_then(|p| match &p.value {
                    PropValue::Str(s) => Some(s.clone()),
                    _ => None,
                }).unwrap_or_default();
                let fade_in = args.get(1).and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v),
                    _ => None,
                }).unwrap_or(0);
                let fade_out = args.get(2).and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v),
                    _ => None,
                }).unwrap_or(0);
                let loop_flag = sub != ELM_BGM_PLAY_ONESHOT && sub != ELM_BGM_READY_ONESHOT;
                let wait_flag = sub == ELM_BGM_PLAY_WAIT;
                let ready_flag = sub == ELM_BGM_READY || sub == ELM_BGM_READY_ONESHOT;
                host.on_bgm_play(&name, loop_flag, wait_flag, fade_in, fade_out, ready_flag);
                true
            }
            ELM_BGM_STOP => {
                let fade_out = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v),
                    _ => None,
                }).unwrap_or(0);
                host.on_bgm_stop(fade_out);
                true
            }
            ELM_BGM_PAUSE => {
                let fade = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v),
                    _ => None,
                }).unwrap_or(0);
                host.on_bgm_pause(fade);
                true
            }
            ELM_BGM_RESUME | ELM_BGM_RESUME_WAIT => {
                let fade = args.first().and_then(|p| match p.value {
                    PropValue::Int(v) => Some(v),
                    _ => None,
                }).unwrap_or(0);
                let wait = sub == ELM_BGM_RESUME_WAIT;
                host.on_bgm_resume(fade, wait);
                true
            }
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
                    PropValue::Int(v) => Some(v),
                    _ => None,
                }).unwrap_or(100);
                host.on_bgm_set_volume(sub, vol);
                true
            }
            ELM_BGM_GET_VOLUME => {
                // C++ tnm_stack_push_int(p_bgm->get_volume())
                self.stack.push_int(100);
                true
            }
            ELM_BGM_GET_REGIST_NAME => {
                // C++ tnm_stack_push_str(p_bgm->get_regist_name())
                self.stack.push_str(String::new());
                true
            }
            ELM_BGM_GET_PLAY_POS => {
                // C++ tnm_stack_push_int(p_bgm->get_play_pos())
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
