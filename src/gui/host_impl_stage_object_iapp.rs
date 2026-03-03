/// STUB(C++: `cmd_object.cpp` / `elm_object.cpp`)
/// - C++ 对照：`cmd_object.cpp` 的 object-int 查询通道（`cmd_object*` -> host `on_object_query`）在未知 sub/selector
///   时返回 0，且不产生额外副作用；Rust 这里复用同一兜底策略。
/// - 当前缺口：原版未公开 selector 符号表；因此此处采用“静态 RIIR 可证明路径”先复刻 C++ 查询车道，
///   不依赖任何特定样本才能定义返回域。
/// - 预期行为：selector 0~8 读取 movie failure snapshot；selector 9~14 读取 create_movie/create_emote named 参数与 check_movie 失败分类快照；selector<0 / selector>=15 / 参数超长 / 无对象 / 无快照 全部返回 0。
/// - 最小验证方向：
///   1) selector 0~8 逐一验证返回域与错误码约束；
///   2) selector 9~14 验证 named 参数与失败分类快照（movie auto_init/real_time/ready_only、emote rep_pos、check_movie failed code）；
///   3) WAIT/CHECK 轮询期间不应修改 failure snapshot（仅读路径）。
#[derive(Clone, Copy)]
enum IappMovieQuerySelector {
    /// selector=0, 返回状态码：0=ok，负值=失败（当前宿主错误域）。
    FailureStatusCode,
    /// selector=1, 返回打包计数：低16位=spawn_fail, 高16位=wait_fail。
    FailureCountersPacked,
    /// selector=2, 返回失败分类枚举（backend/io/unsupported）。
    FailureCategoryCode,
    /// selector=3, 返回是否不可恢复（0/1）。
    FailureUnrecoverableFlag,
    /// selector=4, 返回后端标识 hash。
    FailureBackendHash,
    /// selector=5, 返回细节信息 hash。
    FailureDetailHash,
    /// selector=6, 返回 spawn 失败次数（>=0）。
    FailureSpawnFailCount,
    /// selector=7, 返回 wait 失败次数（>=0）。
    FailureWaitFailCount,
    /// selector=8, 返回 exit 失败次数（>=0）。
    FailureExitFailCount,
    /// selector=9, 返回 create_movie named[0] auto_init(0/1)。
    MovieAutoInitFlag,
    /// selector=10, 返回 create_movie named[1] real_time(0/1)。
    MovieRealTimeFlag,
    /// selector=11, 返回 create_movie named[2] ready_only(0/1)。
    MovieReadyOnlyFlag,
    /// selector=12, 返回 create_emote named[0] rep_x。
    EmoteRepX,
    /// selector=13, 返回 create_emote named[1] rep_y。
    EmoteRepY,
    /// selector=14, 返回 check_movie 失败分类编码（-11..-15）。
    CheckMovieFailedCode,
}

impl IappMovieQuerySelector {
    fn from_i32(v: i32) -> Option<Self> {
        Some(match v {
            0 => Self::FailureStatusCode,
            1 => Self::FailureCountersPacked,
            2 => Self::FailureCategoryCode,
            3 => Self::FailureUnrecoverableFlag,
            4 => Self::FailureBackendHash,
            5 => Self::FailureDetailHash,
            6 => Self::FailureSpawnFailCount,
            7 => Self::FailureWaitFailCount,
            8 => Self::FailureExitFailCount,
            9 => Self::MovieAutoInitFlag,
            10 => Self::MovieRealTimeFlag,
            11 => Self::MovieReadyOnlyFlag,
            12 => Self::EmoteRepX,
            13 => Self::EmoteRepY,
            14 => Self::CheckMovieFailedCode,
            _ => return None,
        })
    }

    fn cxx_branch_hint(self) -> &'static str {
        match self {
            // C++ 侧未公开 selector 名称；这里记录 cmd_object 查询分支对应的 host 返回位槽，
            // 便于按 `cmd_object.cpp: tnm_command_proc_object -> switch(ELM_OBJECT_*) -> int push` 对照。
            Self::FailureStatusCode => "slot0_status",
            Self::FailureCountersPacked => "slot1_counters",
            Self::FailureCategoryCode => "slot2_category",
            Self::FailureUnrecoverableFlag => "slot3_unrecoverable",
            Self::FailureBackendHash => "slot4_backend_hash",
            Self::FailureDetailHash => "slot5_detail_hash",
            Self::FailureSpawnFailCount => "slot6_spawn_fail",
            Self::FailureWaitFailCount => "slot7_wait_fail",
            Self::FailureExitFailCount => "slot8_exit_fail",
            Self::MovieAutoInitFlag => "slot9_movie_auto_init",
            Self::MovieRealTimeFlag => "slot10_movie_real_time",
            Self::MovieReadyOnlyFlag => "slot11_movie_ready_only",
            Self::EmoteRepX => "slot12_emote_rep_x",
            Self::EmoteRepY => "slot13_emote_rep_y",
            Self::CheckMovieFailedCode => "slot14_check_movie_failed_code",
        }
    }

    fn domain_ok(self, value: i32) -> bool {
        match self {
            Self::FailureStatusCode => value != 0,
            Self::FailureCountersPacked => value >= 0,
            Self::FailureCategoryCode => (1..=5).contains(&value),
            Self::FailureUnrecoverableFlag => value == 0 || value == 1,
            Self::FailureBackendHash | Self::FailureDetailHash => value >= 0,
            Self::FailureSpawnFailCount
            | Self::FailureWaitFailCount
            | Self::FailureExitFailCount => value >= 0,
            Self::MovieAutoInitFlag | Self::MovieRealTimeFlag | Self::MovieReadyOnlyFlag => {
                value == 0 || value == 1
            }
            Self::EmoteRepX | Self::EmoteRepY => true,
            Self::CheckMovieFailedCode => (-15..=-11).contains(&value) || value == -1,
        }
    }
}

