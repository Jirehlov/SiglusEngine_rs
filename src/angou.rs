//! Helpers for Siglus encryption/decryption utilities.
//!
//! This module currently implements:
//! - XOR cycling (the common primitive)
//! - `exe_angou_element` (derive the 16-byte exe key element)
//! - Finding `暗号.dat` (first line) or `key.txt` under a directory.
//!
//! Full `SOURCE_ANGOU` (original_sources) decryption is intentionally left as
//! a future step.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use encoding_rs::SHIFT_JIS;

use crate::angou_consts::{self, source_angou};

/// XOR `buf` in-place with `key` cycling, starting from `start_index` into `key`.
pub fn xor_cycle_inplace(buf: &mut [u8], key: &[u8], start_index: usize) {
    if key.is_empty() {
        return;
    }
    let mut j = start_index % key.len();
    for b in buf.iter_mut() {
        *b ^= key[j];
        j += 1;
        if j == key.len() {
            j = 0;
        }
    }
}

/// Derive the 16-byte exe element from the first-line bytes of `暗号.dat`.
///
/// Matches the utility implementation:
/// `r = EXE_ORG; r[b] ^= angou_bytes[a] cycling over max(len(r), len(angou_bytes))`.
pub fn exe_angou_element(angou_bytes: &[u8]) -> [u8; 16] {
    let mut r = [0u8; 16];
    if angou_consts::EXE_ORG.len() == 16 {
        r.copy_from_slice(angou_consts::EXE_ORG);
    }
    if angou_bytes.is_empty() {
        return r;
    }

    let n = angou_bytes.len();
    let m = r.len();
    let cnt = if n < m { m } else { n };
    let mut a = 0usize;
    let mut b = 0usize;
    for _ in 0..cnt {
        r[b] ^= angou_bytes[a];
        a += 1;
        b += 1;
        if a == n {
            a = 0;
        }
        if b == m {
            b = 0;
        }
    }
    r
}

fn first_line_bytes(raw: &[u8]) -> &[u8] {
    let mut end = raw.len();
    for (i, &ch) in raw.iter().enumerate() {
        if ch == b'\n' {
            end = i;
            break;
        }
    }
    let mut slice = &raw[..end];
    if slice.ends_with(b"\r") {
        slice = &slice[..slice.len().saturating_sub(1)];
    }
    slice
}

fn decode_first_line_guess(raw: &[u8]) -> Option<String> {
    let mut line = first_line_bytes(raw);
    if line.starts_with(b"\xEF\xBB\xBF") {
        line = &line[3..];
    }
    if let Ok(s) = std::str::from_utf8(line) {
        return Some(s.to_string());
    }

    let (cow, _, had_errors) = SHIFT_JIS.decode(line);
    if !had_errors {
        return Some(cow.into_owned());
    }
    Some(cow.into_owned())
}

fn angou_line_bytes(raw: &[u8]) -> Option<Vec<u8>> {
    let line = decode_first_line_guess(raw)?;
    let (cow, _, _) = SHIFT_JIS.encode(&line);
    let bytes = cow.into_owned();
    if bytes.len() < 8 {
        return None;
    }
    Some(bytes)
}
pub fn read_exe_el_from_angou_dat(path: &Path) -> Result<[u8; 16]> {
    // Cap read to 4KB to prevent OOM if user has a large file named incorrectly
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::Read::take(file, 4096);
    let mut raw = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut raw)?;

    let line = angou_line_bytes(&raw).ok_or_else(|| anyhow::anyhow!("angou.dat: empty"))?;
    Ok(exe_angou_element(&line))
}

pub fn exe_el_from_angou_bytes(raw: &[u8]) -> Result<[u8; 16]> {
    let line = angou_line_bytes(raw).ok_or_else(|| anyhow::anyhow!("angou.dat: empty"))?;
    Ok(exe_angou_element(&line))
}

