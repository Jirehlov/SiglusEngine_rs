use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Instant;

use anyhow::{Context, Result};
use eframe::egui;
use log::{debug, error, info, warn};
use simplelog::{Config, LevelFilter, WriteLogger};

mod gui_assets;
mod gui_config;
mod resource_bootstrap;
mod stage;

use gui_assets::*;
use gui_config::{RunConfig, load_run_config};
use stage::{
    is_visual_or_flow_command, looks_like_stage_object_path, parse_stage_object_command,
    parse_stage_object_prop, parse_stage_plane_command, summarize_props,
};
mod audio;
mod input_bridge;
use audio::AudioManager;
use input_bridge::*;

const DEFAULT_GAMEEXE_NAME: &str = "Gameexe.dat";

// ── Layout constants (inspired by C++ elm_mwnd layout) ──────────────────

const MSG_WINDOW_HEIGHT_RATIO: f32 = 0.28;
const MSG_WINDOW_MARGIN_X: f32 = 24.0;
const MSG_WINDOW_MARGIN_BOTTOM: f32 = 16.0;
const MSG_WINDOW_ROUNDING: f32 = 12.0;
const MSG_BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(10, 12, 22, 210);

const NAME_PLATE_HEIGHT: f32 = 36.0;
const NAME_PLATE_OFFSET_Y: f32 = -8.0;
const NAME_PLATE_MARGIN_LEFT: f32 = 40.0;
const NAME_PLATE_PADDING_X: f32 = 24.0;
const NAME_PLATE_BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(30, 60, 120, 230);

const MSG_TEXT_PADDING_X: f32 = 36.0;
const MSG_TEXT_PADDING_TOP: f32 = 20.0;

const CLICK_INDICATOR_SIZE: f32 = 12.0;

const SEL_BUTTON_WIDTH: f32 = 520.0;
const SEL_BUTTON_HEIGHT: f32 = 48.0;
const SEL_BUTTON_SPACING: f32 = 10.0;
const SEL_BUTTON_ROUNDING: f32 = 8.0;
const SEL_BUTTON_BG: egui::Color32 = egui::Color32::from_rgba_premultiplied(40, 60, 110, 220);
const SEL_BUTTON_HOVER_BG: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(60, 100, 180, 240);

// ── Run configuration ───────────────────────────────────────────────────

// ── Host events (VM thread → GUI thread) ────────────────────────────────

