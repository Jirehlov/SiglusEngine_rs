# FrameAction_CH Epoch/Slot 回归模板（脚本步骤 + 日志断言）

> 对照：`siglus_engine_source/eng_frame.cpp`、`cmd_object.cpp`。
>
> 目标：把 `frame_action_ch` 在多 stage 切换、resize、counter 并发路径中的 epoch/slot 复用风险，转成可重复执行的脚本步骤模板与日志断言模板。

## 一、统一观测字段

所有场景统一采集以下字段（按事件行输出）：

- `slot`：counter 槽位（包含 frame_action 与 frame_action_ch lane）
- `epoch`：channel 重绑定代次
- `list_id`：对象列表（`stage.object`/front/back/next）
- `obj_idx`：对象号
- `stage_idx`：stage 号
- `ch_idx`：channel 号（`frame_action_ch`）
- `phase`：`tick/start/stop/reset/wait/check_*` 等
- `value`：counter 当前值
- `active`：counter active 状态

建议日志前缀：

- 逐事件：`vm.excall.counter ...`
- 聚合提示：`vm.excall.counter.aggregate ...`

## 二、脚本场景模板

### 场景 A：双 stage 快切 + 同帧 resize + start_frame_loop

**步骤模板**
1. 进入 stage A，创建对象 `obj=0`，`frame_action_ch.resize(4)`。
2. 在 `ch=1` 调 `counter.start_frame_loop(start=0,end=30,frame=15)`。
3. 同帧切换 stage B，复用 `obj=0`，执行 `frame_action_ch.resize(2)`。
4. 下一帧回到 stage A，执行 `counter.check_active`、`counter.get`。

**断言模板**
- A1：stage B resize 后，旧 epoch 的 A/stageA/ch1 slot 不再被 tick。
- A2：回到 stage A 后，若发生重绑，`epoch` 必然递增；旧 `slot+epoch` 组合不再写值。
- A3：`check_active` 结果仅反映当前 epoch 对应实例。

---

### 场景 B：同对象多 channel 交替 resize（N->0->N）+ wait/check_active

**步骤模板**
1. `frame_action_ch.resize(3)`。
2. `ch0.counter.start_frame`、`ch1.counter.start_frame_loop`。
3. 立即 `frame_action_ch.resize(0)`。
4. 下一指令 `frame_action_ch.resize(2)`，再对 `ch1` 执行 `counter.wait` 与 `counter.check_active`。

**断言模板**
- B1：`resize(0)` 后旧 channel 的 slot 必须 stop/inactive。
- B2：二次 `resize(2)` 之后新 channel 不得继承旧 slot 的 `value/active` 残留。
- B3：`wait`/`check_active` 不应唤醒旧 epoch 槽位。

---

### 场景 C：`load_gan/start_gan` 触发与非触发 invalidate 交错

**步骤模板**
1. `frame_action.counter.start_frame_loop(...)`。
2. 连续两次 `load_gan("same_path")`。
3. 再执行 `load_gan("new_path")`。
4. 插入 `start_gan(set_no_same)` 与 `start_gan(set_no_new)` 交错。

**断言模板**
- C1：资源未变化时不触发无意义 invalidate（counter 连续性保留）。
- C2：资源变化时必须触发 invalidate（旧上下文停止推进）。
- C3：日志中 `phase/reclaim` 与 `active=false` 出现顺序稳定。

---

### 场景 D：syscom return_to_menu/sel 与 frame tick 交错

**步骤模板**
1. 持有活动中的 `frame_action_ch` counter。
2. 触发 `return_to_sel` 或 `return_to_menu` proc。
3. 在 proc 阶段插入一帧推进，再恢复脚本。

**断言模板**
- D1：proc 阶段 tick 不推进失效 epoch。
- D2：恢复脚本后 slot 归属仍与当前 stage/object/channel 一致。
- D3：wait owner 与 counter phase 日志可对应（便于后续聚合）。

---

### 场景 E：excall scope0/1 混合访问，不串扰 stage-object slot

**步骤模板**
1. scope0 执行 `excall.counter[array][i].start`。
2. scope1 执行同索引 `excall[1].counter[array][i].start`。
3. 并行执行 `stage.object.frame_action_ch[array][ch].counter.start`。
4. 交替 `get/check_active/reset`。

