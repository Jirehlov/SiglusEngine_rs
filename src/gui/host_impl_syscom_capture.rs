#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct CaptureOptionHeader {
    header_size: i32,
    flag_offset: i32,
    flag_size: i32,
    flag_cnt: i32,
    str_flag_offset: i32,
    str_flag_size: i32,
    str_flag_cnt: i32,
}

impl CaptureOptionHeader {
    fn byte_len() -> usize {
        28
    }

    fn encode(self) -> [u8; 28] {
        let mut out = [0u8; 28];
        out[0..4].copy_from_slice(&self.header_size.to_le_bytes());
        out[4..8].copy_from_slice(&self.flag_offset.to_le_bytes());
        out[8..12].copy_from_slice(&self.flag_size.to_le_bytes());
        out[12..16].copy_from_slice(&self.flag_cnt.to_le_bytes());
        out[16..20].copy_from_slice(&self.str_flag_offset.to_le_bytes());
        out[20..24].copy_from_slice(&self.str_flag_size.to_le_bytes());
        out[24..28].copy_from_slice(&self.str_flag_cnt.to_le_bytes());
        out
    }

    fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < Self::byte_len() {
            return None;
        }
        let i = |off: usize| -> Option<i32> {
            Some(i32::from_le_bytes(data.get(off..off + 4)?.try_into().ok()?))
        };
        Some(Self {
            header_size: i(0)?,
            flag_offset: i(4)?,
            flag_size: i(8)?,
            flag_cnt: i(12)?,
            str_flag_offset: i(16)?,
            str_flag_size: i(20)?,
            str_flag_cnt: i(24)?,
        })
    }
}

macro_rules! impl_host_syscom_capture_methods {
    () => {
        fn on_syscom_create_capture_buffer(&mut self, width: i32, height: i32) {
            self.capture_buffer = Some(HostCaptureBuffer {
                width: width.max(0),
                height: height.max(0),
                ..HostCaptureBuffer::default()
            });
        }

        fn on_syscom_destroy_capture_buffer(&mut self) {
            self.capture_buffer = None;
        }

        fn on_syscom_capture_to_buffer(&mut self, x: i32, y: i32, save_png_path: &str) {
            let st = self.capture_buffer.get_or_insert_with(HostCaptureBuffer::default);
            st.origin_x = x;
            st.origin_y = y;
            st.png_path = save_png_path.to_string();
        }

        fn on_syscom_save_capture_buffer_to_file(&mut self, req: &siglus::vm::VmCaptureFileOp) -> bool {
            let ext = req.extension.trim().trim_start_matches('.').to_ascii_lowercase();
            if ext != "bmp" && ext != "png" {
                return false;
            }
            if ext == "png" && (!req.int_values.is_empty() || !req.str_values.is_empty()) {
                return false;
            }
            let path = resolve_capture_save_file_path(&self.base_dir, req);
            if path.as_os_str().is_empty() {
                return false;
            }
            if let Some(parent) = path.parent() {
                if std::fs::create_dir_all(parent).is_err() {
                    return false;
                }
            }

            let Some(snapshot) = self.capture_buffer.clone() else {
                return false;
            };
            let mut body = Vec::new();
            body.extend_from_slice(b"SSSUCAP2");
            body.extend_from_slice(&snapshot.width.to_le_bytes());
            body.extend_from_slice(&snapshot.height.to_le_bytes());
            body.extend_from_slice(&snapshot.origin_x.to_le_bytes());
            body.extend_from_slice(&snapshot.origin_y.to_le_bytes());
            let png_bytes = snapshot.png_path.as_bytes();
            body.extend_from_slice(&(png_bytes.len() as u32).to_le_bytes());
            body.extend_from_slice(png_bytes);
            if ext == "bmp" {
                let Some(bmp) = make_bmp_from_capture_buffer(&self.base_dir, &snapshot.png_path) else {
                    return false;
                };
                body.extend_from_slice(&bmp);
            }

            let mut option_body = Vec::new();
            let mut header = CaptureOptionHeader {
                header_size: CaptureOptionHeader::byte_len() as i32,
                ..CaptureOptionHeader::default()
            };
            header.flag_offset = header.header_size + option_body.len() as i32;
            for v in &req.int_values {
                option_body.extend_from_slice(&v.to_le_bytes());
            }
            header.flag_cnt = req.int_values.len() as i32;
            header.flag_size = req.int_values.len().saturating_mul(4) as i32;

            header.str_flag_offset = header.header_size + option_body.len() as i32;
            for s in &req.str_values {
                option_body.extend_from_slice(s.as_bytes());
                option_body.push(0);
            }
            header.str_flag_cnt = req.str_values.len() as i32;
            header.str_flag_size = (option_body.len() as i32 + header.header_size) - header.str_flag_offset;

            body.extend_from_slice(&header.encode());
            body.extend_from_slice(&option_body);
            std::fs::write(path, body).is_ok()
        }

        fn on_syscom_load_flag_from_capture_file(
            &mut self,
            req: &siglus::vm::VmCaptureFileOp,
        ) -> Option<siglus::vm::VmCaptureFlagPayload> {
            let ext = req.extension.trim().trim_start_matches('.').to_ascii_lowercase();
            if ext != "bmp" {
                error!("capture load only supports bmp flag payload, extension={}", ext);
                return None;
            }

            let path = resolve_capture_load_file_path(&self.base_dir, req)?;
            let content = std::fs::read(path).ok()?;
            parse_capture_payload(&content).map(|(buf, payload)| {
                self.capture_buffer = Some(buf);
                payload
            })
        }
    };
}