include!("wipe_direction.rs");
include!("host_events.rs");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VmErrorFilter {
    All,
    FatalOnly,
    FileNotFoundOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VmErrorSort {
    TimeDesc,
    SceneLineAsc,
}

include!("movie_events.rs");

include!("stage_plane.rs");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MovieWaitState {
    Pending,
    Ready,
    Failed,
    Interrupted,
}


#[derive(Debug, Clone)]
struct HostObjectState {
    file_name: String,
    pat_no: usize,
    x: f32,
    y: f32,
    center_x: f32,
    center_y: f32,
    visible: bool,
    order: i32,
    layer: i32,
    scale_x: f32,
    scale_y: f32,
    rotate_z_deg: f32,
    alpha: f32,
    alpha_blend: bool,
    dst_clip_use: bool,
    dst_clip_left: f32,
    dst_clip_top: f32,
    dst_clip_right: f32,
    dst_clip_bottom: f32,
    src_clip_use: bool,
    src_clip_left: f32,
    src_clip_top: f32,
    src_clip_right: f32,
    src_clip_bottom: f32,
    color_rate: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
    color_add_r: f32,
    color_add_g: f32,
    color_add_b: f32,
    bright: f32,
    dark: f32,
    mono: f32,
    reverse: bool,
    movie_auto_init: bool,
    movie_real_time: bool,
    movie_ready_only: bool,
    emote_rep_x: i32,
    emote_rep_y: i32,
    seq: u64,
}

#[derive(Debug, Clone, Copy)]
struct ObjectRenderState {
    center_x: f32,
    center_y: f32,
    scale_x: f32,
    scale_y: f32,
    rotate_z_deg: f32,
    alpha: f32,
    dst_clip_use: bool,
    dst_clip_left: f32,
    dst_clip_top: f32,
    dst_clip_right: f32,
    dst_clip_bottom: f32,
    src_clip_use: bool,
    src_clip_left: f32,
    src_clip_top: f32,
    src_clip_right: f32,
    src_clip_bottom: f32,
}

#[derive(Debug, Clone)]
struct HostGroupState {
    order: i32,
    layer: i32,
    cancel_priority: i32,
    hit_button_no: i32,
    pushed_button_no: i32,
    decided_button_no: i32,
    result: i32,
    result_button_no: i32,
    on_hit_no: i32,
    on_pushed_no: i32,
    on_decided_no: i32,
    hover_button_no: i32,
    press_keep_button_no: i32,
    active: bool,
    cancel_enabled: bool,
    cancel_se_no: i32,
}

impl Default for HostGroupState {
    fn default() -> Self {
        Self {
            order: 0,
            layer: 0,
            cancel_priority: 0,
            hit_button_no: -1,
            pushed_button_no: -1,
            decided_button_no: -1,
            result: -1,
            result_button_no: -1,
            on_hit_no: -1,
            on_pushed_no: -1,
            on_decided_no: -1,
            hover_button_no: -1,
            press_keep_button_no: -1,
            active: false,
            cancel_enabled: false,
            cancel_se_no: -1,
        }
    }
}

include!("int_event_state.rs");

// ── VM Host implementation ──────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
struct HostCaptureBuffer {
    width: i32,
    height: i32,
    origin_x: i32,
    origin_y: i32,
    png_path: String,
}

struct GuiHost {
    event_tx: mpsc::Sender<HostEvent>,
    selection_rx: mpsc::Receiver<i32>,
    return_to_menu_warning_rx: mpsc::Receiver<bool>,
    advance_rx: mpsc::Receiver<AdvanceSignal>,
    movie_event_rx: mpsc::Receiver<MoviePlaybackEvent>,
    skip_mode: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
    base_dir: PathBuf,
    append_dirs: Vec<PathBuf>,
    persistent_state_path: PathBuf,
    objects: BTreeMap<(StagePlane, i32), HostObjectState>,
    stage_object_sizes: BTreeMap<StagePlane, i32>,
    stage_group_sizes: BTreeMap<StagePlane, i32>,
    groups: BTreeMap<(StagePlane, i32), HostGroupState>,
    movie_playing_objects: std::collections::BTreeSet<(StagePlane, i32)>,
    movie_ready_objects: std::collections::BTreeSet<(StagePlane, i32)>,
    movie_auto_free_ms: BTreeMap<(StagePlane, i32), i32>,
    movie_generations: BTreeMap<(StagePlane, i32), u64>,
    movie_last_failure: BTreeMap<(StagePlane, i32), MovieFailureInfo>,
    movie_interrupted_objects: std::collections::BTreeSet<(StagePlane, i32)>,
    next_movie_generation: u64,
    global_mov_playing: bool,
    mwnd_list_size: i32,
    world_list_size: i32,
    effect_list_size: i32,
    quake_list_size: i32,
    int_event_list_sizes: BTreeMap<i32, i32>,
    quake_active_until: Option<Instant>,
    last_quake_request: Option<siglus::vm::VmQuakeRequest>,
    next_object_seq: u64,
    int_events: std::collections::HashMap<i32, IntEventState>,
    input_state: Arc<Mutex<SharedInputState>>,
    cancel_se_map: BTreeMap<i32, String>,
    vm_scene: String,
    vm_line_no: i32,
    vm_pc: usize,
    vm_element: Vec<i32>,
    capture_buffer: Option<HostCaptureBuffer>,
    pending_selbtn_request: Option<SelectionRequest>,
}

include!("prop_int_ext.rs");
include!("host_impl_quake_macro.rs");
include!("cancel_se_map.rs");
include!("app_render_overlay_macro.rs");

include!("host_impl.rs");
struct GuiApp {
    event_rx: mpsc::Receiver<HostEvent>,
    selection_tx: mpsc::Sender<i32>,
    return_to_menu_warning_tx: mpsc::Sender<bool>,
    advance_tx: mpsc::Sender<AdvanceSignal>,
    movie_event_tx: mpsc::Sender<MoviePlaybackEvent>,
    skip_mode: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,

    current_name: String,
    current_text: String,
    waiting_for_click: bool, // true when VM is blocked waiting for user advance
    queued_advance_stock: usize,
    hide_message_window: bool,
    message_window_visible: bool,
    pending_options: Vec<SelectionOption>,
    pending_selbtn: Option<SelBtnNamedArgs>,
    backlog: Vec<String>,
    done: bool,
    show_backlog: bool,
    msg_back_display_enabled: bool,
    tweet_dialog_open: bool,
    tweet_text: String,
    tweet_authorized: bool,
    tweet_user_name: String,
    tweet_screen_name: String,
    tweet_status_line: String,
    tweet_confirm_empty: bool,
    show_return_to_menu_warning: bool,
    latest_vm_error: Option<(VmErrorLevel, String, Instant, VmErrorContext)>,
    vm_error_history: Vec<(VmErrorLevel, String, Instant, VmErrorContext)>,
    vm_error_last_copied: Option<String>,
    vm_error_copy_notice_until: Option<Instant>,
    vm_error_filter_recent_copy: bool,
    vm_error_copy_history: Vec<String>,
    vm_error_copy_selected: BTreeSet<String>,
    vm_error_pinned: BTreeSet<String>,
    show_vm_error_panel: bool,
    vm_error_filter: VmErrorFilter,
    vm_error_search: String,
    vm_error_sort: VmErrorSort,
    background_texture: Option<egui::TextureHandle>,
    background_textures: BTreeMap<StagePlane, egui::TextureHandle>,
    missing_background_names: BTreeMap<StagePlane, String>,
    object_textures: BTreeMap<(StagePlane, i32), egui::TextureHandle>,
    missing_object_names: BTreeMap<(StagePlane, i32), String>,
    object_pos: BTreeMap<(StagePlane, i32), egui::Pos2>,
    object_visible: BTreeMap<(StagePlane, i32), bool>,
    object_sort: BTreeMap<(StagePlane, i32), (i32, i32, u64)>,
    object_render: BTreeMap<(StagePlane, i32), ObjectRenderState>,
    base_title: String,
    location_scene_title: String,
    location_scene: String,
    location_line_no: i32,
    location_pc: usize,
    last_window_title: String,
    scene_size: Option<(i32, i32)>,
    wipe_started_at: Option<Instant>,
    wipe_duration_ms: u64,
    wipe_type: i32,
    wipe_direction: WipeDirection,

    start_time: Instant,
    audio_manager: Option<AudioManager>,
    input_state: Arc<Mutex<SharedInputState>>,
    base_dir: PathBuf,
    append_dirs: Vec<PathBuf>,
    movie_backends: Vec<String>,
    movie_stop_flags: Arc<Mutex<BTreeMap<(StagePlane, i32, u64), Arc<AtomicBool>>>>,
    quake_ref_csv: Option<PathBuf>,
    quake_ref_report: PathBuf,
    quake_started_at: Option<Instant>,
    quake_request: Option<siglus::vm::VmQuakeRequest>,
}
include!("app_logic.rs");
include!("app_quake_reference.rs");
include!("app_runtime_effects.rs");
include!("app_render.rs");
include!("app_tweet_dialog.rs");

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.consume_events(ctx);
        self.handle_input(ctx);
        let title = self.compose_window_title();
        if title != self.last_window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_window_title = title;
        }
        ctx.set_visuals(egui::Visuals::dark());

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(8, 8, 16),
                ..Default::default()
            })
            .show(ctx, |ui| {
                self.draw_background(ui);
                self.draw_objects(ui);

                if self.message_window_visible && self.show_backlog {
                    self.draw_backlog(ui);
                } else if (!self.hide_message_window && self.message_window_visible)
                    || !self.pending_options.is_empty()
                {
                    self.draw_message_window(ui);
                    self.draw_selections(ui);
                }

                if self.message_window_visible && !self.hide_message_window {
                    self.draw_toolbar(ui);
                }

                self.draw_return_to_menu_warning(ui);
                self.draw_tweet_dialog(ui);
                self.draw_vm_error_panel(ui);
                self.draw_vm_error_overlay(ui);
                self.draw_wipe_overlay(ui);
            });

        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Signal the VM thread to shut down
        self.shutdown.store(true, Ordering::Relaxed);
        let _ = self.advance_tx.send(AdvanceSignal::Shutdown);
        let _ = self.selection_tx.send(0); // unblock selection wait
        let _ = self.return_to_menu_warning_tx.send(true);
    }
}