**断言模板**
- E1：scope0 与 scope1 的 counter value/active 互不污染。
- E2：excall counter lane 与 object frame_action_ch lane slot 不重叠。
- E3：reset 仅影响目标 lane/scope。

## 三、日志断言格式模板

建议最小断言 DSL（可由 host 或外部脚本实现）：

1. `assert_no_tick_after_invalidate(slot, old_epoch)`
2. `assert_epoch_monotonic(slot)`
3. `assert_active_cleared_on_resize_zero(obj, stage)`
4. `assert_scope_isolation(scope0_slot, scope1_slot)`
5. `assert_lane_isolation(excall_slot, object_slot)`

## 四、执行记录模板（每次回归填写）

- 场景：A/B/C/D/E
- 脚本版本：
- 关键日志片段：
- 断言结果：PASS/FAIL
- 偏差说明（若 FAIL）：
- 关联代码位置（Rust/C++）：

## 五、Host Trace 开关与执行手册（A~E 场景）

### 1) 开关说明

- `SIGLUS_EXCALL_COUNTER_TRACE=1`
  - 打开逐事件 counter trace（`vm.excall.counter slot=... phase=...`）。
- `SIGLUS_EXCALL_COUNTER_TRACE_HINT=1`
  - 打开一次性聚合模板提示（`vm.excall.counter.aggregate ...`）。

建议组合：

```bash
SIGLUS_EXCALL_COUNTER_TRACE=1 \
SIGLUS_EXCALL_COUNTER_TRACE_HINT=1 \
RUST_LOG=debug \
cargo run --bin siglus_rust_gui -- --gameexe <Gameexe.dat> --pck <Scene.pck>
```

### 2) 场景执行步骤（通用）

1. 启动前打开上述开关。
2. 逐个执行 A→E 场景脚本模板。
3. 每个场景结束后抓取对应日志窗口（建议 5s bucket）。
4. 按“日志断言格式模板”执行断言。

### 3) 采样窗口建议

- 高频场景（A/B/E）：`500ms` 或 `1s`。
- 中低频场景（C/D）：`1s` 或 `5s`。
- 每个窗口至少记录：
  - `events_total`
  - `by_phase_count`
  - `active_ratio`
  - `value_min/value_max/value_last`

### 4) 快速故障定位顺序

1. 先看 `assert_lane_isolation`（是否串槽）。
2. 再看 `assert_epoch_monotonic`（是否旧 epoch 回流）。
3. 最后看 `assert_no_tick_after_invalidate`（是否 invalidate 后仍推进）。

## 六、日志样例 + 预期断言样例（双栏模板）

### 样例 1：invalidate 后不再推进旧 epoch

| 日志样例 | 预期断言 |
|---|---|
| `vm.excall.counter slot=5010000 phase=tick value=12 active=true` | 记录当前 epoch=E 的末次 tick。 |
| `vm.excall.counter slot=5010000 phase=reclaim value=12 active=false` | `assert_no_tick_after_invalidate(5010000, E)` 开始生效。 |
| _(后续窗口无 `slot=5010000 phase=tick`)_ | 断言通过。 |

### 样例 2：scope 隔离

| 日志样例 | 预期断言 |
|---|---|
| `vm.excall.counter slot=5100000 phase=start value=0 active=true` | 记为 scope0 槽。 |
| `vm.excall.counter slot=5200000 phase=start value=0 active=true` | 记为 scope1 槽。 |
| `vm.excall.counter slot=5100000 phase=reset value=0 active=false` | `assert_scope_isolation(5100000, 5200000)`：scope1 不受影响。 |

### 样例 3：lane 隔离（excall vs object frame_action_ch）

| 日志样例 | 预期断言 |
|---|---|
| `vm.excall.counter slot=5100001 phase=start value=0 active=true` | excall lane 起点。 |
| `vm.excall.counter slot=5300450 phase=start value=0 active=true` | object frame_action_ch lane 起点。 |
| `vm.excall.counter slot=5300450 phase=check_active value=1 active=true` | `assert_lane_isolation(5100001, 5300450)`：互不串扰。 |

### 样例 4：epoch 单调递增

| 日志样例 | 预期断言 |
|---|---|
| `... slot=5300123 phase=start ... epoch=7 ...` | 旧 epoch。 |
| `... frame_action_ch.resize ...` | 触发重绑定。 |
| `... slot=5300123 phase=start ... epoch=8 ...` | `assert_epoch_monotonic(5300123)`：新 epoch > 旧 epoch。 |

