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
    pub(super) default_global_extra_switch: std::collections::BTreeMap<i32, i32>,
    pub(super) default_global_extra_mode: std::collections::BTreeMap<i32, i32>,
    pub(super) default_object_disp: std::collections::BTreeMap<i32, i32>,
    pub(super) default_local_extra_mode_value: std::collections::BTreeMap<i32, i32>,
    pub(super) default_local_extra_mode_enable: std::collections::BTreeMap<i32, i32>,
    pub(super) default_local_extra_mode_exist: std::collections::BTreeMap<i32, i32>,
    pub(super) default_local_extra_switch_onoff: std::collections::BTreeMap<i32, i32>,
    pub(super) default_local_extra_switch_enable: std::collections::BTreeMap<i32, i32>,
    pub(super) default_local_extra_switch_exist: std::collections::BTreeMap<i32, i32>,
    pub(super) default_global_extra_switch_cnt: usize,
    pub(super) default_global_extra_mode_cnt: usize,
    pub(super) default_object_disp_cnt: usize,
    pub(super) default_local_extra_mode_cnt: usize,
    pub(super) default_local_extra_switch_cnt: usize,
    pub(super) default_charakoe_cnt: usize,
    pub(super) default_charakoe_onoff: std::collections::BTreeMap<i32, i32>,
    pub(super) default_charakoe_volume: std::collections::BTreeMap<i32, i32>,
    pub(super) load_wipe_type: i32,
    pub(super) load_wipe_time_ms: u64,
    pub(super) load_after_call: Option<(String, i32)>,
    pub(super) preload_database_tables: Vec<Vec<Vec<siglus::vm::PropValue>>>,
    pub(super) preload_database_row_calls: Vec<Vec<i32>>,
    pub(super) preload_database_col_calls: Vec<Vec<i32>>,
    pub(super) preload_database_col_types: Vec<Vec<u8>>,
    pub(super) preload_cg_flag_count: usize,
    pub(super) preload_cg_name_to_flag: std::collections::BTreeMap<String, i32>,
    pub(super) preload_cg_group_codes: Vec<[i32; 5]>,
    pub(super) preload_cg_code_exist_cnt: Vec<i32>,
    pub(super) preload_bgm_names: Vec<String>,
    pub(super) preload_counter_count: usize,
    pub(super) preload_frame_action_ch_count: usize,
    pub(super) flick_scene_routes: Vec<siglus::vm::FlickSceneRoute>,
    pub(super) movie_backends: Vec<String>,
    pub(super) quake_ref_csv: Option<PathBuf>,
    pub(super) quake_ref_report: PathBuf,
}

