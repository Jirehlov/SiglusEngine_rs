# CMD_ROUTE_CHECKLIST（重置版）

> 适用范围：`siglus_rust/`

## 目标

- 本清单仅用于 VM 路由与指令层的**静态对齐**推进。
- 非 VM 层（图形、特效、渲染、音频、工具）不作为 C++ 严格对齐对象。

## 当前执行标准（强制）

1. **严格对齐范围**
   - 仅虚拟机层（VM 逻辑内核）严格对应 C++ 原版语义。

2. **非 VM 实现策略**
   - 可按指令名望文生义实现，保证接口可用、逻辑自洽。

3. **检查标准**
   - 动态检查仅 `cargo check`。
   - 对齐工作仅做静态对齐，不做动态对拍要求。

4. **目录约束**
   - `siglus_rust/reference/` 已移除，不再作为流程输入或输出目录。

## 每轮交付最小闭环

1. 识别一个 VM 路由/指令差异点。
2. 提交可编译代码增量。
3. 运行并记录 `cargo check` 结果。
4. 写明下一轮首要 VM 差异项。

## 迭代记录（2026-03-02 / read-flag）

### 本轮完成
1. 完成 SELBTN read-flag 语义接线：选择返回 `selected>=0` 时补充 `read_flag_mark/read_flag_complete` 检查点；取消返回时补充 `read_flag_skip_cancel`。
2. 扩展检查点负载：在 `SelBtnSyncCheckpoint` 中加入 `cancel_enable`，便于静态区分取消可用/不可用路径。
3. 修复并标准化 READY→START 路由细节：去除重复 `start_resolved` 发射，并补齐 `SELBTN_START` 的 `start_resolved` 观测阶段。
4. 继续保持 `capture_flag` 与 `sel_start_call_*` 仅在有效选择路径触发，避免取消分支污染后续阶段语义。

### 下一轮首要 VM 差异项
1. 对齐 SELBTN `template_no` 与 UI 模板槽位消费语义（含默认模板/越界兜底）并补充对应阶段检查点。
