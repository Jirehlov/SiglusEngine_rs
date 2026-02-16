use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use hex_literal::hex;

#[derive(Debug, Clone)]
pub struct G00Info {
    pub ty: u8,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub cut_count: Option<i32>,
    pub cuts: Vec<G00CutInfo>,
}

#[derive(Debug, Clone)]
pub struct G00CutInfo {
    pub index: usize,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug)]
pub struct G00Image {
    pub width: u16,
    pub height: u16,
    pub cuts: Vec<image::DynamicImage>,
}

pub fn read_g00_info(path: &Path) -> Result<G00Info> {
    let data = fs::read(path).with_context(|| format!("read g00: {}", path.display()))?;
    if data.is_empty() {
        bail!("g00 empty");
    }
    let ty = data[0];
    let mut info = G00Info {
        ty,
        width: None,
        height: None,
        cut_count: None,
        cuts: Vec::new(),
    };

    match ty {
        0 | 1 => {
            if data.len() < 5 {
                bail!("g00 header too short");
            }
            info.width = Some(u16::from_le_bytes([data[1], data[2]]));
            info.height = Some(u16::from_le_bytes([data[3], data[4]]));
        }
        2 => {
            if data.len() < 9 {
                bail!("g00 type2 header too short");
            }
            info.width = Some(u16::from_le_bytes([data[1], data[2]]));
            info.height = Some(u16::from_le_bytes([data[3], data[4]]));
            let cut_count = i32::from_le_bytes([data[5], data[6], data[7], data[8]]);
            info.cut_count = Some(cut_count);

            let mut off = 9usize;
            let table_skip = (cut_count.max(0) as usize) * 24;
            off = off.saturating_add(table_skip);
            if off >= data.len() {
                return Ok(info);
            }

            let unp = crate::lzss::unpack(&data[off..]).context("g00 type2 lzss unpack")?;
            if unp.len() < 4 {
                return Ok(info);
            }
            let cnt = u32::from_le_bytes([unp[0], unp[1], unp[2], unp[3]]) as usize;
            for idx in 0..cnt {
                let o = 4 + idx * 8;
                if o + 8 > unp.len() {
                    break;
                }
                let cut_off = u32::from_le_bytes([unp[o], unp[o + 1], unp[o + 2], unp[o + 3]]);
                let cut_size = u32::from_le_bytes([unp[o + 4], unp[o + 5], unp[o + 6], unp[o + 7]]);
                if cut_off != 0
                    && cut_size as usize >= 116
                    && cut_off as usize + cut_size as usize <= unp.len()
                {
                    info.cuts.push(G00CutInfo {
                        index: idx,
                        offset: cut_off,
                        size: cut_size,
                    });
                }
            }
        }
        3 => {
            // JPEG XOR payload. We only expose type so callers can route handling.
        }
        _ => bail!("unknown g00 type: {ty}"),
    }

    Ok(info)
}

const G00_XOR_T: [u8; 256] = hex!(
    "450c85c07514e55d8b55ecc05b8bc38b81ff0000040085ff6a0076b043007649008b7de88b75a1e00c85c0c0757830440085ff7637811dd0ff000075448bb04345f88d55fc52007668000004006a438bb143006a05ff50ffd3a1e0040056152c440085c07409c3a15f5e338be55de030040081c6000081ef04008530440000005dc38b55f88d5e5b4dfc51c4045f8be54300ebd88b45ff15e883c05756522cb101008b7de88900e845f48b20506a4728005053ff1534e46ab143000c8b45006a8b4dec89088a85c045f0848b45107405f528010083c4526a08894583c22000e8e8f4fbffff8b8b5d450c83c074c5f853c40885c0755630448b1dd0f0a1e00083"
);

fn decode_g00_type3_jpeg(width: u16, height: u16, payload: &[u8]) -> Result<image::DynamicImage> {
    let dec: Vec<u8> = payload
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ G00_XOR_T[i & 0xFF])
        .collect();
    let img = image::load_from_memory(&dec).context("g00 type3 jpeg decode")?;
    let rgba = img.to_rgba8();
    if rgba.width() != width as u32 || rgba.height() != height as u32 {
        log::warn!(
            "g00 type3: header {}x{} differs from jpeg {}x{}",
            width,
            height,
            rgba.width(),
            rgba.height()
        );
    }
    Ok(image::DynamicImage::ImageRgba8(rgba))
}

fn bgra_to_rgba_image(width: u32, height: u32, mut data: Vec<u8>) -> Result<image::DynamicImage> {
    let expected = (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4);
    data.resize(expected, 0);
    for px in data.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    let buf = image::RgbaImage::from_raw(width, height, data)
        .context("failed to create RgbaImage from raw buffer")?;
    Ok(image::DynamicImage::ImageRgba8(buf))
}

