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
- 清空并重建“本轮交接”记录，避免历史多轮条目持续累积导致交接噪声。
- 本轮聚焦 `cmd_global.cpp::ELM_GLOBAL_RETURNMENU` 与 `cmd_syscom.cpp::ELM_SYSCOM_SET_RETURN_SCENE_ONCE` 的组合路径：
  - Rust `global.returnmenu(scene[, z])` 现优先采用显式参数目标；
  - 无显式参数时，按“`return_scene_once` 一次性目标 -> `VmOptions.return_menu_scene` 默认目标”回落；
  - `return_scene_once` 仅在被消费时 `take()` 清空，保持 one-shot 语义。
- 本轮补齐 `syscom.return_to_menu` 的最小 VM 消费路径：
  - 参照 C++ `cmd_syscom.cpp::ELM_SYSCOM_RETURN_TO_MENU`，Rust 新增该路由分支并复用当前 VM 跳转语义（优先 one-shot，再回落默认 menu 目标）；
  - 现阶段仍属部分复刻：warning/se/fade/msgback_except 等 C++ proc 细节尚未进入 Rust VM 管线。
- 保持前序已落地的 `GET_SYSTEM_EXTRA_INT_VALUE/STR_VALUE`：
  - 已接入 `Gameexe.dat` 的 `SYSTEM.EXTRA_*`，并按 C++ 边界回落（越界返回 0/空串）；
  - `STR` 分支继续镜像 C++ 当前实现的 `system_int_value_cnt` 维度门控。
- 完成一次全量 `cargo check`，当前无 error 且无 warning。

### Syscom 分支级对照（本轮新增）

> 对照源：`siglus_engine_source/cmd_syscom.cpp` 与 `siglus_rust/src/vm/command_syscom.rs`

| 分组 | C++ 代表分支 | Rust 状态 | 说明 |
|---|---|---|---|
| hide mwnd on/off | `SET/GET/CHECK_HIDE_MWND_*` | 🟡 部分复刻（本轮推进） | 已拆分 onoff/enable/exist 三套状态位并接入 CHECK；仍缺与 UI 层状态联动 |
| wipe anime on/off | `SET_NO_WIPE_ANIME_*` / `SET_SKIP_WIPE_ANIME_*` | 🟡 部分复刻 | 已同步到 VM 选项并参与 wipe wait；其余 syscom 相关联配置仍缺 |
| save/load 主路径 | `SAVE/LOAD/QUICK/INNER/COPY/CHANGE/DELETE/CHECK` | 🟡 部分复刻（已补边界+flag） | 本地内存槽与 enable/exist/check 基础可工作；源/目标 slot 负数均按无效处理；仍缺 C++ 的 UI/对话框/错误类型与持久化细节 |
| slot 时间/文本查询 | `GET_SAVE_*` / `GET_QUICK_SAVE_*` | 🟡 部分复刻（本轮补边界） | 读取不存在或非法 slot 返回 0/空串；时间戳来源仍是 Rust 本地时间快照 |
| open dialog | `OPEN_MSG_BACK` / `CLOSE_MSG_BACK` / `CHECK_MSG_BACK_OPEN` / `OPEN_TWEET_DIALOG` | 🟡 部分复刻（本轮补副作用+快捷键+exist_msg+disable+off写入+disp/proc脚本位+host通知） | msg_back 已接 GUI backlog，`OPEN_MSG_BACK` 成功分支会按 C++ 行为清掉 `read_skip_onoff`；`CHECK_MSG_BACK_ENABLE/OPEN_MSG_BACK` 要求已有历史消息且未被脚本禁用，并由 `CLEAR_MSGBK/INSERT_MSGBK_IMG` 联动历史存在位；文本 print 仅在 `msg_back_off` 关闭时才写入历史可用位；`set_msg_back_disp_off/on` 通过 `on_msg_back_display` 联动 GUI backlog 显隐与手动打开；`set_msg_back_proc_off/on` 当前仅保留状态位 STUB（已撤回推断性输入门控，待找到 C++ 直接消费路径后再补）；tweet 已具备 host 回调 + GUI 占位对话框（认知状态/输入/空文本确认/开闭），但真实上传流程仍未落地 |
| call_ex | `CALL_EX` | 🟡 部分复刻 | 已能发起 farcall-like 流程；参数/返回与 C++ 细节仍待核对 |

### 未完成 / 阻塞
- `cmd_syscom.cpp` 仍有大量分支未进入 Rust 路径（save/load menu 流程、音量/开关矩阵、msg_back 打开状态与 UI 联动等）；`set_return_scene_once` 与 `return_to_menu` 已接 VM 最小消费，但仍缺 C++ 系统菜单/proc 全链路对齐。
- `OPEN_MSG_BACK` 已接入 GUI backlog 显隐；`msg_back_proc_off/on` 仍缺 C++ 侧直接消费路径（当前仅保留 STUB 状态位），tweet/dialog 仍缺真实上传与结果回调流程。
- effect/world/steam 行为实现接近空白，后续需要从 C++ 路由逐条补。

### 下一轮首要任务（可直接执行）
1. 继续 `cmd_script.cpp` + `eng_frame.cpp`：核对 `msg_back_proc_on/off` 的 C++ 细粒度消费路径（当前 Rust 仅保留 STUB 状态位，未再引入推断性输入行为）。
2. 推进 `OPEN_TWEET_DIALOG`：把占位输入/发送状态替换为真实授权与上传回调，逐步对齐 C++ 的 `sys_wnd_solo_tweet`。
3. 继续核对 `SET_RETURN_SCENE_ONCE` + `SYSCOM_RETURN_TO_MENU` 与 C++ 系统菜单/proc 管线的差异（包括 warning/se/fade/msgback_except、调用入口与清理时机）。
4. 继续补齐 `msg_back` 打开态与系统菜单流程联动（含 proc/disp/disable 多位组合边界）。
5. 完成后再次保持 `cargo check` 无 error/warning。
