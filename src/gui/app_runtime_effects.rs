impl GuiApp {
    fn is_unrecoverable_movie_resource(path: &Path) -> Option<&'static str> {
        let Ok(meta) = std::fs::metadata(path) else {
            return Some("metadata_error");
        };
        if !meta.is_file() {
            return Some("not_regular_file");
        }
        if meta.len() == 0 {
            return Some("empty_file");
        }
        if std::fs::File::open(path).is_err() {
            return Some("open_failed");
        }
        None
    }

    fn is_unrecoverable_spawn_error(err: &std::io::Error) -> bool {
        matches!(
            err.kind(),
            std::io::ErrorKind::PermissionDenied
                | std::io::ErrorKind::InvalidInput
                | std::io::ErrorKind::InvalidData
                | std::io::ErrorKind::NotADirectory
        )
    }

    fn is_unrecoverable_wait_error(err: &std::io::Error) -> bool {
        matches!(
            err.kind(),
            std::io::ErrorKind::InvalidInput
                | std::io::ErrorKind::InvalidData
                | std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::UnexpectedEof
        )
    }

    fn is_unrecoverable_exit_status(status: std::process::ExitStatus) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            if let Some(sig) = status.signal() {
                // Fatal signals通常表示后端本身崩溃，继续切后端收益极低，直接短路。
                if matches!(sig, 4 | 6 | 7 | 8 | 11) {
                    return true;
                }
            }
        }
        matches!(status.code(), Some(126 | 127 | 134 | 139))
    }

    fn resolve_movie_asset(&self, file_name: &str) -> Option<PathBuf> {
        if file_name.trim().is_empty() {
            return None;
        }
        let rel = PathBuf::from(file_name.replace('\\', "/"));
        let mut roots = vec![self.base_dir.clone()];
        for p in &self.append_dirs {
            if !roots.iter().any(|r| r == p) {
                roots.push(p.clone());
            }
        }
        let has_ext = rel.extension().is_some();
        let ext_candidates = ["wmv", "mp4", "avi", "mpeg", "mpg", "mov", "webm", "ogv"];
        for root in roots {
            if has_ext {
                let p = root.join(&rel);
                if p.exists() {
                    return Some(p);
                }
            } else {
                for ext in ext_candidates {
                    let p = root.join(&rel).with_extension(ext);
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
        None
    }

    fn movie_backend_candidates(&self) -> Vec<String> {
        let mut out = Vec::new();
        for backend in
            self.movie_backends
                .iter()
                .map(|v| v.as_str())
                .chain(["ffplay", "mpv", "gst-play-1.0"])
        {
            let key = backend.trim().to_ascii_lowercase();
            if key.is_empty() || out.iter().any(|v: &String| v == &key) {
                continue;
            }
            out.push(key);
        }
        out
    }

    fn request_stop_movie_process(&mut self, stage: StagePlane, index: i32, generation: u64) {
        if let Ok(map) = self.movie_stop_flags.lock() {
            if let Some(flag) = map.get(&(stage, index, generation)).cloned() {
                flag.store(true, Ordering::Relaxed);
            }
        }
    }

    fn mark_older_movie_processes_for_stop(
        &mut self,
        stage: StagePlane,
        index: i32,
        generation: u64,
    ) {
        if let Ok(map) = self.movie_stop_flags.lock() {
            for ((s, i, g), flag) in map.iter() {
                if *s == stage && *i == index && *g != generation {
                    flag.store(true, Ordering::Relaxed);
                }
            }
        }
    }

    fn spawn_movie_player_watcher(
        tx: std::sync::mpsc::Sender<MoviePlaybackEvent>,
        stop_registry: Arc<Mutex<BTreeMap<(StagePlane, i32, u64), Arc<AtomicBool>>>>,
        stage: StagePlane,
        index: i32,
        generation: u64,
        path: PathBuf,
        backends: Vec<String>,
        stop_flag: Arc<AtomicBool>,
    ) {
        std::thread::spawn(move || {
            if let Some(reason) = Self::is_unrecoverable_movie_resource(&path) {
                log::warn!(
                    "movie resource unrecoverable before spawn for {}: {}",
                    path.display(),
                    reason
                );
                let _ = tx.send(MoviePlaybackEvent::ObjectFailed {
                    stage,
                    index,
                    generation,
                    info: MovieFailureInfo::simple(MovieFailureCategory::Resource, reason, true),
                });
                if let Ok(mut map) = stop_registry.lock() {
                    map.remove(&(stage, index, generation));
                }
                return;
            }

            let mut spawn_fail = 0usize;
            let mut wait_fail = 0usize;
            let mut exit_fail = 0usize;
            for backend in backends {
                let mut cmd = match backend.as_str() {
                    "ffplay" => {
                        let mut c = std::process::Command::new("ffplay");
                        c.arg("-v")
                            .arg("error")
                            .arg("-autoexit")
                            .arg("-nodisp")
                            .arg("-loglevel")
                            .arg("error")
                            .arg(&path);
                        c
                    }
                    "mpv" => {
                        let mut c = std::process::Command::new("mpv");
                        c.arg("--no-config")
                            .arg("--vo=null")
                            .arg("--ao=null")
                            .arg("--idle=no")
                            .arg("--keep-open=no")
                            .arg("--really-quiet")
                            .arg(&path);
                        c
                    }
                    "gst-play-1.0" | "gstreamer" => {
                        let mut c = std::process::Command::new("gst-play-1.0");
                        c.arg("--videosink=fakesink")
                            .arg("--audiosink=fakesink")
                            .arg("--quiet")
                            .arg(&path);
                        c
                    }
                    _ => continue,
                };

                match cmd.spawn() {
                    Ok(mut child) => loop {
                        if stop_flag.load(Ordering::Relaxed) {
                            let _ = child.kill();
                            let _ = child.wait();
                            let _ = tx.send(MoviePlaybackEvent::ObjectInterrupted {
                                stage,
                                index,
                                generation,
                            });
                            if let Ok(mut map) = stop_registry.lock() {
                                map.remove(&(stage, index, generation));
                            }
                            return;
                        }
                        match child.try_wait() {
                            Ok(Some(status)) if status.success() => {
                                let _ = tx.send(MoviePlaybackEvent::ObjectFinished {
                                    stage,
                                    index,
                                    generation,
                                });
                                if let Ok(mut map) = stop_registry.lock() {
                                    map.remove(&(stage, index, generation));
                                }
                                return;
                            }
                            Ok(Some(status)) => {
                                exit_fail += 1;
                                let unrecoverable = Self::is_unrecoverable_exit_status(status);
                                log::warn!(
                                        "movie backend '{}' exited with status {} for {}, category=exit-code, unrecoverable={}, trying fallback",
                                        backend,
                                        status,
                                        path.display(),
                                        unrecoverable
                                    );
                                if unrecoverable {
                                    let _ = tx.send(MoviePlaybackEvent::ObjectFailed {
                                        stage,
                                        index,
                                        generation,
                                        info: MovieFailureInfo::simple(
                                            MovieFailureCategory::ExitCode,
                                            format!("status={status}"),
                                            true,
                                        )
                                        .with_backend(backend.clone())
                                        .with_counters(spawn_fail, wait_fail, exit_fail),
                                    });
                                    if let Ok(mut map) = stop_registry.lock() {
                                        map.remove(&(stage, index, generation));
                                    }
                                    return;
                                }
                                break;
                            }
                            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(12)),
                            Err(err) => {
                                wait_fail += 1;
                                let unrecoverable = Self::is_unrecoverable_wait_error(&err);
                                log::warn!(
                                        "movie backend '{}' wait failed for {}: {}, category=wait, unrecoverable={}, trying fallback",
                                        backend,
                                        path.display(),
                                        err,
                                        unrecoverable
                                    );
                                if unrecoverable {
                                    let _ = tx.send(MoviePlaybackEvent::ObjectFailed {
                                        stage,
                                        index,
                                        generation,
                                        info: MovieFailureInfo::simple(
                                            MovieFailureCategory::Wait,
                                            err.to_string(),
                                            true,
                                        )
                                        .with_backend(backend.clone())
                                        .with_counters(spawn_fail, wait_fail, exit_fail),
                                    });
                                    if let Ok(mut map) = stop_registry.lock() {
                                        map.remove(&(stage, index, generation));
                                    }
                                    return;
                                }
                                break;
                            }
                        }
                    },
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                        spawn_fail += 1;
                        log::warn!("movie backend '{}' not found, trying next backend", backend);
                        continue;
                    }
                    Err(err) => {
                        spawn_fail += 1;
                        let unrecoverable = Self::is_unrecoverable_spawn_error(&err);
                        log::warn!(
                            "movie backend '{}' failed to spawn for {}: {}, category=spawn, unrecoverable={}",
                            backend,
                            path.display(),
                            err,
                            unrecoverable
                        );
                        if !unrecoverable {
                            continue;
                        }
                        let _ = tx.send(MoviePlaybackEvent::ObjectFailed {
                            stage,
                            index,
                            generation,
                            info: MovieFailureInfo::simple(
                                MovieFailureCategory::Spawn,
                                err.to_string(),
                                true,
                            )
                            .with_backend(backend)
                            .with_counters(spawn_fail, wait_fail, exit_fail),
                        });
                        if let Ok(mut map) = stop_registry.lock() {
                            map.remove(&(stage, index, generation));
                        }
                        return;
                    }
                }
            }
            log::warn!(
                "movie all backends unavailable for {} (candidates exhausted, spawn_fail={}, wait_fail={}, exit_fail={})",
                path.display(),
                spawn_fail,
                wait_fail,
                exit_fail
            );
            let _ = tx.send(MoviePlaybackEvent::ObjectFailed {
                stage,
                index,
                generation,
                info: MovieFailureInfo::simple(
                    MovieFailureCategory::Exhausted,
                    "all_backends_exhausted",
                    false,
                )
                .with_counters(spawn_fail, wait_fail, exit_fail),
            });
            if let Ok(mut map) = stop_registry.lock() {
                map.remove(&(stage, index, generation));
            }
        });
    }

    fn handle_object_movie_play(
        &mut self,
        stage: StagePlane,
        index: i32,
        file_name: String,
        _duration_ms: i32,
        generation: u64,
    ) {
        let Some(path) = self.resolve_movie_asset(&file_name) else {
            let _ = self.movie_event_tx.send(MoviePlaybackEvent::ObjectFailed {
                stage,
                index,
                generation,
                info: MovieFailureInfo::simple(
                    MovieFailureCategory::Resource,
                    "asset_not_found",
                    true,
                ),
            });
            return;
        };
        self.mark_older_movie_processes_for_stop(stage, index, generation);
        let stop_flag = Arc::new(AtomicBool::new(false));
        if let Ok(mut map) = self.movie_stop_flags.lock() {
            map.insert((stage, index, generation), stop_flag.clone());
        }
        let _ = self.movie_event_tx.send(MoviePlaybackEvent::ObjectStarted {
            stage,
            index,
            generation,
        });
        Self::spawn_movie_player_watcher(
            self.movie_event_tx.clone(),
            self.movie_stop_flags.clone(),
            stage,
            index,
            generation,
            path,
            self.movie_backend_candidates(),
            stop_flag,
        );
    }

    fn quake_speed_up_limit(
        cur_time: f32,
        start_time: f32,
        start_value: f32,
        end_time: f32,
        end_value: f32,
    ) -> f32 {
        if (start_time - end_time).abs() < f32::EPSILON {
            return end_value;
        }
        let mut ct = cur_time;
        if start_time < end_time {
            ct = ct.clamp(start_time, end_time);
        } else {
            ct = ct.clamp(end_time, start_time);
        }
        let t = (ct - start_time) / (end_time - start_time);
        t * t * (end_value - start_value) + start_value
    }

    fn quake_speed_down_limit(
        cur_time: f32,
        start_time: f32,
        start_value: f32,
        end_time: f32,
        end_value: f32,
    ) -> f32 {
        if (start_time - end_time).abs() < f32::EPSILON {
            return end_value;
        }
        let mut ct = cur_time;
        if start_time < end_time {
            ct = ct.clamp(start_time, end_time);
        } else {
            ct = ct.clamp(end_time, start_time);
        }
        let t = (ct - end_time) / (end_time - start_time);
        -(t * t) * (end_value - start_value) + end_value
    }

    fn quake_linear_limit(
        cur_time: f32,
        start_time: f32,
        start_value: f32,
        end_time: f32,
        end_value: f32,
    ) -> f32 {
        if (start_time - end_time).abs() < f32::EPSILON {
            return end_value;
        }
        if cur_time <= start_time {
            return start_value;
        }
        if cur_time >= end_time {
            return end_value;
        }
        (end_value - start_value) * (cur_time - start_time) / (end_time - start_time) + start_value
    }

    fn quake_transform_for_order_at(
        req: &siglus::vm::VmQuakeRequest,
        elapsed: f32,
        order: i32,
    ) -> (f32, f32, f32, f32, f32) {
        // C++对照: cmd_effect.cpp 的 begin/end_order 裁剪分支。
        // 当 order 不在区间内时直接返回恒等变换，后续所有位移/缩放分支都不参与。
        if order < req.begin_order || order > req.end_order {
            return (0.0, 0.0, 1.0, 0.0, 0.0);
        }

        let total = req.time_ms.max(1) as f32;
        let total_quake_time = total * (req.cnt + req.end_cnt).max(0) as f32;
        if elapsed >= total_quake_time {
            return (0.0, 0.0, 1.0, 0.0, 0.0);
        }

        let quarter = (total / 4.0).max(1.0);
        let jump_cur_time = elapsed % total;
        let mut pos_x = 0.0f32;
        let mut pos_y = 0.0f32;
        let mut scale = 1.0f32;
        let mut center_x = 0.0f32;
        let mut center_y = 0.0f32;

        match req.kind {
            siglus::vm::VmQuakeKind::Vec => {
                // C++对照: vec 分支按 1/4 周期做加速→减速→反向加速→反向减速。
                // 保持与原分段时序一致，避免在 wait/check 轮询时出现相位偏移。
                let rad = (req.vec as f32).to_radians();
                let x_sign = rad.cos();
                let y_sign = rad.sin();
                let power = req.power as f32;
                if jump_cur_time < quarter {
                    pos_x =
                        Self::quake_speed_up_limit(jump_cur_time, 0.0, 0.0, quarter, power / 2.0)
                            * x_sign;
                    pos_y =
                        Self::quake_speed_up_limit(jump_cur_time, 0.0, 0.0, quarter, power / 2.0)
                            * y_sign;
                } else if jump_cur_time < quarter * 2.0 {
                    let t = jump_cur_time - quarter;
                    pos_x =
                        Self::quake_speed_down_limit(t, 0.0, power / 2.0, quarter, power) * x_sign;
                    pos_y =
                        Self::quake_speed_down_limit(t, 0.0, power / 2.0, quarter, power) * y_sign;
                } else if jump_cur_time < quarter * 3.0 {
                    let t = jump_cur_time - quarter * 2.0;
                    pos_x =
                        Self::quake_speed_up_limit(t, 0.0, power, quarter, power / 2.0) * x_sign;
                    pos_y =
                        Self::quake_speed_up_limit(t, 0.0, power, quarter, power / 2.0) * y_sign;
                } else {
                    let t = jump_cur_time - quarter * 3.0;
                    pos_x =
                        Self::quake_speed_down_limit(t, 0.0, power / 2.0, quarter, 0.0) * x_sign;
                    pos_y =
                        Self::quake_speed_down_limit(t, 0.0, power / 2.0, quarter, 0.0) * y_sign;
                }
            }
            siglus::vm::VmQuakeKind::Dir => {
                // C++对照: dir 分支先离散化 8 方向，再复用同一套 1/4 周期速度曲线。
                let (x_sign, y_sign) = match req.vec {
                    0 => (0.0, -1.0),
                    1 => (1.0, -1.0),
                    2 => (1.0, 0.0),
                    3 => (1.0, 1.0),
                    4 => (0.0, 1.0),
                    5 => (-1.0, 1.0),
                    6 => (-1.0, 0.0),
                    7 => (-1.0, -1.0),
                    _ => (0.0, 0.0),
                };
                let power = req.power as f32;
                if jump_cur_time < quarter {
                    pos_x =
                        Self::quake_speed_down_limit(jump_cur_time, 0.0, 0.0, quarter, power / 2.0)
                            * x_sign;
                    pos_y =
                        Self::quake_speed_down_limit(jump_cur_time, 0.0, 0.0, quarter, power / 2.0)
                            * y_sign;
                } else if jump_cur_time < quarter * 2.0 {
                    let t = jump_cur_time - quarter;
                    pos_x = Self::quake_speed_up_limit(t, 0.0, power / 2.0, quarter, 0.0) * x_sign;
                    pos_y = Self::quake_speed_up_limit(t, 0.0, power / 2.0, quarter, 0.0) * y_sign;
                } else if jump_cur_time < quarter * 3.0 {
                    let t = jump_cur_time - quarter * 2.0;
                    pos_x =
                        Self::quake_speed_down_limit(t, 0.0, 0.0, quarter, -power / 2.0) * x_sign;
                    pos_y =
                        Self::quake_speed_down_limit(t, 0.0, 0.0, quarter, -power / 2.0) * y_sign;
                } else {
                    let t = jump_cur_time - quarter * 3.0;
                    pos_x = Self::quake_speed_up_limit(t, 0.0, -power / 2.0, quarter, 0.0) * x_sign;
                    pos_y = Self::quake_speed_up_limit(t, 0.0, -power / 2.0, quarter, 0.0) * y_sign;
                }
            }
            siglus::vm::VmQuakeKind::Zoom => {
                // C++对照: zoom 分支只改 scale + center，不直接改 pos。
                // center_x/center_y 在脚本可见，必须与 C++ 的参数回放一致。
                let power = req.power.clamp(0, 255) as f32;
                let scale_peak = 256.0 / (256.0 - power);
                let half_scale = (scale_peak - 1.0) / 2.0 + 1.0;
                center_x = req.center_x as f32;
                center_y = req.center_y as f32;
                if jump_cur_time < quarter {
                    scale =
                        Self::quake_speed_up_limit(jump_cur_time, 0.0, 1.0, quarter, half_scale);
                } else if jump_cur_time < quarter * 2.0 {
                    let t = jump_cur_time - quarter;
                    scale = Self::quake_speed_down_limit(t, 0.0, half_scale, quarter, scale_peak);
                } else if jump_cur_time < quarter * 3.0 {
                    let t = jump_cur_time - quarter * 2.0;
                    scale = Self::quake_speed_up_limit(t, 0.0, scale_peak, quarter, half_scale);
                } else {
                    let t = jump_cur_time - quarter * 3.0;
                    scale = Self::quake_speed_down_limit(t, 0.0, half_scale, quarter, 1.0);
                }
            }
        }

        // C++对照: end_cnt 衰减段线性收敛到单位变换；该段对 vec/dir/zoom 共用。
        if elapsed >= total * req.cnt.max(0) as f32 {
            let fade = Self::quake_linear_limit(
                elapsed,
                total * req.cnt.max(0) as f32,
                1.0,
                total_quake_time,
                0.0,
            );
            pos_x *= fade;
            pos_y *= fade;
            scale = (scale - 1.0) * fade + 1.0;
        }

        (pos_x, pos_y, scale, center_x, center_y)
    }

    fn quake_transform_for_order(&self, order: i32) -> (f32, f32, f32, f32, f32) {
        let (Some(req), Some(started_at)) = (self.quake_request, self.quake_started_at) else {
            return (0.0, 0.0, 1.0, 0.0, 0.0);
        };
        Self::quake_transform_for_order_at(&req, started_at.elapsed().as_millis() as f32, order)
    }
}