fn unpack_lzss32(container: &[u8]) -> Result<Vec<u8>> {
    if container.len() < 8 {
        bail!("lzss32: too small");
    }
    let org_sz =
        u32::from_le_bytes([container[4], container[5], container[6], container[7]]) as usize;
    let mut out = Vec::with_capacity(org_sz);
    let mut si = 8usize;
    while out.len() < org_sz {
        if si >= container.len() {
            bail!("lzss32: eof in flag stream");
        }
        let mut flag = container[si];
        si += 1;
        for _ in 0..8 {
            if out.len() >= org_sz {
                break;
            }
            if (flag & 1) != 0 {
                if si + 3 > container.len() {
                    bail!("lzss32: eof in literal");
                }
                out.extend_from_slice(&[container[si], container[si + 1], container[si + 2], 255]);
                si += 3;
            } else {
                if si + 2 > container.len() {
                    bail!("lzss32: eof in backref");
                }
                let tok = u16::from_le_bytes([container[si], container[si + 1]]) as usize;
                si += 2;
                let off = (tok >> 4) * 4;
                let len = ((tok & 0xF) + 1) * 4;
                if off == 0 || off > out.len() {
                    bail!("lzss32: invalid backref");
                }
                let st = out.len() - off;
                for j in 0..len {
                    if out.len() >= org_sz {
                        break;
                    }
                    out.push(out[st + j]);
                }
            }
            flag >>= 1;
        }
    }
    Ok(out)
}

fn decode_type1_bgra(unpacked: &[u8], width: u16, height: u16) -> Result<Vec<u8>> {
    if unpacked.len() < 2 {
        bail!("g00 type1: palette header too short");
    }
    let pc = u16::from_le_bytes([unpacked[0], unpacked[1]]) as usize;
    let pal_off = 2usize;
    let idx_off = pal_off + pc * 4;
    let px_cnt = (width as usize) * (height as usize);
    if unpacked.len() < idx_off + px_cnt {
        bail!("g00 type1: palette/index data truncated");
    }
    let mut out = vec![0u8; px_cnt * 4];
    for i in 0..px_cnt {
        let pi = unpacked[idx_off + i] as usize;
        if pi >= pc {
            continue;
        }
        let p = pal_off + pi * 4;
        out[i * 4..i * 4 + 4].copy_from_slice(&unpacked[p..p + 4]);
    }
    Ok(out)
}

fn alpha_blit_bgra(
    canvas: &mut [u8],
    canvas_w: usize,
    canvas_h: usize,
    chip: &[u8],
    chip_w: usize,
    chip_h: usize,
    dst_x: usize,
    dst_y: usize,
) {
    for y in 0..chip_h {
        if dst_y + y >= canvas_h {
            break;
        }
        for x in 0..chip_w {
            if dst_x + x >= canvas_w {
                break;
            }
            let si = (y * chip_w + x) * 4;
            let di = ((dst_y + y) * canvas_w + (dst_x + x)) * 4;
            let a = chip[si + 3] as u32;
            if a == 0 {
                continue;
            }
            if a == 255 {
                canvas[di..di + 4].copy_from_slice(&chip[si..si + 4]);
                continue;
            }
            let ia = 255 - a;
            for c in 0..3 {
                canvas[di + c] =
                    ((chip[si + c] as u32 * a + canvas[di + c] as u32 * ia) / 255) as u8;
            }
            canvas[di + 3] = (a + (canvas[di + 3] as u32 * ia) / 255) as u8;
        }
    }
}