// ── Configuration loading ───────────────────────────────────────────────

fn setup_multilingual_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for (idx, path) in [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansSC-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
        "/System/Library/Fonts/PingFang.ttc",
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/simsun.ttc",
    ]
    .iter()
    .enumerate()
    {
        if let Ok(bytes) = std::fs::read(path) {
            let font_name = format!("fallback-{idx}");
            fonts
                .font_data
                .insert(font_name.clone(), egui::FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push(font_name.clone());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push(font_name);
        }
    }
    ctx.set_fonts(fonts);
}

fn load_persistent_state(path: &Path) -> Result<Option<siglus::vm::VmPersistentState>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path)
        .with_context(|| format!("failed to read persistent state: {}", path.display()))?;
    let st = siglus::vm::VmPersistentState::decode_binary(&bytes)
        .with_context(|| format!("failed to parse persistent state: {}", path.display()))?;
    Ok(Some(st))
}

fn save_persistent_state(path: &Path, state: &siglus::vm::VmPersistentState) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create state dir: {}", parent.display()))?;
    }
    std::fs::write(path, state.encode_binary())
        .with_context(|| format!("failed to write persistent state: {}", path.display()))
}

fn load_end_save_state(path: &Path) -> Result<Option<siglus::vm::VmEndSaveState>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path)
        .with_context(|| format!("failed to read end-save state: {}", path.display()))?;
    let st = siglus::vm::VmEndSaveState::decode_binary(&bytes)
        .with_context(|| format!("failed to parse end-save state: {}", path.display()))?;
    Ok(Some(st))
}

