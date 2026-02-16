//! .pck scene pack reader.
//!
//! Supported:
//! - UTF-16LE scene name tables
//! - Scene blob extraction
//! - Optional `exe angou` XOR (when `scn_data_exe_angou_mod != 0`) if an `暗号.dat`
//!   or `key.txt` can be found near the pack file
//! - Easy XOR + LZSS unpack when the payload matches the Siglus LZSS header

use std::{collections::HashMap, path::Path};

use anyhow::{bail, Context, Result};
use widestring::U16String;

use crate::{angou, lzss};

#[derive(Clone, Debug)]
pub struct PackHeader {
    pub header_size: i32,

    pub inc_prop_list_ofs: i32,
    pub inc_prop_cnt: i32,
    pub inc_prop_name_index_list_ofs: i32,
    pub inc_prop_name_index_cnt: i32,
    pub inc_prop_name_list_ofs: i32,
    pub inc_prop_name_cnt: i32,

    pub inc_cmd_list_ofs: i32,
    pub inc_cmd_cnt: i32,
    pub inc_cmd_name_index_list_ofs: i32,
    pub inc_cmd_name_index_cnt: i32,
    pub inc_cmd_name_list_ofs: i32,
    pub inc_cmd_name_cnt: i32,

    pub scn_name_index_list_ofs: i32,
    pub scn_name_index_cnt: i32,
    pub scn_name_list_ofs: i32,
    pub scn_name_cnt: i32,

    pub scn_data_index_list_ofs: i32,
    pub scn_data_index_cnt: i32,
    pub scn_data_list_ofs: i32,
    pub scn_data_cnt: i32,

    pub scn_data_exe_angou_mod: i32,
    pub original_source_header_size: i32,
}

#[derive(Clone, Debug)]
pub struct Pack {
    pub header: PackHeader,

    pub inc_prop_list: Vec<(i32, i32)>,
    pub inc_cmd_list: Vec<(i32, i32)>,

    pub inc_prop_names: Vec<U16String>,
    pub inc_cmd_names: Vec<U16String>,
    pub scene_names: Vec<U16String>,

    pub scene_name_to_index: HashMap<String, usize>,
    pub inc_prop_name_to_index: HashMap<String, usize>,
    pub inc_cmd_name_to_index: HashMap<String, usize>,

    /// Optional exe-angou element used to decrypt scene data.
    pub exe_el: Option<[u8; 16]>,

    /// Per-scene raw .dat bytes (decrypted+decompressed when detectable).
    pub scenes: Vec<Vec<u8>>,
}

pub fn read_file(path: &Path) -> Result<Pack> {
    let b = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    parse_with_dir(&b, Some(dir))
}

/// Parse a pack from bytes.
///
/// Note: This version does not try to find `暗号.dat`/`key.txt` and therefore will
/// not decrypt `scn_data_exe_angou_mod` packs.
pub fn parse(data: &[u8]) -> Result<Pack> {
    parse_with_dir(data, None)
}

