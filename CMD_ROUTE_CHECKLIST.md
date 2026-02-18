# C++ `cmd_*` 路由对齐清单（首轮）

> 目的：把“按 C++ `cmd_*` 路由逐 form 建已实现/缺失/stub 清单”落地成可持续维护的基线文档。
>
> 对照源：
> - C++：`siglus_engine_source/cmd_*.cpp`
> - Rust：`siglus_rust/src/vm/*.rs` + `siglus_rust/src/gui/*.rs`

## 0) 交接记录维护规则（新增）

- 为避免交接段落持续膨胀，`## 4) 本轮交接（Iteration Handoff）` 必须按“单轮覆盖”维护：
  - 每轮结束时，先清空上一轮的 `### 本轮完成` 内容，再写入本轮真实增量；
  - `### 未完成 / 阻塞` 与 `### 下一轮首要任务` 允许延续，但必须按本轮状态重写，不做机械累积；
  - 禁止把多轮历史流水账长期保留在同一交接块中。
- 如需保留长期脉络，应沉淀到上方稳定分区（路由总览/优先级/对照表），而不是堆叠在“本轮交接”。

## 1) 路由总览（按 C++ 文件）

| C++ 路由文件 | Rust 当前入口 | 状态 | 备注 |
|---|---|---|---|
| `cmd_global.cpp` | `vm/command_head.rs` + `vm/command_tail.rs` | 🟡 部分实现 | 全局流控/文本/wipe 有实现；大量命令仍 passthrough 或 stub |
| `cmd_wipe.cpp` | `vm/command_tail.rs` + `gui/host_impl.rs` | 🟡 部分实现 | 参数/等待时序有实现；渲染特效行为仍未对齐 `eng_disp_wipe.cpp` |
| `cmd_syscom.cpp` | `vm/command_syscom.rs` | 🟡 部分实现 | 存档与部分配置读写已实现；大量项仍默认/占位 |
| `cmd_stage.cpp` | `gui/stage.rs` + `gui/host_stage_*.rs` | 🟡 部分实现 | GUI host 侧解析与部分对象状态生效；VM 核心未完全复刻 C++ 各分支 |
| `cmd_object.cpp` | `gui/host_stage_object_cmd.rs` + `gui/host_stage_object_assign.rs` | 🟡 部分实现 | 对象常见变换/可见性有实现；高级对象命令大量缺失 |
| `cmd_mwnd.cpp` | `gui/host_impl.rs` + global/mwnd 常量表 | 🟡 部分实现 | 文本窗口开关/文本流存在；mwnd 细分行为未全覆盖 |
| `cmd_input.cpp` | `vm/command_tail.rs`（root passthrough）+ host | 🟡 部分实现 | 当前以 host 透传为主，缺 C++ 级输入状态机复刻 |
| `cmd_sound.cpp` / `cmd_koe.cpp` | `vm/command_tail.rs`（sound passthrough + 少量默认返回） | 🟡 部分实现 | 主体仍透传/默认值，需按 C++ 细化 |
| `cmd_script.cpp` | `vm/command_try.rs` + `vm/core.rs` | 🟡 部分实现 | CALL/JUMP/FARCALL 主流程可跑，脚本子系统细节仍缺 |
| `cmd_call.cpp` | `vm/command_try.rs`（call.L/call.K） | 🟡 部分实现 | 列表辅助实现，但 call 子命令行为未逐项对齐 |
| `cmd_effect.cpp` | 暂无专门行为实现 | ❌ 缺失 | 仅常量表已补齐 |
| `cmd_world.cpp` | 暂无专门行为实现 | ❌ 缺失 | 仅常量表已补齐 |
| `cmd_steam.cpp` | root passthrough | ❌ 缺失 | 仅入口透传 |
| `cmd_others.cpp` | 分散于 VM/host | 🟡 部分实现 | 需按 C++ 逐项归档到明确模块 |

## 2) 高频路径细化（优先级）

### P0（高优先，直接影响脚本行为）

1. **Global + Wipe**（`cmd_global.cpp` / `cmd_wipe.cpp`）
   - 已有：`wipe/wait_wipe/check_wipe` 时序、部分返回值策略。
   - 缺口：显示/捕获/消息窗口细粒度分支大量仍走 passthrough/stub。

