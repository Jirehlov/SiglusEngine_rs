#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VmErrorLevel {
    Fatal,
    FileNotFound,
}

#[derive(Debug, Clone, Default)]
struct VmErrorContext {
    scene: String,
    line_no: i32,
    pc: usize,
    element: Vec<i32>,
}

enum HostEvent {
    Name(String),
    Text {
        text: String,
    },
    Selection(Vec<String>),
    VmError {
        level: VmErrorLevel,
        message: String,
        context: VmErrorContext,
    },
    LoadImage {
        image: Arc<image::DynamicImage>,
    },
    LoadPlaneImage {
        stage: StagePlane,
        image: Arc<image::DynamicImage>,
    },
    MissingPlaneImage {
        stage: StagePlane,
        name: String,
    },
    UpsertObjectImage {
        stage: StagePlane,
        index: i32,
        image: Arc<image::DynamicImage>,
    },
    MissingObjectImage {
        stage: StagePlane,
        index: i32,
        name: String,
    },
    SetObjectPos {
        stage: StagePlane,
        index: i32,
        x: f32,
        y: f32,
    },
    SetObjectVisible {
        stage: StagePlane,
        index: i32,
        visible: bool,
    },
    RemoveObject {
        stage: StagePlane,
        index: i32,
    },
    SetObjectSort {
        stage: StagePlane,
        index: i32,
        order: i32,
        layer: i32,
        seq: u64,
    },
    SetObjectRenderState {
        stage: StagePlane,
        index: i32,
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
    },
    ClearPlaneObjects {
        stage: StagePlane,
    },
    Location {
        scene_title: String,
        scene: String,
        line_no: i32,
        pc: usize,
    },
    MessageWindowVisible(bool),
    MsgBackState(bool),
    MsgBackDisplayEnabled(bool),
    OpenTweetDialog,
    ConfirmReturnToMenuWarning,
    StartWipe {
        duration_ms: u64,
        wipe_type: i32,
        wipe_direction: WipeDirection,
    },
    SetCursorPos {
        x: i32,
        y: i32,
    },
    PlayBgm {
        name: String,
        loop_flag: bool,
        fade_in_ms: i32,
    },
    StopBgm {
        fade_out_ms: i32,
    },
    PlaySe {
        name: String,
    },
    StopSe,
    PlayPcm {
        ch: i32,
        name: String,
        loop_flag: bool,
    },
    StopPcm {
        ch: i32,
    },
    PlayObjectMovie {
        stage: StagePlane,
        index: i32,
        file_name: String,
        duration_ms: i32,
        generation: u64,
    },
    StopObjectMovie {
        stage: StagePlane,
        index: i32,
        generation: u64,
    },
    StartQuake {
        req: siglus::vm::VmQuakeRequest,
        started_at: Instant,
    },
    EndQuake,
    Done,
}

/// Sent by the GUI to unblock the VM when the user clicks to advance text.
enum AdvanceSignal {
    Proceed,
    Shutdown,
}