## 七、A~E 场景日志+断言样例（每场景最少 2 对）

### 场景 A（双 stage 快切 + 同帧 resize）

| 日志样例 | 预期断言 |
|---|---|
| `... stage=A slot=5300101 epoch=12 phase=start value=0 active=true` | A-1: 建立 A/stage 初始绑定。 |
| `... stage=B slot=5300101 epoch=13 phase=reclaim value=0 active=false` | A-2: `assert_no_tick_after_invalidate(5300101, 12)`。 |

### 场景 B（N->0->N resize）

| 日志样例 | 预期断言 |
|---|---|
| `... slot=5300200 phase=stop value=5 active=false` | B-1: `resize(0)` 后旧槽停止。 |
| `... slot=5300200 phase=start value=0 active=true` | B-2: 二次 resize 后重启，不继承旧 active/value。 |

### 场景 C（load_gan/start_gan 交错）

| 日志样例 | 预期断言 |
|---|---|
| `... load_gan same_path ... phase=tick value=9 active=true` | C-1: 同资源不触发额外 invalidate。 |
| `... load_gan new_path ... phase=reclaim value=9 active=false` | C-2: 资源变化触发 reclaim。 |

### 场景 D（syscom return 与 tick 交错）

| 日志样例 | 预期断言 |
|---|---|
| `... counter_observe wait owner=-10000 phase=proc_other depth=2 top=1 idx=... active=1` | D-1: 记录 proc 阶段入口。 |
| `... vm.wait owner=-10012 phase=return_to_sel status=1 key_skip=false depth=2 top=1` | D-2: `assert_no_tick_after_invalidate(slot, old_epoch)`。 |

### 场景 E（scope0/1 + object lane 混合）

| 日志样例 | 预期断言 |
|---|---|
| `... slot=5100002 scope=0 phase=start value=0 active=true` | E-1: scope0 建立。 |
| `... slot=5200002 scope=1 phase=start value=0 active=true` | E-2: `assert_scope_isolation(5100002, 5200002)`。 |

## 八、`frame_action.counter` 参数矩阵（method-by-method）

> 适用路径：`array[idx].up.counter.<method>`。
>
> 当前实现策略：参数不匹配统一 `frame_action.counter` fatal + typed 默认返回；
> 若存在深层 tail（`...counter.<method>.tail`）则优先命中 `frame_action_ch` fatal。

| method | 期望参数个数 | 缺参/多参错误出口 |
|---|---:|---|
| `set` | 1 | `無効なコマンドが指定されました。(frame_action.counter)` |
| `get` | 0 | 同上 |
| `reset` | 0 | 同上 |
| `start` / `start_real` / `resume` | 0 | 同上 |
| `start_frame` / `start_frame_real` | 3 | 同上 |
| `start_frame_loop` / `start_frame_loop_real` | 3 | 同上 |
| `stop` | 0 | 同上 |
| `wait` / `wait_key` | 1 | 同上 |
| `check_value` | 1 | 同上 |
| `check_active` | 1 | 同上 |

## 九、A~E 场景“通过/失败”样例对照

### A：双 stage 快切

| 类型 | 日志样例 |
|---|---|
| 通过 | `... stage=B slot=5300101 epoch=13 phase=reclaim ...` 后无 `slot=5300101 epoch=12 phase=tick` |
| 失败 | `... stage=B ...` 之后仍出现 `slot=5300101 epoch=12 phase=tick` |

### B：N->0->N resize

| 类型 | 日志样例 |
|---|---|
| 通过 | `resize(0)` 后 `slot=5300200 phase=stop active=false`，重建后 `start value=0` |
| 失败 | 重建后直接出现 `slot=5300200 phase=tick value=5`（继承旧值） |

### C：load_gan/start_gan 交错

| 类型 | 日志样例 |
|---|---|
| 通过 | `same_path` 阶段无 reclaim；`new_path` 后出现 `phase=reclaim` |
| 失败 | `same_path` 也频繁触发 reclaim，或 `new_path` 不触发 reclaim |

### D：syscom return 交错

