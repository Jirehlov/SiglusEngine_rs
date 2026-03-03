#[derive(Debug, Clone)]
struct ObjectStringStyleState {
    moji_size: i32,
    moji_space_x: i32,
    moji_space_y: i32,
    moji_cnt: i32,
    moji_color: i32,
    shadow_color: i32,
    fuchi_color: i32,
    shadow_mode: i32,
}

impl Default for ObjectStringStyleState {
    fn default() -> Self {
        Self {
            moji_size: 18,
            moji_space_x: 0,
            moji_space_y: 0,
            moji_cnt: 0,
            moji_color: 0xFFFFFF,
            shadow_color: 0,
            fuchi_color: 0,
            shadow_mode: -1,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ObjectNumberStyleState {
    keta_max: i32,
    disp_zero: i32,
    disp_sign: i32,
    tumeru_sign: i32,
    space_mod: i32,
    space: i32,
}

#[derive(Debug, Clone, Default)]
pub(super) struct ObjectButtonState {
    pub(super) button_no: i32,
    pub(super) group_no: i32,
    pub(super) action_no: i32,
    pub(super) se_no: i32,
    pub(super) push_keep: i32,
    pub(super) alpha_test: i32,
    pub(super) state: i32,
    pub(super) hit_state: i32,
    pub(super) real_state: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub(super) struct ObjectWeatherState {
    last_type: i32,
    params: Vec<i32>,
}

static OBJECT_STRING_STATE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), String>>,
> = std::sync::OnceLock::new();
static OBJECT_STRING_STYLE_STATE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), ObjectStringStyleState>>,
> = std::sync::OnceLock::new();
static OBJECT_NUMBER_STATE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), i32>>,
> = std::sync::OnceLock::new();
static OBJECT_NUMBER_STYLE_STATE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), ObjectNumberStyleState>>,
> = std::sync::OnceLock::new();
static CUSTOM_BITMAP_FONT_FAMILIES: std::sync::OnceLock<
    Vec<std::collections::BTreeMap<char, [u8; 7]>>,
> = std::sync::OnceLock::new();
static OBJECT_BUTTON_STATE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), ObjectButtonState>>,
> = std::sync::OnceLock::new();
static OBJECT_WEATHER_STATE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), ObjectWeatherState>>,
> = std::sync::OnceLock::new();
static OBJECT_MOVIE_SEEK_STATE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), i32>>,
> = std::sync::OnceLock::new();

