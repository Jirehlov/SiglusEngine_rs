#!/usr/bin/env python3
"""Frame-action / excall epoch-slot regression sampler.

Consumes VM trace logs and performs lightweight A~E scenario checks documented in
`docs/frame_action_epoch_slot_regression_template.md`.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List

COUNTER_OBS_RE = re.compile(
    r"counter_observe\s+(?P<kind>wait|wait_key)(?:\s+owner=(?P<owner>-?\d+)\s+phase=(?P<phase>[a-z_]+)\s+depth=(?P<depth>-?\d+)\s+top=(?P<top>-?\d+))?\s+idx=(?P<idx>-?\d+)\s+option=(?P<option>-?\d+)\s+value=(?P<value>-?\d+)\s+active=(?P<active>[01])"
)
EPOCH_RE = re.compile(r"epoch=(?P<epoch>-?\d+)")
SLOT_RE = re.compile(r"slot=(?P<slot>-?\d+)")
STAGE_RE = re.compile(r"stage=(?P<stage>-?\d+)")


@dataclass
class CheckResult:
    case_id: str
    ok: bool
    details: str


def parse_log_lines(path: Path) -> List[str]:
    return path.read_text(encoding="utf-8", errors="replace").splitlines()


def find_numeric(lines: List[str], pattern: re.Pattern[str], key: str) -> List[int]:
    vals: List[int] = []
    for line in lines:
        m = pattern.search(line)
        if m:
            vals.append(int(m.group(key)))
    return vals


def check_case_a(lines: List[str]) -> CheckResult:
    epochs = find_numeric(lines, EPOCH_RE, "epoch")
    if len(epochs) < 2:
        return CheckResult("A", False, "epoch 样本不足，至少需要两次采样")
    ok = epochs[-1] >= epochs[-2]
    return CheckResult("A", ok, f"epoch tail={epochs[-2:]}")


def check_case_b(lines: List[str]) -> CheckResult:
    slots = find_numeric(lines, SLOT_RE, "slot")
    unique_cnt = len(set(slots))
    ok = unique_cnt >= 2
    return CheckResult("B", ok, f"slot unique={unique_cnt}")


def check_case_c(lines: List[str]) -> CheckResult:
    stages = find_numeric(lines, STAGE_RE, "stage")
    ok = any(stage in (0, 1, 2) for stage in stages)
    return CheckResult("C", ok, f"stages sampled={sorted(set(stages))[:10]}")


def check_case_d(lines: List[str]) -> CheckResult:
    observed = []
    for line in lines:
        m = COUNTER_OBS_RE.search(line)
        if m:
            observed.append((m.group("kind"), int(m.group("option"))))
    ok = any(kind == "wait" for kind, _ in observed) and any(
        kind == "wait_key" for kind, _ in observed
    )
    return CheckResult("D", ok, f"counter_observe entries={len(observed)}")


def check_case_e(lines: List[str]) -> CheckResult:
    # Type-mismatch path is considered covered when log contains explicit counter fatal.
    hits = [line for line in lines if "(frame_action.counter)" in line or "(counter)" in line]
    ok = len(hits) > 0
    return CheckResult("E", ok, f"fatal/mismatch lines={len(hits)}")


def run_all(lines: List[str]) -> Dict[str, CheckResult]:
    checks = [check_case_a, check_case_b, check_case_c, check_case_d, check_case_e]
    results = {r.case_id: r for r in (check(lines) for check in checks)}
    return results


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", type=Path, required=True, help="VM trace log file")
    ap.add_argument(
        "--json", action="store_true", help="Print machine-readable JSON output"
    )
    args = ap.parse_args()

    lines = parse_log_lines(args.log)
    results = run_all(lines)
    failed = [r for r in results.values() if not r.ok]

    if args.json:
        print(
            json.dumps(
                {
                    "ok": not failed,
                    "results": {
                        cid: {"ok": r.ok, "details": r.details}
                        for cid, r in results.items()
                    },
                },
                ensure_ascii=False,
                indent=2,
            )
        )
    else:
        for cid in ["A", "B", "C", "D", "E"]:
            r = results[cid]
            mark = "PASS" if r.ok else "FAIL"
            print(f"[{mark}] case {cid}: {r.details}")

    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
