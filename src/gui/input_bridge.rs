const REPEAT_START_FRAMES: i32 = 24;
const REPEAT_INTERVAL_FRAMES: i32 = 4;
// C++ reference: button flick threshold in std_input path (empirical parity target).
const FLICK_MIN_PIXEL: i32 = 24;
const MM_PER_INCH: f32 = 25.4;
const FALLBACK_DPI: f32 = 96.0;

#[derive(Debug, Clone, Copy, Default)]
struct FlickSample {
    angle: i32,
    angle_radian: f64,
    pixel: i32,
    mm: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct InputButtonTracker {
    is_down: bool,
    down_stock: u8,
    up_stock: u8,
    repeat_stock: u8,
    hold_frames: i32,
    flick_stock: u8,
    flick_sample: FlickSample,
    press_origin: Option<(i32, i32)>,
    cursor_pos: (i32, i32),
    pixels_per_point: f32,
}

impl InputButtonTracker {
    pub(super) fn update(&mut self, down: bool) {
        self.update_with_pointer(
            down,
            self.cursor_pos.0,
            self.cursor_pos.1,
            self.pixels_per_point,
        );
    }

    pub(super) fn update_with_pointer(
        &mut self,
        down: bool,
        cursor_x: i32,
        cursor_y: i32,
        pixels_per_point: f32,
    ) {
        self.cursor_pos = (cursor_x, cursor_y);
        self.pixels_per_point = pixels_per_point.max(0.01);
        if down && !self.is_down {
            self.down_stock = self.down_stock.saturating_add(1);
            self.hold_frames = 0;
            self.press_origin = Some((cursor_x, cursor_y));
        } else if down && self.is_down {
            self.hold_frames = self.hold_frames.saturating_add(1);
            if self.hold_frames >= REPEAT_START_FRAMES
                && (self.hold_frames - REPEAT_START_FRAMES) % REPEAT_INTERVAL_FRAMES == 0
            {
                self.repeat_stock = self.repeat_stock.saturating_add(1);
            }
        } else if !down && self.is_down {
            self.up_stock = self.up_stock.saturating_add(1);
            self.hold_frames = 0;
            self.capture_flick(cursor_x, cursor_y);
            self.press_origin = None;
        }
        self.is_down = down;
    }

    fn capture_flick(&mut self, release_x: i32, release_y: i32) {
        let Some((start_x, start_y)) = self.press_origin else {
            return;
        };
        let dx = release_x - start_x;
        let dy = release_y - start_y;
        let pixel = ((dx * dx + dy * dy) as f64).sqrt().round() as i32;
        if pixel < FLICK_MIN_PIXEL {
            return;
        }

        let angle_radian = (dy as f64).atan2(dx as f64);
        // C++ cmd_input.cpp::ELM_KEY_GET_FLICK_ANGLE
        // int((180.0 - rad/PI*180) * TNM_ANGLE_UNIT), TNM_ANGLE_UNIT=10.
        // Keep truncation semantics (`as i32`) instead of rounding.
        let angle = ((180.0 - angle_radian / std::f64::consts::PI * 180.0) * 10.0) as i32;
        let dpi = (self.pixels_per_point * 72.0).max(FALLBACK_DPI);
        let mm = ((pixel as f32) * MM_PER_INCH / dpi).round() as i32;

        self.flick_stock = self.flick_stock.saturating_add(1);
        self.flick_sample = FlickSample {
            angle,
            angle_radian,
            pixel,
            mm,
        };
    }

    pub(super) fn has_flick_stock(&self) -> bool {
        self.flick_stock > 0
    }

    pub(super) fn flick_angle_radian(&self) -> f64 {
        self.flick_sample.angle_radian
    }

    pub(super) fn use_flick_stock(&mut self) -> bool {
        if self.flick_stock == 0 {
            return false;
        }
        self.flick_stock -= 1;
        true
    }

    pub(super) fn clear(&mut self) {
        self.is_down = false;
        self.down_stock = 0;
        self.up_stock = 0;
        self.repeat_stock = 0;
        self.hold_frames = 0;
        self.flick_stock = 0;
        self.flick_sample = FlickSample::default();
        self.press_origin = None;
    }

