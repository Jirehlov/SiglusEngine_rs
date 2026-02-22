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

---

## 6) 本轮交接（Iteration Handoff）

### 本轮完成

1. **`__iapp_dummy` 路由补齐“函数级到分支级”的 C++ 证据注释**
   - 在 `src/vm/command_object.rs` 的 `ELM_OBJECT__IAPP_DUMMY` 分支补充对照注释：明确该路径对应 C++ `cmd_object.cpp` 的 int 查询车道（`tnm_stack_push_int`），并保持 query-only 语义。
   - 在 `src/gui/host_impl_stage_object.rs::on_object_query` 增补路由证据链注释：覆盖 list_id/stage/sub/object 存在性门禁、参数超长回落、未知 selector 回落 0 等分支。

2. **新增 selector 返回值域开发期开关断言（仅日志，不改外部行为）**
   - `on_object_query` 新增环境变量开关 `SIGLUS_DEV_ASSERT_IAPP_SELECTOR_DOMAIN`。
   - 开关开启时对 selector 0~8 的返回值域执行断言校验，发现漂移仅 `log::warn!`，不改变返回值与脚本可见行为。
   - 为 selector 0~8 增加 `cxx_branch_hint`，日志直接定位 slot，便于和 C++ 分支逐项对照。

3. **补充 quake 分类报告的 Rust/C++ 差值列生成脚本**
   - 新增 `src/bin/quake_kind_outlier_diff.rs`。
   - 读取 Rust/C++ 两份 quake 报告 CSV（`kind_outlier` 行），输出 `rust_err/cpp_err/delta_err/delta_abs`，并按 `vec/dir/zoom` 各自 top-N rank 生成对照结果。
   - 支持 `--top N` 和 `--out <file>`，用于直接导出“每类 top-N 偏差”结果。

### 未完成 / 阻塞

- `selector 0~8` 的**符号级命名**仍缺少 C++ 原始符号表证据；当前仍以“slot 对照 + 行为域约束”方式追踪。
- 当前环境 `cargo check` 受系统 `alsa.pc` 缺失阻塞，尚不能完成整仓无 warning 编译闭环。

### 下一轮首要任务（可直接执行）

1. 在 `siglus_engine_source` 进一步定位 movie/object 查询的常量来源（头文件/反编译符号），把 selector 0~8 从 slot 命名升级为“C++ 原名 + slot”双标注。
2. 将 `quake_kind_outlier_diff` 接入现有 quake 参考流程（统一输入/输出目录与命名），并在报告中自动引用最新对照差值文件。
3. 对 `__iapp_dummy` 的非 int 参数退化路径做 C++ 实机对照记录（缺省值、混合类型、超长参数），完善返回域约束文档。
