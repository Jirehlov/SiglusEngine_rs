# C++ `cmd_*` 路由对齐清单（重建版）

> 目标：把 Rust VM/Host 的命令路由与 C++ Siglus `cmd_*` 行为逐项对齐，并保持可持续迭代。
>
> 对照源：
> - C++ 行为源：`siglus_engine_source/cmd_*.cpp`
> - Rust 实现：`siglus_rust/src/vm/*.rs` + `siglus_rust/src/gui/*.rs`
> - 格式辅助：`siglus_scene_script_utility/`

---

## 0) 交接记录维护规则（强制）

- `## 6) 本轮交接（Iteration Handoff）` 必须采用“单轮覆盖”，禁止历史流水账堆叠。
- 每轮结束时：
  1. 重写 `### 本轮完成`，仅保留本轮真实增量；
  2. 重写 `### 未完成 / 阻塞`，反映当前状态；
  3. 重写 `### 下一轮首要任务`，要求可直接开工。
- 长期信息只放在稳定分区（总览、差异矩阵、优先级），不放在交接区。

---

## 1) 路由总览（按 C++ 文件）

| C++ 文件 | Rust 路径 | 状态 | 说明 |
|---|---|---|---|
| `cmd_global.cpp` | `vm/command_head.rs` + `vm/command_tail.rs` | 🟡 部分实现 | 基础文本流控/wipe 有实现，仍有 passthrough |
| `cmd_wipe.cpp` | `vm/command_tail.rs` + `gui/host_impl.rs` | 🟡 部分实现 | 等待时序可跑，渲染细节未对齐 `eng_disp_wipe.cpp` |
| `cmd_syscom.cpp` | `vm/command_syscom.rs` | 🟡 部分实现 | save/load 主路径已落地，配置项仍大量占位 |
| `cmd_stage.cpp` | `vm/command_stage.rs` + `gui/stage.rs` | 🟡 部分实现 | stage/group 有基础骨架，边界与错误分支不足 |
| `cmd_object.cpp` | `vm/command_object.rs` + `gui/host_stage_object_*.rs` | 🟡 部分实现 | 常见对象属性可用，高级命令缺失较多 |
| `cmd_mwnd.cpp` | `vm/command_mwnd.rs` + `gui/host_impl.rs` | 🟡 部分实现 | 文本窗口主开关可用，细粒度行为未齐 |
| `cmd_input.cpp` | `vm/command_input.rs` + `gui/input_bridge.rs` | 🟡 部分实现 | 坐标双向映射 + KEY_WAIT proc 已落地，flick 角度截断语义已对齐，方向分区仍待对齐 |
| `cmd_sound.cpp` / `cmd_koe.cpp` | `vm/command_sound.rs` + Host API | 🟡 部分实现 | 路由齐全但播放态/等待态语义未全对齐 |
| `cmd_script.cpp` | `vm/command_script.rs` + `vm/core.rs` | 🟡 部分实现 | CALL/JUMP/FARCALL 基础可跑，脚本时序细节仍缺 |
| `cmd_call.cpp` | `vm/command_call.rs` | 🟡 部分实现 | call flag/列表辅助可用，子命令行为不完整 |
| `cmd_effect.cpp` | `vm/command_effect.rs` + Host API | 🟡 部分实现 | effect/quake 路由有，行为一致性待补 |
| `cmd_world.cpp` | `vm/command_world.rs` + Host API | 🟡 部分实现 | world 属性与 camera 部分可用，子分发未齐 |
| `cmd_steam.cpp` | root passthrough | ❌ 缺失 | 尚未进入可验证实现阶段 |
| `cmd_others.cpp` | `vm/command_others.rs` 等 | 🟡 部分实现 | 多命令散落实现，需收敛到明确语义表 |

---

## 2) 差异矩阵（以“脚本可见行为”衡量）

### P0（会造成脚本行为错误/崩溃）

1. **`cmd_stage.cpp` / `cmd_object.cpp`：错误路径与边界不足**
   - 现状：常规路径可跑，非法参数/越界/状态机边界不完整。
   - 风险：脚本分支结果与 C++ 偏离，可能触发“能跑但错语义”。

2. **`cmd_syscom.cpp`：配置项语义不全**
   - 现状：大量项可接收但未完整生效。
   - 风险：系统菜单/返回流程行为偏差。

### P1（高频但短期可降级）

3. **`cmd_sound.cpp` / `cmd_koe.cpp`：等待与状态查询语义**
   - 现状：播放控制入口齐，状态一致性不足。
   - 风险：脚本等待分支条件偏差。

4. **`cmd_effect.cpp` / `cmd_world.cpp`：Host 回调行为细节**
   - 现状：路由存在，底层效果实现有 stub。
   - 风险：视觉/状态返回与 C++ 不一致。

