use anyhow::{bail, Result};
use widestring::U16String;

#[derive(Debug, Clone)]
pub struct SceneHeader {
    pub header_size: i32,
    pub scn_ofs: i32,
    pub scn_size: i32,
    pub str_index_list_ofs: i32,
    pub str_index_cnt: i32,
    pub str_list_ofs: i32,
    pub str_cnt: i32,
    pub label_list_ofs: i32,
    pub label_cnt: i32,
    pub z_label_list_ofs: i32,
    pub z_label_cnt: i32,
    pub cmd_label_list_ofs: i32,
    pub cmd_label_cnt: i32,
    pub scn_prop_list_ofs: i32,
    pub scn_prop_cnt: i32,
    pub scn_prop_name_index_list_ofs: i32,
    pub scn_prop_name_index_cnt: i32,
    pub scn_prop_name_list_ofs: i32,
    pub scn_prop_name_cnt: i32,
    pub scn_cmd_list_ofs: i32,
    pub scn_cmd_cnt: i32,
    pub scn_cmd_name_index_list_ofs: i32,
    pub scn_cmd_name_index_cnt: i32,
    pub scn_cmd_name_list_ofs: i32,
    pub scn_cmd_name_cnt: i32,
    pub call_prop_name_index_list_ofs: i32,
    pub call_prop_name_index_cnt: i32,
    pub call_prop_name_list_ofs: i32,
    pub call_prop_name_cnt: i32,
    pub namae_list_ofs: i32,
    pub namae_cnt: i32,
    pub read_flag_list_ofs: i32,
    pub read_flag_cnt: i32,
}

impl SceneHeader {
    pub const SIZE: usize = 132;

