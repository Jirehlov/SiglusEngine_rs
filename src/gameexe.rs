use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use encoding_rs::SHIFT_JIS;

const GAMEEXE_DAT_ANGOU_CODE: [u8; 256] = hex_literal::hex!(
    "d829b9163d1a76d0879b2d0c7bd1a919229f91736a35b17ed1b5e7e6d5f506d6babff3453ff161dd4c676a6f74ec7a6f26740edb274ca5f10e2d70c4405d4fda9ec5497bbde8dfeecaf492dee47610dd2a52dc734e548c303d9ab29bb8932955fa7ac9da1097e5b62302dd384c9b1f9ad549e9340f282d1b52395c368956a79614be2ec53e085f47a9df889fd4cc691f309fe7cd8045f3e72a1d16b2f154c86c2b0dd465f7e336d4a53bd1794c54f02ab4b256452eab7b88c5fa74ad03b89ed5f56fdcfa444931f68332ffc2b1e9e1983d6f310dacb108839d0d10d141f900ba1acf1371e486212f2365c345a0c392489deadd312ce9e21022aae1ad2cc42d7f"
);
const EXE_ORG: [u8; 16] = hex_literal::hex!("3659c9732eb509bae44cf26aa234ec7c");

#[derive(Debug, Clone)]
pub struct GameexeConfig {
    pub game_id: Option<String>,
    pub game_name: Option<String>,
    pub game_version: Option<String>,
    pub disc_mark: Option<String>,
    pub manual_path: Option<String>,
    pub screen_size: Option<(i32, i32)>,
    pub start_scene: String,
    pub start_scene_z: i32,
    pub scene_pack: String,
    pub menu_scene: String,
    pub menu_scene_z: i32,
    pub cancel_scene: Option<(String, i32)>,
    pub config_scene: Option<(String, i32)>,
    pub save_scene: Option<(String, i32)>,
    pub load_scene: Option<(String, i32)>,
    pub load_after_call: Option<(String, i32)>,
    pub dummy_check_str: Option<String>,
    pub dummy_check_ok_str: Option<String>,
    pub user_config: GameexeUserConfig,
    /// Parsed normalized entries for all directives (including repeated keys).
    pub entries: Vec<GameexeEntry>,
    /// Fast index for directive lookups; values are indexes into `entries`.
    pub entry_index: BTreeMap<String, Vec<usize>>,
}

#[derive(Debug, Clone)]
pub struct GameexeEntry {
    pub key: String,
    pub raw_value: String,
    pub values: Vec<String>,
}

impl GameexeConfig {
    pub fn first_raw_value(&self, key: &str) -> Option<&str> {
        let norm = normalize_key(key);
        let idx = self.entry_index.get(&norm)?.first().copied()?;
        self.entries.get(idx).map(|e| e.raw_value.as_str())
    }

