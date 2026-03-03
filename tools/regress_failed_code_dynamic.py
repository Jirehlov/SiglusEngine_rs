#!/usr/bin/env python3
"""Dynamic log-driven regression for failed code parity across create/check/query paths."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

CREATE_RE = re.compile(r"vm\.failed_code_trace create stage=(?P<stage>\w+) index=(?P<index>-?\d+) auto_init=(?P<auto_init>true|false) real_time=(?P<real_time>true|false) ready_only=(?P<ready_only>true|false)")
CHECK_RE = re.compile(r"vm\.failed_code_trace check stage=(?P<stage>\w+) index=(?P<index>-?\d+) state=(?P<state>\w+) value=(?P<value>-?\d+)")
QUERY_RE = re.compile(r"vm\.failed_code_trace query stage=(?P<stage>\w+) index=(?P<index>-?\d+) selector=(?P<selector>-?\d+) value=(?P<value>-?\d+)")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", type=Path, required=True)
    args = ap.parse_args()

    text = args.log.read_text(encoding="utf-8", errors="replace").splitlines()
    creates = {}
    checks = {}
    queries = {}
    observed_failed_codes = set()
    errors = []

    for i, ln in enumerate(text, 1):
        m = CREATE_RE.search(ln)
        if m:
            key = (m.group("stage"), int(m.group("index")))
            creates[key] = {
                "auto_init": m.group("auto_init") == "true",
                "real_time": m.group("real_time") == "true",
                "ready_only": m.group("ready_only") == "true",
            }
            continue

        m = CHECK_RE.search(ln)
        if m:
            key = (m.group("stage"), int(m.group("index")))
            state = m.group("state")
            value = int(m.group("value"))
            checks[key] = {"state": state, "value": value}
            if state == "Failed":
                observed_failed_codes.add(value)
                if value not in {-11, -12, -13, -14, -15}:
                    errors.append(f"line {i}: Failed code must be in -11..-15, got {value}")
            continue

        m = QUERY_RE.search(ln)
        if m:
            key = (m.group("stage"), int(m.group("index")))
            selector = int(m.group("selector"))
            value = int(m.group("value"))
            queries[key] = {"selector": selector, "value": value}
            if selector != 14:
                errors.append(f"line {i}: query selector must be 14, got {selector}")
            continue

    # cross-path parity: check_failed value must equal query selector14 value for same object
    for key, chk in checks.items():
        if chk["state"] != "Failed":
            continue
        q = queries.get(key)
        if not q:
            errors.append(f"missing selector14 query for failed check at {key}")
            continue
        if q["value"] != chk["value"]:
            errors.append(f"check/query mismatch at {key}: check={chk['value']} query={q['value']}")
        if key not in creates:
            errors.append(f"missing create trace before check/query for {key}")

    # require full -11..-15 coverage in sample/runtime log
    required = {-11, -12, -13, -14, -15}
    missing = sorted(required - observed_failed_codes)
    if missing:
        errors.append(f"missing failed code coverage: {missing}")

    out = {
        "ok": not errors,
        "create_count": len(creates),
        "check_count": len(checks),
        "query_count": len(queries),
        "observed_failed_codes": sorted(observed_failed_codes),
        "errors": errors,
    }
    print(json.dumps(out, ensure_ascii=False, indent=2))
    return 0 if not errors else 1


if __name__ == "__main__":
    raise SystemExit(main())