fn parse_with_dir(data: &[u8], search_dir: Option<&Path>) -> Result<Pack> {
    if data.len() < 92 {
        bail!("pck: too small");
    }

    let h = PackHeader {
        header_size: read_i32(data, 0)?,
        inc_prop_list_ofs: read_i32(data, 4)?,
        inc_prop_cnt: read_i32(data, 8)?,
        inc_prop_name_index_list_ofs: read_i32(data, 12)?,
        inc_prop_name_index_cnt: read_i32(data, 16)?,
        inc_prop_name_list_ofs: read_i32(data, 20)?,
        inc_prop_name_cnt: read_i32(data, 24)?,
        inc_cmd_list_ofs: read_i32(data, 28)?,
        inc_cmd_cnt: read_i32(data, 32)?,
        inc_cmd_name_index_list_ofs: read_i32(data, 36)?,
        inc_cmd_name_index_cnt: read_i32(data, 40)?,
        inc_cmd_name_list_ofs: read_i32(data, 44)?,
        inc_cmd_name_cnt: read_i32(data, 48)?,
        scn_name_index_list_ofs: read_i32(data, 52)?,
        scn_name_index_cnt: read_i32(data, 56)?,
        scn_name_list_ofs: read_i32(data, 60)?,
        scn_name_cnt: read_i32(data, 64)?,
        scn_data_index_list_ofs: read_i32(data, 68)?,
        scn_data_index_cnt: read_i32(data, 72)?,
        scn_data_list_ofs: read_i32(data, 76)?,
        scn_data_cnt: read_i32(data, 80)?,
        scn_data_exe_angou_mod: read_i32(data, 84)?,
        original_source_header_size: read_i32(data, 88)?,
    };

    // Header sanity (matches util/pck.py validation)
    let hs = if h.header_size != 0 {
        h.header_size as usize
    } else {
        92
    };
    if hs < 92 || hs > data.len() {
        bail!("pck: invalid header_size={}", h.header_size);
    }
    for &(k, v) in &[
        ("scn_name_index_list_ofs", h.scn_name_index_list_ofs),
        ("scn_data_index_list_ofs", h.scn_data_index_list_ofs),
        ("scn_data_list_ofs", h.scn_data_list_ofs),
    ] {
        if v < 0 || (v as usize) > data.len() {
            bail!("pck: invalid {}={}", k, v);
        }
    }

    let inc_prop_list = read_i32_pairs(data, h.inc_prop_list_ofs, h.inc_prop_cnt)?;
    let inc_cmd_list = read_i32_pairs(data, h.inc_cmd_list_ofs, h.inc_cmd_cnt)?;

    let inc_prop_name_pairs = read_i32_pairs(
        data,
        h.inc_prop_name_index_list_ofs,
        h.inc_prop_name_index_cnt,
    )?;
    let inc_cmd_name_pairs = read_i32_pairs(
        data,
        h.inc_cmd_name_index_list_ofs,
        h.inc_cmd_name_index_cnt,
    )?;
    let scn_name_pairs = read_i32_pairs(data, h.scn_name_index_list_ofs, h.scn_name_index_cnt)?;

    let inc_prop_names =
        decode_utf16le_strings(data, &inc_prop_name_pairs, h.inc_prop_name_list_ofs)?;
    let inc_cmd_names = decode_utf16le_strings(data, &inc_cmd_name_pairs, h.inc_cmd_name_list_ofs)?;
    let scene_names = decode_utf16le_strings(data, &scn_name_pairs, h.scn_name_list_ofs)?;

    let mut scene_name_to_index = HashMap::new();
    for (i, nm) in scene_names.iter().enumerate() {
        scene_name_to_index.insert(nm.to_string_lossy(), i);
    }
    let mut inc_prop_name_to_index = HashMap::new();
    for (i, nm) in inc_prop_names.iter().enumerate() {
        inc_prop_name_to_index.insert(nm.to_string_lossy(), i);
    }
    let mut inc_cmd_name_to_index = HashMap::new();
    for (i, nm) in inc_cmd_names.iter().enumerate() {
        inc_cmd_name_to_index.insert(nm.to_string_lossy(), i);
    }

    let scn_data_pairs = read_i32_pairs(data, h.scn_data_index_list_ofs, h.scn_data_index_cnt)?;
    if h.scn_data_cnt < 0 {
        bail!("pck: negative scn_data_cnt");
    }
    if scn_data_pairs.len() < h.scn_data_cnt as usize {
        bail!("pck: scn_data_index_cnt too small");
    }

    let mut exe_el = if h.scn_data_exe_angou_mod != 0 {
        search_dir.and_then(|d| angou::find_exe_el(d, false))
    } else {
        None
    };
    if exe_el.is_none() && h.scn_data_exe_angou_mod != 0 && h.original_source_header_size > 0 {
        exe_el = find_exe_el_from_original_sources(data, &h, &scn_data_pairs);
    }

    let mut scenes = Vec::with_capacity(h.scn_data_cnt as usize);
    for i in 0..(h.scn_data_cnt as usize) {
        let (ofs, sz) = scn_data_pairs[i];
        if ofs < 0 || sz < 0 {
            scenes.push(Vec::new());
            continue;
        }
        let a = (h.scn_data_list_ofs as isize) + (ofs as isize);
        let b = a + (sz as isize);
        if a < 0 || b < 0 || (b as usize) > data.len() {
            bail!("pck: scene[{i}] out of range");
        }

        let mut blob = data[a as usize..b as usize].to_vec();

        let out = if h.original_source_header_size > 0 {
            // Matches tnm_lexer.cpp: exe-angou -> easy-angou -> lzss when original sources exist.
            if let Some(el) = &exe_el {
                if h.scn_data_exe_angou_mod != 0 {
                    angou::xor_cycle_inplace(&mut blob, el, 0);
                }
            }
            if !crate::angou_consts::EASY_ANGOU_CODE.is_empty() {
                angou::xor_cycle_inplace(&mut blob, crate::angou_consts::EASY_ANGOU_CODE, 0);
            }
            lzss::unpack(&blob).with_context(|| format!("lzss unpack scene[{i}]"))?
        } else {
            // Heuristic fallback for packs without original sources.
            let mut cand = blob.clone();
            if !crate::angou_consts::EASY_ANGOU_CODE.is_empty() {
                angou::xor_cycle_inplace(&mut cand, crate::angou_consts::EASY_ANGOU_CODE, 0);
            }
            if lzss::looks_like_lzss(&cand) {
                lzss::unpack(&cand).with_context(|| format!("lzss unpack scene[{i}]"))?
            } else if lzss::looks_like_lzss(&blob) {
                lzss::unpack(&blob).with_context(|| format!("lzss unpack scene[{i}]"))?
            } else {
                blob
            }
        };

        scenes.push(out);
    }

    Ok(Pack {
        header: h,
        inc_prop_list,
        inc_cmd_list,
        inc_prop_names,
        inc_cmd_names,
        scene_names,
        scene_name_to_index,
        inc_prop_name_to_index,
        inc_cmd_name_to_index,
        exe_el,
        scenes,
    })
}