    pub fn all_raw_values(&self, key: &str) -> Vec<&str> {
        let norm = normalize_key(key);
        self.entry_index
            .get(&norm)
            .map(|indexes| {
                indexes
                    .iter()
                    .filter_map(|i| self.entries.get(*i))
                    .map(|e| e.raw_value.as_str())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn first_values(&self, key: &str) -> Option<&[String]> {
        let norm = normalize_key(key);
        let idx = self.entry_index.get(&norm)?.first().copied()?;
        self.entries.get(idx).map(|e| e.values.as_slice())
    }
}

#[derive(Debug, Clone)]
pub struct GameexeUserConfig {
    pub screen_size_mode: Option<ScreenSizeMode>,
    pub all_user_volume: Option<i32>,
    pub bgm_user_volume: Option<i32>,
    pub koe_user_volume: Option<i32>,
    pub pcm_user_volume: Option<i32>,
    pub se_user_volume: Option<i32>,
    pub mov_user_volume: Option<i32>,
    pub bgmfade_volume: Option<i32>,
    pub bgmfade_enabled: Option<bool>,
    pub message_speed: Option<i32>,
    pub message_speed_nowait: Option<bool>,
    pub mouse_cursor_hide_onoff: Option<bool>,
    pub mouse_cursor_hide_time: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenSizeMode {
    Window,
    Full,
}

impl Default for GameexeUserConfig {
    fn default() -> Self {
        Self {
            screen_size_mode: None,
            all_user_volume: None,
            bgm_user_volume: None,
            koe_user_volume: None,
            pcm_user_volume: None,
            se_user_volume: None,
            mov_user_volume: None,
            bgmfade_volume: None,
            bgmfade_enabled: None,
            message_speed: None,
            message_speed_nowait: None,
            mouse_cursor_hide_onoff: None,
            mouse_cursor_hide_time: None,
        }
    }
}

pub fn read_file(path: &Path) -> Result<GameexeConfig> {
    let exe_key = discover_exe_key(path.parent().unwrap_or(Path::new(".")));
    read_file_with_key(path, exe_key.as_deref())
}

pub fn read_file_with_key(path: &Path, exe_key: Option<&[u8]>) -> Result<GameexeConfig> {
    let dat = fs::read(path).with_context(|| format!("read Gameexe.dat: {}", path.display()))?;
    if dat.len() < 8 {
        bail!("Gameexe.dat too small");
    }

    let mode = i32::from_le_bytes([dat[4], dat[5], dat[6], dat[7]]);
    let mut payload = dat[8..].to_vec();
    xor_cycle(&mut payload, &GAMEEXE_DAT_ANGOU_CODE);

    if mode != 0 {
        let key = exe_key
            .context("Gameexe.dat requires exe-angou key (key.txt / 暗号.dat / SIGLUS_EXE_KEY)")?;
        if key.len() != 16 {
            bail!("exe-angou key must be 16 bytes");
        }
        xor_cycle(&mut payload, key);
    }

    let raw = crate::lzss::unpack(&payload).context("Gameexe.dat lzss unpack failed")?;
    let text = decode_utf16le_lossy(&raw);
    parse_gameexe_ini(&text)
}

fn parse_gameexe_ini(text: &str) -> Result<GameexeConfig> {
    let mut game_id = None;
    let mut game_name = None;
    let mut game_version = None;
    let mut disc_mark = None;
    let mut manual_path = None;
    let mut screen_size = None;
    let mut start_scene = String::from("_start");
    let mut start_scene_z = 0;
    let mut scene_pack = String::from("Scene.pck");
    let mut menu_scene = String::from("__sys_menu");
    let mut menu_scene_z = 0;
    let mut cancel_scene = None;
    let mut config_scene = None;
    let mut save_scene = None;
    let mut load_scene = None;
    let mut load_after_call = None;
    let mut dummy_check_str = None;
    let mut dummy_check_ok_str = None;
    let mut user_config = GameexeUserConfig::default();
    let mut entries = Vec::new();
    let mut entry_index: BTreeMap<String, Vec<usize>> = BTreeMap::new();

    for raw in text.lines() {
        let Some(line) = strip_inline_comment(raw) else {
            continue;
        };
        if line.is_empty() || line.starts_with("//") || line.starts_with(';') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let key = normalize_key(k.trim());
        let value = v.trim();
        let idx = entries.len();
        entries.push(GameexeEntry {
            key: key.clone(),
            raw_value: value.to_string(),
            values: split_gameexe_csv(value)
                .into_iter()
                .map(|x| parse_text(&x))
                .collect(),
        });
        entry_index.entry(key.clone()).or_default().push(idx);

        match key.as_str() {
            "GAMEID" => game_id = Some(parse_text(value)),
            "GAMENAME" => game_name = Some(parse_text(value)),
            "GAMEVERSION" => game_version = Some(parse_text(value)),
            "DISCMARK" => disc_mark = Some(parse_text(value)),
            "MANUAL_PATH" => manual_path = Some(parse_text(value)),
            "START_SCENE" => {
                if let Some((name, z)) = parse_scene_spec(value) {
                    if !name.is_empty() {
                        start_scene = name;
                    }
                    if let Some(z) = z {
                        start_scene_z = z;
                    }
                }
            }
            "MENU_SCENE" => {
                if let Some((name, z)) = parse_scene_spec(value) {
                    if !name.is_empty() {
                        menu_scene = name;
                    }
                    if let Some(z) = z {
                        menu_scene_z = z;
                    }
                }
            }
            "MENU_SCENE_Z" => {
                if let Ok(z) = value.parse::<i32>() {
                    menu_scene_z = z;
                }
            }
            "SCREEN_SIZE" => {
                let parts = split_gameexe_csv(value);
                if parts.len() >= 2 {
                    if let (Ok(w), Ok(h)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        screen_size = Some((w, h));
                    }
                }
            }
            "SCENE_PACK" | "SCENE_PCK" | "SCENEFILE" | "SCENE_FILE" | "SCENEPACK" => {
                let pack = parse_text(value);
                if !pack.is_empty() {
                    scene_pack = pack;
                }
            }
            "CANCEL_SCENE" => cancel_scene = parse_optional_scene_spec(value),
            "CONFIG_SCENE" => config_scene = parse_optional_scene_spec(value),
            "SAVE_SCENE" => save_scene = parse_optional_scene_spec(value),
            "LOAD_SCENE" => load_scene = parse_optional_scene_spec(value),
            "LOAD_AFTER_CALL" => load_after_call = parse_optional_scene_spec(value),
            "DUMMY_CHECK_STR" => dummy_check_str = Some(parse_text(value)),
            "DUMMY_CHECK_OK_STR" => dummy_check_ok_str = Some(parse_text(value)),
            "CONFIG.WINDOW_MODE" => {
                user_config.screen_size_mode = match parse_int(value) {
                    Some(0) => Some(ScreenSizeMode::Window),
                    Some(1) => Some(ScreenSizeMode::Full),
                    _ => None,
                }
            }
            "CONFIG.VOLUME.ALL" => user_config.all_user_volume = parse_int(value),
            "CONFIG.VOLUME.BGM" => user_config.bgm_user_volume = parse_int(value),
            "CONFIG.VOLUME.KOE" => user_config.koe_user_volume = parse_int(value),
            "CONFIG.VOLUME.PCM" => user_config.pcm_user_volume = parse_int(value),
            "CONFIG.VOLUME.SE" => user_config.se_user_volume = parse_int(value),
            "CONFIG.VOLUME.MOV" => user_config.mov_user_volume = parse_int(value),
            "CONFIG.BGMFADE_VOLUME" => user_config.bgmfade_volume = parse_int(value),
            "CONFIG.BGMFADE_ONOFF" => user_config.bgmfade_enabled = parse_bool01(value),
            "CONFIG.MESSAGE_SPEED" => user_config.message_speed = parse_int(value),
            "CONFIG.MESSAGE_SPEED_NOWAIT.ONOFF" => {
                user_config.message_speed_nowait = parse_bool01(value)
            }
            "CONFIG.MOUSE_CURSOR_HIDE_ONOFF" => {
                user_config.mouse_cursor_hide_onoff = parse_bool01(value)
            }
            "CONFIG.MOUSE_CURSOR_HIDE_TIME" => {
                user_config.mouse_cursor_hide_time = parse_int(value)
            }
            _ => {}
        }
    }

    Ok(GameexeConfig {
        game_id,
        game_name,
        game_version,
        disc_mark,
        manual_path,
        screen_size,
        start_scene,
        start_scene_z,
        scene_pack,
        menu_scene,
        menu_scene_z,
        cancel_scene,
        config_scene,
        save_scene,
        load_scene,
        load_after_call,
        dummy_check_str,
        dummy_check_ok_str,
        user_config,
        entries,
        entry_index,
    })
}

fn normalize_key(raw: &str) -> String {
    raw.trim_start_matches('#')
        .split('.')
        .map(|seg| seg.trim().replace(' ', ""))
        .filter(|seg| !seg.is_empty())
        .collect::<Vec<_>>()
        .join(".")
        .to_ascii_uppercase()
}

fn parse_int(s: &str) -> Option<i32> {
    s.trim().parse::<i32>().ok()
}

fn parse_bool01(s: &str) -> Option<bool> {
    match parse_int(s) {
        Some(0) => Some(false),
        Some(_) => Some(true),
        None => None,
    }
}

fn parse_scene_spec(value: &str) -> Option<(String, Option<i32>)> {
    let parts = split_gameexe_csv(value);
    let name = parts.first().map(|s| parse_text(s))?;
    let z = parts.get(1).and_then(|s| parse_int(s));
    Some((name, z))
}

fn split_gameexe_csv(value: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;

    for ch in value.chars() {
        match ch {
            '"' => {
                in_quote = !in_quote;
                cur.push(ch);
            }
            ',' if !in_quote => {
                parts.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }

    if !cur.is_empty() || value.ends_with(',') {
        parts.push(cur.trim().to_string());
    }

    parts
}

fn parse_optional_scene_spec(value: &str) -> Option<(String, i32)> {
    let (name, z) = parse_scene_spec(value)?;
    if name.is_empty() {
        return None;
    }
    Some((name, z.unwrap_or(0)))
}

fn parse_text(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

fn strip_inline_comment(line: &str) -> Option<&str> {
    let mut in_quote = false;
    let mut prev = '\0';
    let bytes = line.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '"' && prev != '\\' {
            in_quote = !in_quote;
        }
        if !in_quote {
            if ch == ';' {
                let s = line[..i].trim();
                return if s.is_empty() { None } else { Some(s) };
            }
            if ch == '/' && i + 1 < bytes.len() && bytes[i + 1] as char == '/' {
                let s = line[..i].trim();
                return if s.is_empty() { None } else { Some(s) };
            }
        }
        prev = ch;
        i += 1;
    }

    let s = line.trim();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn decode_utf16le_lossy(raw: &[u8]) -> String {
    let mut u16s = Vec::with_capacity(raw.len() / 2);
    let mut i = 0usize;
    while i + 1 < raw.len() {
        u16s.push(u16::from_le_bytes([raw[i], raw[i + 1]]));
        i += 2;
    }
    String::from_utf16_lossy(&u16s)
}

fn xor_cycle(data: &mut [u8], key: &[u8]) {
    if key.is_empty() {
        return;
    }
    for (i, b) in data.iter_mut().enumerate() {
        *b ^= key[i % key.len()];
    }
}

fn discover_exe_key(base_dir: &Path) -> Option<Vec<u8>> {
    if let Ok(env) = std::env::var("SIGLUS_EXE_KEY") {
        if let Some(k) = parse_key_text(&env) {
            return Some(k);
        }
    }

    let key_txt = base_dir.join("key.txt");
    if let Ok(text) = fs::read_to_string(&key_txt) {
        if let Some(k) = parse_key_text(&text) {
            return Some(k);
        }
    }

    let angou_dat = base_dir.join("暗号.dat");
    if let Ok(bytes) = fs::read(&angou_dat) {
        let first_line = bytes.split(|b| *b == b'\n').next().unwrap_or_default();
        let (decoded, _, _) = SHIFT_JIS.decode(first_line);
        let line = decoded.trim();
        if !line.is_empty() {
            let sjis = SHIFT_JIS.encode(line).0;
            if sjis.len() >= 8 {
                return Some(exe_angou_element(&sjis));
            }
        }
    }

    // Fallback: try to extract 暗号.dat from PCK original-sources (OS) segment
    // User restriction: Only look for "Scene.pck" (case-insensitive) to avoid scanning
    // unrelated large .pck files which causes OOM / crashes.
    if let Ok(rd) = fs::read_dir(base_dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if name.eq_ignore_ascii_case("Scene.pck") {
                    if let Some(el) = crate::pck::find_exe_el_from_pck_file(&p) {
                        return Some(el.to_vec());
                    }
                }
            }
        }
    }

    None
}

fn parse_key_text(s: &str) -> Option<Vec<u8>> {
    let hex_only: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex_only.len() >= 32 {
        let mut out = Vec::with_capacity(16);
        for i in 0..16 {
            let part = &hex_only[i * 2..i * 2 + 2];
            out.push(u8::from_str_radix(part, 16).ok()?);
        }
        return Some(out);
    }
    None
}

fn exe_angou_element(angou_bytes: &[u8]) -> Vec<u8> {
    let mut out = EXE_ORG.to_vec();
    if angou_bytes.is_empty() {
        return out;
    }
    let mut ai = 0usize;
    let mut bi = 0usize;
    let count = out.len().max(angou_bytes.len());
    for _ in 0..count {
        out[bi] ^= angou_bytes[ai];
        ai = (ai + 1) % angou_bytes.len();
        bi = (bi + 1) % out.len();
    }
    out
}

pub fn resolve_scene_pack_path(gameexe_path: &Path, scene_pack: &str) -> PathBuf {
    let base_dir = gameexe_path.parent().unwrap_or(Path::new("."));
    base_dir.join(scene_pack)
}
