fn on_trace(&mut self, msg: &str) {
    static TRACE_HINT_ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if std::env::var("SIGLUS_EXCALL_COUNTER_TRACE_HINT")
        .map(|v| v != "0")
        .unwrap_or(false)
        && !TRACE_HINT_ONCE.swap(true, std::sync::atomic::Ordering::Relaxed)
    {
        debug!("{}", siglus::vm::format_excall_counter_aggregate_hint("5s"));
    }

    if !msg.starts_with("vm: counter_observe ") {
        return;
    }

    #[derive(Default)]
    struct CounterObserveAggregateState {
        samples: Vec<(std::time::Instant, i32, String, bool)>,
        total_seen: u64,
    }

    static COUNTER_WAIT_REPORT: std::sync::OnceLock<
        std::sync::Mutex<CounterObserveAggregateState>,
    > = std::sync::OnceLock::new();

    let window_sec = std::env::var("SIGLUS_COUNTER_OBSERVE_WINDOW_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(5);
    let report_every = std::env::var("SIGLUS_COUNTER_OBSERVE_REPORT_EVERY")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(64);

    let mut owner = 0;
    let mut phase = String::from("non_syscom");
    let mut active = false;
    for part in msg.split_whitespace() {
        if let Some(v) = part.strip_prefix("owner=") {
            owner = v.parse::<i32>().unwrap_or(0);
        } else if let Some(v) = part.strip_prefix("phase=") {
            phase = v.to_string();
        } else if let Some(v) = part.strip_prefix("active=") {
            active = v == "1" || v.eq_ignore_ascii_case("true");
        }
    }

    let map = COUNTER_WAIT_REPORT.get_or_init(|| {
        std::sync::Mutex::new(CounterObserveAggregateState::default())
    });
    if let Ok(mut guard) = map.lock() {
        let now = std::time::Instant::now();
        guard.total_seen += 1;
        guard.samples.push((now, owner, phase.clone(), active));
        let cutoff = now
            .checked_sub(std::time::Duration::from_secs(window_sec))
            .unwrap_or(now);
        guard.samples.retain(|(t, _, _, _)| *t >= cutoff);

        if guard.total_seen == 1 || guard.total_seen % report_every == 0 {
            let mut by_owner_phase: std::collections::BTreeMap<(i32, String), u64> =
                std::collections::BTreeMap::new();
            let mut active_cnt = 0u64;
            for (_, s_owner, s_phase, s_active) in &guard.samples {
                *by_owner_phase
                    .entry((*s_owner, s_phase.clone()))
                    .or_insert(0) += 1;
                if *s_active {
                    active_cnt += 1;
                }
            }
            let total = guard.samples.len() as u64;
            let active_ratio = if total == 0 {
                0.0
            } else {
                active_cnt as f64 / total as f64
            };
            let mut top_clusters = Vec::new();
            for ((c_owner, c_phase), c_count) in by_owner_phase.iter().take(4) {
                let ratio = if total == 0 {
                    0.0
                } else {
                    *c_count as f64 / total as f64
                };
                top_clusters.push(format!(
                    "owner={} phase={} count={} ratio={:.2}",
                    c_owner, c_phase, c_count, ratio
                ));
            }
            debug!(
                "vm.counter_observe.report window={}s samples={} active_ratio={:.2} clusters=[{}]",
                window_sec,
                total,
                active_ratio,
                top_clusters.join("; ")
            );
            if let Ok(path) = std::env::var("SIGLUS_COUNTER_OBSERVE_JSON_PATH") {
                if !path.is_empty() {
                    let mut cluster_rows = Vec::new();
                    for ((c_owner, c_phase), c_count) in &by_owner_phase {
                        let ratio = if total == 0 {
                            0.0
                        } else {
                            *c_count as f64 / total as f64
                        };
                        cluster_rows.push(serde_json::json!({
                            "owner": c_owner,
                            "phase": c_phase,
                            "count": c_count,
                            "ratio": ratio,
                        }));
                    }
                    let row = serde_json::json!({
                        "window_sec": window_sec,
                        "samples": total,
                        "active_ratio": active_ratio,
                        "clusters": cluster_rows,
                    });
                    if let Ok(mut f) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(path)
                    {
                        let _ = std::io::Write::write_all(&mut f, row.to_string().as_bytes());
                        let _ = std::io::Write::write_all(&mut f, b"\n");
                    }
                }
            }
        }
    }
}

fn on_int_event_wait_status_with_proc(
    &mut self,
    owner_id: i32,
    key_skip: bool,
    status: i32,
    proc_depth: i32,
    proc_top: i32,
) {
    let trace_on = std::env::var("SIGLUS_WAIT_TRACE")
        .map(|v| v != "0")
        .unwrap_or(true);
    if !trace_on {
        return;
    }
    let phase = siglus::vm::classify_syscom_wait_owner(owner_id);
    if phase == siglus::vm::VmSyscomWaitPhase::NonSyscom {
        return;
    }
    let line =
        siglus::vm::format_syscom_wait_trace(owner_id, key_skip, status, proc_depth, proc_top);
    debug!("{}", line);
}