fn resolve_capture_save_file_path(
    base_dir: &std::path::Path,
    req: &siglus::vm::VmCaptureFileOp,
) -> std::path::PathBuf {
    let ext = req.extension.trim().trim_start_matches('.');
    let mut path = std::path::PathBuf::from(req.file_name.trim());
    if req.dialog_flag {
        if path.as_os_str().is_empty() {
            return std::path::PathBuf::new();
        }
        if !ext.is_empty() && path.extension().is_none() {
            path.set_extension(ext);
        }
        return if path.is_absolute() { path } else { base_dir.join(path) };
    }

    // C++ non-dialog path appends ".ext" to save_dir + file_name.
    let mut raw = req.file_name.trim().to_string();
    if !ext.is_empty() {
        raw.push('.');
        raw.push_str(ext);
    }
    base_dir.join(raw)
}

fn resolve_capture_load_file_path(
    base_dir: &std::path::Path,
    req: &siglus::vm::VmCaptureFileOp,
) -> Option<std::path::PathBuf> {
    let ext = req.extension.trim().trim_start_matches('.');
    let mut path = std::path::PathBuf::from(req.file_name.trim());
    if req.dialog_flag {
        if path.as_os_str().is_empty() {
            return None;
        }
        return Some(if path.is_absolute() { path } else { base_dir.join(path) });
    }
    if path.as_os_str().is_empty() {
        return None;
    }
    if !ext.is_empty() {
        let mut raw = req.file_name.trim().to_string();
        raw.push('.');
        raw.push_str(ext);
        path = std::path::PathBuf::from(raw);
    }
    Some(if path.is_absolute() { path } else { base_dir.join(path) })
}

fn parse_capture_payload(content: &[u8]) -> Option<(HostCaptureBuffer, siglus::vm::VmCaptureFlagPayload)> {
    if content.starts_with(b"SSSUCAP2") {
        return parse_capture_payload_v2(content);
    }
    if content.starts_with(b"SSSUCAP1") {
        return parse_capture_payload_v1(content);
    }
    error!("capture payload magic mismatch or truncated");
    None
}