### P2（后续推进）

5. **`cmd_steam.cpp` / `cmd_others.cpp` 未系统化对照**
   - 现状：尚未形成“逐命令语义表 + 验证路径”。

---

## 3) `cmd_input.cpp` 专项状态（当前重点）

### 已完成（本轮之前累积）

- `mouse.set_pos` 与 `mouse.pos` 已统一到 stage 逻辑坐标系，采用正反双向映射，避免 letterbox / 非 1:1 缩放偏差。
- flick `mm` 已改为实时 DPI 计算，去除固定 96DPI 近似。
- KEY_WAIT 判断与 consume 已拆为 Host 三段式接口（便于 proc 化接入）。

### 未完成（必须继续）

- KEY_WAIT 已进入 VM 可恢复 proc 调度，并对齐到 C++ 的 decide down_up stock 判定/消费路径（不再误用“任意按键 down”）。
- flick 方向分区（eng_frame 的 1..8 映射）已按 C++ 角区条件落地到 VM 侧路由匹配。
- 多显示器与 DPI 切换边界行为仍缺少系统性验证记录。

---

## 4) 实施策略（未来 3 轮建议）

### 第 1 轮（已完成）

- 已落地 KEY_WAIT proc 节点：
  - 命令接收阶段只入队；
  - 每次 VM tick 先执行 KEY_WAIT proc 检查 `has_press_stock`；
  - 命中后调用 `consume_frame` 并返回脚本执行；
  - 未命中时调用 `on_wait_frame` 持续轮询，不提前结束单次 `vm.run()`。
- skip/interrupt 优先级已按 C++ 路径并入 proc 检查。

### 第 2 轮

- 对齐 flick 角度/阈值：
  - 角度零点、旋转方向、量化单位；
  - 像素阈值与 repeat/flick 竞争关系。

### 第 3 轮

- 补齐跨显示器边界策略：
  - stage 坐标与 OS cursor 坐标夹逼规则；
  - 窗口缩放变化时的坐标稳定性。

---

## 5) 代码证据点（Rust）

- 输入命令分发：`src/vm/command_input.rs`
- VM 主循环与调度：`src/vm/core.rs`
- Host 输入 API：`src/vm/api.rs`
- GUI 输入桥：`src/gui/input_bridge.rs`
- GUI 事件与坐标映射：`src/gui/app_logic.rs`
- Host 输入回调实现：`src/gui/host_impl.rs`

---

## 6) 本轮交接（Iteration Handoff）

### 本轮完成

- 对齐 `eng_frame.cpp::cancel_call_proc` 的 flick scene 前置 gating 最小闭环：
  - VM 新增 `is_flick_scene_allowed()`，在 flick proc 执行前先检查 `game_timer_move_flag / msg_back_open_flag / syscom_menu_disable_flag / hide_mwnd` 组合条件；
  - `hide_mwnd` 判定采用与 syscom query 相同的 `onoff && enable && exist` 语义，并兼容脚本态 `hide_mwnd_disable`。
- 补齐 `syscom_menu_disable` 运行时状态接入：
  - 在 syscom 命令分发层接入 `set_syscom_menu_enable/disable`，直接驱动 VM 侧 `syscom_menu_disable_flag`；
  - `init_syscom_flag` 时重置该标志，避免菜单禁用状态泄漏到新流程。
- 补齐 game timer 对 flick gating 的联动：
  - syscom proc queue 执行期默认置 `game_timer_move_flag=0`，在 `GameTimerStart` proc 恢复为 `1`；
  - 本地快照/恢复路径（local save state）已纳入 `game_timer_move_flag` 与 `syscom_menu_disable_flag`，保证读档后 gating 行为连续。
- 明确保留 stub 缺口（已写入代码 TODO）：movie 播放态与 excall 活跃态的 flick 禁止条件尚未并入，已标注 C++ 对照位置、预期行为与最小验证方向。

### 未完成 / 阻塞

- `eng_frame.cpp` flick scene gating 仍缺：
  - `m_sound.m_mov.is_playing()`（movie 播放中禁止 flick）；
  - `tnm_excall_is_excall()`（excall 活跃时禁止 flick）。
- `cmd_stage` / `cmd_object` 的非法参数与越界错误路径仍是 P0 差异项。
- `mouse.set_pos` 多显示器 + DPI 动态切换夹逼策略仍缺系统化验证记录。

### 下一轮首要任务（可直接执行）

1. 在 Host/VM 之间补齐 movie/excall 活跃态可观测标志，并并入 flick scene gating。
2. 推进 `cmd_stage` / `cmd_object` 越界错误路径（优先 stage/group/object index 的 C++ 错误分支）。
3. 整理 `mouse.set_pos` 跨显示器/DPI 变化验证记录，形成可回归的对照清单。