    pub(super) fn next_frame(&mut self) {
        self.down_stock = 0;
        self.up_stock = 0;
        self.repeat_stock = 0;
        self.flick_stock = 0;
    }

    pub(super) fn has_down_up_stock(&self) -> bool {
        self.down_stock > 0 || self.up_stock > 0
    }

    pub(super) fn use_down_up_stock(&mut self) -> bool {
        if self.down_stock > 0 {
            self.down_stock -= 1;
            return true;
        }
        if self.up_stock > 0 {
            self.up_stock -= 1;
            return true;
        }
        false
    }

    pub(super) fn snapshot_and_consume(&mut self) -> siglus::vm::VmInputButtonState {
        let on_down = self.down_stock > 0;
        let on_up = self.up_stock > 0;
        let on_repeat = self.repeat_stock > 0;
        let on_flick = self.flick_stock > 0;
        if on_down {
            self.down_stock -= 1;
        }
        if on_up {
            self.up_stock -= 1;
        }
        if on_repeat {
            self.repeat_stock -= 1;
        }
        if on_flick {
            self.flick_stock -= 1;
        }
        siglus::vm::VmInputButtonState {
            on_down,
            on_up,
            on_down_up: on_down || on_up,
            is_down: self.is_down,
            is_up: !self.is_down,
            on_flick,
            on_repeat,
            flick_angle: self.flick_sample.angle,
            flick_pixel: self.flick_sample.pixel,
            flick_mm: self.flick_sample.mm,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct SharedInputState {
    pub(super) mouse_x: i32,
    pub(super) mouse_y: i32,
    pub(super) wheel_delta: i32,
    pub(super) mouse_left: InputButtonTracker,
    pub(super) mouse_right: InputButtonTracker,
    pub(super) decide: InputButtonTracker,
    pub(super) cancel: InputButtonTracker,
    pub(super) keyboard: [InputButtonTracker; 256],
    pub(super) pixels_per_point: f32,
}

impl Default for SharedInputState {
    fn default() -> Self {
        Self {
            mouse_x: 0,
            mouse_y: 0,
            wheel_delta: 0,
            mouse_left: InputButtonTracker::default(),
            mouse_right: InputButtonTracker::default(),
            decide: InputButtonTracker::default(),
            cancel: InputButtonTracker::default(),
            keyboard: [InputButtonTracker::default(); 256],
            pixels_per_point: 1.0,
        }
    }
}

impl SharedInputState {
    pub(super) fn clear_mouse(&mut self) {
        self.wheel_delta = 0;
        self.mouse_left.clear();
        self.mouse_right.clear();
    }

    pub(super) fn next_mouse(&mut self) {
        self.wheel_delta = 0;
        self.mouse_left.next_frame();
        self.mouse_right.next_frame();
    }

    pub(super) fn clear_keyboard(&mut self) {
        self.decide.clear();
        self.cancel.clear();
        for key in &mut self.keyboard {
            key.clear();
        }
    }

    pub(super) fn next_keyboard(&mut self) {
        self.decide.next_frame();
        self.cancel.next_frame();
        for key in &mut self.keyboard {
            key.next_frame();
        }
    }

    pub(super) fn clear_all(&mut self) {
        self.clear_mouse();
        self.clear_keyboard();
    }

    pub(super) fn has_key_wait_press_stock(&self) -> bool {
        // C++ flow_proc.cpp::tnm_key_wait_proc checks only VK_EX_DECIDE down_up stock.
        self.decide.has_down_up_stock()
    }

    pub(super) fn consume_key_wait_press_stock(&mut self) -> bool {
        self.decide.use_down_up_stock()
    }

    pub(super) fn left_flick_state(&self) -> siglus::vm::VmFlickState {
        siglus::vm::VmFlickState {
            has_flick_stock: self.mouse_left.has_flick_stock(),
            angle_radian: self.mouse_left.flick_angle_radian(),
        }
    }

    pub(super) fn consume_left_flick_stock(&mut self) -> bool {
        self.mouse_left.use_flick_stock()
    }
}
