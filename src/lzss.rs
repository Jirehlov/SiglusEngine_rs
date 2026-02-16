//! LZSS decompression (Siglus-style wrapper: <packed_size:u32><orig_size:u32><payload...>)

use anyhow::{bail, Result};

/// Detects the Siglus LZSS container header.
pub fn looks_like_lzss(blob: &[u8]) -> bool {
    if blob.len() < 8 {
        return false;
    }
    let pack_sz = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    let org_sz = u32::from_le_bytes([blob[4], blob[5], blob[6], blob[7]]) as usize;
    if pack_sz != blob.len() {
        return false;
    }
    if org_sz == 0 || org_sz > 0x4000_0000 {
        return false;
    }
    true
}

/// Unpack Siglus LZSS container.
pub fn unpack(container: &[u8]) -> Result<Vec<u8>> {
    if container.len() < 8 {
        bail!("lzss: too small");
    }

    let org_sz =
        u32::from_le_bytes([container[4], container[5], container[6], container[7]]) as usize;
    if org_sz == 0 {
        return Ok(Vec::new());
    }

    let mut out = Vec::with_capacity(org_sz);
    let mut si = 8usize;

    while out.len() < org_sz && si < container.len() {
        let mut fl = container[si];
        si += 1;

        for _ in 0..8 {
            if out.len() >= org_sz {
                break;
            }

            if (fl & 1) != 0 {
                if si < container.len() {
                    out.push(container[si]);
                    si += 1;
                }
            } else {
                if si + 1 >= container.len() {
                    break;
                }
                let tok = (container[si] as usize) | ((container[si + 1] as usize) << 8);
                si += 2;
                let ofs = tok >> 4;
                let len = (tok & 0xF) + 2;
                let st = out.len().wrapping_sub(ofs);
                for j in 0..len {
                    if out.len() >= org_sz {
                        break;
                    }
                    let idx = st.wrapping_add(j);
                    if idx < out.len() {
                        out.push(out[idx]);
                    }
                }
            }
            fl >>= 1;
        }
    }

    Ok(out)
}
