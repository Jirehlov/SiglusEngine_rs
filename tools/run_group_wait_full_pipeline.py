#!/usr/bin/env python3
"""Run full group-wait pipeline: launch engine, capture logs, extract sample, run regression."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

ROUTE_SOURCES = ("on_hit", "on_pushed", "on_decided")


def run(cmd: list[str], cwd: Path, env: dict[str, str]) -> int:
    proc = subprocess.run(cmd, cwd=str(cwd), env=env, text=True)
    return proc.returncode


def diagnose_runtime_log(log_path: Path) -> tuple[str, str]:
    if not log_path.exists():
        return ("missing_runtime_log", "未找到 runtime 日志；可先使用 --skip-engine-run 验证抽取/回归链路。")
    text = log_path.read_text(encoding="utf-8", errors="replace")
    low = text.lower()
    if "requires the features: `gui-bin`" in text:
        return ("missing_gui_feature", "请使用 `cargo build --bin siglus --features gui-bin` 构建 GUI 目标。")
    if "non-impl item macro in impl item position" in text:
        return (
            "gui_bin_compile_error",
            "检测到 gui-bin 专属编译错误；建议先运行 cargo build --bin siglus --features gui-bin 并修复后再采集。",
        )
    if "wayland" in low and "display" in low:
        return ("display_unavailable", "检测到显示服务不可用；在 CI/无头环境请使用 --skip-engine-run 或提供 Xvfb/Wayland。")
    if "x11" in low and "display" in low:
        return ("display_unavailable", "检测到 X11 DISPLAY 不可用；可改用 --skip-engine-run 或设置 DISPLAY。")
    if "permission denied" in low:
        return ("permission_denied", "运行权限不足；请确认二进制可执行并允许写入 reference 目录。")
    return ("unknown", "运行失败但未命中已知模式；请检查 runtime 日志末尾错误并重放。")


def write_diagnostic(diag_path: Path, code: str, message: str, runtime_log: Path) -> None:
    diag_path.parent.mkdir(parents=True, exist_ok=True)
    diag_path.write_text(
        f"code={code}\nmessage={message}\nruntime_log={runtime_log}\n",
        encoding="utf-8",
    )




def run_engine_capture(root: Path, runtime_log: Path, gameexe: Path, pck: Path, timeout_sec: int, env: dict[str, str]) -> int:
    bin_path = root / "target" / "debug" / "siglus"
    launch_cmd = [
        "timeout",
        f"{max(1, timeout_sec)}s",
        str(bin_path),
        "--gameexe",
        str(gameexe),
        "--pck",
        str(pck),
    ]
    with runtime_log.open("w", encoding="utf-8") as out:
        launched = subprocess.run(launch_cmd, cwd=str(root), env=env, stdout=out, stderr=subprocess.STDOUT, text=True)
    return launched.returncode

def write_sample_diff(diff_path: Path, before: str, after: str) -> dict[str, int]:
    before_set = [ln for ln in before.splitlines() if ln.strip()]
    after_set = [ln for ln in after.splitlines() if ln.strip()]
    added = [ln for ln in after_set if ln not in before_set]
    removed = [ln for ln in before_set if ln not in after_set]
    lines = [f"added={len(added)}", f"removed={len(removed)}"]
    for ln in added[:8]:
        lines.append(f"+ {ln}")
    for ln in removed[:8]:
        lines.append(f"- {ln}")
    diff_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return {"added": len(added), "removed": len(removed)}

def extract_failed_code_trace(runtime_log: Path, out_log: Path) -> int:
    if not runtime_log.exists():
        return 1
    lines = []
    for ln in runtime_log.read_text(encoding="utf-8", errors="replace").splitlines():
        if "vm.failed_code_trace" in ln:
            lines.append(ln)
    out_log.parent.mkdir(parents=True, exist_ok=True)
    out_log.write_text("\n".join(lines) + ("\n" if lines else ""), encoding="utf-8")
    return 0 if lines else 2


def build_source_hotspots(drift_by_source: dict[str, int], weighted_drift_by_source: dict[str, int]) -> list[dict]:
    rows = []
    for src in ROUTE_SOURCES:
        rows.append(
            {
                "source": src,
                "drift_count": drift_by_source.get(src, 0),
                "weighted_drift_count": weighted_drift_by_source.get(src, 0),
            }
        )
    rows.sort(key=lambda r: (-r["weighted_drift_count"], -r["drift_count"], r["source"]))
    return rows


def classify_risk(ok: bool, case_a: bool, case_b: bool, case_c: bool, route_groups: int, diff_added: int, diff_removed: int) -> tuple[str, str]:
    if not ok or not (case_a and case_b and case_c):
        return ("high", "case failure detected in regression summary")
    if route_groups <= 0:
        return ("high", "route_groups <= 0 indicates invalid extraction")
    diff_total = diff_added + diff_removed
    if diff_total >= 8:
        return ("medium", f"sample diff churn is high ({diff_total})")
    if diff_total > 0:
        return ("low", f"sample diff churn is non-zero ({diff_total})")
    return ("low", "stable sample diff and all cases passed")




def risk_rank(level: str) -> int:
    return {"low": 0, "medium": 1, "high": 2}.get(level, 0)


def build_degradation_summary(rows: list[dict]) -> dict:
    ordered = sorted(rows, key=lambda r: r.get("report", ""))
    streak = 0
    max_streak = 0
    prev_rank = None
    upgraded_reports = []
    for row in ordered:
        rank = risk_rank(row.get("risk_level", "low"))
        if prev_rank is not None and rank > prev_rank:
            streak += 1
            upgraded_reports.append(row.get("report", ""))
        else:
            streak = 0
        max_streak = max(max_streak, streak)
        prev_rank = rank
    alert = max_streak >= 2
    hint = (
        "rollback to last stable report before consecutive upgrades and rerun pipeline with --prefer-runtime-log"
        if alert
        else "no rollback needed"
    )
    return {
        "max_consecutive_risk_upgrade": max_streak,
        "upgraded_reports": upgraded_reports,
        "degradation_alert": alert,
        "rollback_hint": hint,
    }


def parse_signature(sig: str) -> tuple[str, str, int, int, int]:
    parts = sig.split(":")
    if len(parts) != 5:
        return ("-1", "-1", -1, -1, -1)
    return (parts[0], parts[1], int(parts[2]), int(parts[3]), int(parts[4]))


def classify_signature_drift(prev_signatures: tuple[str, ...], cur_signatures: tuple[str, ...]) -> dict[str, int]:
    prev_map = {}
    cur_map = {}
    for s in prev_signatures:
        st, gp, hit, pushed, decided = parse_signature(s)
        prev_map[(st, gp)] = {"on_hit": hit, "on_pushed": pushed, "on_decided": decided}
    for s in cur_signatures:
        st, gp, hit, pushed, decided = parse_signature(s)
        cur_map[(st, gp)] = {"on_hit": hit, "on_pushed": pushed, "on_decided": decided}
    out = {"on_hit": 0, "on_pushed": 0, "on_decided": 0}
    for key in sorted(set(prev_map.keys()) & set(cur_map.keys())):
        for route in ("on_hit", "on_pushed", "on_decided"):
            if prev_map[key][route] != cur_map[key][route]:
                out[route] += 1
    return out


def build_route_group_degradation(rows: list[dict]) -> dict:
    ordered = sorted(rows, key=lambda r: r.get("report", ""))
    prev_keys = None
    prev_signatures = None
    consecutive_route_change = 0
    max_consecutive_route_change = 0
    consecutive_value_drift = 0
    max_consecutive_value_drift = 0
    changed_reports = []
    drift_reports = []
    drift_by_source = {src: 0 for src in ROUTE_SOURCES}
    weighted_drift_by_source = {src: 0 for src in ROUTE_SOURCES}
    source_consecutive_batches = {src: 0 for src in ROUTE_SOURCES}
    for row in ordered:
        keys = tuple(sorted(row.get("route_keys", [])))
        signatures = tuple(sorted(row.get("route_signatures", [])))
        if prev_keys is not None and keys != prev_keys:
            consecutive_route_change += 1
            changed_reports.append(row.get("report", ""))
        else:
            consecutive_route_change = 0
        if prev_signatures is not None and signatures != prev_signatures:
            consecutive_value_drift += 1
            drift_reports.append(row.get("report", ""))
            delta = classify_signature_drift(prev_signatures, signatures)
            for source in ROUTE_SOURCES:
                drift_count = delta.get(source, 0)
                drift_by_source[source] += drift_count
                if drift_count > 0:
                    source_consecutive_batches[source] += 1
                    weighted_drift_by_source[source] += drift_count * source_consecutive_batches[source]
                else:
                    source_consecutive_batches[source] = 0
        else:
            consecutive_value_drift = 0
            for source in ROUTE_SOURCES:
                source_consecutive_batches[source] = 0
        max_consecutive_route_change = max(max_consecutive_route_change, consecutive_route_change)
        max_consecutive_value_drift = max(max_consecutive_value_drift, consecutive_value_drift)
        prev_keys = keys
        prev_signatures = signatures
    alert = max_consecutive_route_change >= 2 or max_consecutive_value_drift >= 2
    source_hotspots = build_source_hotspots(drift_by_source, weighted_drift_by_source)
    rollback_priority = [row["source"] for row in source_hotspots if row["weighted_drift_count"] > 0]
    hint = (
        "route topology/value drift changed in consecutive runs; rollback to last stable route signature and revalidate extraction log"
        if alert
        else "route-group topology/value stable"
    )
    return {
        "max_consecutive_route_change": max_consecutive_route_change,
        "max_consecutive_route_value_drift": max_consecutive_value_drift,
        "changed_reports": changed_reports,
        "value_drift_reports": drift_reports,
        "value_drift_by_source": drift_by_source,
        "weighted_value_drift_by_source": weighted_drift_by_source,
        "source_hotspots": source_hotspots,
        "rollback_priority_by_source": rollback_priority,
        "route_degradation_alert": alert,
        "route_rollback_hint": hint,
    }
def write_report_index(index_path: Path, report_root: Path, sample_diff_path: Path, diff_metrics: dict[str, int]) -> None:
    rows = []
    for d in sorted(report_root.glob("*")):
        if not d.is_dir():
            continue
        summary = d / "summary.json"
        if not summary.exists():
            continue
        try:
            data = json.loads(summary.read_text(encoding="utf-8"))
        except Exception:
            continue
        ok = bool(data.get("ok"))
        route_groups = len(data.get("route_groups", []))
        case_a = bool(data.get("results", {}).get("A", {}).get("ok", False))
        case_b = bool(data.get("results", {}).get("B", {}).get("ok", False))
        case_c = bool(data.get("results", {}).get("C", {}).get("ok", False))
        risk_level, risk_reason = classify_risk(
            ok,
            case_a,
            case_b,
            case_c,
            route_groups,
            diff_metrics.get("added", 0),
            diff_metrics.get("removed", 0),
        )
        route_keys = [f"{r.get('stage', -1)}:{r.get('group', -1)}" for r in data.get("route_groups", [])]
        route_signatures = [
            f"{r.get('stage', -1)}:{r.get('group', -1)}:{r.get('on_hit', -1)}:{r.get('on_pushed', -1)}:{r.get('on_decided', -1)}"
            for r in data.get("route_groups", [])
        ]
        rows.append(
            {
                "report": d.name,
                "ok": ok,
                "route_groups": route_groups,
                "route_keys": route_keys,
                "route_signatures": route_signatures,
                "case_a": case_a,
                "case_b": case_b,
                "case_c": case_c,
                "risk_level": risk_level,
                "risk_reason": risk_reason,
            }
        )

    risk_counts = {"low": 0, "medium": 0, "high": 0}
    for row in rows:
        risk_counts[row["risk_level"]] += 1

    degradation = build_degradation_summary(rows)
    route_group_degradation = build_route_group_degradation(rows)
    index = {
        "rows": rows,
        "row_count": len(rows),
        "sample_diff": sample_diff_path.as_posix(),
        "sample_diff_metrics": diff_metrics,
        "risk_counts": risk_counts,
        "degradation": degradation,
        "route_group_degradation": route_group_degradation,
    }
    index_path.write_text(json.dumps(index, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--gameexe", type=Path, default=Path("../Gameexe.dat"))
    ap.add_argument("--pck", type=Path, default=Path("../Scene.pck"))
    ap.add_argument("--runtime-log", type=Path, default=Path("reference/group_wait_runtime_capture.log"))
    ap.add_argument("--sample-log", type=Path, default=Path("reference/group_wait_input_stock_sample.log"))
    ap.add_argument("--timeout-sec", type=int, default=20)
    ap.add_argument("--skip-engine-run", action="store_true", help="skip engine run and only use existing runtime log")
    ap.add_argument("--diagnostic", type=Path, default=Path("reference/group_wait_pipeline_diagnostic.txt"))
    ap.add_argument("--prefer-runtime-log", action="store_true", help="prefer existing runtime log and skip engine run when available")
    ap.add_argument("--sample-diff", type=Path, default=Path("reference/group_wait_sample_diff.txt"))
    ap.add_argument("--emit-report-index", type=Path, default=Path("reference/group_wait_report_index.json"))
    ap.add_argument("--run-failed-code-replay", action="store_true", help="extract vm.failed_code_trace from runtime log and run dynamic replay checker")
    ap.add_argument("--failed-code-log", type=Path, default=Path("reference/failed_code_dynamic_runtime.log"))
    args = ap.parse_args()

    root = Path(__file__).resolve().parents[1]
    runtime_log = (root / args.runtime_log) if not args.runtime_log.is_absolute() else args.runtime_log
    sample_log = (root / args.sample_log) if not args.sample_log.is_absolute() else args.sample_log
    diagnostic = (root / args.diagnostic) if not args.diagnostic.is_absolute() else args.diagnostic
    sample_diff = (root / args.sample_diff) if not args.sample_diff.is_absolute() else args.sample_diff
    report_index = (root / args.emit_report_index) if not args.emit_report_index.is_absolute() else args.emit_report_index
    failed_code_log = (root / args.failed_code_log) if not args.failed_code_log.is_absolute() else args.failed_code_log

    runtime_log.parent.mkdir(parents=True, exist_ok=True)
    sample_log.parent.mkdir(parents=True, exist_ok=True)

    env = os.environ.copy()
    env.setdefault("RUST_LOG", "debug")
    env.setdefault("SIGLUS_GROUP_WAIT_TRACE", "1")
    if args.run_failed_code_replay:
        env.setdefault("SIGLUS_MOVIE_WAIT_TRACE", "1")

    sample_before = sample_log.read_text(encoding="utf-8", errors="replace") if sample_log.exists() else ""

    if args.prefer_runtime_log and runtime_log.exists():
        pass
    elif not args.skip_engine_run:
        build_cmd = ["cargo", "build", "--bin", "siglus", "--features", "gui-bin"]
        if run(build_cmd, root, env) != 0:
            code = "build_failed"
            msg = "GUI 构建失败；请先修复 gui-bin 构建错误，或使用 --skip-engine-run 仅验证抽取/回归链路。"
            write_diagnostic(diagnostic, code, msg, runtime_log)
            print(f"diagnostic={diagnostic}")
            return 2

        launched_rc = run_engine_capture(root, runtime_log, args.gameexe, args.pck, args.timeout_sec, env)
        if launched_rc not in (0, 124):
            code, msg = diagnose_runtime_log(runtime_log)
            write_diagnostic(diagnostic, code, msg, runtime_log)
            print(f"diagnostic={diagnostic}")
            return 4

    extract_cmd = [
        sys.executable,
        str(root / "tools" / "extract_group_wait_trace.py"),
        "--input",
        str(runtime_log),
        "--output",
        str(sample_log),
    ]
    if run(extract_cmd, root, env) != 0:
        code, msg = diagnose_runtime_log(runtime_log)
        write_diagnostic(diagnostic, code, msg, runtime_log)
        print(f"diagnostic={diagnostic}")
        return 3

    sample_after = sample_log.read_text(encoding="utf-8", errors="replace") if sample_log.exists() else ""
    diff_metrics = write_sample_diff(sample_diff, sample_before, sample_after)

    regress_cmd = [sys.executable, str(root / "tools" / "group_wait_input_stock_regression.py"), "--log", str(sample_log)]
    rc = run(regress_cmd, root, env)

    failed_code_rc = 0
    if args.run_failed_code_replay:
        ex_rc = extract_failed_code_trace(runtime_log, failed_code_log)
        if ex_rc != 0 and not args.skip_engine_run and not args.prefer_runtime_log:
            retry_env = env.copy()
            retry_env["SIGLUS_FAILED_CODE_AUTORUN"] = "1"
            retry_timeout = max(args.timeout_sec, 45)
            retry_log = runtime_log.with_name(runtime_log.stem + "_failed_retry.log")
            retry_rc = run_engine_capture(root, retry_log, args.gameexe, args.pck, retry_timeout, retry_env)
            if retry_rc in (0, 124):
                runtime_log.write_text(retry_log.read_text(encoding="utf-8", errors="replace"), encoding="utf-8")
                ex_rc = extract_failed_code_trace(runtime_log, failed_code_log)
        if ex_rc != 0:
            code = "missing_failed_code_trace"
            msg = "runtime 日志未提取到 vm.failed_code_trace；pipeline 已自动二次重放 failed 场景，仍缺失，请检查场景脚本入口。"
            write_diagnostic(diagnostic, code, msg, runtime_log)
            print(f"diagnostic={diagnostic}")
            failed_code_rc = 6
        else:
            replay_cmd = [sys.executable, str(root / "tools" / "regress_failed_code_dynamic.py"), "--log", str(failed_code_log)]
            failed_code_rc = run(replay_cmd, root, env)
            print(f"failed_code_log={failed_code_log}")

    write_report_index(report_index, root / "reference" / "group_wait_stock_reports", sample_diff, diff_metrics)
    print(f"report_index={report_index}")
    return rc if failed_code_rc == 0 else failed_code_rc


if __name__ == "__main__":
    raise SystemExit(main())