fn parse_capture_payload_v2(content: &[u8]) -> Option<(HostCaptureBuffer, siglus::vm::VmCaptureFlagPayload)> {
    if content.len() < 28 {
        return None;
    }
    let mut off = 8usize;
    let next_i32 = |data: &[u8], off: &mut usize| -> Option<i32> {
        let end = off.checked_add(4)?;
        let b = data.get(*off..end)?;
        *off = end;
        Some(i32::from_le_bytes(b.try_into().ok()?))
    };
    let next_u32 = |data: &[u8], off: &mut usize| -> Option<u32> {
        let end = off.checked_add(4)?;
        let b = data.get(*off..end)?;
        *off = end;
        Some(u32::from_le_bytes(b.try_into().ok()?))
    };

    let width = next_i32(content, &mut off)?;
    let height = next_i32(content, &mut off)?;
    let origin_x = next_i32(content, &mut off)?;
    let origin_y = next_i32(content, &mut off)?;
    let png_len = next_u32(content, &mut off)? as usize;
    let png_end = off.checked_add(png_len)?;
    let png_path = String::from_utf8(content.get(off..png_end)?.to_vec()).ok()?;
    off = png_end;

    // C++ eng_syscom_capture.cpp: option area starts at bmp_size.
    let bmp_size = parse_bmp_size_and_validate(content, off)?;
    off = off.checked_add(bmp_size)?;
    let header_slice = content.get(off..off + CaptureOptionHeader::byte_len())?;
    let header = CaptureOptionHeader::decode(header_slice)?;
    off += CaptureOptionHeader::byte_len();

    if header.header_size as usize != CaptureOptionHeader::byte_len() {
        error!("capture option header_size mismatch: {}", header.header_size);
        return None;
    }

    let option_base = off.checked_sub(CaptureOptionHeader::byte_len())?;
    let option_end = content.len();
    let section = |offset: i32, size: i32| -> Option<&[u8]> {
        if offset < header.header_size || size < 0 {
            return None;
        }
        let start = option_base.checked_add((offset - header.header_size) as usize)?;
        let end = start.checked_add(size as usize)?;
        if end > option_end {
            return None;
        }
        content.get(start..end)
    };

    let int_raw = section(header.flag_offset, header.flag_size)?;
    let mut payload = siglus::vm::VmCaptureFlagPayload::default();
    for i in 0..header.flag_cnt.max(0) as usize {
        let b = int_raw.get(i * 4..i * 4 + 4)?;
        payload.int_values.push(i32::from_le_bytes(b.try_into().ok()?));
    }

    let str_raw = section(header.str_flag_offset, header.str_flag_size)?;
    let mut parts = str_raw.split(|b| *b == 0);
    for _ in 0..header.str_flag_cnt.max(0) {
        let Some(part) = parts.next() else {
            error!("capture string flag payload truncated");
            return None;
        };
        payload.str_values.push(String::from_utf8(part.to_vec()).ok()?);
    }

    Some((
        HostCaptureBuffer {
            width,
            height,
            origin_x,
            origin_y,
            png_path,
        },
        payload,
    ))
}

fn make_bmp_stub(width: i32, height: i32) -> Vec<u8> {
    // Minimal BMP header (14 + 40) with BGRA32 payload layout.
    let w = width.max(1) as u32;
    let h = height.max(1) as u32;
    let pixel_bytes = w.saturating_mul(h).saturating_mul(4);
    let file_size = 54u32.saturating_add(pixel_bytes);
    let mut out = vec![0u8; file_size as usize];
    out[0] = b'B';
    out[1] = b'M';
    out[2..6].copy_from_slice(&file_size.to_le_bytes());
    out[10..14].copy_from_slice(&54u32.to_le_bytes());
    out[14..18].copy_from_slice(&40u32.to_le_bytes());
    out[18..22].copy_from_slice(&(w as i32).to_le_bytes());
    out[22..26].copy_from_slice(&(h as i32).to_le_bytes());
    out[26..28].copy_from_slice(&1u16.to_le_bytes());
    out[28..30].copy_from_slice(&32u16.to_le_bytes());
    out[30..34].copy_from_slice(&0u32.to_le_bytes());
    out[34..38].copy_from_slice(&pixel_bytes.to_le_bytes());
    out
}