fn parse_key_txt_str(s: &str) -> Result<[u8; 16]> {
    let mut bytes: Vec<u8> = Vec::new();

    for tok in s
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == ':')
        .filter(|t| !t.is_empty())
    {
        let t = tok.trim();
        if t.is_empty() {
            continue;
        }
        let v = if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
            u32::from_str_radix(hex, 16).ok()
        } else if t
            .chars()
            .any(|c| c.is_ascii_hexdigit() && c.is_ascii_alphabetic())
        {
            // contains a-f
            u32::from_str_radix(t, 16).ok()
        } else {
            t.parse::<u32>().ok()
        };
        if let Some(v) = v {
            bytes.push((v & 0xFF) as u8);
        }
    }

    if bytes.len() < 16 {
        bail!("key.txt: need 16 bytes, got {}", bytes.len());
    }
    bytes.truncate(16);
    let mut out = [0u8; 16];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub fn read_exe_el_from_key_txt(path: &Path) -> Result<[u8; 16]> {
    // Cap read to 4KB
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::Read::take(file, 4096);
    let mut raw = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut raw)?;

    if raw.len() == 16 {
        let mut out = [0u8; 16];
        out.copy_from_slice(&raw);
        return Ok(out);
    }
    let s = String::from_utf8_lossy(&raw);
    parse_key_txt_str(&s)
}

fn is_named(p: &Path, name: &str) -> bool {
    p.file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case(name))
}

pub fn is_angou_dat_name(name: &str) -> bool {
    name.starts_with("暗号") && name.to_ascii_lowercase().ends_with(".dat")
}

fn is_angou_dat_path(p: &Path) -> bool {
    p.file_name()
        .and_then(|s| s.to_str())
        .is_some_and(is_angou_dat_name)
}

fn walk_paths(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let rd = match fs::read_dir(&d) {
            Ok(x) => x,
            Err(_) => continue,
        };
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                if recursive {
                    stack.push(p);
                }
            } else {
                out.push(p);
            }
        }
    }
    out
}

/// Find exe_el candidates under a directory.
///
/// Priority:
/// 1) `暗号.dat` (first line -> exe_angou_element)
/// 2) `key.txt` (16 bytes or a list)
pub fn find_exe_el(dir: &Path, recursive: bool) -> Option<[u8; 16]> {
    let files = walk_paths(dir, recursive);

    // Prefer angou*.dat
    for p in &files {
        if is_angou_dat_path(p) {
            if let Ok(el) = read_exe_el_from_angou_dat(p) {
                return Some(el);
            }
        }
    }

    // Fallback key.txt
    for p in &files {
        if is_named(p, "key.txt") {
            if let Ok(el) = read_exe_el_from_key_txt(p) {
                return Some(el);
            }
        }
    }

    None
}

fn md5_dword(md5_code: &[u8], ofs: usize) -> u32 {
    if ofs + 4 > md5_code.len() {
        return 0;
    }
    u32::from_le_bytes([
        md5_code[ofs],
        md5_code[ofs + 1],
        md5_code[ofs + 2],
        md5_code[ofs + 3],
    ])
}

fn decode_utf16le(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    let mut u16s = Vec::with_capacity(bytes.len() / 2);
    let mut iter = bytes.chunks_exact(2);
    for ch in iter.by_ref() {
        u16s.push(u16::from_le_bytes([ch[0], ch[1]]));
    }
    String::from_utf16_lossy(&u16s)
}

#[allow(clippy::too_many_arguments)]
fn tile_copy(
    dst: &mut [u8],
    src: &[u8],
    bx: usize,
    by: usize,
    mask: &[u8],
    tx: usize,
    ty: usize,
    repx: i32,
    repy: i32,
    rev: bool,
    lim: u8,
) {
    if dst.is_empty() || src.is_empty() || tx == 0 || ty == 0 {
        return;
    }

    let x0 = if repx <= 0 {
        ((-repx) as usize) % tx
    } else {
        (tx - ((repx as usize) % tx)) % tx
    };

    let y0 = if repy <= 0 {
        ((-repy) as usize) % ty
    } else {
        (ty - ((repy as usize) % ty)) % ty
    };

    for y in 0..by {
        let tyi = (y0 + y) % ty;
        let ty_offset = tyi * tx;
        let y_offset = y * bx;

        for x in 0..bx {
            let mask_idx = ty_offset + ((x0 + x) % tx);
            if mask_idx >= mask.len() {
                continue;
            }

            let v = mask[mask_idx];
            let i = (y_offset + x) * 4;

            let condition = if rev { v < lim } else { v >= lim };

            if condition && i + 4 <= dst.len() && i + 4 <= src.len() {
                dst[i..i + 4].copy_from_slice(&src[i..i + 4]);
            }
        }
    }
}