2. **Syscom**（`cmd_syscom.cpp`）
   - 已有：save/quick/inner save 及部分 query。
   - 缺口：大量配置菜单项仅“接受命令但无完整语义”。

3. **Stage/Object**（`cmd_stage.cpp` / `cmd_object.cpp`）
   - 已有：host 侧可解析 stage/object 路径并应用部分属性/命令。
   - 缺口：对象高级命令、effect/world/group/quake 及错误路径未完整复刻。

### P1（中优先）

4. **Sound/Koe/Bgm/Pcm/Se/Mov**（`cmd_sound.cpp` / `cmd_koe.cpp`）
   - 已有：root 透传 + 少量默认返回（如检查类）。
   - 缺口：播放状态、等待语义、错误分支与 C++ 不一致。

5. **Input/Keyboard/Joy**（`cmd_input.cpp`）
   - 已有：透传路径。
   - 缺口：输入状态流与 keylist 相关行为未对齐。

### P2（后续）

6. **Effect / World / Steam / Others**
   - 现状：常量表有，行为基本未落实。

## 3) 代码内证据点（Rust）

- `try_command` 总入口：`src/vm/command_try.rs`
- Global head/tail：`src/vm/command_head.rs` / `src/vm/command_tail.rs`
- Syscom：`src/vm/command_syscom.rs`
- Stage/Object host 解析：`src/gui/stage.rs` 与 `src/gui/host_stage_*`
- Wipe host 协同：`src/gui/host_impl.rs`

## 4) 本轮交接（Iteration Handoff）

### 本轮完成
- 继续对齐 `flow_proc.cpp` 与 `eng_frame.cpp` 的 load 系列全局标志时序，本轮把 C++ 的 `system_wipe_flag / do_frame_action_flag / do_load_after_call_flag` 显式落地到 Rust VM 可观测状态：
  1) 在 VM Host API 新增 `VmLoadFlowState` 与 `on_syscom_load_flow_state` 回调，建立 VM→Host 的统一状态上报面；
  2) 在 `run_syscom_proc_queue` 的 `GAME_END_WIPE / GAME_START_WIPE / RETURN_TO_MENU / RETURN_TO_SEL / END_LOAD` 分支中补齐标志位写入与即时上报；
  3) `END_LOAD` 与 `RETURN_TO_SEL` 现在会在 proc 阶段设置 `do_frame_action/do_load_after_call=true`，与 C++ `tnm_end_load_proc/tnm_return_to_sel_proc` 的关键副作用对齐。
- 将上述三类标志纳入本地状态与 END_SAVE runtime 快照链路，避免跨命令/跨进程恢复时被丢失：
  - `VmLocalState` 的 snapshot/apply 已覆盖三标志；
  - `SESV3` 新格式新增三标志字段，`decode` 兼容旧 `SESV2`（自动回落为 0）。
- GUI host 实现新增 load-flow 状态日志，后续可以直接对照 `eng_frame.cpp::frame_action_proc` 触发前置条件。
- 完成 `cargo check` 全量编译校验：当前无 error 且无 warning。

### 未完成 / 阻塞
- 目前仅完成“标志位状态对齐 + 可观测性”；`eng_frame.cpp::frame_action_proc` 对这三标志的**消费语义**尚未在 Rust 侧形成等价调度（尤其 `load_after_call_scene` 的 farcall 时机）。
- `ELM_SYSCOM_LOAD / QUICK_LOAD / INNER_LOAD` 仍走 Rust 直接恢复路径，尚未完全改造成 C++ 的 proc 驱动模型（含 fade/wipe 与 frame 驱动耦合）。
- `SESV2` 历史文件会按默认 0 补齐新标志，虽然可读，但不具备新增字段的原始语义信息。

### 下一轮首要任务（可直接执行）
1. 对照 `eng_frame.cpp::frame_action_proc`，在 Rust VM/Host 帧循环补齐 `do_load_after_call_flag` 的一次性消费与 `load_after_call_scene` 调度。
2. 将 `ELM_SYSCOM_LOAD / QUICK_LOAD / INNER_LOAD` 迁移到统一 proc 队列路径，复用本轮新增的三标志写入与上报语义。
3. 继续对照 `eng_disp_wipe.cpp`，拆分 `GAME_START_WIPE` 与 `GAME_END_WIPE` 的视觉/范围差异并落地到 host 渲染侧。