fn make_bmp_from_capture_buffer(base_dir: &std::path::Path, png_path: &str) -> Option<Vec<u8>> {
    if png_path.trim().is_empty() {
        return None;
    }
    let png = std::path::PathBuf::from(png_path);
    let png_full = if png.is_absolute() { png } else { base_dir.join(png) };
    if let Ok(img) = image::open(&png_full) {
        use std::io::Cursor;
        let rgba = img.to_rgba8();
        let dyn_img = image::DynamicImage::ImageRgba8(rgba);
        let mut out = Vec::new();
        if dyn_img
            .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Bmp)
            .is_ok()
            && out.len() >= 54
        {
            return Some(out);
        }
    }
    None
}


fn parse_bmp_size_and_validate(content: &[u8], bmp_off: usize) -> Option<usize> {
    let hdr = content.get(bmp_off..bmp_off + 54)?;
    if hdr[0] != b'B' || hdr[1] != b'M' {
        error!("capture bmp header magic mismatch");
        return None;
    }
    let bmp_size = u32::from_le_bytes(hdr[2..6].try_into().ok()?) as usize;
    let bpp = u16::from_le_bytes(hdr[28..30].try_into().ok()?);
    let compression = u32::from_le_bytes(hdr[30..34].try_into().ok()?);
    if !(bpp == 24 || bpp == 32) {
        error!("capture bmp header invalid bit depth: {}", bpp);
        return None;
    }
    if compression != 0 {
        error!("capture bmp header unsupported compression: {}", compression);
        return None;
    }
    if bmp_size < 54 || bmp_off.saturating_add(bmp_size) > content.len() {
        error!("capture bmp size out of range: {}", bmp_size);
        return None;
    }
    Some(bmp_size)
}

fn parse_capture_payload_v1(content: &[u8]) -> Option<(HostCaptureBuffer, siglus::vm::VmCaptureFlagPayload)> {
    let mut off = 8usize;
    let next_i32 = |data: &[u8], off: &mut usize| -> Option<i32> {
        let end = off.checked_add(4)?;
        let b = data.get(*off..end)?;
        *off = end;
        Some(i32::from_le_bytes(b.try_into().ok()?))
    };
    let next_u32 = |data: &[u8], off: &mut usize| -> Option<u32> {
        let end = off.checked_add(4)?;
        let b = data.get(*off..end)?;
        *off = end;
        Some(u32::from_le_bytes(b.try_into().ok()?))
    };
    let width = next_i32(content, &mut off)?;
    let height = next_i32(content, &mut off)?;
    let origin_x = next_i32(content, &mut off)?;
    let origin_y = next_i32(content, &mut off)?;
    let png_len = next_u32(content, &mut off)? as usize;
    let int_cnt = next_u32(content, &mut off)? as usize;
    let str_cnt = next_u32(content, &mut off)? as usize;
    let png_end = off.checked_add(png_len)?;
    let png_path = String::from_utf8(content.get(off..png_end)?.to_vec()).ok()?;
    off = png_end;
    let mut payload = siglus::vm::VmCaptureFlagPayload::default();
    for _ in 0..int_cnt {
        payload.int_values.push(next_i32(content, &mut off)?);
    }
    for _ in 0..str_cnt {
        let len = next_u32(content, &mut off)? as usize;
        let end = off.checked_add(len)?;
        payload
            .str_values
            .push(String::from_utf8(content.get(off..end)?.to_vec()).ok()?);
        off = end;
    }
    Some((
        HostCaptureBuffer {
            width,
            height,
            origin_x,
            origin_y,
            png_path,
        },
        payload,
    ))
}