pub fn source_angou_decrypt(enc: &[u8]) -> Result<(Vec<u8>, String)> {
    let eg = source_angou::EASY_CODE;
    let mg = source_angou::MASK_CODE;
    let lg = source_angou::LAST_CODE;
    let ng = source_angou::NAME_CODE;
    let hs = source_angou::HEADER_SIZE as usize;

    if eg.is_empty() || mg.is_empty() || lg.is_empty() || ng.is_empty() || hs == 0 {
        bail!("source_angou: missing codes/params");
    }
    if enc.len() < hs + 4 {
        return Ok((Vec::new(), String::new()));
    }

    let mut dec = enc.to_vec();
    xor_cycle_inplace(&mut dec, lg, source_angou::LAST_INDEX as usize);
    let ver = u32::from_le_bytes([dec[0], dec[1], dec[2], dec[3]]);
    if ver != 1 {
        bail!("source_angou: bad version");
    }
    let md5_code = &dec[4..hs];
    let name_len = u32::from_le_bytes([dec[hs], dec[hs + 1], dec[hs + 2], dec[hs + 3]]) as usize;
    let mut p = hs + 4;
    if p + name_len > dec.len() {
        bail!("source_angou: truncated name");
    }
    let mut name_bytes = dec[p..p + name_len].to_vec();
    xor_cycle_inplace(&mut name_bytes, ng, source_angou::NAME_INDEX as usize);
    let name = decode_utf16le(&name_bytes);
    p += name_len;

    let lzsz = md5_dword(md5_code, 64) as usize;
    let mw = (md5_dword(md5_code, source_angou::MASK_W_MD5_I as usize)
        % source_angou::MASK_W_SUR as u32)
        + source_angou::MASK_W_ADD as u32;
    let mh = (md5_dword(md5_code, source_angou::MASK_H_MD5_I as usize)
        % source_angou::MASK_H_SUR as u32)
        + source_angou::MASK_H_ADD as u32;
    let mw = mw as usize;
    let mh = mh as usize;
    let mut mask = vec![0u8; mw * mh];
    let mut ind = source_angou::MASK_INDEX as usize;
    let mut mi = source_angou::MASK_MD5_INDEX as usize;
    for v in mask.iter_mut() {
        let mask_md5_ofs = (mi % 16) * 4;
        *v = mg[ind % mg.len()] ^ md5_code[mask_md5_ofs];
        ind += 1;
        mi = (mi + 1) % 16;
    }

    let mapw = (md5_dword(md5_code, source_angou::MAP_W_MD5_I as usize)
        % source_angou::MAP_W_SUR as u32)
        + source_angou::MAP_W_ADD as u32;
    let mapw = mapw as usize;
    let bh = (lzsz + 1) / 2;
    let dh = (bh + 3) / 4;
    let maph = (dh + (mapw - 1)) / mapw;
    let mapt = mapw * maph * 4;
    let dp1 = dec
        .get(p..p + mapt)
        .ok_or_else(|| anyhow::anyhow!("source_angou: truncated payload"))?;
    let dp2 = dec
        .get(p + mapt..p + mapt * 2)
        .ok_or_else(|| anyhow::anyhow!("source_angou: truncated payload"))?;

    let mut lzb = vec![0u8; mapt * 2];
    let repx = source_angou::TILE_REPX;
    let repy = source_angou::TILE_REPY;
    let lim = source_angou::TILE_LIMIT as u8;

    if bh + mapt > lzb.len() {
        bail!("source_angou: invalid buffer geometry");
    }
    {
        let sp1 = &mut lzb[0..mapt];
        tile_copy(sp1, dp1, mapw, maph, &mask, mw, mh, repx, repy, false, lim);
        tile_copy(sp1, dp2, mapw, maph, &mask, mw, mh, repx, repy, true, lim);
    }
    {
        let sp2 = &mut lzb[bh..bh + mapt];
        tile_copy(sp2, dp2, mapw, maph, &mask, mw, mh, repx, repy, false, lim);
        tile_copy(sp2, dp1, mapw, maph, &mask, mw, mh, repx, repy, true, lim);
    }

    let mut lz = lzb;
    lz.truncate(lzsz);
    xor_cycle_inplace(&mut lz, eg, source_angou::EASY_INDEX as usize);
    let raw = crate::lzss::unpack(&lz)?;
    Ok((raw, name))
}
