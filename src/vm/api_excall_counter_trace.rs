use std::sync::atomic::{AtomicBool, Ordering};

static EXCALL_COUNTER_HINT_EMITTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmExcallCounterPhase {
    Tick,
    Start,
    Stop,
    Reset,
    Wait,
    WaitKey,
    CheckValue,
    CheckActive,
    Reclaim,
}

impl VmExcallCounterPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tick => "tick",
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Reset => "reset",
            Self::Wait => "wait",
            Self::WaitKey => "wait_key",
            Self::CheckValue => "check_value",
            Self::CheckActive => "check_active",
            Self::Reclaim => "reclaim",
        }
    }
}

pub fn format_excall_counter_trace(
    slot: usize,
    phase: VmExcallCounterPhase,
    value: i32,
    active: bool,
) -> String {
    format!(
        "vm.excall.counter slot={} phase={} value={} active={}",
        slot,
        phase.as_str(),
        value,
        active
    )
}

/// Host-side default aggregation template for `vm.excall.counter` logs.
///
/// Recommended minimal fields:
/// - `slot`   : counter identity (stable within process lifetime)
/// - `phase`  : lifecycle/observation point (`tick/start/stop/...`)
/// - `value`  : counter value at this phase
/// - `active` : whether counter is active after this phase
///
/// Recommended window stats (e.g. 1s/5s bucket):
/// - events_total
/// - by_phase_count
/// - active_ratio (active=true / events_total)
/// - value_min/value_max/value_last
pub fn format_excall_counter_aggregate_hint(window_label: &str) -> String {
    format!(
        "vm.excall.counter.aggregate window={} fields=[slot,phase,value,active] stats=[events_total,by_phase_count,active_ratio,value_min,value_max,value_last]",
        window_label
    )
}

pub fn take_excall_counter_aggregate_hint(window_label: &str) -> Option<String> {
    let enabled = std::env::var("SIGLUS_EXCALL_COUNTER_TRACE_HINT")
        .map(|v| v != "0")
        .unwrap_or(false);
    if !enabled {
        return None;
    }
    if EXCALL_COUNTER_HINT_EMITTED.swap(true, Ordering::AcqRel) {
        return None;
    }
    Some(format_excall_counter_aggregate_hint(window_label))
}
