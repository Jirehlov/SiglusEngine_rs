use super::*;

pub(super) struct RunConfig {
    pub(super) gameexe: PathBuf,
    pub(super) pck: PathBuf,
    pub(super) scene: String,
    pub(super) z: i32,
    pub(super) max_steps: Option<u64>,
    pub(super) menu_scene: String,
    pub(super) menu_z: i32,
    pub(super) persistent_state_path: PathBuf,
    pub(super) append_search_dirs: Vec<PathBuf>,
    pub(super) title: String,
    pub(super) window_size: Option<(i32, i32)>,
    pub(super) scene_size: Option<(i32, i32)>,
    pub(super) system_extra_int_values: Vec<i32>,
    pub(super) system_extra_str_values: Vec<String>,
    pub(super) load_wipe_type: i32,
    pub(super) load_wipe_time_ms: u64,
    pub(super) load_after_call: Option<(String, i32)>,
}

fn parse_system_extra_values(cfg: &siglus::gameexe::GameexeConfig) -> (Vec<i32>, Vec<String>) {
    let int_cnt = cfg
        .first_values("SYSTEM.EXTRA_INT_VALUE.CNT")
        .and_then(|v| v.first())
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);

    let mut int_values = vec![0; int_cnt];
    for (idx, slot) in int_values.iter_mut().enumerate() {
        let key = format!("SYSTEM.EXTRA_INT_VALUE.{idx}");
        if let Some(v) = cfg
            .first_values(&key)
            .and_then(|v| v.first())
            .and_then(|s| s.trim().parse::<i32>().ok())
        {
            *slot = v;
        }
    }

    let str_cnt = cfg
        .first_values("SYSTEM.EXTRA_STR_VALUE.CNT")
        .and_then(|v| v.first())
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let mut str_values = vec![String::new(); str_cnt];
    for (idx, slot) in str_values.iter_mut().enumerate() {
        let key = format!("SYSTEM.EXTRA_STR_VALUE.{idx}");
        if let Some(v) = cfg.first_values(&key).and_then(|v| v.first()) {
            *slot = v.clone();
        }
    }

    (int_values, str_values)
}

fn parse_load_wipe(cfg: &siglus::gameexe::GameexeConfig) -> (i32, u64) {
    // C++ reference: tnm_ini.cpp::LOAD_WIPE(num[0]=type, num[1]=time), defaults 0/1000.
    let wipe_type = cfg
        .first_values("LOAD_WIPE")
        .and_then(|vals| vals.first())
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(0);
    let wipe_time_ms = cfg
        .first_values("LOAD_WIPE")
        .and_then(|vals| vals.get(1))
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(1000);
    (wipe_type, wipe_time_ms)
}

fn parse_select_ini_dirs(exe_dir: &std::path::Path) -> Result<Vec<PathBuf>> {
    let ini_path = exe_dir.join("Select.ini");
    if !ini_path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read(&ini_path).context("failed to read Select.ini")?;
    let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(&content);
    let text = cow.as_ref();

    // Select.ini format: DirName \t GameName
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        if let Some((dir, _)) = line.split_once('\t') {
            let dir = dir.trim();
            if !dir.is_empty() {
                out.push(PathBuf::from(dir));
            }
        }
    }
    Ok(out)
}

pub(super) fn load_run_config() -> Result<RunConfig> {
    let exe_path = std::env::current_exe().context("failed to get current exe path")?;
    let exe_dir = exe_path.parent().context("failed to get exe dir")?;
    let cwd = std::env::current_dir().context("failed to get current dir")?;

    let base_probe = std::env::var_os("SIGLUS_BASE_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            if cwd.join(DEFAULT_GAMEEXE_NAME).exists() {
                Some(cwd.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| exe_dir.to_path_buf());

    // Align with C++ File Search:
    // 1. Parse Select.ini append_dir list.
    // 2. Base directory is ExeDir + first append_dir.
    // 3. Image search uses append_dir chain in listed order.
    let select_dirs = parse_select_ini_dirs(&base_probe)
        .or_else(|_| parse_select_ini_dirs(exe_dir))
        .unwrap_or_default();
    let base_dir = if let Some(sub_dir) = select_dirs.first() {
        base_probe.join(sub_dir)
    } else {
        base_probe.clone()
    };

    let mut append_search_dirs: Vec<PathBuf> = if select_dirs.is_empty() {
        vec![base_dir.clone()]
    } else {
        select_dirs.iter().map(|d| base_probe.join(d)).collect()
    };
    if !append_search_dirs.iter().any(|p| p == &base_dir) {
        append_search_dirs.insert(0, base_dir.clone());
    }
    if append_search_dirs.len() <= 1 {
        for extra in collect_append_dirs(&base_dir) {
            if !append_search_dirs.iter().any(|p| p == &extra) {
                append_search_dirs.push(extra);
            }
        }
    }

    let gameexe = base_dir.join(DEFAULT_GAMEEXE_NAME);
    let cfg = siglus::gameexe::read_file(&gameexe)
        .with_context(|| format!("failed to parse {}", gameexe.display()))?;

    // Resolve Scene.pck relative to Gameexe.dat (which is in base_dir)
    let pck = siglus::gameexe::resolve_scene_pack_path(&gameexe, &cfg.scene_pack);

    let title = cfg
        .game_name
        .clone()
        .or(cfg.game_id.clone())
        .unwrap_or_else(|| "Siglus Game".to_string());

    // Always start windowed GUI in 720p by default as requested.
    let window_size = Some((1280, 720));

    let persistent_state_path = base_dir.join("siglus_vm_state.bin");
    let (system_extra_int_values, system_extra_str_values) = parse_system_extra_values(&cfg);
    let (load_wipe_type, load_wipe_time_ms) = parse_load_wipe(&cfg);

    let load_after_call = cfg.load_after_call.clone();

    Ok(RunConfig {
        gameexe,
        pck,
        scene: cfg.start_scene,
        z: cfg.start_scene_z,
        max_steps: None,
        menu_scene: cfg.menu_scene,
        menu_z: cfg.menu_scene_z,
        persistent_state_path,
        append_search_dirs,
        title,
        window_size,
        scene_size: cfg.screen_size,
        system_extra_int_values,
        system_extra_str_values,
        load_wipe_type,
        load_wipe_time_ms,
        load_after_call,
    })
}
