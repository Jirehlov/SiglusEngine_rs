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
| `cmd_sound.cpp` / `cmd_koe.cpp` | `vm/command_sound.rs` + `vm/command_tail.rs` + Host API | 🟡 部分实现 | BGM/PCM/PCMCH/SE/MOV/KOE 路由已建立；PCMCH named-arg 已完整对齐 C++；Host 回调为 stub |
| `cmd_script.cpp` | `vm/command_try.rs` + `vm/core.rs` | 🟡 部分实现 | CALL/JUMP/FARCALL 主流程可跑，脚本子系统细节仍缺 |
| `cmd_call.cpp` | `vm/command_try.rs`（call.L/call.K） | 🟡 部分实现 | 列表辅助实现，但 call 子命令行为未逐项对齐 |
| `cmd_effect.cpp` | `vm/command_effect.rs` + `vm/command_tail.rs` + Host API | 🟡 部分实现 | screen/effect/quake 路由已建立；属性 get/set 对齐 C++；int_event 子路由仅 accept |
| `cmd_world.cpp` | `vm/command_world.rs` + Host API | 🟡 部分实现 | world_list/world 路由已建立（camera/属性 get/set/calc）；需 stage 子分发器接入 |
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
- 新增 `src/vm/command_effect.rs`，完整路由 `global.screen.*` 命令对齐 C++ `cmd_effect.cpp`。包含 screen 顶层属性分发（x/y/z/mono/reverse/bright/dark/color_*/color_add_*）、effect_list 管理（array/resize/get_size）、per-effect 属性获取/设定（含 init, wipe_copy/erase, begin/end order/layer）、quake_list/quake 完整路由（start/end/wait/check 全变体）。
- 新增 `src/vm/command_world.rs`，完整路由 world 命令对齐 C++ `cmd_world.cpp`。包含 world_list 管理（create/destroy/array）、per-world camera_eye/pint/up xyz 属性获取/设定、calc_camera_eye/pint（三角计算）、set_camera helpers、camera_view_angle/mono/order/layer/wipe_copy/wipe_erase、事件分发器、set_camera_eve_xz_rotate。暂标 `#[allow(dead_code)]`，待 stage 子分发器接入。
- `command_sound.rs::try_command_pcmch()` 重写：完整解析 C++ 11 个 named-arg（loop_flag, wait_flag, fade_in_time, volume_type, bgm_fade_target/fade2_target, chara_no, pcm_name, koe_no, se_no, bgm_name, bgm_fade_source），提取频道索引，PLAY/READY/STOP/PAUSE/RESUME/SET_VOLUME 均调用专用 Host 回调。
- `api.rs` Host trait 新增 17 个回调：screen/effect/quake 系列（on_screen_property, on_effect_property, on_effect_init, on_quake_start, on_quake_end）、world 系列（on_world_property, on_world_create, on_world_destroy, on_world_init, on_world_set_camera, on_world_calc_camera）、PCMCH 系列（on_pcmch_play, on_pcmch_stop, on_pcmch_pause, on_pcmch_resume, on_pcmch_set_volume），均有默认 no-op 实现。
- `command_tail.rs` 新增 `ELM_GLOBAL_SCREEN` 路由臂，调用 `try_command_screen()`；`global.rs` 中从 `is_host_passthrough_root` 移除 `ELM_GLOBAL_SCREEN`。
- `mod.rs` 注册 `command_effect` 和 `command_world` 模块。
- `cargo check` 通过，无 error 无 warning。

### 未完成 / 阻塞
- Host 侧所有新回调均为 no-op stub，无真实渲染/音频/动画效果。
- int_event 子路由（`tnm_command_proc_int_event`）在 effect/screen/world 的 `*_EVE` 命令中仅 accept，未完整复刻 C++ 事件控制逻辑。
- World 路由需通过 stage 子分发器接入（C++ 中 world_list 挂在 stage 下），当前标记 `dead_code`。
- BGM PLAY/READY 的 named-arg 覆盖逻辑（6 个 named-arg: regist_name, loop_flag, wait_flag, start_pos, fade_in_time, fade_out_time）尚未对齐（当前仅用位置参数）。

### 下一轮首要任务（可直接执行）
1. BGM PLAY/READY named-arg 覆盖对齐 C++ `tnm_command_proc_bgm` 的 `for (arg = al_end; arg < named_al_end; ...)` 循环。
2. Stage 子分发器：从 host passthrough 中拆出 ELM_GLOBAL_STAGE/FRONT/BACK 至 VM 侧，接入 world_list 路由。
3. int_event 通用子路由实现（start/end/wait/check/set_target 等），供 effect/screen/world event 属性使用。
4. 接入真实音频后端（rodio / cpal），让 BGM/SE/PCM/PCMCH Host 回调实际播放音频。

