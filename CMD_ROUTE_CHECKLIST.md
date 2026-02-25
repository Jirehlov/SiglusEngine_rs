# C++ `cmd_*` 路由对齐清单（VM 主线 / 重置版）

> 本清单已重置：不再保留历史流水账，改为“当前实现进度评估 + 下一轮可直接执行任务”。
>
> 评估范围：`siglus_rust/src/vm`，对照 `siglus_engine_source/cmd_*.cpp`、`flow_proc.cpp`、`eng_frame.cpp`。

## 一、VM 层实现进度评估（当前快照）

### 1) `stage.object` 主路由（`cmd_object.cpp`）

**已完成（高优先级骨架已对齐）**
- object 主命令分发具备完整骨架，lifecycle / create / movie / string-number / emote / frame_action 等主要族已进入 VM 内部分流。
- `stage.object.child` 已拆分为独立路由，具备 getter/setter lane、越界与 `is_use` 防护、typed 默认返回与 fatal 行为收敛。
- child 参数化 getter（size/pixel）已通过 `on_object_child_query(..., args, ...)` 打通“有参读取”路径。

**部分完成（仍有 C++ 语义细节差距）**
- `frame_action` 子分发中的 `load_gan/start_gan` 已按“真实资源替换才触发 invalidate”收敛，当前剩余差距转移到错误文案逐分支对照。
- child lane 对复杂 tail / command-only 误入虽有防护，但尚缺更系统的 C++ 错误文案对照清单（逐分支逐文案）。

### 2) `*_eve.wait / wait_key` 协议与 proc 观测（`flow_proc.cpp` / `ifc_proc_stack`）

**已完成**
- VM 已统一 wait 状态常量：`EVE_WAIT_DONE / EVE_WAIT_PENDING / EVE_WAIT_KEY_SKIPPED`。
- property lane + command lane 已统一上报 `on_int_event_wait_status` 与 `on_int_event_wait_status_with_proc`。
- syscom 观测 owner 常量已集中到共享区，并细化 phase：
  - `SYSCOM_WAIT_OWNER_PROC_RETURN_TO_MENU`
  - `SYSCOM_WAIT_OWNER_PROC_RETURN_TO_SEL`
  - `SYSCOM_WAIT_OWNER_PROC_END_GAME`
  - `SYSCOM_WAIT_OWNER_END_LOAD_PRE_QUEUE / POST_QUEUE`

**部分完成**
- host 默认适配层仍缺“推荐消费模板”（日志/统计/phase 聚类）的内建示例；目前以接口能力为主，接入规范仍偏口头约定。

### 3) frame_action counter 生命周期与帧推进（`eng_frame.cpp`）

**已完成**
- VM 已具备 `frame_action_counter_tick_all`，并接入主循环与 wait 路径，避免仅靠 wait/check 才推进。
- counter 具备对象上下文 bind/invalidate/stop/reclaim。
- `frame_action_ch.list_resize` 已引入 epoch guard（rebind 竞争保护）：
  - 先 bump epoch
  - 再 host resize
  - 后 invalidate
- slot 匹配已同时校验“slot 反算一致性 + epoch 一致性”，可阻断旧通道槽位误推进。

**部分完成**
- 多 stage 并发切换帧下的 epoch/slot 复用边界尚未形成专项回归清单（当前为逻辑防护到位，但缺系统化场景验证）。

## 二、当前阻塞项

- 环境缺少 `alsa` 系统库（`alsa.pc`），`cargo check` 被 `alsa-sys` 构建脚本阻塞，无法在该容器内给出“无 warning/无 error”完整编译结论。

## 三、本轮完成项（已完成“下一轮首要任务”全部任务）

1. **对照 `cmd_call.cpp` + `cmd_script.cpp`：补齐 `excall.script` 在 scope0/scope1 下字体参数落点与恢复细节**
   - 修正 `ELM_EXCALL_SCRIPT` 路由：不再直接以 `excall_slot.unwrap_or(1)` 作为字体存储槽，而是先走 `resolve_excall_scope`，再通过 `excall_script_font_scope(scope)` 统一映射到 scope1。
   - 对齐依据：C++ `tnm_command_proc_excall` 无论 `excall[0]` 还是 `excall[1]`，最终都进入 `tnm_command_proc_script_excall`，其字体状态均落在 `Gp_excall`。
   - 行为变化：`excall[0].script.set/get_font_*` 与 `excall[1].script.set/get_font_*` 现在共享同一字体状态池，不再出现 scope0 私有字体副本。

2. **对照 `cmd_others.cpp` + `flow_proc.cpp`：补齐 counter.wait/wait_key 的 host 观测字段标准化（owner/phase）**
   - `counter_observe` trace 由旧格式：
     - `counter_observe <kind> idx=... option=... value=... active=...`
     升级为新格式：
     - `counter_observe <kind> owner=... phase=... depth=... top=... idx=... option=... value=... active=...`
   - 新增 owner/phase 归一策略：
     - proc 深度 > 1：`owner = SYSCOM_WAIT_OWNER_PROC_BASE`，`phase = proc_other`；
     - 否则：`owner = 0`，`phase = non_syscom`。
   - 结果：counter wait 观测可直接和 `vm.wait owner/phase` 聚类字段对齐，不需要再做字段重写。

3. **对照 `eng_frame.cpp`：将采样脚本接入一键回归命令（脚本驱动 + 结果归档）**
   - 新增 `tools/run_frame_action_epoch_slot_regression.py`：
     - 调用 `frame_action_epoch_slot_sampler.py --json`；
     - 自动复制输入日志并输出 `summary.json` / `summary.txt`；
     - 归档到时间戳目录（默认 `reference/frame_action_epoch_slot_reports/<timestamp>/`）。
   - `frame_action_epoch_slot_sampler.py` 同步升级：兼容带 `owner/phase/depth/top` 的新版 `counter_observe` 行。
   - 回归文档已补“一键回归命令 + 归档结构 + 参数说明”，形成脚本化闭环。

## 四、下一轮首要任务（可直接开工）

1. **对照 `cmd_object.cpp`：补齐 `stage.object.child` lane 的错误文案逐分支对照清单并修正剩余差异**
   - 目标：把 getter/setter/command-only 误入/越界/未 use 的错误文本统一到 C++ 文案粒度。
2. **对照 `cmd_object.cpp` + `eng_frame.cpp`：补齐 object `frame_action/load_gan/start_gan` 的逐分支错误文案与返回值对齐**
   - 目标：将当前“行为已对齐、文案未完全对齐”的分支收敛到可逐项核销状态。
3. **对照 `flow_proc.cpp`：补齐 counter wait/proc 观测的 host 消费示例（聚类模板 + 最小报表）**
   - 目标：在默认 host 中提供可直接复用的 owner/phase 聚类示例，减少接入方重复实现。

## 四、执行约束（持续生效）

- 每轮必须给出可运行增量，禁止“仅注释/仅格式化”提交。
- 每轮结束必须回填：
  - 本轮完成项；
  - 阻塞；
  - 下一轮首要任务。
- 变更后的 `.rs` 文件必须保持 ≤700 行；若超限，同轮拆分。