#[inline]
fn read_i32(data: &[u8], ofs: usize) -> Result<i32> {
    if ofs + 4 > data.len() {
        bail!("truncated i32 at {ofs}");
    }
    Ok(i32::from_le_bytes([
        data[ofs],
        data[ofs + 1],
        data[ofs + 2],
        data[ofs + 3],
    ]))
}

fn read_i32_pairs(data: &[u8], ofs: i32, cnt: i32) -> Result<Vec<(i32, i32)>> {
    if ofs <= 0 || cnt <= 0 {
        return Ok(Vec::new());
    }
    let ofs = ofs as usize;
    let cnt = cnt as usize;
    let mut out = Vec::with_capacity(cnt);
    for k in 0..cnt {
        let p = ofs + k * 8;
        if p + 8 > data.len() {
            bail!("truncated pair list at {p}");
        }
        let a = i32::from_le_bytes([data[p], data[p + 1], data[p + 2], data[p + 3]]);
        let b = i32::from_le_bytes([data[p + 4], data[p + 5], data[p + 6], data[p + 7]]);
        out.push((a, b));
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

fn scn_data_blob_end(header: &PackHeader, scn_data_pairs: &[(i32, i32)]) -> Option<usize> {
    let mut max_end: i64 = 0;
    for (ofs, sz) in scn_data_pairs {
        let ofs = *ofs as i64;
        let sz = *sz as i64;
        if ofs < 0 || sz < 0 {
            continue;
        }
        max_end = max_end.max(ofs + sz);
    }
    if max_end <= 0 {
        return None;
    }
    let base = header.scn_data_list_ofs as i64;
    let end = base + max_end;
    if end <= 0 {
        None
    } else {
        Some(end as usize)
    }
}

/// Try to find the exe-angou key from a PCK file's original-sources (OS) segment.
///
/// This reads the PCK, parses just enough of the header to locate the OS segment,
/// then searches for `暗号.dat` among the encrypted original-source blobs.
pub fn find_exe_el_from_pck_file(path: &Path) -> Option<[u8; 16]> {
    let data = std::fs::read(path).ok()?;
    if data.len() < 92 {
        return None;
    }
    let h = PackHeader {
        header_size: read_i32(&data, 0).ok()?,
        inc_prop_list_ofs: read_i32(&data, 4).ok()?,
        inc_prop_cnt: read_i32(&data, 8).ok()?,
        inc_prop_name_index_list_ofs: read_i32(&data, 12).ok()?,
        inc_prop_name_index_cnt: read_i32(&data, 16).ok()?,
        inc_prop_name_list_ofs: read_i32(&data, 20).ok()?,
        inc_prop_name_cnt: read_i32(&data, 24).ok()?,
        inc_cmd_list_ofs: read_i32(&data, 28).ok()?,
        inc_cmd_cnt: read_i32(&data, 32).ok()?,
        inc_cmd_name_index_list_ofs: read_i32(&data, 36).ok()?,
        inc_cmd_name_index_cnt: read_i32(&data, 40).ok()?,
        inc_cmd_name_list_ofs: read_i32(&data, 44).ok()?,
        inc_cmd_name_cnt: read_i32(&data, 48).ok()?,
        scn_name_index_list_ofs: read_i32(&data, 52).ok()?,
        scn_name_index_cnt: read_i32(&data, 56).ok()?,
        scn_name_list_ofs: read_i32(&data, 60).ok()?,
        scn_name_cnt: read_i32(&data, 64).ok()?,
        scn_data_index_list_ofs: read_i32(&data, 68).ok()?,
        scn_data_index_cnt: read_i32(&data, 72).ok()?,
        scn_data_list_ofs: read_i32(&data, 76).ok()?,
        scn_data_cnt: read_i32(&data, 80).ok()?,
        scn_data_exe_angou_mod: read_i32(&data, 84).ok()?,
        original_source_header_size: read_i32(&data, 88).ok()?,
    };
    if h.original_source_header_size <= 0 {
        return None;
    }
    let scn_data_pairs =
        read_i32_pairs(&data, h.scn_data_index_list_ofs, h.scn_data_index_cnt).ok()?;
    find_exe_el_from_original_sources(&data, &h, &scn_data_pairs)
}

fn find_exe_el_from_original_sources(
    data: &[u8],
    header: &PackHeader,
    scn_data_pairs: &[(i32, i32)],
) -> Option<[u8; 16]> {
    let hsz = header.original_source_header_size as usize;
    let blob_end = scn_data_blob_end(header, scn_data_pairs)?;
    let mut pos = blob_end;
    if hsz == 0 || pos + hsz > data.len() {
        return None;
    }

    let size_list_enc = &data[pos..pos + hsz];
    let (size_bytes, _) = angou::source_angou_decrypt(size_list_enc).ok()?;
    if size_bytes.len() % 4 != 0 {
        return None;
    }
    let mut sizes = Vec::with_capacity(size_bytes.len() / 4);
    for chunk in size_bytes.chunks_exact(4) {
        sizes.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as usize);
    }
    pos += hsz;

    for sz in sizes {
        if sz == 0 || pos + sz > data.len() {
            break;
        }
        let enc_blob = &data[pos..pos + sz];
        if let Ok((raw, name)) = angou::source_angou_decrypt(enc_blob) {
            if let Some(file_name) = Path::new(&name).file_name().and_then(|s| s.to_str()) {
                if angou::is_angou_dat_name(file_name) {
                    if let Ok(el) = angou::exe_el_from_angou_bytes(&raw) {
                        return Some(el);
                    }
                }
            }
        }
        pos += sz;
    }
    None
}
