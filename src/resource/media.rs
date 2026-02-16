use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone)]
pub struct NwaHeader {
    pub channels: u16,
    pub bits_per_sample: u16,
    pub samples_per_sec: u32,
    pub pack_mod: i32,
    pub zero_mod: i32,
    pub unit_cnt: u32,
    pub original_size: u32,
    pub pack_size: u32,
    pub sample_cnt: u32,
    pub unit_sample_cnt: u32,
    pub last_sample_cnt: u32,
    pub last_sample_pack_size: u32,
}

#[derive(Debug, Clone)]
pub struct OvkEntry {
    pub entry_no: i32,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct OvkInfo {
    pub entries: Vec<OvkEntry>,
}

#[derive(Debug, Clone)]
pub struct OmvInfo {
    pub file_size: usize,
    pub oggs_offset: usize,
    pub ogv_size: usize,
    pub stream_kinds: Vec<String>,
}

pub fn read_nwa_header(path: &Path) -> Result<NwaHeader> {
    let data = fs::read(path).with_context(|| format!("read nwa: {}", path.display()))?;
    if data.len() < 44 {
        bail!("nwa header too short");
    }
    Ok(NwaHeader {
        channels: u16::from_le_bytes([data[0], data[1]]),
        bits_per_sample: u16::from_le_bytes([data[2], data[3]]),
        samples_per_sec: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        pack_mod: i32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        zero_mod: i32::from_le_bytes([data[12], data[13], data[14], data[15]]),
        unit_cnt: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
        original_size: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),
        pack_size: u32::from_le_bytes([data[24], data[25], data[26], data[27]]),
        sample_cnt: u32::from_le_bytes([data[28], data[29], data[30], data[31]]),
        unit_sample_cnt: u32::from_le_bytes([data[32], data[33], data[34], data[35]]),
        last_sample_cnt: u32::from_le_bytes([data[36], data[37], data[38], data[39]]),
        last_sample_pack_size: u32::from_le_bytes([data[40], data[41], data[42], data[43]]),
    })
}

pub fn decode_owp(path: &Path, key: u8) -> Result<Vec<u8>> {
    let data = fs::read(path).with_context(|| format!("read owp: {}", path.display()))?;
    if data.starts_with(b"OggS") {
        return Ok(data);
    }

    let xored: Vec<u8> = data.iter().map(|b| b ^ key).collect();
    if xored.starts_with(b"OggS") {
        return Ok(xored);
    }

    if data.len() >= 4 {
        let auto_key = data[0] ^ b'O';
        let auto: Vec<u8> = data.iter().map(|b| b ^ auto_key).collect();
        if auto.starts_with(b"OggS") {
            return Ok(auto);
        }
    }

    bail!("owp decode failed: output is not OggS")
}

pub fn read_ovk(path: &Path) -> Result<OvkInfo> {
    let data = fs::read(path).with_context(|| format!("read ovk: {}", path.display()))?;
    if data.len() < 4 {
        bail!("ovk too short");
    }
    let cnt = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut entries = Vec::with_capacity(cnt);
    let mut off = 4usize;
    for _ in 0..cnt {
        if off + 16 > data.len() {
            bail!("ovk table truncated");
        }
        let size = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        let offset =
            u32::from_le_bytes([data[off + 4], data[off + 5], data[off + 6], data[off + 7]]);
        let entry_no =
            i32::from_le_bytes([data[off + 8], data[off + 9], data[off + 10], data[off + 11]]);
        entries.push(OvkEntry {
            entry_no,
            offset,
            size,
        });
        off += 16;
    }
    Ok(OvkInfo { entries })
}

pub fn read_omv(path: &Path) -> Result<OmvInfo> {
    let data = fs::read(path).with_context(|| format!("read omv: {}", path.display()))?;
    let oggs_offset = find_oggs_offset(&data).context("omv missing embedded ogg")?;
    let kinds = parse_ogg_stream_kinds(&data, oggs_offset);
    Ok(OmvInfo {
        file_size: data.len(),
        oggs_offset,
        ogv_size: data.len() - oggs_offset,
        stream_kinds: kinds,
    })
}

fn find_oggs_offset(data: &[u8]) -> Result<usize> {
    for i in 0..data.len().saturating_sub(4) {
        if &data[i..i + 4] == b"OggS" {
            if i + 4 < data.len() && data[i + 4] == 0 {
                return Ok(i);
            }
        }
    }
    bail!("OggS not found")
}

fn parse_ogg_stream_kinds(data: &[u8], start: usize) -> Vec<String> {
    let mut kinds_by_serial = BTreeMap::<u32, String>::new();
    let mut bufs = BTreeMap::<u32, Vec<u8>>::new();

    let mut off = start;
    let mut pages = 0usize;
    while off + 27 <= data.len() && pages < 128 {
        if &data[off..off + 4] != b"OggS" {
            break;
        }
        let ver = data[off + 4];
        let header_type = data[off + 5];
        if ver != 0 {
            break;
        }
        let serial = u32::from_le_bytes([
            data[off + 14],
            data[off + 15],
            data[off + 16],
            data[off + 17],
        ]);
        let seg_cnt = data[off + 26] as usize;
        off += 27;
        if off + seg_cnt > data.len() {
            break;
        }
        let segs = &data[off..off + seg_cnt];
        off += seg_cnt;
        let payload_len: usize = segs.iter().map(|&v| v as usize).sum();
        if off + payload_len > data.len() {
            break;
        }
        let payload = &data[off..off + payload_len];
        off += payload_len;

        if (header_type & 0x01) == 0 {
            bufs.entry(serial).or_default().clear();
        }
        let cur = bufs.entry(serial).or_default();
        let mut p = 0usize;
        for &seg_len in segs {
            let seg_len = seg_len as usize;
            cur.extend_from_slice(&payload[p..p + seg_len]);
            p += seg_len;
            if seg_len < 255 {
                if !kinds_by_serial.contains_key(&serial) {
                    if let Some(k) = detect_packet_kind(cur) {
                        kinds_by_serial.insert(serial, k.to_string());
                    }
                }
                cur.clear();
            }
        }

        pages += 1;
        if kinds_by_serial.len() >= 2 && pages >= 8 {
            break;
        }
    }

    let mut ordered = BTreeSet::new();
    for (_s, k) in kinds_by_serial {
        ordered.insert(k);
    }
    ordered.into_iter().collect()
}

fn detect_packet_kind(pkt: &[u8]) -> Option<&'static str> {
    if pkt.len() >= 7 && pkt[0] == 0x01 && &pkt[1..7] == b"vorbis" {
        return Some("vorbis");
    }
    if pkt.len() >= 7 && pkt[0] == 0x80 && &pkt[1..7] == b"theora" {
        return Some("theora");
    }
    if pkt.len() >= 8 && &pkt[0..8] == b"OpusHead" {
        return Some("opus");
    }
    if pkt.len() >= 8 && &pkt[0..8] == b"Speex   " {
        return Some("speex");
    }
    None
}