fn parse_movie_backends(cfg: &siglus::gameexe::GameexeConfig) -> Vec<String> {
    let from_env = std::env::var("SIGLUS_MOVIE_BACKENDS")
        .ok()
        .map(|s| {
            s.split(',')
                .map(|v| v.trim().to_ascii_lowercase())
                .filter(|v| !v.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !from_env.is_empty() {
        return from_env;
    }

    // 项目样本常见命名：MOVIE.BACKEND / SYSTEM.MOVIE.BACKEND
    let from_gameexe = ["MOVIE.BACKEND", "SYSTEM.MOVIE.BACKEND"]
        .iter()
        .find_map(|key| cfg.first_values(key))
        .map(|vals| {
            vals.iter()
                .flat_map(|v| v.split(','))
                .map(|v| v.trim().to_ascii_lowercase())
                .filter(|v| !v.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !from_gameexe.is_empty() {
        return from_gameexe;
    }

    vec![
        "ffplay".to_string(),
        "mpv".to_string(),
        "gst-play-1.0".to_string(),
    ]
}

fn parse_quake_reference_paths(base_dir: &Path) -> (Option<PathBuf>, PathBuf) {
    let ref_csv = std::env::var_os("SIGLUS_QUAKE_REF_CSV")
        .map(PathBuf::from)
        .or_else(|| {
            let repo_default =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("reference/quake_baseline.csv");
            if repo_default.exists() {
                Some(repo_default)
            } else {
                None
            }
        });
    let report = std::env::var_os("SIGLUS_QUAKE_REF_REPORT")
        .map(PathBuf::from)
        .unwrap_or_else(|| base_dir.join("quake_ref_report.txt"));
    (ref_csv, report)
}

fn parse_config_count(cfg: &siglus::gameexe::GameexeConfig, cnt_keys: &[&str]) -> usize {
    cnt_keys
        .iter()
        .find_map(|k| cfg.first_values(k))
        .and_then(|v| v.first())
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0)
}

fn parse_indexed_config_map(
    cfg: &siglus::gameexe::GameexeConfig,
    cnt_keys: &[&str],
    value_key_patterns: &[&str],
) -> std::collections::BTreeMap<i32, i32> {
    let cnt = parse_config_count(cfg, cnt_keys);
    let mut out = std::collections::BTreeMap::new();
    for idx in 0..cnt {
        let mut v = None;
        for pat in value_key_patterns {
            let key = pat.replace("{idx}", &idx.to_string());
            v = cfg
                .first_values(&key)
                .and_then(|vals| vals.first())
                .and_then(|s| s.trim().parse::<i32>().ok());
            if v.is_some() {
                break;
            }
        }
        if let Some(v) = v {
            out.insert(idx as i32, v);
        }
    }
    out
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

use super::resource_bootstrap::parse_vm_resource_bootstrap;

fn parse_flick_scene_routes(
    cfg: &siglus::gameexe::GameexeConfig,
) -> Vec<siglus::vm::FlickSceneRoute> {
    let cnt = cfg
        .first_values("FLICK_SCENE.CNT")
        .and_then(|v| v.first())
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);

    let mut routes = Vec::new();
    for idx in 0..cnt {
        let scene_key = format!("FLICK_SCENE.{idx}.SCENE");
        let Some(values) = cfg.first_values(&scene_key) else {
            continue;
        };
        let Some(scene_name) = values.first().map(|v| v.trim()).filter(|v| !v.is_empty()) else {
            continue;
        };
        let z_no = values
            .get(1)
            .and_then(|v| v.trim().parse::<i32>().ok())
            .unwrap_or(0);
        let angle_key = format!("FLICK_SCENE.{idx}.ANGLE");
        let angle_type = cfg
            .first_values(&angle_key)
            .and_then(|v| v.first())
            .and_then(|v| v.trim().parse::<i32>().ok())
            .unwrap_or(0);
        if !(1..=8).contains(&angle_type) {
            continue;
        }
        routes.push(siglus::vm::FlickSceneRoute {
            scene: scene_name.to_string(),
            z_no,
            angle_type,
        });
    }
    routes
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
    let default_global_extra_switch_cnt =
        parse_config_count(&cfg, &["CONFIG.GLOBAL_EXTRA_SWITCH.CNT"]);
    let default_global_extra_mode_cnt = parse_config_count(&cfg, &["CONFIG.GLOBAL_EXTRA_MODE.CNT"]);
    let default_object_disp_cnt = parse_config_count(
        &cfg,
        &["CONFIG.OBJECT_DISP.CNT", "CONFIG.GLOBAL_EXTRA_SWITCH.CNT"],
    );
    let default_local_extra_mode_cnt = parse_config_count(&cfg, &["CONFIG.LOCAL_EXTRA_MODE.CNT"]);
    let default_local_extra_switch_cnt =
        parse_config_count(&cfg, &["CONFIG.LOCAL_EXTRA_SWITCH.CNT"]);
    let default_charakoe_cnt = parse_config_count(&cfg, &["CHRKOE.CNT", "CONFIG.CHRKOE.CNT"]);
    let default_global_extra_switch = parse_indexed_config_map(
        &cfg,
        &["CONFIG.GLOBAL_EXTRA_SWITCH.CNT"],
        &[
            "CONFIG.GLOBAL_EXTRA_SWITCH.{idx}.ONOFF",
            "CONFIG.GLOBAL_EXTRA_SWITCH.{idx}",
        ],
    );
    let default_global_extra_mode = parse_indexed_config_map(
        &cfg,
        &["CONFIG.GLOBAL_EXTRA_MODE.CNT"],
        &[
            "CONFIG.GLOBAL_EXTRA_MODE.{idx}.MODE",
            "CONFIG.GLOBAL_EXTRA_MODE.{idx}",
        ],
    );
    let default_object_disp = parse_indexed_config_map(
        &cfg,
        &["CONFIG.OBJECT_DISP.CNT", "CONFIG.GLOBAL_EXTRA_SWITCH.CNT"],
        &[
            "CONFIG.OBJECT_DISP.{idx}.ONOFF",
            "CONFIG.OBJECT_DISP.{idx}",
            "CONFIG.GLOBAL_EXTRA_SWITCH.{idx}.ONOFF",
        ],
    );
    let default_local_extra_mode_value = parse_indexed_config_map(
        &cfg,
        &["CONFIG.LOCAL_EXTRA_MODE.CNT"],
        &[
            "CONFIG.LOCAL_EXTRA_MODE.{idx}.MODE",
            "CONFIG.LOCAL_EXTRA_MODE.{idx}",
        ],
    );
    let default_local_extra_mode_enable = parse_indexed_config_map(
        &cfg,
        &["CONFIG.LOCAL_EXTRA_MODE.CNT"],
        &[
            "CONFIG.LOCAL_EXTRA_MODE.{idx}.ENABLE",
            "CONFIG.LOCAL_EXTRA_MODE.{idx}.ONOFF",
        ],
    );
    let default_local_extra_mode_exist = parse_indexed_config_map(
        &cfg,
        &["CONFIG.LOCAL_EXTRA_MODE.CNT"],
        &[
            "CONFIG.LOCAL_EXTRA_MODE.{idx}.EXIST",
            "CONFIG.LOCAL_EXTRA_MODE.{idx}.USE",
        ],
    );
    let default_local_extra_switch_onoff = parse_indexed_config_map(
        &cfg,
        &["CONFIG.LOCAL_EXTRA_SWITCH.CNT"],
        &[
            "CONFIG.LOCAL_EXTRA_SWITCH.{idx}.ONOFF",
            "CONFIG.LOCAL_EXTRA_SWITCH.{idx}",
        ],
    );
    let default_local_extra_switch_enable = parse_indexed_config_map(
        &cfg,
        &["CONFIG.LOCAL_EXTRA_SWITCH.CNT"],
        &[
            "CONFIG.LOCAL_EXTRA_SWITCH.{idx}.ENABLE",
            "CONFIG.LOCAL_EXTRA_SWITCH.{idx}.ON",
        ],
    );
    let default_local_extra_switch_exist = parse_indexed_config_map(
        &cfg,
        &["CONFIG.LOCAL_EXTRA_SWITCH.CNT"],
        &[
            "CONFIG.LOCAL_EXTRA_SWITCH.{idx}.EXIST",
            "CONFIG.LOCAL_EXTRA_SWITCH.{idx}.USE",
        ],
    );
    let default_charakoe_onoff = parse_indexed_config_map(
        &cfg,
        &["CHRKOE.CNT", "CONFIG.CHRKOE.CNT"],
        &["CHRKOE.{idx}.ONOFF", "CONFIG.CHRKOE.{idx}.ONOFF"],
    );
    let default_charakoe_volume = parse_indexed_config_map(
        &cfg,
        &["CHRKOE.CNT", "CONFIG.CHRKOE.CNT"],
        &["CHRKOE.{idx}.VOLUME", "CONFIG.CHRKOE.{idx}.VOLUME"],
    );
    let (load_wipe_type, load_wipe_time_ms) = parse_load_wipe(&cfg);

    let preload_counter_count = cfg
        .first_values("COUNTER.CNT")
        .and_then(|v| v.first())
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(32)
        .max(1);
    let preload_frame_action_ch_count = cfg
        .first_values("FRAME_ACTION_CH.CNT")
        .or_else(|| cfg.first_values("FRAME_ACTION.CH.CNT"))
        .or_else(|| cfg.first_values("EXCALL.FRAME_ACTION_CH.CNT"))
        .and_then(|v| v.first())
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let load_after_call = cfg.load_after_call.clone();
    let (
        preload_database_tables,
        preload_cg_flag_count,
        preload_cg_name_to_flag,
        preload_bgm_names,
        preload_database_row_calls,
        preload_database_col_calls,
        preload_database_col_types,
        preload_cg_group_codes,
        preload_cg_code_exist_cnt,
    ) = parse_vm_resource_bootstrap(&cfg, &base_dir);
    let flick_scene_routes = parse_flick_scene_routes(&cfg);
    let movie_backends = parse_movie_backends(&cfg);
    let (quake_ref_csv, quake_ref_report) = parse_quake_reference_paths(&base_dir);

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
        default_global_extra_switch,
        default_global_extra_mode,
        default_object_disp,
        default_local_extra_mode_value,
        default_local_extra_mode_enable,
        default_local_extra_mode_exist,
        default_local_extra_switch_onoff,
        default_local_extra_switch_enable,
        default_local_extra_switch_exist,
        default_global_extra_switch_cnt,
        default_global_extra_mode_cnt,
        default_object_disp_cnt,
        default_local_extra_mode_cnt,
        default_local_extra_switch_cnt,
        default_charakoe_cnt,
        default_charakoe_onoff,
        default_charakoe_volume,
        load_wipe_type,
        load_wipe_time_ms,
        load_after_call,
        preload_database_tables,
        preload_database_row_calls,
        preload_database_col_calls,
        preload_database_col_types,
        preload_cg_flag_count,
        preload_cg_name_to_flag,
        preload_cg_group_codes,
        preload_cg_code_exist_cnt,
        preload_bgm_names,
        preload_counter_count,
        preload_frame_action_ch_count,
        flick_scene_routes,
        movie_backends,
        quake_ref_csv,
        quake_ref_report,
    })
}