| 类型 | 日志样例 |
|---|---|
| 通过 | `pre_queue` 与 `post_queue` 间无旧 epoch tick |
| 失败 | `post_queue` 后仍看到旧 epoch `phase=tick` |

### E：scope/lane 混合

| 类型 | 日志样例 |
|---|---|
| 通过 | `scope0 slot=5100002` reset 不影响 `scope1 slot=5200002` |
| 失败 | `scope0` reset 后 `scope1` 同索引同时变 inactive |



## 十、自动化采样脚本骨架（伪代码）

> 目标：把 A~E 场景从“手工观测”收敛为“固定采样 + 自动判定”。

```text
function run_case(case_id):
    reset_vm_session()
    set_env("SIGLUS_EXCALL_COUNTER_TRACE", "1")
    set_env("SIGLUS_EXCALL_COUNTER_TRACE_HINT", "1")

    script = build_case_script(case_id)
    run_script(script)

    logs = collect_logs([
        "vm: frame_counter",
        "vm: excall_counter",
        "vm: excall_counter_hint",
        "on_object_action",
    ])

    sample = {
        "slot": parse_last_int(logs, "slot="),
        "epoch": parse_last_int(logs, "epoch="),
        "list_id": parse_last_int(logs, "list="),
        "obj_idx": parse_last_int(logs, "obj="),
        "stage_idx": parse_last_int(logs, "stage="),
        "ch_idx": parse_last_int(logs, "ch="),
        "active": parse_last_bool(logs, "active="),
        "value": parse_last_int(logs, "value="),
    }

    assertions = build_case_assertions(case_id)
    return evaluate(sample, logs, assertions)

function main():
    report = {}
    for case_id in ["A", "B", "C", "D", "E"]:
        result = run_case(case_id)
        report[case_id] = result
        print_case_summary(case_id, result)

    if any_failed(report):
        dump_fail_artifacts(report)
        exit(1)
    exit(0)
```

### 关键断言建议

- A：`FREE -> ALLOC` 后同路径 slot 不得沿用旧 epoch；旧 slot 必须 inactive。
- B：`frame_action_ch.resize` 后旧 ch 绑定计数器必须失效，且新 ch 触发新上下文绑定。
- C：stage 切换（含 back/front/next）后旧 stage counter 不再推进。
- D：`wait/wait_key/check_*` 的返回值与 active/value 状态一致。
- E：`Int/Str` 混入参数时，合法数字串可解析；非法串必须走 `frame_action.counter` fatal + 默认返回。


## 十一、可执行采样脚本（tools）

仓库已提供最小可运行采样脚本：

- `tools/frame_action_epoch_slot_sampler.py`

### 用法

```bash
python siglus_rust/tools/frame_action_epoch_slot_sampler.py --log <trace.log>
python siglus_rust/tools/frame_action_epoch_slot_sampler.py --log <trace.log> --json
```

### 输出解释

- A：检查 epoch 样本是否形成有效更新序列（覆盖 free->alloc 后 epoch 变化）。
- B：检查 slot 样本是否出现重绑定（覆盖 resize/重建路径）。
- C：检查 stage 样本是否覆盖 `0/1/2`（stage/back/front/next）。
- D：检查 `counter_observe wait/wait_key` 观测是否都有出现。
- E：检查类型偏差/非法参数路径是否触发 `frame_action.counter` 或 `counter` 错误出口。

> 说明：脚本是“最小自动化采样器”，用于回归初筛；失败后仍应结合上文 A~E 详细断言与原始日志逐条定位。


## 十、一键回归命令（脚本驱动 + 结果归档）

新增脚本：`tools/run_frame_action_epoch_slot_regression.py`。

用途：
- 调用 `tools/frame_action_epoch_slot_sampler.py --json` 运行 A~E 初筛；
- 自动归档输入日志与结果到时间戳目录；
- 输出 `summary.json` + `summary.txt`，便于下一轮直接接力。

示例：

```bash
python tools/run_frame_action_epoch_slot_regression.py   --log /path/to/vm_trace.log
```

可选参数：
- `--out-dir`：归档根目录（默认 `reference/frame_action_epoch_slot_reports`）
- `--python`：指定运行 sampler 的 Python 可执行文件

归档目录结构示例：

```text
reference/frame_action_epoch_slot_reports/20260225_120102/
  ├── vm_trace.log
  ├── summary.json
  └── summary.txt
```
