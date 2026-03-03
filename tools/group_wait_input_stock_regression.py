#!/usr/bin/env python3
"""Group wait/input stock regression sampler for cmd_stage route parity."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
from pathlib import Path

ROUTE_RE = re.compile(r"vm\.group_wait\.route\s+stage=(?P<stage>-?\d+)\s+group=(?P<group>-?\d+)\s+route=(?P<route>on_hit|on_pushed|on_decided)\s+no=(?P<no>-?\d+)")
STOCK_RE = re.compile(r"vm\.group_wait\.stock\s+stage=(?P<stage>-?\d+)\s+group=(?P<group>-?\d+)\s+hit=(?P<hit>[^\s]+)\s+on_hit=(?P<on_hit>-?\d+)\s+on_pushed=(?P<on_pushed>-?\d+)\s+on_decided=(?P<on_decided>-?\d+)\s+mouse_stock=(?P<mouse>[01])\s+decide_stock=(?P<decide>[01])\s+cancel_input=(?P<cancel>[01])\s+direct_decided=(?P<direct>.+)$")
RESULT_RE = re.compile(r"vm\.group_wait\.result\s+stage=(?P<stage>-?\d+)\s+group=(?P<group>-?\d+)\s+decided=(?P<decided>-?\d+)\s+cancel_se_no=(?P<cancel_se>-?\d+)\s+route_hit=(?P<on_hit>-?\d+)\s+route_pushed=(?P<on_pushed>-?\d+)\s+route_decided=(?P<on_decided>-?\d+)")


def run_checks(lines: list[str]) -> dict:
    routes: dict[tuple[int, int], dict[str, int]] = {}
    stock_rows = []
    results = []
    for line in lines:
        if m := ROUTE_RE.search(line):
            key = (int(m.group("stage")), int(m.group("group")))
            routes.setdefault(key, {})[m.group("route")] = int(m.group("no"))
        if m := STOCK_RE.search(line):
            stock_rows.append({
                "stage": int(m.group("stage")),
                "group": int(m.group("group")),
                "mouse_stock": int(m.group("mouse")),
                "decide_stock": int(m.group("decide")),
                "cancel_input": int(m.group("cancel")),
                "on_hit": int(m.group("on_hit")),
                "on_pushed": int(m.group("on_pushed")),
                "on_decided": int(m.group("on_decided")),
            })
        if m := RESULT_RE.search(line):
            results.append({
                "stage": int(m.group("stage")),
                "group": int(m.group("group")),
                "decided": int(m.group("decided")),
                "on_hit": int(m.group("on_hit")),
                "on_pushed": int(m.group("on_pushed")),
                "on_decided": int(m.group("on_decided")),
            })

    case_a = len(routes) > 0 and all({"on_hit", "on_pushed", "on_decided"}.issubset(v.keys()) for v in routes.values())
    case_b = any(r["mouse_stock"] == 1 and r["decide_stock"] == 1 for r in stock_rows)
    case_c = any(res["decided"] in (res["on_hit"], res["on_pushed"], res["on_decided"]) for res in results)

    return {
        "ok": case_a and case_b and case_c,
        "results": {
            "A": {"ok": case_a, "details": f"route_groups={len(routes)}"},
            "B": {"ok": case_b, "details": f"stock_rows={len(stock_rows)}"},
            "C": {"ok": case_c, "details": f"result_rows={len(results)}"},
        },
        "route_groups": [{"stage": s, "group": g, **rv} for (s, g), rv in sorted(routes.items())],
    }


def format_txt(data: dict) -> str:
    lines = [f"overall={'PASS' if data['ok'] else 'FAIL'}"]
    for cid in ["A", "B", "C"]:
        row = data["results"][cid]
        lines.append(f"[{'PASS' if row['ok'] else 'FAIL'}] case {cid}: {row['details']}")
    for row in data.get("route_groups", []):
        lines.append(f"route stage={row['stage']} group={row['group']} on_hit={row.get('on_hit')} on_pushed={row.get('on_pushed')} on_decided={row.get('on_decided')}")
    return "\n".join(lines) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", type=Path, required=True)
    ap.add_argument("--out-dir", type=Path, default=Path("reference/group_wait_stock_reports"))
    args = ap.parse_args()

    lines = args.log.read_text(encoding="utf-8", errors="replace").splitlines()
    data = run_checks(lines)
    stamp = dt.datetime.now().strftime("%Y%m%d_%H%M%S")
    root = Path(__file__).resolve().parents[1]
    out_dir = args.out_dir if args.out_dir.is_absolute() else root / args.out_dir
    out_dir = out_dir / stamp
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "summary.json").write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    (out_dir / "summary.txt").write_text(format_txt(data), encoding="utf-8")
    (out_dir / args.log.name).write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"archive_dir={out_dir}")
    print(f"summary_json={out_dir / 'summary.json'}")
    print(f"summary_txt={out_dir / 'summary.txt'}")
    return 0 if data["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
