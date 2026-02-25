use super::*;

struct DbsParseResult {
    rows: Vec<Vec<siglus::vm::PropValue>>,
    row_calls: Vec<i32>,
    col_calls: Vec<i32>,
    col_types: Vec<u8>,
}

fn parse_int_cell_token(tok: &str) -> siglus::vm::PropValue {
    let t = tok.trim();
    if let Ok(v) = t.parse::<i32>() {
        siglus::vm::PropValue::Int(v)
    } else {
        siglus::vm::PropValue::Str(t.trim_matches('"').to_string())
    }
}

fn tile_copy_rgba_mask(
    dst: &mut [u8],
    src: &[u8],
    bx: usize,
    by: usize,
    mask: &[u8],
    tx: usize,
    ty: usize,
    rev: bool,
    lim: u8,
) {
    if tx == 0 || ty == 0 {
        return;
    }
    for y in 0..by {
        for x in 0..bx {
            let m = mask[(y % ty) * tx + (x % tx)];
            let ok = if rev { m < lim } else { m >= lim };
            let i = (y * bx + x) * 4;
            if ok && i + 4 <= dst.len() && i + 4 <= src.len() {
                dst[i..i + 4].copy_from_slice(&src[i..i + 4]);
            }
        }
    }
}

fn xor_u32_inplace(buf: &mut [u8], key: u32) {
    let n = (buf.len() / 4) * 4;
    for i in (0..n).step_by(4) {
        let mut v = [0u8; 4];
        v.copy_from_slice(&buf[i..i + 4]);
        let x = u32::from_le_bytes(v) ^ key;
        buf[i..i + 4].copy_from_slice(&x.to_le_bytes());
    }
}