impl GuiHost {
    fn object_string_state_map(
        &self,
    ) -> &std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), String>> {
        OBJECT_STRING_STATE.get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()))
    }

    fn object_string_style_map(
        &self,
    ) -> &std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), ObjectStringStyleState>>
    {
        OBJECT_STRING_STYLE_STATE
            .get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()))
    }

    fn object_number_state_map(
        &self,
    ) -> &std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), i32>> {
        OBJECT_NUMBER_STATE.get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()))
    }

    fn object_number_style_map(
        &self,
    ) -> &std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), ObjectNumberStyleState>>
    {
        OBJECT_NUMBER_STYLE_STATE
            .get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()))
    }

    fn object_button_state_map(
        &self,
    ) -> &std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), ObjectButtonState>> {
        OBJECT_BUTTON_STATE.get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()))
    }

    fn object_weather_state_map(
        &self,
    ) -> &std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), ObjectWeatherState>> {
        OBJECT_WEATHER_STATE
            .get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()))
    }

    fn object_movie_seek_map(
        &self,
    ) -> &std::sync::Mutex<std::collections::BTreeMap<(StagePlane, i32), i32>> {
        OBJECT_MOVIE_SEEK_STATE
            .get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()))
    }

    fn set_object_string_state(&self, plane: StagePlane, object_index: i32, value: String) {
        if let Ok(mut guard) = self.object_string_state_map().lock() {
            guard.insert((plane, object_index), value);
        }
    }

    fn clear_object_string_state(&self, plane: StagePlane, object_index: i32) {
        if let Ok(mut guard) = self.object_string_state_map().lock() {
            guard.remove(&(plane, object_index));
        }
    }

    fn set_object_string_style_state(
        &self,
        plane: StagePlane,
        object_index: i32,
        style: ObjectStringStyleState,
    ) {
        if let Ok(mut guard) = self.object_string_style_map().lock() {
            guard.insert((plane, object_index), style);
        }
    }

    fn clear_object_string_style_state(&self, plane: StagePlane, object_index: i32) {
        if let Ok(mut guard) = self.object_string_style_map().lock() {
            guard.remove(&(plane, object_index));
        }
    }

    fn get_object_string_style_state(
        &self,
        plane: StagePlane,
        object_index: i32,
    ) -> ObjectStringStyleState {
        if let Ok(guard) = self.object_string_style_map().lock() {
            return guard
                .get(&(plane, object_index))
                .cloned()
                .unwrap_or_default();
        }
        ObjectStringStyleState::default()
    }

    fn set_object_number_state(&self, plane: StagePlane, object_index: i32, value: i32) {
        if let Ok(mut guard) = self.object_number_state_map().lock() {
            guard.insert((plane, object_index), value);
        }
    }

    fn clear_object_number_state(&self, plane: StagePlane, object_index: i32) {
        if let Ok(mut guard) = self.object_number_state_map().lock() {
            guard.remove(&(plane, object_index));
        }
    }

    pub(super) fn get_object_number_state(&self, plane: StagePlane, object_index: i32) -> i32 {
        if let Ok(guard) = self.object_number_state_map().lock() {
            return guard.get(&(plane, object_index)).copied().unwrap_or(0);
        }
        0
    }

    fn set_object_number_style_state(
        &self,
        plane: StagePlane,
        object_index: i32,
        style: ObjectNumberStyleState,
    ) {
        if let Ok(mut guard) = self.object_number_style_map().lock() {
            guard.insert((plane, object_index), style);
        }
    }

    fn clear_object_number_style_state(&self, plane: StagePlane, object_index: i32) {
        if let Ok(mut guard) = self.object_number_style_map().lock() {
            guard.remove(&(plane, object_index));
        }
    }

    fn get_object_number_style_state(
        &self,
        plane: StagePlane,
        object_index: i32,
    ) -> ObjectNumberStyleState {
        if let Ok(guard) = self.object_number_style_map().lock() {
            return guard
                .get(&(plane, object_index))
                .cloned()
                .unwrap_or_default();
        }
        ObjectNumberStyleState::default()
    }

    pub(super) fn set_object_button_state(
        &self,
        plane: StagePlane,
        object_index: i32,
        state: ObjectButtonState,
    ) {
        if let Ok(mut guard) = self.object_button_state_map().lock() {
            guard.insert((plane, object_index), state);
        }
    }

    pub(super) fn get_object_button_state(&self, plane: StagePlane, object_index: i32) -> ObjectButtonState {
        if let Ok(guard) = self.object_button_state_map().lock() {
            return guard
                .get(&(plane, object_index))
                .cloned()
                .unwrap_or_default();
        }
        ObjectButtonState::default()
    }

    pub(super) fn clear_object_button_state(&self, plane: StagePlane, object_index: i32) {
        if let Ok(mut guard) = self.object_button_state_map().lock() {
            guard.remove(&(plane, object_index));
        }
    }

    pub(super) fn set_object_weather_state(
        &self,
        plane: StagePlane,
        object_index: i32,
        state: ObjectWeatherState,
    ) {
        if let Ok(mut guard) = self.object_weather_state_map().lock() {
            guard.insert((plane, object_index), state);
        }
    }

    pub(super) fn clear_object_weather_state(&self, plane: StagePlane, object_index: i32) {
        if let Ok(mut guard) = self.object_weather_state_map().lock() {
            guard.remove(&(plane, object_index));
        }
    }

    pub(super) fn set_object_movie_seek_state(&self, plane: StagePlane, object_index: i32, seek: i32) {
        if let Ok(mut guard) = self.object_movie_seek_map().lock() {
            guard.insert((plane, object_index), seek);
        }
    }

    pub(super) fn get_object_movie_seek_state(&self, plane: StagePlane, object_index: i32) -> i32 {
        if let Ok(guard) = self.object_movie_seek_map().lock() {
            return guard.get(&(plane, object_index)).copied().unwrap_or(0);
        }
        0
    }

    pub(super) fn clear_object_movie_seek_state(&self, plane: StagePlane, object_index: i32) {
        if let Ok(mut guard) = self.object_movie_seek_map().lock() {
            guard.remove(&(plane, object_index));
        }
    }

    pub(super) fn get_object_string_state(&self, plane: StagePlane, object_index: i32) -> String {
        if let Ok(guard) = self.object_string_state_map().lock() {
            return guard
                .get(&(plane, object_index))
                .cloned()
                .unwrap_or_default();
        }
        String::new()
    }

    fn emit_object_sort_and_visibility(&mut self, plane: StagePlane, object_index: i32) {
        if let Some(state) = self.objects.get(&(plane, object_index)) {
            let _ = self.event_tx.send(HostEvent::SetObjectSort {
                stage: plane,
                index: object_index,
                order: state.order,
                layer: state.layer,
                seq: state.seq,
            });
            let _ = self.event_tx.send(HostEvent::SetObjectVisible {
                stage: plane,
                index: object_index,
                visible: state.visible,
            });
        }
    }

    fn emit_generated_object_image(
        &mut self,
        plane: StagePlane,
        object_index: i32,
        image: image::DynamicImage,
    ) {
        let _ = self.event_tx.send(HostEvent::UpsertObjectImage {
            stage: plane,
            index: object_index,
            image: std::sync::Arc::new(image),
        });
        if let Some(state) = self.objects.get(&(plane, object_index)) {
            let _ = self.event_tx.send(HostEvent::SetObjectPos {
                stage: plane,
                index: object_index,
                x: state.x,
                y: state.y,
            });
            self.emit_object_sort_and_visibility(plane, object_index);
            self.emit_object_render_state(plane, object_index);
        }
    }

    fn build_rect_image(args: &[siglus::vm::Prop]) -> image::DynamicImage {
        let l = args.first().and_then(|p| p.as_int()).unwrap_or(0);
        let t = args.get(1).and_then(|p| p.as_int()).unwrap_or(0);
        let r = args.get(2).and_then(|p| p.as_int()).unwrap_or(l + 1);
        let b = args.get(3).and_then(|p| p.as_int()).unwrap_or(t + 1);
        let width = (r - l).unsigned_abs().max(1);
        let height = (b - t).unsigned_abs().max(1);
        let a = args
            .get(7)
            .and_then(|p| p.as_int())
            .unwrap_or(255)
            .clamp(0, 255) as u8;
        let rr = args
            .get(4)
            .and_then(|p| p.as_int())
            .unwrap_or(255)
            .clamp(0, 255) as u8;
        let gg = args
            .get(5)
            .and_then(|p| p.as_int())
            .unwrap_or(255)
            .clamp(0, 255) as u8;
        let bb = args
            .get(6)
            .and_then(|p| p.as_int())
            .unwrap_or(255)
            .clamp(0, 255) as u8;
        let img = image::RgbaImage::from_pixel(width, height, image::Rgba([rr, gg, bb, a]));
        image::DynamicImage::ImageRgba8(img)
    }

    fn color_from_script_i32(v: i32, default_rgb: [u8; 3]) -> [u8; 3] {
        if v < 0 {
            return default_rgb;
        }
        [
            ((v >> 16) & 0xFF) as u8,
            ((v >> 8) & 0xFF) as u8,
            (v & 0xFF) as u8,
        ]
    }

    fn parse_bitmap_font_file(path: &std::path::Path) -> std::collections::BTreeMap<char, [u8; 7]> {
        let mut out = std::collections::BTreeMap::new();
        let Ok(content) = std::fs::read_to_string(path) else {
            return out;
        };
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let ch = k.chars().next().unwrap_or(' ');
            let cols: Vec<&str> = v.split(',').map(|s| s.trim()).collect();
            if cols.len() != 7 {
                continue;
            }
            let mut rows = [0u8; 7];
            let mut ok = true;
            for (i, c) in cols.iter().enumerate() {
                match u8::from_str_radix(c.trim_start_matches("0x"), 16) {
                    Ok(v) => rows[i] = v,
                    Err(_) => {
                        ok = false;
                        break;
                    }
                }
            }
            if ok {
                out.insert(ch, rows);
            }
        }
        out
    }

    fn load_custom_bitmap_font_families() -> Vec<std::collections::BTreeMap<char, [u8; 7]>> {
        let mut families = Vec::new();
        let Some(raw) = std::env::var_os("SIGLUS_BITMAP_FONT_PATH") else {
            return families;
        };
        let raw = raw.to_string_lossy();
        let delim = if raw.contains(';') { ';' } else { ':' };
        for seg in raw.split(delim) {
            let seg = seg.trim();
            if seg.is_empty() {
                continue;
            }
            let map = Self::parse_bitmap_font_file(std::path::Path::new(seg));
            if !map.is_empty() {
                families.push(map);
            }
        }
        families
    }

    fn builtin_glyph5x7(ch: char) -> [u8; 7] {
        match ch {
            '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
            '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
            '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
            '3' => [0x1E, 0x01, 0x01, 0x0E, 0x01, 0x01, 0x1E],
            '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
            '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
            '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
            '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
            '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
            '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
            '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
            '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
            '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
            ',' => [0x00, 0x00, 0x00, 0x00, 0x0C, 0x08, 0x10],
            ':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00],
            '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
            '?' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x00, 0x04],
            ' ' => [0x00; 7],
            _ => [0x1F, 0x11, 0x15, 0x11, 0x15, 0x11, 0x1F],
        }
    }

    fn glyph5x7(ch: char) -> [u8; 7] {
        let families =
            CUSTOM_BITMAP_FONT_FAMILIES.get_or_init(Self::load_custom_bitmap_font_families);
        for fam in families {
            if let Some(g) = fam.get(&ch) {
                return *g;
            }
        }
        Self::builtin_glyph5x7(ch)
    }

    fn draw_bitmap_glyph(
        img: &mut image::RgbaImage,
        x0: i32,
        y0: i32,
        scale: u32,
        glyph: [u8; 7],
        fill: [u8; 4],
    ) {
        if scale == 0 {
            return;
        }
        let iw = img.width() as i32;
        let ih = img.height() as i32;
        for (row_idx, row) in glyph.iter().enumerate() {
            for col in 0..5 {
                if (row & (1 << (4 - col))) == 0 {
                    continue;
                }
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = x0 + (col as i32) * scale as i32 + sx as i32;
                        let py = y0 + (row_idx as i32) * scale as i32 + sy as i32;
                        if px < 0 || py < 0 || px >= iw || py >= ih {
                            continue;
                        }
                        img.put_pixel(px as u32, py as u32, image::Rgba(fill));
                    }
                }
            }
        }
    }

    fn build_string_raster_image(
        text: &str,
        style: &ObjectStringStyleState,
    ) -> image::DynamicImage {
        let scale = (style.moji_size.max(8) as u32).saturating_div(8).max(1);
        let spacing = style.moji_space_x.max(0) as u32;
        let pad_x = 2u32;
        let pad_y = 2u32;

        let mut chars: Vec<char> = if text.is_empty() {
            vec![' ']
        } else {
            text.chars().collect()
        };
        if style.moji_cnt > 0 {
            let max_chars = (style.moji_cnt as usize).saturating_mul(2);
            if chars.len() > max_chars {
                chars.truncate(max_chars);
            }
        }

        let glyph_w = 5 * scale;
        let glyph_h = 7 * scale;
        let advance = glyph_w + spacing;
        let width = (pad_x * 2 + advance.saturating_mul(chars.len() as u32)).max(16);
        let height = (pad_y * 2 + glyph_h + style.moji_space_y.max(0) as u32 + scale).max(16);

        let mut img = image::RgbaImage::from_pixel(width, height, image::Rgba([0, 0, 0, 0]));
        let fill_rgb = Self::color_from_script_i32(style.moji_color, [255, 255, 255]);
        let shadow_rgb = Self::color_from_script_i32(style.shadow_color, [0, 0, 0]);
        let outline_rgb = Self::color_from_script_i32(style.fuchi_color, [0, 0, 0]);
        let shadow_enabled = style.shadow_mode >= 0;
        let outline_enabled = style.fuchi_color >= 0;

        for (i, ch) in chars.iter().enumerate() {
            let x = pad_x as i32 + (i as i32) * advance as i32;
            let y = pad_y as i32;
            let glyph = Self::glyph5x7(*ch);

            if outline_enabled {
                for oy in -1..=1 {
                    for ox in -1..=1 {
                        if ox == 0 && oy == 0 {
                            continue;
                        }
                        Self::draw_bitmap_glyph(
                            &mut img,
                            x + ox * scale as i32,
                            y + oy * scale as i32,
                            scale,
                            glyph,
                            [outline_rgb[0], outline_rgb[1], outline_rgb[2], 255],
                        );
                    }
                }
            }

            if shadow_enabled {
                Self::draw_bitmap_glyph(
                    &mut img,
                    x + scale as i32,
                    y + scale as i32,
                    scale,
                    glyph,
                    [shadow_rgb[0], shadow_rgb[1], shadow_rgb[2], 220],
                );
            }

            Self::draw_bitmap_glyph(
                &mut img,
                x,
                y,
                scale,
                glyph,
                [fill_rgb[0], fill_rgb[1], fill_rgb[2], 255],
            );
        }

        image::DynamicImage::ImageRgba8(img)
    }

    fn build_number_display_text(value: i32, style: &ObjectNumberStyleState) -> String {
        let mut slots = vec![' '; 16];
        let sign = if value > 0 {
            1
        } else if value < 0 {
            -1
        } else {
            0
        };
        let mut digits: Vec<char> = value.abs().to_string().chars().collect();
        if digits.is_empty() {
            digits.push('0');
        }
        let keta = digits.len();
        let keta_max = style.keta_max.max(0) as usize;

        if style.disp_zero != 0 {
            for i in 0..keta_max.min(slots.len()) {
                slots[i] = '0';
            }
        }

        let mut num_pos = keta_max.saturating_sub(keta);
        let disp_sign = style.disp_sign != 0 || sign < 0;
        let tumeru_sign = style.tumeru_sign != 0 && style.disp_zero == 0;
        let mut sign_pos: Option<usize> = None;
        if disp_sign {
            let p = if tumeru_sign {
                num_pos.saturating_sub(1)
            } else {
                0
            };
            sign_pos = Some(p);
            num_pos = num_pos.max(p + 1);
        }

        for (i, ch) in digits.into_iter().enumerate() {
            let p = num_pos + i;
            if p >= slots.len() {
                break;
            }
            slots[p] = ch;
        }

        if let Some(p) = sign_pos {
            if p < slots.len() {
                slots[p] = if sign < 0 {
                    '-'
                } else if sign > 0 {
                    '+'
                } else {
                    ' '
                };
            }
        }

        let mut end = slots.len();
        while end > 1 && slots[end - 1] == ' ' {
            end -= 1;
        }
        slots[..end].iter().collect()
    }

    fn build_number_raster_image(
        value: i32,
        style: &ObjectNumberStyleState,
    ) -> image::DynamicImage {
        let text = Self::build_number_display_text(value, style);
        let mut sstyle = ObjectStringStyleState::default();
        sstyle.moji_size = 18;
        sstyle.moji_space_x = style.space.max(0);
        sstyle.moji_color = 0xFFFFFF;
        sstyle.shadow_mode = if style.space_mod == 0 { 0 } else { -1 };
        Self::build_string_raster_image(&text, &sstyle)
    }

    fn apply_create_tail_disp_xy_pat(
        &mut self,
        plane: StagePlane,
        object_index: i32,
        disp_idx: usize,
        x_idx: usize,
        y_idx: usize,
        pat_idx: Option<usize>,
        args: &[siglus::vm::Prop],
    ) {
        let visible = args.get(disp_idx).and_then(|p| p.as_int()).unwrap_or(1) != 0;
        let x_opt = args.get(x_idx).and_then(|p| p.as_int()).map(|v| v as f32);
        let y_opt = args.get(y_idx).and_then(|p| p.as_int()).map(|v| v as f32);
        let pat_opt = pat_idx
            .and_then(|idx| args.get(idx))
            .and_then(|p| p.as_int())
            .map(|v| v.max(0) as usize);

        let state = self.get_or_create_object_state(plane, object_index);
        state.visible = visible;
        if let Some(x) = x_opt {
            state.x = x;
        }
        if let Some(y) = y_opt {
            state.y = y;
        }
        if let Some(pat_no) = pat_opt {
            state.pat_no = pat_no;
        }
    }
}
