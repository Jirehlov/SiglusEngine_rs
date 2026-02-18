use std::io::Cursor;

use anyhow::{Context, Result, bail};

use super::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VmEndSaveState {
    pub scene_title: String,
    pub message: String,
    pub persistent: VmPersistentState,
    pub runtime: Option<VmEndSaveRuntimeState>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VmEndSaveRuntimeCallPropState {
    pub prop_id: i32,
    pub form: i32,
    pub value: VmEndSaveRuntimePropValue,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VmEndSaveRuntimeFrameState {
    pub return_pc: usize,
    pub return_scene: String,
    pub return_line_no: i32,
    pub expect_ret_form: i32,
    pub frame_action_flag: bool,
    pub arg_cnt: usize,
    pub call_l: Vec<i32>,
    pub call_k: Vec<String>,
    pub call_user_props: Vec<VmEndSaveRuntimeCallPropState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmEndSaveRuntimePropValue {
    Int(i32),
    Str(String),
    List(Vec<VmEndSaveRuntimeProp>),
    Element(Vec<i32>),
    IntList(Vec<i32>),
    StrList(Vec<String>),
}

impl Default for VmEndSaveRuntimePropValue {
    fn default() -> Self {
        Self::Int(0)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VmEndSaveRuntimeProp {
    pub id: i32,
    pub form: i32,
    pub value: VmEndSaveRuntimePropValue,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VmEndSaveRuntimeFrameActionState {
    pub end_time: i32,
    pub real_flag: i32,
    pub scn_name: String,
    pub cmd_name: String,
    pub args: Vec<VmEndSaveRuntimeProp>,
    pub end_action_flag: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VmEndSaveRuntimeState {
    pub scene: String,
    pub lexer_scene: String,
    pub lexer_pc: usize,
    pub lexer_line_no: i32,
    pub stack_ints: Vec<i32>,
    pub stack_strs: Vec<String>,
    pub stack_points: Vec<usize>,
    pub frames: Vec<VmEndSaveRuntimeFrameState>,
    pub user_prop_forms: Vec<i32>,
    pub user_prop_values: Vec<VmEndSaveRuntimePropValue>,
    pub frame_action: VmEndSaveRuntimeFrameActionState,
    pub frame_action_ch: Vec<VmEndSaveRuntimeFrameActionState>,
    pub save_point_snapshot: Option<VmPersistentState>,
    pub sel_point_snapshot: Option<VmPersistentState>,
    pub sel_point_stock: Option<VmPersistentState>,
    pub cur_mwnd_element: Vec<i32>,
    pub cur_sel_mwnd_element: Vec<i32>,
    pub hide_mwnd_onoff_flag: i32,
    pub msg_back_open_flag: i32,
    pub msg_back_has_message: i32,
    pub msg_back_disable_flag: i32,
    pub msg_back_off_flag: i32,
    pub msg_back_disp_off_flag: i32,
    pub msg_back_proc_off_flag: i32,
    pub system_wipe_flag: i32,
    pub do_frame_action_flag: i32,
    pub do_load_after_call_flag: i32,
    pub last_pc: usize,
    pub last_line_no: i32,
    pub last_scene: String,
}

impl VmEndSaveState {
    const MAGIC_V1: &'static [u8; 5] = b"SESV1";
    const MAGIC_V2: &'static [u8; 5] = b"SESV2";
    const MAGIC_V3: &'static [u8; 5] = b"SESV3";
    const MAX_VEC_LEN: usize = 1 << 20;
    const MAX_STR_BYTES: usize = 16 * 1024 * 1024;
    const MAX_TOTAL_STR_BYTES: usize = 128 * 1024 * 1024;

    pub fn encode_binary(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(Self::MAGIC_V3);

        let title = self.scene_title.as_bytes();
        let title_len: u32 = title
            .len()
            .try_into()
            .expect("end-save title length exceeds u32");
        out.extend_from_slice(&title_len.to_le_bytes());
        out.extend_from_slice(title);

        let message = self.message.as_bytes();
        let message_len: u32 = message
            .len()
            .try_into()
            .expect("end-save message length exceeds u32");
        out.extend_from_slice(&message_len.to_le_bytes());
        out.extend_from_slice(message);

        let p = self.persistent.encode_binary();
        let p_len: u32 = p
            .len()
            .try_into()
            .expect("end-save persistent payload length exceeds u32");
        out.extend_from_slice(&p_len.to_le_bytes());
        out.extend_from_slice(&p);

        match &self.runtime {
            Some(runtime) => {
                out.push(1);
                Self::encode_runtime_state(&mut out, runtime);
            }
            None => out.push(0),
        }
        out
    }

    fn push_i32_vec(buf: &mut Vec<u8>, vals: &[i32]) {
        let len: u32 = vals
            .len()
            .try_into()
            .expect("end-save runtime i32 vec length exceeds u32");
        buf.extend_from_slice(&len.to_le_bytes());
        for v in vals {
            buf.extend_from_slice(&v.to_le_bytes());
        }
    }

    fn push_usize_vec(buf: &mut Vec<u8>, vals: &[usize]) {
        let len: u32 = vals
            .len()
            .try_into()
            .expect("end-save runtime usize vec length exceeds u32");
        buf.extend_from_slice(&len.to_le_bytes());
        for v in vals {
            let vv: u64 = (*v).try_into().expect("end-save runtime usize exceeds u64");
            buf.extend_from_slice(&vv.to_le_bytes());
        }
    }

    fn push_string(buf: &mut Vec<u8>, s: &str) {
        let bytes = s.as_bytes();
        let len: u32 = bytes
            .len()
            .try_into()
            .expect("end-save runtime string length exceeds u32");
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(bytes);
    }

    fn push_str_vec(buf: &mut Vec<u8>, vals: &[String]) {
        let len: u32 = vals
            .len()
            .try_into()
            .expect("end-save runtime string vec length exceeds u32");
        buf.extend_from_slice(&len.to_le_bytes());
        for v in vals {
            Self::push_string(buf, v);
        }
    }

    fn push_opt_persistent(buf: &mut Vec<u8>, st: &Option<VmPersistentState>) {
        match st {
            Some(v) => {
                buf.push(1);
                let payload = v.encode_binary();
                let n: u32 = payload
                    .len()
                    .try_into()
                    .expect("runtime optional persistent payload too large");
                buf.extend_from_slice(&n.to_le_bytes());
                buf.extend_from_slice(&payload);
            }
            None => buf.push(0),
        }
    }

    fn push_prop_value(buf: &mut Vec<u8>, v: &VmEndSaveRuntimePropValue) {
        match v {
            VmEndSaveRuntimePropValue::Int(i) => {
                buf.push(0);
                buf.extend_from_slice(&i.to_le_bytes());
            }
            VmEndSaveRuntimePropValue::Str(s) => {
                buf.push(1);
                Self::push_string(buf, s);
            }
            VmEndSaveRuntimePropValue::List(items) => {
                buf.push(2);
                let n: u32 = items.len().try_into().expect("runtime prop list too large");
                buf.extend_from_slice(&n.to_le_bytes());
                for item in items {
                    Self::push_prop(buf, item);
                }
            }
            VmEndSaveRuntimePropValue::Element(vals) => {
                buf.push(3);
                Self::push_i32_vec(buf, vals);
            }
            VmEndSaveRuntimePropValue::IntList(vals) => {
                buf.push(4);
                Self::push_i32_vec(buf, vals);
            }
            VmEndSaveRuntimePropValue::StrList(vals) => {
                buf.push(5);
                Self::push_str_vec(buf, vals);
            }
        }
    }

    fn push_prop(buf: &mut Vec<u8>, p: &VmEndSaveRuntimeProp) {
        buf.extend_from_slice(&p.id.to_le_bytes());
        buf.extend_from_slice(&p.form.to_le_bytes());
        Self::push_prop_value(buf, &p.value);
    }

    fn push_call_prop(buf: &mut Vec<u8>, p: &VmEndSaveRuntimeCallPropState) {
        buf.extend_from_slice(&p.prop_id.to_le_bytes());
        buf.extend_from_slice(&p.form.to_le_bytes());
        Self::push_prop_value(buf, &p.value);
    }

    fn push_frame_action(buf: &mut Vec<u8>, fa: &VmEndSaveRuntimeFrameActionState) {
        buf.extend_from_slice(&fa.end_time.to_le_bytes());
        buf.extend_from_slice(&fa.real_flag.to_le_bytes());
        Self::push_string(buf, &fa.scn_name);
        Self::push_string(buf, &fa.cmd_name);
        let n: u32 = fa
            .args
            .len()
            .try_into()
            .expect("frame_action args too large");
        buf.extend_from_slice(&n.to_le_bytes());
        for arg in &fa.args {
            Self::push_prop(buf, arg);
        }
        buf.push(if fa.end_action_flag { 1 } else { 0 });
    }

    fn encode_runtime_state(buf: &mut Vec<u8>, rt: &VmEndSaveRuntimeState) {
        Self::push_string(buf, &rt.scene);
        Self::push_string(buf, &rt.lexer_scene);
        buf.extend_from_slice(&(rt.lexer_pc as u64).to_le_bytes());
        buf.extend_from_slice(&rt.lexer_line_no.to_le_bytes());
        Self::push_i32_vec(buf, &rt.stack_ints);
        Self::push_str_vec(buf, &rt.stack_strs);
        Self::push_usize_vec(buf, &rt.stack_points);

        let frame_len: u32 = rt
            .frames
            .len()
            .try_into()
            .expect("end-save runtime frame vec length exceeds u32");
        buf.extend_from_slice(&frame_len.to_le_bytes());
        for frame in &rt.frames {
            buf.extend_from_slice(&(frame.return_pc as u64).to_le_bytes());
            Self::push_string(buf, &frame.return_scene);
            buf.extend_from_slice(&frame.return_line_no.to_le_bytes());
            buf.extend_from_slice(&frame.expect_ret_form.to_le_bytes());
            buf.push(if frame.frame_action_flag { 1 } else { 0 });
            buf.extend_from_slice(&(frame.arg_cnt as u64).to_le_bytes());
            Self::push_i32_vec(buf, &frame.call_l);
            Self::push_str_vec(buf, &frame.call_k);
            let n: u32 = frame
                .call_user_props
                .len()
                .try_into()
                .expect("runtime call user_props too large");
            buf.extend_from_slice(&n.to_le_bytes());
            for cp in &frame.call_user_props {
                Self::push_call_prop(buf, cp);
            }
        }

        Self::push_i32_vec(buf, &rt.user_prop_forms);
        let upn: u32 = rt
            .user_prop_values
            .len()
            .try_into()
            .expect("runtime user_prop_values too large");
        buf.extend_from_slice(&upn.to_le_bytes());
        for up in &rt.user_prop_values {
            Self::push_prop_value(buf, up);
        }

        Self::push_frame_action(buf, &rt.frame_action);
        let fan: u32 = rt
            .frame_action_ch
            .len()
            .try_into()
            .expect("runtime frame_action_ch too large");
        buf.extend_from_slice(&fan.to_le_bytes());
        for fa in &rt.frame_action_ch {
            Self::push_frame_action(buf, fa);
        }

        Self::push_opt_persistent(buf, &rt.save_point_snapshot);
        Self::push_opt_persistent(buf, &rt.sel_point_snapshot);
        Self::push_opt_persistent(buf, &rt.sel_point_stock);
        Self::push_i32_vec(buf, &rt.cur_mwnd_element);
        Self::push_i32_vec(buf, &rt.cur_sel_mwnd_element);
        buf.extend_from_slice(&rt.hide_mwnd_onoff_flag.to_le_bytes());
        buf.extend_from_slice(&rt.msg_back_open_flag.to_le_bytes());
        buf.extend_from_slice(&rt.msg_back_has_message.to_le_bytes());
        buf.extend_from_slice(&rt.msg_back_disable_flag.to_le_bytes());
        buf.extend_from_slice(&rt.msg_back_off_flag.to_le_bytes());
        buf.extend_from_slice(&rt.msg_back_disp_off_flag.to_le_bytes());
        buf.extend_from_slice(&rt.msg_back_proc_off_flag.to_le_bytes());
        buf.extend_from_slice(&rt.system_wipe_flag.to_le_bytes());
        buf.extend_from_slice(&rt.do_frame_action_flag.to_le_bytes());
        buf.extend_from_slice(&rt.do_load_after_call_flag.to_le_bytes());

        buf.extend_from_slice(&(rt.last_pc as u64).to_le_bytes());
        buf.extend_from_slice(&rt.last_line_no.to_le_bytes());
        Self::push_string(buf, &rt.last_scene);
    }

    pub fn decode_binary(bytes: &[u8]) -> Result<Self> {
        fn read_u32(cur: &mut Cursor<&[u8]>) -> Result<u32> {
            let mut b = [0u8; 4];
            std::io::Read::read_exact(cur, &mut b)?;
            Ok(u32::from_le_bytes(b))
        }

        fn read_vec(cur: &mut Cursor<&[u8]>, n: usize) -> Result<Vec<u8>> {
            let mut v = vec![0u8; n];
            std::io::Read::read_exact(cur, &mut v)?;
            Ok(v)
        }

        fn read_i32(cur: &mut Cursor<&[u8]>) -> Result<i32> {
            let mut b = [0u8; 4];
            std::io::Read::read_exact(cur, &mut b)?;
            Ok(i32::from_le_bytes(b))
        }

        fn read_u64(cur: &mut Cursor<&[u8]>) -> Result<u64> {
            let mut b = [0u8; 8];
            std::io::Read::read_exact(cur, &mut b)?;
            Ok(u64::from_le_bytes(b))
        }

        fn read_string(cur: &mut Cursor<&[u8]>, what: &str) -> Result<String> {
            let len = read_u32(cur)? as usize;
            if len > VmEndSaveState::MAX_STR_BYTES {
                bail!("end-save runtime {} string too large: {}", what, len);
            }
            String::from_utf8(read_vec(cur, len)?)
                .with_context(|| format!("invalid utf8 in {}", what))
        }

        fn read_i32_vec(cur: &mut Cursor<&[u8]>, what: &str) -> Result<Vec<i32>> {
            let n = read_u32(cur)? as usize;
            if n > VmEndSaveState::MAX_VEC_LEN {
                bail!("end-save runtime {} vec too large: {}", what, n);
            }
            let mut v = Vec::with_capacity(n);
            for _ in 0..n {
                v.push(read_i32(cur)?);
            }
            Ok(v)
        }

        fn read_usize_vec(cur: &mut Cursor<&[u8]>, what: &str) -> Result<Vec<usize>> {
            let n = read_u32(cur)? as usize;
            if n > VmEndSaveState::MAX_VEC_LEN {
                bail!("end-save runtime {} vec too large: {}", what, n);
            }
            let mut v = Vec::with_capacity(n);
            for _ in 0..n {
                v.push(read_u64(cur)? as usize);
            }
            Ok(v)
        }

        fn read_str_vec(cur: &mut Cursor<&[u8]>, what: &str) -> Result<Vec<String>> {
            let n = read_u32(cur)? as usize;
            if n > VmEndSaveState::MAX_VEC_LEN {
                bail!("end-save runtime {} vec too large: {}", what, n);
            }
            let mut vals = Vec::with_capacity(n);
            let mut total_bytes = 0usize;
            for _ in 0..n {
                let s = read_string(cur, what)?;
                total_bytes = total_bytes
                    .checked_add(s.len())
                    .context("end-save runtime string bytes overflow")?;
                if total_bytes > VmEndSaveState::MAX_TOTAL_STR_BYTES {
                    bail!(
                        "end-save runtime total string bytes too large: {}",
                        total_bytes
                    );
                }
                vals.push(s);
            }
            Ok(vals)
        }

        fn read_prop_value(cur: &mut Cursor<&[u8]>) -> Result<VmEndSaveRuntimePropValue> {
            let mut kind = [0u8; 1];
            std::io::Read::read_exact(cur, &mut kind)?;
            match kind[0] {
                0 => Ok(VmEndSaveRuntimePropValue::Int(read_i32(cur)?)),
                1 => Ok(VmEndSaveRuntimePropValue::Str(read_string(
                    cur,
                    "runtime prop str",
                )?)),
                2 => {
                    let n = read_u32(cur)? as usize;
                    if n > VmEndSaveState::MAX_VEC_LEN {
                        bail!("runtime prop list too large: {}", n);
                    }
                    let mut vals = Vec::with_capacity(n);
                    for _ in 0..n {
                        vals.push(read_prop(cur)?);
                    }
                    Ok(VmEndSaveRuntimePropValue::List(vals))
                }
                3 => Ok(VmEndSaveRuntimePropValue::Element(read_i32_vec(
                    cur,
                    "runtime prop element",
                )?)),
                4 => Ok(VmEndSaveRuntimePropValue::IntList(read_i32_vec(
                    cur,
                    "runtime prop intlist",
                )?)),
                5 => Ok(VmEndSaveRuntimePropValue::StrList(read_str_vec(
                    cur,
                    "runtime prop strlist",
                )?)),
                _ => bail!("invalid runtime prop kind: {}", kind[0]),
            }
        }

        fn read_prop(cur: &mut Cursor<&[u8]>) -> Result<VmEndSaveRuntimeProp> {
            Ok(VmEndSaveRuntimeProp {
                id: read_i32(cur)?,
                form: read_i32(cur)?,
                value: read_prop_value(cur)?,
            })
        }

        fn read_call_prop(cur: &mut Cursor<&[u8]>) -> Result<VmEndSaveRuntimeCallPropState> {
            Ok(VmEndSaveRuntimeCallPropState {
                prop_id: read_i32(cur)?,
                form: read_i32(cur)?,
                value: read_prop_value(cur)?,
            })
        }

        fn read_frame_action(cur: &mut Cursor<&[u8]>) -> Result<VmEndSaveRuntimeFrameActionState> {
            let end_time = read_i32(cur)?;
            let real_flag = read_i32(cur)?;
            let scn_name = read_string(cur, "runtime frame_action scn_name")?;
            let cmd_name = read_string(cur, "runtime frame_action cmd_name")?;
            let n = read_u32(cur)? as usize;
            if n > VmEndSaveState::MAX_VEC_LEN {
                bail!("runtime frame_action args too large: {}", n);
            }
            let mut args = Vec::with_capacity(n);
            for _ in 0..n {
                args.push(read_prop(cur)?);
            }
            let mut end_flag = [0u8; 1];
            std::io::Read::read_exact(cur, &mut end_flag)?;
            Ok(VmEndSaveRuntimeFrameActionState {
                end_time,
                real_flag,
                scn_name,
                cmd_name,
                args,
                end_action_flag: end_flag[0] != 0,
            })
        }

        fn read_opt_persistent(cur: &mut Cursor<&[u8]>) -> Result<Option<VmPersistentState>> {
            let mut has = [0u8; 1];
            std::io::Read::read_exact(cur, &mut has)?;
            if has[0] == 0 {
                return Ok(None);
            }
            let n = read_u32(cur)? as usize;
            if n > VmEndSaveState::MAX_TOTAL_STR_BYTES {
                bail!("runtime optional persistent payload too large: {}", n);
            }
            let payload = read_vec(cur, n)?;
            Ok(Some(VmPersistentState::decode_binary(&payload)?))
        }

        let mut cur = Cursor::new(bytes);
        let mut magic = [0u8; 5];
        std::io::Read::read_exact(&mut cur, &mut magic)?;
        if &magic != Self::MAGIC_V1 && &magic != Self::MAGIC_V2 && &magic != Self::MAGIC_V3 {
            bail!("invalid end-save state magic")
        }

        let title_len = read_u32(&mut cur)? as usize;
        let scene_title = String::from_utf8(read_vec(&mut cur, title_len)?)
            .context("invalid utf8 in end-save title")?;

        let msg_len = read_u32(&mut cur)? as usize;
        let message = String::from_utf8(read_vec(&mut cur, msg_len)?)
            .context("invalid utf8 in end-save message")?;

        let payload_len = read_u32(&mut cur)? as usize;
        let payload = read_vec(&mut cur, payload_len)?;
        let persistent = VmPersistentState::decode_binary(&payload)
            .context("invalid persistent payload in end-save state")?;

        let mut state = Self {
            scene_title,
            message,
            persistent,
            runtime: None,
        };

        if &magic == Self::MAGIC_V2 || &magic == Self::MAGIC_V3 {
            let mut has_runtime = [0u8; 1];
            std::io::Read::read_exact(&mut cur, &mut has_runtime)?;
            if has_runtime[0] != 0 {
                let scene = read_string(&mut cur, "runtime scene")?;
                let lexer_scene = read_string(&mut cur, "runtime lexer scene")?;
                let lexer_pc = read_u64(&mut cur)? as usize;
                let lexer_line_no = read_i32(&mut cur)?;
                let stack_ints = read_i32_vec(&mut cur, "runtime stack_ints")?;
                let stack_strs = read_str_vec(&mut cur, "runtime stack_strs")?;
                let stack_points = read_usize_vec(&mut cur, "runtime stack_points")?;
                let frame_len = read_u32(&mut cur)? as usize;
                if frame_len > Self::MAX_VEC_LEN {
                    bail!("end-save runtime frames too large: {}", frame_len);
                }
                let mut frames = Vec::with_capacity(frame_len);
                for _ in 0..frame_len {
                    let return_pc = read_u64(&mut cur)? as usize;
                    let return_scene = read_string(&mut cur, "runtime return scene")?;
                    let return_line_no = read_i32(&mut cur)?;
                    let expect_ret_form = read_i32(&mut cur)?;
                    let mut flag = [0u8; 1];
                    std::io::Read::read_exact(&mut cur, &mut flag)?;
                    let arg_cnt = read_u64(&mut cur)? as usize;
                    let call_l = read_i32_vec(&mut cur, "runtime call_l")?;
                    let call_k = read_str_vec(&mut cur, "runtime call_k")?;
                    let cpn = read_u32(&mut cur)? as usize;
                    if cpn > Self::MAX_VEC_LEN {
                        bail!("runtime call user_props too large: {}", cpn);
                    }
                    let mut call_user_props = Vec::with_capacity(cpn);
                    for _ in 0..cpn {
                        call_user_props.push(read_call_prop(&mut cur)?);
                    }
                    frames.push(VmEndSaveRuntimeFrameState {
                        return_pc,
                        return_scene,
                        return_line_no,
                        expect_ret_form,
                        frame_action_flag: flag[0] != 0,
                        arg_cnt,
                        call_l,
                        call_k,
                        call_user_props,
                    });
                }
                let user_prop_forms = read_i32_vec(&mut cur, "runtime user_prop_forms")?;
                let upn = read_u32(&mut cur)? as usize;
                if upn > Self::MAX_VEC_LEN {
                    bail!("runtime user_prop_values too large: {}", upn);
                }
                let mut user_prop_values = Vec::with_capacity(upn);
                for _ in 0..upn {
                    user_prop_values.push(read_prop_value(&mut cur)?);
                }
                let frame_action = read_frame_action(&mut cur)?;
                let fan = read_u32(&mut cur)? as usize;
                if fan > Self::MAX_VEC_LEN {
                    bail!("runtime frame_action_ch too large: {}", fan);
                }
                let mut frame_action_ch = Vec::with_capacity(fan);
                for _ in 0..fan {
                    frame_action_ch.push(read_frame_action(&mut cur)?);
                }
                let save_point_snapshot = read_opt_persistent(&mut cur)?;
                let sel_point_snapshot = read_opt_persistent(&mut cur)?;
                let sel_point_stock = read_opt_persistent(&mut cur)?;
                let cur_mwnd_element = read_i32_vec(&mut cur, "runtime cur_mwnd_element")?;
                let cur_sel_mwnd_element = read_i32_vec(&mut cur, "runtime cur_sel_mwnd_element")?;
                let hide_mwnd_onoff_flag = read_i32(&mut cur)?;
                let msg_back_open_flag = read_i32(&mut cur)?;
                let msg_back_has_message = read_i32(&mut cur)?;
                let msg_back_disable_flag = read_i32(&mut cur)?;
                let msg_back_off_flag = read_i32(&mut cur)?;
                let msg_back_disp_off_flag = read_i32(&mut cur)?;
                let msg_back_proc_off_flag = read_i32(&mut cur)?;
                let (system_wipe_flag, do_frame_action_flag, do_load_after_call_flag) =
                    if &magic == Self::MAGIC_V3 {
                        (
                            read_i32(&mut cur)?,
                            read_i32(&mut cur)?,
                            read_i32(&mut cur)?,
                        )
                    } else {
                        (0, 0, 0)
                    };
                let last_pc = read_u64(&mut cur)? as usize;
                let last_line_no = read_i32(&mut cur)?;
                let last_scene = read_string(&mut cur, "runtime last scene")?;
                state.runtime = Some(VmEndSaveRuntimeState {
                    scene,
                    lexer_scene,
                    lexer_pc,
                    lexer_line_no,
                    stack_ints,
                    stack_strs,
                    stack_points,
                    frames,
                    user_prop_forms,
                    user_prop_values,
                    frame_action,
                    frame_action_ch,
                    save_point_snapshot,
                    sel_point_snapshot,
                    sel_point_stock,
                    cur_mwnd_element,
                    cur_sel_mwnd_element,
                    hide_mwnd_onoff_flag,
                    msg_back_open_flag,
                    msg_back_has_message,
                    msg_back_disable_flag,
                    msg_back_off_flag,
                    msg_back_disp_off_flag,
                    msg_back_proc_off_flag,
                    system_wipe_flag,
                    do_frame_action_flag,
                    do_load_after_call_flag,
                    last_pc,
                    last_line_no,
                    last_scene,
                });
            }
        }

        if cur.position() != bytes.len() as u64 {
            bail!("unexpected trailing bytes in end-save state")
        }

        Ok(state)
    }
}