fn parse_dbs_table(path: &Path) -> Option<DbsParseResult> {
    const DBS_XOR32_CODE: u32 = 0x89F4622D;
    const DBS_XOR32_CODE_A: u32 = 0x7190C70E;
    const DBS_XOR32_CODE_B: u32 = 0x499BF135;
    const DBS_MAP_WIDTH: usize = 16;
    const DBS_TILE: [u8; 25] = [
        255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 255, 255, 255, 0, 255, 0, 0, 255, 0, 0, 255, 0,
        255, 255, 255,
    ];

    let blob = std::fs::read(path).ok()?;
    if blob.len() < 12 {
        return None;
    }
    let m_type = i32::from_le_bytes(blob[0..4].try_into().ok()?);
    let mut packed = blob[4..].to_vec();
    xor_u32_inplace(&mut packed, DBS_XOR32_CODE);
    let unpack = siglus::lzss::unpack(&packed).ok()?;
    if unpack.is_empty() {
        return None;
    }

    let yl = unpack.len() / (DBS_MAP_WIDTH * 4);
    if yl == 0 {
        return None;
    }
    let mut temp_a = vec![0u8; unpack.len()];
    let mut temp_b = vec![0u8; unpack.len()];
    tile_copy_rgba_mask(
        &mut temp_a,
        &unpack,
        DBS_MAP_WIDTH,
        yl,
        &DBS_TILE,
        5,
        5,
        false,
        128,
    );
    tile_copy_rgba_mask(
        &mut temp_b,
        &unpack,
        DBS_MAP_WIDTH,
        yl,
        &DBS_TILE,
        5,
        5,
        true,
        128,
    );
    xor_u32_inplace(&mut temp_a, DBS_XOR32_CODE_A);
    xor_u32_inplace(&mut temp_b, DBS_XOR32_CODE_B);
    let mut decoded = vec![0u8; unpack.len()];
    tile_copy_rgba_mask(
        &mut decoded,
        &temp_a,
        DBS_MAP_WIDTH,
        yl,
        &DBS_TILE,
        5,
        5,
        false,
        128,
    );
    tile_copy_rgba_mask(
        &mut decoded,
        &temp_b,
        DBS_MAP_WIDTH,
        yl,
        &DBS_TILE,
        5,
        5,
        true,
        128,
    );

    if decoded.len() < 28 {
        return None;
    }
    let i32at = |off: usize| -> Option<i32> {
        Some(i32::from_le_bytes(
            decoded.get(off..off + 4)?.try_into().ok()?,
        ))
    };
    let data_size_raw = i32at(0)?;
    let row_cnt = i32at(4)?.max(0) as usize;
    let col_cnt = i32at(8)?.max(0) as usize;
    let row_ofs_raw = i32at(12)?;
    let col_ofs_raw = i32at(16)?;
    let data_ofs_raw = i32at(20)?;
    let str_ofs_raw = i32at(24)?;

    let mut chosen = None;
    for scale in [1usize, 4usize] {
        let row_ofs = row_ofs_raw.max(0) as usize * scale;
        let col_ofs = col_ofs_raw.max(0) as usize * scale;
        let data_ofs = data_ofs_raw.max(0) as usize * scale;
        let str_ofs = str_ofs_raw.max(0) as usize * scale;
        let data_size = if data_size_raw <= 0 {
            decoded.len()
        } else {
            (data_size_raw.max(0) as usize * scale).min(decoded.len())
        };
        let row_hdr = row_cnt.saturating_mul(4);
        let col_hdr = col_cnt.saturating_mul(8);
        let cell_bytes = row_cnt.saturating_mul(col_cnt).saturating_mul(4);
        if row_ofs + row_hdr <= decoded.len()
            && col_ofs + col_hdr <= decoded.len()
            && data_ofs + cell_bytes <= decoded.len()
            && str_ofs <= data_size
            && row_ofs <= col_ofs
            && col_ofs <= data_ofs
            && data_ofs <= str_ofs
        {
            chosen = Some((data_size, row_ofs, col_ofs, data_ofs, str_ofs));
            break;
        }
    }
    let (data_size, row_ofs, col_ofs, data_ofs, str_ofs) = chosen?;

    let mut row_calls = Vec::with_capacity(row_cnt);
    for r in 0..row_cnt {
        let off = row_ofs + r * 4;
        row_calls.push(i32at(off).unwrap_or(r as i32));
    }

    let mut col_calls = Vec::with_capacity(col_cnt);
    let mut col_types = Vec::with_capacity(col_cnt);
    for c in 0..col_cnt {
        let off = col_ofs + c * 8;
        col_calls.push(i32at(off).unwrap_or(c as i32));
        col_types.push(i32at(off + 4).unwrap_or(0) as u8);
    }
    let str_blob = &decoded[str_ofs..data_size];
    let mut rows = Vec::with_capacity(row_cnt);
    for r in 0..row_cnt {
        let mut row = Vec::with_capacity(col_cnt);
        for c in 0..col_cnt {
            let off = data_ofs + (r * col_cnt + c) * 4;
            let raw = u32::from_le_bytes(decoded.get(off..off + 4)?.try_into().ok()?);
            let dt = col_types.get(c).copied().unwrap_or(0);
            if dt == b'S' {
                let s = if m_type == 0 {
                    let o = raw as usize;
                    if o < str_blob.len() {
                        let e = str_blob[o..]
                            .iter()
                            .position(|b| *b == 0)
                            .map(|v| o + v)
                            .unwrap_or(str_blob.len());
                        encoding_rs::SHIFT_JIS.decode(&str_blob[o..e]).0.to_string()
                    } else {
                        String::new()
                    }
                } else {
                    let o = raw as usize;
                    if o + 1 < str_blob.len() {
                        let mut e = o;
                        while e + 1 < str_blob.len() {
                            if str_blob[e] == 0 && str_blob[e + 1] == 0 {
                                break;
                            }
                            e += 2;
                        }
                        String::from_utf16_lossy(
                            &str_blob[o..e]
                                .chunks(2)
                                .map(|ch| u16::from_le_bytes([ch[0], *ch.get(1).unwrap_or(&0)]))
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        String::new()
                    }
                };
                row.push(siglus::vm::PropValue::Str(s));
            } else {
                row.push(siglus::vm::PropValue::Int(i32::from_le_bytes(
                    raw.to_le_bytes(),
                )));
            }
        }
        rows.push(row);
    }
    Some(DbsParseResult {
        rows,
        row_calls,
        col_calls,
        col_types,
    })
}

fn parse_cgtable_file(
    path: &Path,
) -> Option<(
    usize,
    std::collections::BTreeMap<String, i32>,
    Vec<[i32; 5]>,
    Vec<i32>,
)> {
    let blob = std::fs::read(path).ok()?;
    if blob.len() < 32 {
        return None;
    }
    let head = &blob[0..16];
    let is_v1 = head.starts_with(b"CGTABLE\0") || head.starts_with(b"CGTABLE");
    let is_v2 = head.starts_with(b"CGTABLE2\0") || head.starts_with(b"CGTABLE2");
    if !is_v1 && !is_v2 {
        return None;
    }
    let rec_size = if is_v2 { 60usize } else { 36usize };
    let cnt = i32::from_le_bytes(blob[16..20].try_into().ok()?).max(0) as usize;
    let mut packed = blob[32..].to_vec();
    siglus::angou::xor_cycle_inplace(&mut packed, siglus::angou_consts::EASY_ANGOU_CODE, 0);
    let payload = siglus::lzss::unpack(&packed).ok()?;
    if payload.len() < cnt.saturating_mul(rec_size) {
        return None;
    }
    let mut map = std::collections::BTreeMap::new();
    let mut codes = vec![[0; 5]; cnt];
    let mut code_exist_cnt = vec![0; cnt];
    for i in 0..cnt {
        let o = i * rec_size;
        let name_raw = &payload[o..o + 32];
        let end = name_raw.iter().position(|b| *b == 0).unwrap_or(32);
        let mut name = encoding_rs::SHIFT_JIS
            .decode(&name_raw[..end])
            .0
            .to_string();
        name = name.to_ascii_uppercase();
        let flag = i32::from_le_bytes(payload[o + 32..o + 36].try_into().ok()?);
        map.insert(name, flag);
        if rec_size >= 60 {
            codes[i] = [
                i32::from_le_bytes(payload[o + 36..o + 40].try_into().ok()?),
                i32::from_le_bytes(payload[o + 40..o + 44].try_into().ok()?),
                i32::from_le_bytes(payload[o + 44..o + 48].try_into().ok()?),
                i32::from_le_bytes(payload[o + 48..o + 52].try_into().ok()?),
                i32::from_le_bytes(payload[o + 52..o + 56].try_into().ok()?),
            ];
            code_exist_cnt[i] = i32::from_le_bytes(payload[o + 56..o + 60].try_into().ok()?);
        }
    }
    Some((cnt, map, codes, code_exist_cnt))
}

fn load_database_table_file(
    path: &Path,
) -> (Vec<Vec<siglus::vm::PropValue>>, Vec<i32>, Vec<i32>, Vec<u8>) {
    if path
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("dbs"))
    {
        if let Some(parsed) = parse_dbs_table(path) {
            return (
                parsed.rows,
                parsed.row_calls,
                parsed.col_calls,
                parsed.col_types,
            );
        }
    }

    let raw = match std::fs::read(path) {
        Ok(v) => v,
        Err(_) => return (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
    };
    let (txt, _, _) = encoding_rs::SHIFT_JIS.decode(&raw);
    let mut rows = Vec::new();
    for line in txt.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with("//") {
            continue;
        }
        let delim = if line.contains('\t') { '\t' } else { ',' };
        let row = line
            .split(delim)
            .map(parse_int_cell_token)
            .collect::<Vec<_>>();
        rows.push(row);
    }
    (rows, Vec::new(), Vec::new(), Vec::new())
}

