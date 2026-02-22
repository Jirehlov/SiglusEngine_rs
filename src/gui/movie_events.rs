#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MovieFailureCategory {
    Resource,
    Spawn,
    Wait,
    ExitCode,
    Exhausted,
}

#[derive(Debug, Clone)]
struct MovieFailureInfo {
    category: MovieFailureCategory,
    backend: Option<String>,
    detail: String,
    unrecoverable: bool,
    spawn_fail: usize,
    wait_fail: usize,
    exit_fail: usize,
}

impl MovieFailureInfo {
    fn simple(
        category: MovieFailureCategory,
        detail: impl Into<String>,
        unrecoverable: bool,
    ) -> Self {
        Self {
            category,
            backend: None,
            detail: detail.into(),
            unrecoverable,
            spawn_fail: 0,
            wait_fail: 0,
            exit_fail: 0,
        }
    }

    fn with_backend(mut self, backend: impl Into<String>) -> Self {
        self.backend = Some(backend.into());
        self
    }

    fn with_counters(mut self, spawn_fail: usize, wait_fail: usize, exit_fail: usize) -> Self {
        self.spawn_fail = spawn_fail;
        self.wait_fail = wait_fail;
        self.exit_fail = exit_fail;
        self
    }
}

impl MovieFailureCategory {
    fn as_code(self) -> i32 {
        match self {
            MovieFailureCategory::Resource => 1,
            MovieFailureCategory::Spawn => 2,
            MovieFailureCategory::Wait => 3,
            MovieFailureCategory::ExitCode => 4,
            MovieFailureCategory::Exhausted => 5,
        }
    }
}

impl MovieFailureInfo {
    fn status_code(&self) -> i32 {
        let base = self.category.as_code();
        if self.unrecoverable {
            base
        } else {
            -base
        }
    }

    fn counters_packed(&self) -> i32 {
        let s = (self.spawn_fail.min(255) as i32) << 16;
        let w = (self.wait_fail.min(255) as i32) << 8;
        let e = self.exit_fail.min(255) as i32;
        s | w | e
    }

    fn category_code(&self) -> i32 {
        self.category.as_code()
    }

    fn unrecoverable_flag(&self) -> i32 {
        i32::from(self.unrecoverable)
    }

    fn backend_hash(&self) -> i32 {
        self.backend
            .as_deref()
            .map(stable_iapp_hash)
            .unwrap_or_default()
    }

    fn detail_hash(&self) -> i32 {
        stable_iapp_hash(&self.detail)
    }

    fn spawn_fail_count(&self) -> i32 {
        self.spawn_fail.min(i32::MAX as usize) as i32
    }

    fn wait_fail_count(&self) -> i32 {
        self.wait_fail.min(i32::MAX as usize) as i32
    }

    fn exit_fail_count(&self) -> i32 {
        self.exit_fail.min(i32::MAX as usize) as i32
    }
}

fn stable_iapp_hash(text: &str) -> i32 {
    let mut h: u32 = 0x811C_9DC5;
    for b in text.as_bytes() {
        h ^= *b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    (h & 0x7FFF_FFFF) as i32
}

#[derive(Debug, Clone)]
enum MoviePlaybackEvent {
    ObjectStarted {
        stage: StagePlane,
        index: i32,
        generation: u64,
    },
    ObjectFinished {
        stage: StagePlane,
        index: i32,
        generation: u64,
    },
    ObjectFailed {
        stage: StagePlane,
        index: i32,
        generation: u64,
        info: MovieFailureInfo,
    },
    ObjectInterrupted {
        stage: StagePlane,
        index: i32,
        generation: u64,
    },
}
