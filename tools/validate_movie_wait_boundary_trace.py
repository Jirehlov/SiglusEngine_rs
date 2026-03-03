#!/usr/bin/env python3
"""Validate movie check/wait boundary trace semantics from SIGLUS_MOVIE_WAIT_TRACE logs."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

CHECK_RE = re.compile(r"vm\.movie_wait_trace check_movie stage=(?P<stage>\w+) index=(?P<index>-?\d+) state=(?P<state>\w+) value=(?P<value>-?\d+)(?:\s+ready_only=(?P<ready_only>true|false)\s+generation_live=(?P<generation_live>true|false)\s+failed_live=(?P<failed_live>true|false)\s+interrupted_live=(?P<interrupted_live>true|false))?")
START_RE = re.compile(r"vm\.movie_wait_trace gate_start stage=(?P<stage>\w+) index=(?P<index>-?\d+) key_skip=(?P<key_skip>true|false) state=(?P<state>\w+)")
END_RE = re.compile(r"vm\.movie_wait_trace gate_end stage=(?P<stage>\w+) index=(?P<index>-?\d+) key_skip=(?P<key_skip>true|false) state=(?P<state>\w+)")
SKIP_RE = re.compile(r"vm\.movie_wait_trace key_skip_consumed stage=(?P<stage>\w+) index=(?P<index>-?\d+) state=(?P<state>\w+)")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", type=Path, required=True)
    args = ap.parse_args()

    text = args.log.read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()

    starts = {}
    gate_count = 0
    key_skip_count = 0
    errors = []

    for i, ln in enumerate(lines, 1):
        m = CHECK_RE.search(ln)
        if m:
            state = m.group("state")
            value = int(m.group("value"))
            expected = {
                "Pending": 1,
                "Ready": 0,
                "Interrupted": -2,
            }
            if state in expected and value != expected[state]:
                errors.append(f"line {i}: check_movie state={state} expected {expected[state]} got {value}")
            if state == "Failed" and value >= 0:
                errors.append(f"line {i}: Failed must return negative code, got {value}")
            if m.group("ready_only") is not None:
                ready_only = m.group("ready_only") == "true"
                generation_live = m.group("generation_live") == "true"
                failed_live = m.group("failed_live") == "true"
                interrupted_live = m.group("interrupted_live") == "true"
                if state == "Pending" and ready_only and not generation_live:
                    errors.append(f"line {i}: ready_only Pending should keep generation_live=true during pre-start/wait")
                if state == "Failed" and not failed_live:
                    errors.append(f"line {i}: Failed state requires failed_live=true")
                if state == "Interrupted" and not interrupted_live:
                    errors.append(f"line {i}: Interrupted state requires interrupted_live=true")

        m = START_RE.search(ln)
        if m:
            gate_count += 1
            key = (m.group("stage"), int(m.group("index")))
            starts[key] = starts.get(key, 0) + 1

        m = SKIP_RE.search(ln)
        if m:
            key_skip_count += 1

        m = END_RE.search(ln)
        if m:
            key = (m.group("stage"), int(m.group("index")))
            if starts.get(key, 0) <= 0:
                errors.append(f"line {i}: gate_end without gate_start for {key}")
            else:
                starts[key] -= 1

    dangling = [k for k, v in starts.items() if v > 0]
    for key in dangling:
        errors.append(f"dangling gate_start without gate_end for {key}")

    out = {
        "ok": not errors,
        "gate_count": gate_count,
        "key_skip_count": key_skip_count,
        "checks": {
            "state_value_mapping": True,
            "gate_pairing": True,
            "ready_only_liveness": True,
        },
        "errors": errors,
    }
    print(json.dumps(out, ensure_ascii=False, indent=2))
    return 0 if not errors else 1


if __name__ == "__main__":
    raise SystemExit(main())