pub(super) fn parse_vm_resource_bootstrap(
    cfg: &siglus::gameexe::GameexeConfig,
    base_dir: &Path,
) -> (
    Vec<Vec<Vec<siglus::vm::PropValue>>>,
    usize,
    std::collections::BTreeMap<String, i32>,
    Vec<String>,
    Vec<Vec<i32>>,
    Vec<Vec<i32>>,
    Vec<Vec<u8>>,
    Vec<[i32; 5]>,
    Vec<i32>,
) {
    let db_cnt = cfg
        .first_values("DATABASE.CNT")
        .and_then(|v| v.first())
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let mut database_tables = vec![Vec::new(); db_cnt];
    let mut database_row_calls = vec![Vec::new(); db_cnt];
    let mut database_col_calls = vec![Vec::new(); db_cnt];
    let mut database_col_types = vec![Vec::new(); db_cnt];
    for (idx, table) in database_tables.iter_mut().enumerate() {
        let key = format!("DATABASE.{idx}");
        if let Some(path) = cfg.first_values(&key).and_then(|v| v.first()) {
            let p = base_dir.join(path.trim_matches('"'));
            let (rows, row_calls, col_calls, col_types) = load_database_table_file(&p);
            *table = rows;
            database_row_calls[idx] = row_calls;
            database_col_calls[idx] = col_calls;
            database_col_types[idx] = col_types;
        }
    }

    let mut cg_flag_cnt = cfg
        .first_values("CGTABLE_FLAG_CNT")
        .and_then(|v| v.first())
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let mut cg_name_to_flag = std::collections::BTreeMap::new();
    let mut cg_group_codes = Vec::new();
    let mut cg_code_exist_cnt = Vec::new();
    if let Some(cg_path) = cfg
        .first_values("CGTABLE_FILE")
        .and_then(|v| v.first())
        .map(|s| base_dir.join(s.trim_matches('"')))
        && let Some((cnt, map, codes, code_exist)) = parse_cgtable_file(&cg_path)
    {
        cg_flag_cnt = cg_flag_cnt.max(cnt);
        cg_name_to_flag = map;
        cg_group_codes = codes;
        cg_code_exist_cnt = code_exist;
    }
    if cg_name_to_flag.is_empty() {
        for idx in 0..cg_flag_cnt {
            let key = format!("CGTABLE_NAME.{idx}");
            if let Some(name) = cfg.first_values(&key).and_then(|v| v.first())
                && !name.is_empty()
            {
                cg_name_to_flag.insert(name.clone(), idx as i32);
            }
        }
    }

    if cg_group_codes.len() < cg_flag_cnt {
        cg_group_codes.resize(cg_flag_cnt, [0; 5]);
    }
    if cg_code_exist_cnt.len() < cg_flag_cnt {
        cg_code_exist_cnt.resize(cg_flag_cnt, 0);
    }

    let bgm_cnt = cfg
        .first_values("BGM.CNT")
        .and_then(|v| v.first())
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let mut bgm_names = Vec::new();
    for idx in 0..bgm_cnt {
        let key = format!("BGM.{idx}");
        if let Some(values) = cfg.first_values(&key)
            && let Some(name) = values.first()
            && !name.is_empty()
        {
            bgm_names.push(name.clone());
        }
    }

    (
        database_tables,
        cg_flag_cnt,
        cg_name_to_flag,
        bgm_names,
        database_row_calls,
        database_col_calls,
        database_col_types,
        cg_group_codes,
        cg_code_exist_cnt,
    )
}