fn save_end_save_state(path: &Path, state: &siglus::vm::VmEndSaveState) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create end-save dir: {}", parent.display()))?;
    }
    std::fs::write(path, state.encode_binary())
        .with_context(|| format!("failed to write end-save state: {}", path.display()))
}

fn run_gui(args: RunConfig) -> Result<()> {
    // Initialize logging
    if let Err(e) = GuiApp::init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
    }
    info!("Gui started with args: {:?}", args.title);

    let (event_tx, event_rx) = mpsc::channel::<HostEvent>();
    let (selection_tx, selection_rx) = mpsc::channel::<i32>();
    let (return_to_menu_warning_tx, return_to_menu_warning_rx) = mpsc::channel::<bool>();
    let (advance_tx, advance_rx) = mpsc::channel::<AdvanceSignal>();
    let (movie_event_tx, movie_event_rx) = mpsc::channel::<MoviePlaybackEvent>();
    let skip_mode = Arc::new(AtomicBool::new(false));
    let shutdown = Arc::new(AtomicBool::new(false));
    let input_state = Arc::new(Mutex::new(SharedInputState::default()));

    let worker_event_tx = event_tx.clone();
    let worker_skip = skip_mode.clone();
    let worker_shutdown = shutdown.clone();
    let worker_input_state = input_state.clone();

    let base_dir = args
        .pck
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .to_path_buf();
    let audio_manager = audio::AudioManager::new(base_dir.clone()).ok();
    let app_append_dirs = args.append_search_dirs.clone();
    let cancel_se_map = load_cancel_se_map_from_gameexe(&args.gameexe);

    let _worker = thread::spawn(move || {
        let run = || -> Result<u64> {
            let pack = siglus::pck::read_file(&args.pck)
                .with_context(|| format!("failed to read pack: {}", args.pck.display()))?;
            let mut rt = siglus::runtime::Runtime::new(pack)?;

            let mut host = GuiHost {
                event_tx: worker_event_tx,
                selection_rx,
                return_to_menu_warning_rx,
                advance_rx,
                movie_event_rx,
                skip_mode: worker_skip,
                shutdown: worker_shutdown,
                base_dir: args
                    .pck
                    .parent()
                    .unwrap_or(&PathBuf::from("."))
                    .to_path_buf(),
                append_dirs: args.append_search_dirs.clone(),
                persistent_state_path: args.persistent_state_path.clone(),
                objects: BTreeMap::new(),
                stage_object_sizes: BTreeMap::new(),
                stage_group_sizes: BTreeMap::new(),
                groups: BTreeMap::new(),
                movie_playing_objects: std::collections::BTreeSet::new(),
                movie_ready_objects: std::collections::BTreeSet::new(),
                movie_auto_free_ms: BTreeMap::new(),
                movie_generations: BTreeMap::new(),
                movie_last_failure: BTreeMap::new(),
                movie_interrupted_objects: std::collections::BTreeSet::new(),
                next_movie_generation: 1,
                global_mov_playing: false,
                mwnd_list_size: 1,
                world_list_size: 1,
                effect_list_size: 0,
                quake_list_size: 0,
                int_event_list_sizes: BTreeMap::new(),
                quake_active_until: None,
                last_quake_request: None,
                next_object_seq: 1,
                int_events: std::collections::HashMap::new(),
                input_state: worker_input_state,
                cancel_se_map: cancel_se_map.clone(),
                vm_scene: String::new(),
                vm_line_no: 0,
                vm_pc: 0,
                vm_element: Vec::new(),
                capture_buffer: None,
                pending_selbtn_request: None,
            };

            let state_in = match load_persistent_state(&args.persistent_state_path) {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to load persistent state: {:#}", e);
                    None
                }
            };

            let (steps, _stats, state_out) = rt.run_scene_z_with_options_and_persistent_state(
                &args.scene,
                args.z,
                &mut host,
                args.max_steps,
                siglus::vm::VmOptions {
                    return_menu_scene: Some((args.menu_scene.clone(), args.menu_z)),
                    system_extra_int_values: args.system_extra_int_values.clone(),
                    system_extra_str_values: args.system_extra_str_values.clone(),
                    default_global_extra_switch: args.default_global_extra_switch.clone(),
                    default_global_extra_mode: args.default_global_extra_mode.clone(),
                    default_object_disp: args.default_object_disp.clone(),
                    default_local_extra_mode_value: args.default_local_extra_mode_value.clone(),
                    default_local_extra_mode_enable: args.default_local_extra_mode_enable.clone(),
                    default_local_extra_mode_exist: args.default_local_extra_mode_exist.clone(),
                    default_local_extra_switch_onoff: args.default_local_extra_switch_onoff.clone(),
                    default_local_extra_switch_enable: args
                        .default_local_extra_switch_enable
                        .clone(),
                    default_local_extra_switch_exist: args.default_local_extra_switch_exist.clone(),
                    default_global_extra_switch_cnt: args.default_global_extra_switch_cnt,
                    default_global_extra_mode_cnt: args.default_global_extra_mode_cnt,
                    default_object_disp_cnt: args.default_object_disp_cnt,
                    default_local_extra_mode_cnt: args.default_local_extra_mode_cnt,
                    default_local_extra_switch_cnt: args.default_local_extra_switch_cnt,
                    default_charakoe_cnt: args.default_charakoe_cnt,
                    default_charakoe_onoff: args.default_charakoe_onoff.clone(),
                    default_charakoe_volume: args.default_charakoe_volume.clone(),
                    load_wipe_type: args.load_wipe_type,
                    load_wipe_time_ms: args.load_wipe_time_ms,
                    load_after_call_scene: args.load_after_call.as_ref().map(|(s, _)| s.clone()),
                    load_after_call_z_no: args
                        .load_after_call
                        .as_ref()
                        .map(|(_, z)| *z)
                        .unwrap_or(0),
                    preloaded_database_tables: args.preload_database_tables.clone(),
                    preloaded_database_row_calls: args.preload_database_row_calls.clone(),
                    preloaded_database_col_calls: args.preload_database_col_calls.clone(),
                    preloaded_database_col_types: args.preload_database_col_types.clone(),
                    preloaded_cg_flag_count: args.preload_cg_flag_count,
                    preloaded_cg_name_to_flag: args.preload_cg_name_to_flag.clone(),
                    preloaded_cg_group_codes: args.preload_cg_group_codes.clone(),
                    preloaded_cg_code_exist_cnt: args.preload_cg_code_exist_cnt.clone(),
                    preloaded_bgm_names: args.preload_bgm_names.clone(),
                    preloaded_counter_count: args.preload_counter_count,
                    preloaded_frame_action_ch_count: args.preload_frame_action_ch_count,
                    flick_scene_routes: args.flick_scene_routes.clone(),
                    ..siglus::vm::VmOptions::default()
                },
                state_in.as_ref(),
            )?;

            if let Err(e) = save_persistent_state(&args.persistent_state_path, &state_out) {
                error!("Failed to save persistent state: {:#}", e);
            }
            Ok(steps)
        };

        match run() {
            Ok(_steps) => {
                let _ = event_tx.send(HostEvent::Done);
            }
            Err(err) => {
                error!("Worker thread error: {:#}", err);
                // let _ = event_tx.send(HostEvent::Error(format!("{err:#}")));
                let _ = event_tx.send(HostEvent::Done);
            }
        }
    });

    let app = GuiApp::new(
        event_rx,
        selection_tx.clone(),
        return_to_menu_warning_tx.clone(),
        advance_tx.clone(),
        movie_event_tx.clone(),
        skip_mode.clone(),
        shutdown.clone(),
        args.title.clone(),
        args.scene_size,
        audio_manager,
        input_state,
        base_dir.clone(),
        app_append_dirs,
        args.movie_backends.clone(),
        args.quake_ref_csv.clone(),
        args.quake_ref_report.clone(),
    );

    let mut native_options = eframe::NativeOptions::default();
    let (w, h) = args.window_size.unwrap_or((1280, 720));
    native_options.viewport = native_options
        .viewport
        .with_inner_size([w as f32, h as f32]);

    eframe::run_native(
        &format!("{} - Siglus", args.title),
        native_options,
        Box::new(|cc| {
            setup_multilingual_fonts(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
    .map_err(|err| anyhow::anyhow!("failed to run GUI: {err}"))?;

    Ok(())
}

pub(crate) fn run() -> Result<()> {
    // Initialize logging
    if let Err(e) = GuiApp::init_logging() {
        eprintln!("Failed to initialize logging: {e}");
    }
    info!("Starting Siglus GUI...");

    let config = load_run_config()?;
    if !config.pck.exists() {
        anyhow::bail!(
            "scene pack not found: {} (from {})",
            config.pck.display(),
            config.gameexe.display()
        );
    }
    run_gui(config)
}