    pub fn parse(b: &[u8]) -> Result<Self> {
        if b.len() < Self::SIZE {
            bail!("dat: too small for header");
        }
        let r = |ofs: usize| -> i32 {
            i32::from_le_bytes([b[ofs], b[ofs + 1], b[ofs + 2], b[ofs + 3]])
        };
        Ok(Self {
            header_size: r(0),
            scn_ofs: r(4),
            scn_size: r(8),
            str_index_list_ofs: r(12),
            str_index_cnt: r(16),
            str_list_ofs: r(20),
            str_cnt: r(24),
            label_list_ofs: r(28),
            label_cnt: r(32),
            z_label_list_ofs: r(36),
            z_label_cnt: r(40),
            cmd_label_list_ofs: r(44),
            cmd_label_cnt: r(48),
            scn_prop_list_ofs: r(52),
            scn_prop_cnt: r(56),
            scn_prop_name_index_list_ofs: r(60),
            scn_prop_name_index_cnt: r(64),
            scn_prop_name_list_ofs: r(68),
            scn_prop_name_cnt: r(72),
            scn_cmd_list_ofs: r(76),
            scn_cmd_cnt: r(80),
            scn_cmd_name_index_list_ofs: r(84),
            scn_cmd_name_index_cnt: r(88),
            scn_cmd_name_list_ofs: r(92),
            scn_cmd_name_cnt: r(96),
            call_prop_name_index_list_ofs: r(100),
            call_prop_name_index_cnt: r(104),
            call_prop_name_list_ofs: r(108),
            call_prop_name_cnt: r(112),
            namae_list_ofs: r(116),
            namae_cnt: r(120),
            read_flag_list_ofs: r(124),
            read_flag_cnt: r(128),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SceneDat {
    pub header: SceneHeader,
    pub scn_bytes: Vec<u8>,

    pub strings: Vec<U16String>,
    pub labels: Vec<i32>,
    pub z_labels: Vec<i32>,

    pub cmd_labels: Vec<(i32, i32)>,
    pub scn_props: Vec<(i32, i32)>,
    pub scn_prop_names: Vec<U16String>,

    pub scn_cmds: Vec<i32>,
    pub scn_cmd_names: Vec<U16String>,

    pub call_prop_names: Vec<U16String>,

    pub namae_list: Vec<i32>,
    pub read_flag_list: Vec<i32>,
}

impl SceneDat {
    /// Convenience wrapper used by runtime/cli.
    /// The `name` is currently unused but kept for API parity with the C++ tooling.
    pub fn parse(name: String, bytes: Vec<u8>) -> anyhow::Result<Self> {
        let _ = name;
        crate::dat::parse(&bytes)
    }
}

pub fn parse(dat: &[u8]) -> Result<SceneDat> {
    let header = SceneHeader::parse(dat)?;

    let so = header.scn_ofs as isize;
    let ss = header.scn_size as isize;
    if so < 0 || ss <= 0 || (so + ss) as usize > dat.len() {
        bail!("dat: invalid scn_ofs/scn_size");
    }
    let scn_bytes = dat[so as usize..(so + ss) as usize].to_vec();

    let str_pairs = read_i32_pairs(dat, header.str_index_list_ofs, header.str_index_cnt)?;
    let strings = decode_xor_utf16le_strings(dat, &str_pairs, header.str_list_ofs)?;

    let labels = read_i32_list(dat, header.label_list_ofs, header.label_cnt)?;
    let z_labels = read_i32_list(dat, header.z_label_list_ofs, header.z_label_cnt)?;

    let cmd_labels = read_i32_pairs(dat, header.cmd_label_list_ofs, header.cmd_label_cnt)?;
    let scn_props = read_i32_pairs(dat, header.scn_prop_list_ofs, header.scn_prop_cnt)?;

    let spn_pairs = read_i32_pairs(
        dat,
        header.scn_prop_name_index_list_ofs,
        header.scn_prop_name_index_cnt,
    )?;
    let scn_prop_names = decode_utf16le_strings(dat, &spn_pairs, header.scn_prop_name_list_ofs)?;

    let scn_cmds = read_i32_list(dat, header.scn_cmd_list_ofs, header.scn_cmd_cnt)?;
    let scn_cmd_name_pairs = read_i32_pairs(
        dat,
        header.scn_cmd_name_index_list_ofs,
        header.scn_cmd_name_index_cnt,
    )?;
    let scn_cmd_names =
        decode_utf16le_strings(dat, &scn_cmd_name_pairs, header.scn_cmd_name_list_ofs)?;

    let call_prop_pairs = read_i32_pairs(
        dat,
        header.call_prop_name_index_list_ofs,
        header.call_prop_name_index_cnt,
    )?;
    let call_prop_names =
        decode_utf16le_strings(dat, &call_prop_pairs, header.call_prop_name_list_ofs)?;

    let namae_list = read_i32_list(dat, header.namae_list_ofs, header.namae_cnt)?;
    let read_flag_list = read_i32_list(dat, header.read_flag_list_ofs, header.read_flag_cnt)?;

    Ok(SceneDat {
        header,
        scn_bytes,
        strings,
        labels,
        z_labels,
        cmd_labels,
        scn_props,
        scn_prop_names,
        scn_cmds,
        scn_cmd_names,
        call_prop_names,
        namae_list,
        read_flag_list,
    })
}

fn read_i32_pairs(data: &[u8], ofs: i32, cnt: i32) -> Result<Vec<(i32, i32)>> {
    if ofs <= 0 || cnt <= 0 {
        return Ok(Vec::new());
    }
    let o = ofs as usize;
    let n = cnt as usize;
    let need = o + n * 8;
    if need > data.len() {
        bail!("dat: i32 pair list out of range");
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let p = o + i * 8;
        let a = i32::from_le_bytes([data[p], data[p + 1], data[p + 2], data[p + 3]]);
        let b = i32::from_le_bytes([data[p + 4], data[p + 5], data[p + 6], data[p + 7]]);
        out.push((a, b));
    }
    Ok(out)
}

fn read_i32_list(data: &[u8], ofs: i32, cnt: i32) -> Result<Vec<i32>> {
    if ofs <= 0 || cnt <= 0 {
        return Ok(Vec::new());
    }
    let o = ofs as usize;
    let n = cnt as usize;
    let need = o + n * 4;
    if need > data.len() {
        bail!("dat: i32 list out of range");
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let p = o + i * 4;
        let v = i32::from_le_bytes([data[p], data[p + 1], data[p + 2], data[p + 3]]);
        out.push(v);
    }
    Ok(out)
}

fn decode_utf16le_strings(
    data: &[u8],
    pairs: &[(i32, i32)],
    blob_ofs: i32,
) -> Result<Vec<U16String>> {
    let base = blob_ofs as isize;
    let mut out = Vec::with_capacity(pairs.len());
    for &(ofs_u16, ln_u16) in pairs {
        if ofs_u16 < 0 || ln_u16 < 0 {
            out.push(U16String::new());
            continue;
        }
        let a = base + (ofs_u16 as isize) * 2;
        let b = a + (ln_u16 as isize) * 2;
        if a < 0 || b < 0 || b as usize > data.len() {
            out.push(U16String::new());
            continue;
        }
        let mut u16s = Vec::with_capacity(ln_u16 as usize);
        let mut p = a as usize;
        for _ in 0..(ln_u16 as usize) {
            let w = u16::from_le_bytes([data[p], data[p + 1]]);
            u16s.push(w);
            p += 2;
        }
        out.push(U16String::from_vec(u16s));
    }
    Ok(out)
}

fn decode_xor_utf16le_strings(
    data: &[u8],
    pairs: &[(i32, i32)],
    blob_ofs: i32,
) -> Result<Vec<U16String>> {
    let base = blob_ofs as isize;
    let mut out = Vec::with_capacity(pairs.len());
    for (si, &(ofs_u16, ln_u16)) in pairs.iter().enumerate() {
        if ofs_u16 < 0 || ln_u16 < 0 {
            out.push(U16String::new());
            continue;
        }
        let a = base + (ofs_u16 as isize) * 2;
        let b = a + (ln_u16 as isize) * 2;
        if a < 0 || b < 0 || b as usize > data.len() {
            out.push(U16String::new());
            continue;
        }
        let key = (28807u32.wrapping_mul(si as u32) & 0xFFFF) as u16;
        let mut u16s = Vec::with_capacity(ln_u16 as usize);
        let mut p = a as usize;
        for _ in 0..(ln_u16 as usize) {
            let w = u16::from_le_bytes([data[p], data[p + 1]]);
            u16s.push(w ^ key);
            p += 2;
        }
        out.push(U16String::from_vec(u16s));
    }
    Ok(out)
}
