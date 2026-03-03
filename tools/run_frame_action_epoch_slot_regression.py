#!/usr/bin/env python3
"""One-shot driver for frame-action epoch/slot regression sampling.

Runs `frame_action_epoch_slot_sampler.py`, writes archived JSON/TXT summaries,
and keeps a copy of the input trace log for A~E regression handoff.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import shutil
import statistics
import subprocess
import sys
from pathlib import Path


COUNTER_OBS_RE = re.compile(
    r"counter_observe\s+(?P<kind>wait|wait_key)(?:\s+owner=(?P<owner>-?\d+)\s+phase=(?P<phase>[a-z_]+)\s+depth=(?P<depth>-?\d+)\s+top=(?P<top>-?\d+))?\s+idx=(?P<idx>-?\d+)\s+option=(?P<option>-?\d+)\s+value=(?P<value>-?\d+)\s+active=(?P<active>[01])"
)


def run_sampler(py: str, sampler: Path, log_path: Path) -> dict:
    proc = subprocess.run(
        [py, str(sampler), "--log", str(log_path), "--json"],
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode not in (0, 1):
        raise RuntimeError(proc.stderr.strip() or "sampler failed")
    try:
        data = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"sampler json parse failed: {exc}") from exc
    data["exit_code"] = proc.returncode
    return data


def format_text_report(data: dict) -> str:
    lines = []
    lines.append(f"overall={'PASS' if data.get('ok') else 'FAIL'}")
    for cid in ["A", "B", "C", "D", "E", "F"]:
        row = data["results"].get(cid, {})
        ok = row.get("ok", False)
        details = row.get("details", "")
        lines.append(f"[{ 'PASS' if ok else 'FAIL' }] case {cid}: {details}")
    return "\n".join(lines) + "\n"


def build_counter_window_aggregate(
    log_path: Path,
    alert_min_samples: int,
    alert_high_ratio: float,
    alert_low_ratio: float,
    phase_window: int,
    phase_min_streak: int,
    phase_jitter_ratio: float,
) -> dict:
    rows = {}
    total = 0
    matched = 0
    option_dist = {}
    value_dist = {}
    phase_samples = {}
    for line in log_path.read_text(encoding="utf-8", errors="replace").splitlines():
        total += 1
        m = COUNTER_OBS_RE.search(line)
        if not m:
            continue
        matched += 1
        kind = m.group("kind")
        owner = int(m.group("owner") or -1)
        phase = m.group("phase") or "unknown"
        option = int(m.group("option"))
        value = int(m.group("value"))
        active = int(m.group("active"))
        key = (kind, owner, phase)
        row = rows.setdefault(key, {"samples": 0, "active_samples": 0, "option_dist": {}, "value_dist": {}})
        row["samples"] += 1
        row["active_samples"] += active
        row["option_dist"][option] = row["option_dist"].get(option, 0) + 1
        row["value_dist"][value] = row["value_dist"].get(value, 0) + 1
        option_dist[option] = option_dist.get(option, 0) + 1
        value_dist[value] = value_dist.get(value, 0) + 1
        phase_samples.setdefault((kind, owner, phase), []).append(active)

    buckets = []
    alerts = []
    phase_baseline = {}
    phase_windows = []
    for (kind, owner, phase), row in sorted(rows.items(), key=lambda item: (-item[1]["samples"], item[0])):
        samples = row["samples"]
        active_samples = row["active_samples"]
        active_ratio = active_samples / samples if samples else 0.0
        option_top = sorted(row["option_dist"].items(), key=lambda kv: (-kv[1], kv[0]))[:4]
        value_top = sorted(row["value_dist"].items(), key=lambda kv: (-kv[1], kv[0]))[:4]
        bucket = {
            "kind": kind,
            "owner": owner,
            "phase": phase,
            "samples": samples,
            "active_samples": active_samples,
            "active_ratio": round(active_ratio, 4),
            "option_dist": {str(k): v for k, v in sorted(row["option_dist"].items())},
            "value_dist": {str(k): v for k, v in sorted(row["value_dist"].items())},
            "option_top": [{"option": k, "count": v} for k, v in option_top],
            "value_top": [{"value": k, "count": v} for k, v in value_top],
        }
        buckets.append(bucket)
        if samples >= alert_min_samples and (active_ratio >= alert_high_ratio or active_ratio <= alert_low_ratio):
            alerts.append({
                "kind": kind,
                "owner": owner,
                "phase": phase,
                "samples": samples,
                "active_ratio": round(active_ratio, 4),
                "reason": "high_active_ratio" if active_ratio >= alert_high_ratio else "low_active_ratio",
            })

        seq = phase_samples.get((kind, owner, phase), [])
        if len(seq) < phase_window:
            continue
        window_ratios = []
        for i in range(0, len(seq) - phase_window + 1):
            chunk = seq[i : i + phase_window]
            ratio = sum(chunk) / phase_window
            window_ratios.append(round(ratio, 4))
            phase_windows.append({
                "kind": kind,
                "owner": owner,
                "phase": phase,
                "start": i,
                "end": i + phase_window - 1,
                "active_ratio": round(ratio, 4),
            })
        baseline = statistics.mean(window_ratios) if window_ratios else active_ratio
        phase_baseline_key = f"{kind}:{owner}:{phase}"
        phase_baseline[phase_baseline_key] = {
            "window_size": phase_window,
            "window_count": len(window_ratios),
            "baseline_ratio": round(baseline, 4),
            "jitter_threshold": phase_jitter_ratio,
        }
        streak = 0
        for idx, wr in enumerate(window_ratios):
            if abs(wr - baseline) >= phase_jitter_ratio:
                streak += 1
            else:
                streak = 0
            if streak >= phase_min_streak:
                alerts.append({
                    "kind": kind,
                    "owner": owner,
                    "phase": phase,
                    "samples": len(seq),
                    "active_ratio": wr,
                    "reason": "phase_window_jitter_streak",
                    "window_size": phase_window,
                    "window_end": idx + phase_window - 1,
                    "streak": streak,
                    "baseline_ratio": round(baseline, 4),
                })
                break

    return {
        "line_count": total,
        "observe_match_count": matched,
        "bucket_count": len(buckets),
        "alerts": alerts,
        "alert_thresholds": {
            "min_samples": alert_min_samples,
            "high_ratio": alert_high_ratio,
            "low_ratio": alert_low_ratio,
            "phase_window": phase_window,
            "phase_min_streak": phase_min_streak,
            "phase_jitter_ratio": phase_jitter_ratio,
        },
        "global_option_dist": {str(k): v for k, v in sorted(option_dist.items())},
        "global_value_dist": {str(k): v for k, v in sorted(value_dist.items())},
        "phase_baseline": phase_baseline,
        "phase_windows": phase_windows,
        "buckets": buckets,
    }


def format_aggregate_text(data: dict) -> str:
    lines = [
        f"line_count={data['line_count']}",
        f"observe_match_count={data['observe_match_count']}",
        f"bucket_count={data['bucket_count']}",
        f"alert_count={len(data['alerts'])}",
        f"alert_thresholds={data['alert_thresholds']}",
        f"global_option_dist={data['global_option_dist']}",
        f"global_value_dist={data['global_value_dist']}",
        f"phase_baseline_count={len(data.get('phase_baseline', {}))}",
    ]
    for key, row in sorted(data.get("phase_baseline", {}).items()):
        lines.append(f"phase_baseline[{key}]={row}")
    for row in data["buckets"]:
        lines.append(
            "kind={kind} owner={owner} phase={phase} samples={samples} active_samples={active_samples} active_ratio={active_ratio:.4f} option_top={option_top} value_top={value_top}".format(
                **row
            )
        )
    if data["alerts"]:
        lines.append("alerts:")
        for alert in data["alerts"]:
            lines.append(
                "  - kind={kind} owner={owner} phase={phase} samples={samples} active_ratio={active_ratio:.4f} reason={reason}".format(**alert)
            )
    return "\n".join(lines) + "\n"


def build_phase_baseline_trend(report_root: Path, keep_last: int, drift_ratio_threshold: float, drift_min_streak: int, drift_alert_delta_threshold: int) -> dict:
    entries = []
    for d in sorted(report_root.glob("*")):
        if not d.is_dir():
            continue
        agg = d / "counter_observe_aggregate.json"
        if not agg.exists():
            continue
        try:
            data = json.loads(agg.read_text(encoding="utf-8"))
        except Exception:
            continue
        phase_baseline = data.get("phase_baseline", {})
        avg_baseline = 0.0
        if phase_baseline:
            avg_baseline = sum(v.get("baseline_ratio", 0.0) for v in phase_baseline.values()) / len(phase_baseline)
        entries.append({
            "report": d.name,
            "phase_baseline_count": len(phase_baseline),
            "avg_baseline_ratio": round(avg_baseline, 4),
            "alert_count": len(data.get("alerts", [])),
        })
    if keep_last > 0:
        entries = entries[-keep_last:]
    deltas = []
    drift_alerts = []
    streak = 0
    for prev, cur in zip(entries, entries[1:]):
        ratio_delta = round(cur["avg_baseline_ratio"] - prev["avg_baseline_ratio"], 4)
        alert_delta = cur["alert_count"] - prev["alert_count"]
        row = {
            "from": prev["report"],
            "to": cur["report"],
            "avg_baseline_ratio_delta": ratio_delta,
            "alert_delta": alert_delta,
        }
        deltas.append(row)
        ratio_drift = abs(ratio_delta) >= drift_ratio_threshold
        alert_drift = abs(alert_delta) >= drift_alert_delta_threshold
        if ratio_drift or alert_drift:
            streak += 1
        else:
            streak = 0
        if streak >= drift_min_streak:
            drift_alerts.append({
                "to": cur["report"],
                "streak": streak,
                "ratio_delta": ratio_delta,
                "alert_delta": alert_delta,
                "reason": "continuous_drift",
                "rollback_hint": "建议先回退最近一批 wait/counter 观测改动，再重放同窗口样本确认是否收敛",
            })
    return {
        "entries": entries,
        "deltas": deltas,
        "entry_count": len(entries),
        "drift_alerts": drift_alerts,
        "drift_thresholds": {
            "ratio_threshold": drift_ratio_threshold,
            "min_streak": drift_min_streak,
            "alert_delta_threshold": drift_alert_delta_threshold,
        },
    }


def format_trend_text(data: dict) -> str:
    lines = [f"entry_count={data.get('entry_count', 0)}"]
    for row in data.get("entries", []):
        lines.append(
            "report={report} phase_baseline_count={phase_baseline_count} avg_baseline_ratio={avg_baseline_ratio:.4f} alert_count={alert_count}".format(**row)
        )
    for row in data.get("deltas", []):
        lines.append(
            "delta {from}->{to} avg_baseline_ratio_delta={avg_baseline_ratio_delta:.4f} alert_delta={alert_delta}".format(**row)
        )
    lines.append(f"drift_thresholds={data.get('drift_thresholds', {})}")
    for row in data.get("drift_alerts", []):
        lines.append(
            "drift_alert to={to} streak={streak} ratio_delta={ratio_delta:.4f} alert_delta={alert_delta} reason={reason} rollback_hint={rollback_hint}".format(**row)
        )
    return "\n".join(lines) + "\n"


def build_phase_trend_archive_index(report_root: Path, keep_last: int) -> dict:
    rows = []
    for d in sorted(report_root.glob("*")):
        if not d.is_dir():
            continue
        trend_path = d / "phase_baseline_trend.json"
        if not trend_path.exists():
            continue
        try:
            trend = json.loads(trend_path.read_text(encoding="utf-8"))
        except Exception:
            continue
        drift_alerts = trend.get("drift_alerts", [])
        first_drift_to = drift_alerts[0]["to"] if drift_alerts else ""
        rows.append({
            "report": d.name,
            "entry_count": trend.get("entry_count", 0),
            "drift_alert_count": len(drift_alerts),
            "first_drift_to": first_drift_to,
        })
    if keep_last > 0:
        rows = rows[-keep_last:]
    return {"rows": rows, "row_count": len(rows)}


def format_phase_trend_archive_index_text(data: dict) -> str:
    lines = [f"row_count={data.get('row_count', 0)}"]
    for row in data.get("rows", []):
        lines.append(
            "report={report} entry_count={entry_count} drift_alert_count={drift_alert_count} first_drift_to={first_drift_to}".format(**row)
        )
    return "\n".join(lines) + "\n"


def load_owner_phase_thresholds(path: Path | None) -> dict:
    if path is None:
        return {}
    if not path.exists():
        return {}
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}
    if not isinstance(raw, dict):
        return {}
    out = {}
    for key, row in raw.items():
        if not isinstance(row, dict):
            continue
        high = int(row.get("high", 4))
        medium = int(row.get("medium", 2))
        out[str(key)] = {
            "high": max(1, high),
            "medium": max(1, min(high, medium)),
            "reason": str(row.get("reason", "custom threshold")),
        }
    return out


def resolve_cluster_threshold(owner: int, phase: str, thresholds: dict) -> tuple[int, int, str]:
    exact = f"{owner}:{phase}"
    owner_any = f"{owner}:*"
    any_phase = f"*:{phase}"
    any_any = "*:*"
    row = thresholds.get(exact) or thresholds.get(owner_any) or thresholds.get(any_phase) or thresholds.get(any_any)
    if not row:
        return (4, 2, "default threshold")
    return (int(row.get("high", 4)), int(row.get("medium", 2)), str(row.get("reason", "custom threshold")))


def build_phase_trend_overview(report_root: Path, keep_last: int, threshold_cfg: dict, high_risk_only: bool) -> dict:
    rows = []
    cluster_scores = {}
    for d in sorted(report_root.glob("*")):
        if not d.is_dir():
            continue
        agg_path = d / "counter_observe_aggregate.json"
        trend_idx_path = d / "phase_baseline_trend_index.json"
        if not agg_path.exists() or not trend_idx_path.exists():
            continue
        try:
            agg = json.loads(agg_path.read_text(encoding="utf-8"))
            trend_idx = json.loads(trend_idx_path.read_text(encoding="utf-8"))
        except Exception:
            continue
        buckets = agg.get("buckets", [])
        drift_rows = trend_idx.get("rows", [])
        drift_alert_total = sum(r.get("drift_alert_count", 0) for r in drift_rows)
        aggregate_txt_path = d / "counter_observe_aggregate.txt"
        aggregate_txt_line_map = {}
        if aggregate_txt_path.exists():
            for ln_no, ln in enumerate(aggregate_txt_path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
                m = re.search(r"kind=(?P<kind>\w+) owner=(?P<owner>-?\d+) phase=(?P<phase>[\w_]+)", ln)
                if not m:
                    continue
                k = (m.group("kind"), int(m.group("owner")), m.group("phase"))
                if k not in aggregate_txt_line_map:
                    aggregate_txt_line_map[k] = {"start": ln_no, "end": ln_no}
                else:
                    aggregate_txt_line_map[k]["end"] = ln_no
        summary_txt_line = -1
        summary_txt_path = d / "summary.txt"
        if summary_txt_path.exists():
            for ln_no, ln in enumerate(summary_txt_path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
                if "case D:" in ln:
                    summary_txt_line = ln_no
                    break
        for bucket_idx, b in enumerate(buckets):
            row = {
                "report": d.name,
                "kind": b.get("kind", ""),
                "owner": b.get("owner", -1),
                "phase": b.get("phase", ""),
                "samples": b.get("samples", 0),
                "active_ratio": b.get("active_ratio", 0.0),
                "drift_alert_total": drift_alert_total,
            }
            rows.append(row)
            key = (row["owner"], row["phase"], row["kind"])
            score = cluster_scores.setdefault(
                key,
                {
                    "samples": 0,
                    "drift_alert_total": 0,
                    "hits": 0,
                    "first_report": d.name,
                    "bucket_refs": [],
                },
            )
            score["samples"] += row["samples"]
            score["drift_alert_total"] += drift_alert_total
            score["hits"] += 1
            agg_range = aggregate_txt_line_map.get((row["kind"], row["owner"], row["phase"]), {"start": -1, "end": -1})
            score["bucket_refs"].append(
                {
                    "report": d.name,
                    "owner": row["owner"],
                    "phase": row["phase"],
                    "kind": row["kind"],
                    "samples": row["samples"],
                    "drift_alert_total": drift_alert_total,
                    "aggregate_txt_line": agg_range["start"],
                    "aggregate_txt_line_range": [agg_range["start"], agg_range["end"]],
                    "summary_txt_line": summary_txt_line,
                    "summary_txt_line_range": [summary_txt_line, summary_txt_line],
                    "summary_json_pointer": "/results/D",
                    "aggregate_json_pointer": f"/buckets/{bucket_idx}",
                }
            )
    if keep_last > 0:
        rows = rows[-(keep_last * 16):]
    top_clusters = []
    for (owner, phase, kind), sc in cluster_scores.items():
        high_th, medium_th, reason_prefix = resolve_cluster_threshold(owner, phase, threshold_cfg)
        drift = sc["drift_alert_total"]
        risk_level = "high" if drift >= high_th else ("medium" if drift >= medium_th else "low")
        threshold_reason = f"{reason_prefix}; drift_alert_total={drift}; high>={high_th}; medium>={medium_th}"
        refs = sorted(sc["bucket_refs"], key=lambda r: (-r["drift_alert_total"], -r["samples"], r["report"]))
        top_clusters.append(
            {
                "owner": owner,
                "phase": phase,
                "kind": kind,
                "samples": sc["samples"],
                "drift_alert_total": drift,
                "hits": sc["hits"],
                "score": sc["samples"] + drift * 10,
                "risk_level": risk_level,
                "threshold_reason": threshold_reason,
                "minimal_repro_pointer": {
                    "first_report": sc["first_report"],
                    "top_bucket_ref": refs[0] if refs else None,
                },
            }
        )
    if high_risk_only:
        top_clusters = [r for r in top_clusters if r.get("risk_level") == "high"]
    top_clusters = sorted(
        top_clusters,
        key=lambda r: (-r["score"], -r["samples"], r["owner"], r["phase"], r["kind"]),
    )[:10]
    overview_line_base = len(rows) + 3
    for idx, cluster in enumerate(top_clusters):
        line_no = overview_line_base + idx
        pointer = cluster.get("minimal_repro_pointer", {})
        pointer["trend_overview_txt_line_range"] = [line_no, line_no]
    return {
        "rows": rows,
        "row_count": len(rows),
        "top_drift_clusters": top_clusters,
        "threshold_config_keys": sorted(threshold_cfg.keys()),
    }

def format_phase_trend_overview_text(data: dict) -> str:
    lines = [f"row_count={data.get('row_count', 0)}"]
    for row in data.get("rows", []):
        lines.append(
            "report={report} kind={kind} owner={owner} phase={phase} samples={samples} active_ratio={active_ratio} drift_alert_total={drift_alert_total}".format(**row)
        )
    lines.append(f"top_drift_cluster_count={len(data.get('top_drift_clusters', []))}")
    for row in data.get("top_drift_clusters", []):
        ptr = row.get("minimal_repro_pointer", {})
        lines.append(
            "top_cluster owner={owner} phase={phase} kind={kind} score={score} samples={samples} drift_alert_total={drift_alert_total} hits={hits} risk_level={risk_level} threshold_reason={threshold_reason} repro_first_report={repro_first} repro_top={repro_top}".format(
                **row,
                repro_first=ptr.get("first_report", ""),
                repro_top=ptr.get("top_bucket_ref", ""),
            )
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", type=Path, required=True, help="VM trace log file")
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=Path("reference/frame_action_epoch_slot_reports"),
        help="Archive root directory",
    )
    ap.add_argument(
        "--python",
        default=sys.executable,
        help="Python executable used to run sampler",
    )
    ap.add_argument("--alert-min-samples", type=int, default=8, help="minimum samples required before ratio alerting")
    ap.add_argument("--alert-high-ratio", type=float, default=0.95, help="high active-ratio alert threshold")
    ap.add_argument("--alert-low-ratio", type=float, default=0.05, help="low active-ratio alert threshold")
    ap.add_argument("--phase-window", type=int, default=4, help="window size for phase baseline/jitter aggregation")
    ap.add_argument("--phase-min-streak", type=int, default=2, help="minimum consecutive anomalous windows before phase jitter alert")
    ap.add_argument("--phase-jitter-ratio", type=float, default=0.25, help="ratio diff threshold against phase baseline for jitter alerts")
    ap.add_argument("--trend-keep-last", type=int, default=12, help="number of recent archives to include in phase trend summary")
    ap.add_argument("--trend-drift-ratio-threshold", type=float, default=0.15, help="cross-archive baseline-ratio drift threshold")
    ap.add_argument("--trend-drift-min-streak", type=int, default=2, help="minimum continuous drift streak for trend alerts")
    ap.add_argument("--trend-drift-alert-delta-threshold", type=int, default=1, help="cross-archive alert-count delta threshold")
    ap.add_argument("--trend-threshold-config", type=Path, default=None, help="optional JSON config for owner/phase risk thresholds")
    ap.add_argument("--trend-high-risk-only", action="store_true", help="only keep high-risk clusters in trend overview output")
    args = ap.parse_args()

    log_path = args.log
    if not log_path.exists():
        raise SystemExit(f"log file not found: {log_path}")

    root = Path(__file__).resolve().parents[1]
    sampler = root / "tools" / "frame_action_epoch_slot_sampler.py"
    stamp = dt.datetime.now().strftime("%Y%m%d_%H%M%S")
    out_dir = args.out_dir
    if not out_dir.is_absolute():
        out_dir = root / out_dir
    out_dir = out_dir / stamp
    out_dir.mkdir(parents=True, exist_ok=True)

    data = run_sampler(args.python, sampler, log_path)
    copied_log = out_dir / log_path.name
    shutil.copyfile(log_path, copied_log)

    json_path = out_dir / "summary.json"
    txt_path = out_dir / "summary.txt"
    aggregate_json_path = out_dir / "counter_observe_aggregate.json"
    aggregate_txt_path = out_dir / "counter_observe_aggregate.txt"
    trend_json_path = out_dir / "phase_baseline_trend.json"
    trend_txt_path = out_dir / "phase_baseline_trend.txt"
    trend_index_json_path = out_dir / "phase_baseline_trend_index.json"
    trend_index_txt_path = out_dir / "phase_baseline_trend_index.txt"
    trend_overview_json_path = out_dir / "phase_baseline_trend_overview.json"
    trend_overview_txt_path = out_dir / "phase_baseline_trend_overview.txt"
    json_path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    txt_path.write_text(format_text_report(data), encoding="utf-8")

    aggregate = build_counter_window_aggregate(
        log_path,
        alert_min_samples=max(1, args.alert_min_samples),
        alert_high_ratio=max(0.0, min(1.0, args.alert_high_ratio)),
        alert_low_ratio=max(0.0, min(1.0, args.alert_low_ratio)),
        phase_window=max(1, args.phase_window),
        phase_min_streak=max(1, args.phase_min_streak),
        phase_jitter_ratio=max(0.0, min(1.0, args.phase_jitter_ratio)),
    )
    aggregate_json_path.write_text(
        json.dumps(aggregate, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    aggregate_txt_path.write_text(format_aggregate_text(aggregate), encoding="utf-8")

    trend = build_phase_baseline_trend(
        out_dir.parent,
        max(1, args.trend_keep_last),
        max(0.0, min(1.0, args.trend_drift_ratio_threshold)),
        max(1, args.trend_drift_min_streak),
        max(0, args.trend_drift_alert_delta_threshold),
    )
    trend_json_path.write_text(json.dumps(trend, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    trend_txt_path.write_text(format_trend_text(trend), encoding="utf-8")

    trend_index = build_phase_trend_archive_index(out_dir.parent, max(1, args.trend_keep_last))
    trend_index_json_path.write_text(json.dumps(trend_index, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    trend_index_txt_path.write_text(format_phase_trend_archive_index_text(trend_index), encoding="utf-8")

    threshold_cfg = load_owner_phase_thresholds(args.trend_threshold_config)
    trend_overview = build_phase_trend_overview(
        out_dir.parent,
        max(1, args.trend_keep_last),
        threshold_cfg,
        args.trend_high_risk_only,
    )
    trend_overview_json_path.write_text(json.dumps(trend_overview, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    trend_overview_txt_path.write_text(format_phase_trend_overview_text(trend_overview), encoding="utf-8")

    print(f"archive_dir={out_dir}")
    print(f"summary_json={json_path}")
    print(f"summary_txt={txt_path}")
    print(f"aggregate_json={aggregate_json_path}")
    print(f"aggregate_txt={aggregate_txt_path}")
    print(f"trend_json={trend_json_path}")
    print(f"trend_txt={trend_txt_path}")
    print(f"trend_index_json={trend_index_json_path}")
    print(f"trend_index_txt={trend_index_txt_path}")
    print(f"trend_overview_json={trend_overview_json_path}")
    print(f"trend_overview_txt={trend_overview_txt_path}")
    print(f"copied_log={copied_log}")

    return int(data.get("exit_code", 1))


if __name__ == "__main__":
    raise SystemExit(main())