fn decode_type2_cut(block: &[u8]) -> Result<Option<image::DynamicImage>> {
    const G00_CUT_SZ: usize = 116;
    const G00_CHIP_SZ: usize = 92;
    if block.len() < G00_CUT_SZ {
        return Ok(None);
    }

    let chip_count = u16::from_le_bytes([block[2], block[3]]) as usize;
    // Align with utility parser: header is "<B x H 8i>", so cw/ch are the 7th/8th i32 fields
    // located at byte offsets 28..35.
    let cw = i32::from_le_bytes([block[28], block[29], block[30], block[31]]);
    let ch = i32::from_le_bytes([block[32], block[33], block[34], block[35]]);
    if cw <= 0 || ch <= 0 {
        return Ok(None);
    }
    let (cw, ch) = (cw as usize, ch as usize);
    let mut canvas = vec![0u8; cw.saturating_mul(ch).saturating_mul(4)];

    let mut pos = G00_CUT_SZ;
    for _ in 0..chip_count {
        if pos + G00_CHIP_SZ > block.len() {
            break;
        }
        let px = u16::from_le_bytes([block[pos], block[pos + 1]]) as usize;
        let py = u16::from_le_bytes([block[pos + 2], block[pos + 3]]) as usize;
        let xl = u16::from_le_bytes([block[pos + 6], block[pos + 7]]) as usize;
        let yl = u16::from_le_bytes([block[pos + 8], block[pos + 9]]) as usize;
        pos += G00_CHIP_SZ;
        let chip_len = xl.saturating_mul(yl).saturating_mul(4);
        if pos + chip_len > block.len() {
            break;
        }
        let chip = &block[pos..pos + chip_len];
        pos += chip_len;
        alpha_blit_bgra(&mut canvas, cw, ch, chip, xl, yl, px, py);
    }

    Ok(Some(bgra_to_rgba_image(cw as u32, ch as u32, canvas)?))
}

pub fn load_g00_images(path: &Path) -> Result<G00Image> {
    let data = fs::read(path).with_context(|| format!("read g00: {}", path.display()))?;
    if data.is_empty() {
        bail!("g00 empty");
    }
    let ty = data[0];

    match ty {
        0 | 1 => {
            if data.len() < 5 {
                bail!("g00 header too short");
            }
            let w = u16::from_le_bytes([data[1], data[2]]);
            let h = u16::from_le_bytes([data[3], data[4]]);
            let payload = &data[5..];

            let bgra = if ty == 0 {
                unpack_lzss32(payload).context("g00 type0 lzss32 unpack")?
            } else {
                let unp = crate::lzss::unpack(payload).context("g00 type1 lzss unpack")?;
                decode_type1_bgra(&unp, w, h)?
            };

            let img = bgra_to_rgba_image(w as u32, h as u32, bgra)?;
            Ok(G00Image {
                width: w,
                height: h,
                cuts: vec![img],
            })
        }
        2 => {
            if data.len() < 9 {
                bail!("g00 type2 header too short");
            }
            let w = u16::from_le_bytes([data[1], data[2]]);
            let h = u16::from_le_bytes([data[3], data[4]]);
            let cut_count = i32::from_le_bytes([data[5], data[6], data[7], data[8]]);
            let table_off = 9usize.saturating_add(cut_count.max(0) as usize * 24);
            if table_off >= data.len() {
                bail!(
                    "g00 type2 invalid cut table offset: cut_count={}, table_off={}, file_size={}",
                    cut_count,
                    table_off,
                    data.len()
                );
            }

            let unp = crate::lzss::unpack(&data[table_off..]).context("g00 type2 table unpack")?;
            if unp.len() < 4 {
                bail!("g00 type2 unpacked table too short: {}", unp.len());
            }
            let cnt = u32::from_le_bytes([unp[0], unp[1], unp[2], unp[3]]) as usize;
            let mut cuts = Vec::with_capacity(cnt);
            let mut valid_entries = 0usize;
            for idx in 0..cnt {
                let o = 4 + idx * 8;
                if o + 8 > unp.len() {
                    break;
                }
                let cut_off =
                    u32::from_le_bytes([unp[o], unp[o + 1], unp[o + 2], unp[o + 3]]) as usize;
                let cut_size =
                    u32::from_le_bytes([unp[o + 4], unp[o + 5], unp[o + 6], unp[o + 7]]) as usize;
                if cut_off == 0 || cut_size < 116 || cut_off + cut_size > unp.len() {
                    continue;
                }
                valid_entries += 1;
                if let Some(img) = decode_type2_cut(&unp[cut_off..cut_off + cut_size])? {
                    cuts.push(img);
                }
            }
            if cuts.is_empty() {
                bail!(
                    "g00 type2 no cuts: declared_cut_count={}, unpacked_table_count={}, valid_entries={}, unpacked_size={}",
                    cut_count,
                    cnt,
                    valid_entries,
                    unp.len()
                );
            }
            Ok(G00Image {
                width: w,
                height: h,
                cuts,
            })
        }
        3 => {
            if data.len() < 5 {
                bail!("g00 type3 header too short");
            }
            let w = u16::from_le_bytes([data[1], data[2]]);
            let h = u16::from_le_bytes([data[3], data[4]]);
            let img = decode_g00_type3_jpeg(w, h, &data[5..])?;
            Ok(G00Image {
                width: w,
                height: h,
                cuts: vec![img],
            })
        }
        _ => bail!("unknown g00 type: {ty}"),
    }
}
