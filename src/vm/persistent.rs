use std::io::Cursor;

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VmPersistentState {
    pub flags_a: Vec<i32>,
    pub flags_b: Vec<i32>,
    pub flags_c: Vec<i32>,
    pub flags_d: Vec<i32>,
    pub flags_e: Vec<i32>,
    pub flags_f: Vec<i32>,
    pub flags_x: Vec<i32>,
    pub flags_g: Vec<i32>,
    pub flags_z: Vec<i32>,
    pub flags_s: Vec<String>,
    pub flags_m: Vec<String>,
    pub global_namae: Vec<String>,
    pub local_namae: Vec<String>,
    pub save_point_set: bool,
    pub sel_point_set: bool,
}

impl VmPersistentState {
    const MAGIC: &'static [u8; 5] = b"SVMS1";
    const MAX_VEC_LEN: usize = 1 << 20;
    const MAX_STR_BYTES: usize = 16 * 1024 * 1024;
    const MAX_TOTAL_STR_BYTES: usize = 64 * 1024 * 1024;

    pub fn encode_binary(&self) -> Vec<u8> {
        fn push_i32_vec(buf: &mut Vec<u8>, vals: &[i32]) {
            let len: u32 = vals
                .len()
                .try_into()
                .expect("persistent state i32 vec length exceeds u32");
            buf.extend_from_slice(&len.to_le_bytes());
            for v in vals {
                buf.extend_from_slice(&v.to_le_bytes());
            }
        }

        fn push_str_vec(buf: &mut Vec<u8>, vals: &[String]) {
            let len: u32 = vals
                .len()
                .try_into()
                .expect("persistent state string vec length exceeds u32");
            buf.extend_from_slice(&len.to_le_bytes());
            for v in vals {
                let bytes = v.as_bytes();
                let byte_len: u32 = bytes
                    .len()
                    .try_into()
                    .expect("persistent state string length exceeds u32");
                buf.extend_from_slice(&byte_len.to_le_bytes());
                buf.extend_from_slice(bytes);
            }
        }

        let mut out = Vec::new();
        out.extend_from_slice(Self::MAGIC);
        push_i32_vec(&mut out, &self.flags_a);
        push_i32_vec(&mut out, &self.flags_b);
        push_i32_vec(&mut out, &self.flags_c);
        push_i32_vec(&mut out, &self.flags_d);
        push_i32_vec(&mut out, &self.flags_e);
        push_i32_vec(&mut out, &self.flags_f);
        push_i32_vec(&mut out, &self.flags_x);
        push_i32_vec(&mut out, &self.flags_g);
        push_i32_vec(&mut out, &self.flags_z);
        push_str_vec(&mut out, &self.flags_s);
        push_str_vec(&mut out, &self.flags_m);
        push_str_vec(&mut out, &self.global_namae);
        push_str_vec(&mut out, &self.local_namae);
        out.push(if self.save_point_set { 1 } else { 0 });
        out.push(if self.sel_point_set { 1 } else { 0 });
        out
    }

    pub fn decode_binary(bytes: &[u8]) -> Result<Self> {
        fn ensure_remaining(r: &Cursor<&[u8]>, need: usize) -> Result<()> {
            let len = r.get_ref().len();
            let pos = r.position() as usize;
            let left = len.saturating_sub(pos);
            if need > left {
                bail!(
                    "truncated persistent state byte stream: need {} bytes at {}, remaining {}",
                    need,
                    pos,
                    left
                );
            }
            Ok(())
        }

        fn read_exact<const N: usize>(r: &mut Cursor<&[u8]>) -> Result<[u8; N]> {
            ensure_remaining(r, N)?;
            let mut buf = [0u8; N];
            std::io::Read::read_exact(r, &mut buf)?;
            Ok(buf)
        }

        fn read_u32(r: &mut Cursor<&[u8]>) -> Result<u32> {
            Ok(u32::from_le_bytes(read_exact::<4>(r)?))
        }

        fn read_i32_vec(r: &mut Cursor<&[u8]>) -> Result<Vec<i32>> {
            let n = read_u32(r)? as usize;
            if n > VmPersistentState::MAX_VEC_LEN {
                bail!(
                    "persistent state i32 vec too large: {} > {}",
                    n,
                    VmPersistentState::MAX_VEC_LEN
                );
            }
            ensure_remaining(r, n.saturating_mul(4))?;
            let mut vals = Vec::with_capacity(n);
            for _ in 0..n {
                vals.push(i32::from_le_bytes(read_exact::<4>(r)?));
            }
            Ok(vals)
        }

        fn read_str_vec(r: &mut Cursor<&[u8]>) -> Result<Vec<String>> {
            let n = read_u32(r)? as usize;
            if n > VmPersistentState::MAX_VEC_LEN {
                bail!(
                    "persistent state string vec too large: {} > {}",
                    n,
                    VmPersistentState::MAX_VEC_LEN
                );
            }
            let mut vals = Vec::with_capacity(n);
            let mut total_bytes = 0usize;
            for _ in 0..n {
                let len = read_u32(r)? as usize;
                if len > VmPersistentState::MAX_STR_BYTES {
                    bail!(
                        "persistent state string too large: {} > {}",
                        len,
                        VmPersistentState::MAX_STR_BYTES
                    );
                }
                total_bytes = total_bytes
                    .checked_add(len)
                    .context("persistent state string bytes overflow")?;
                if total_bytes > VmPersistentState::MAX_TOTAL_STR_BYTES {
                    bail!(
                        "persistent state total string bytes too large: {} > {}",
                        total_bytes,
                        VmPersistentState::MAX_TOTAL_STR_BYTES
                    );
                }
                ensure_remaining(r, len)?;
                let mut data = vec![0u8; len];
                std::io::Read::read_exact(r, &mut data)?;
                vals.push(String::from_utf8(data).context("invalid utf8 in persistent state")?);
            }
            Ok(vals)
        }

        let mut cur = Cursor::new(bytes);
        let magic = read_exact::<5>(&mut cur)?;
        if &magic != Self::MAGIC {
            bail!("invalid persistent state magic")
        }

        let st = Self {
            flags_a: read_i32_vec(&mut cur)?,
            flags_b: read_i32_vec(&mut cur)?,
            flags_c: read_i32_vec(&mut cur)?,
            flags_d: read_i32_vec(&mut cur)?,
            flags_e: read_i32_vec(&mut cur)?,
            flags_f: read_i32_vec(&mut cur)?,
            flags_x: read_i32_vec(&mut cur)?,
            flags_g: read_i32_vec(&mut cur)?,
            flags_z: read_i32_vec(&mut cur)?,
            flags_s: read_str_vec(&mut cur)?,
            flags_m: read_str_vec(&mut cur)?,
            global_namae: read_str_vec(&mut cur)?,
            local_namae: read_str_vec(&mut cur)?,
            save_point_set: read_exact::<1>(&mut cur)?[0] != 0,
            sel_point_set: read_exact::<1>(&mut cur)?[0] != 0,
        };

        if cur.position() != bytes.len() as u64 {
            bail!("unexpected trailing bytes in persistent state")
        }

        Ok(st)
    }
}
