/// C++ wipe range classification:
/// - `Normal`    = script-level WIPE / MASK_WIPE commands
/// - `SystemIn`  = TNM_WIPE_RANGE_SYSTEM_IN  (fade *from* black after load)
/// - `SystemOut` = TNM_WIPE_RANGE_SYSTEM_OUT (fade *to* black before load)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WipeDirection {
    Normal,
    SystemIn,
    SystemOut,
}
