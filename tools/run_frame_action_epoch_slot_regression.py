#!/usr/bin/env python3
"""One-shot driver for frame-action epoch/slot regression sampling.

Runs `frame_action_epoch_slot_sampler.py`, writes archived JSON/TXT summaries,
and keeps a copy of the input trace log for A~E regression handoff.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import shutil
import subprocess
import sys
from pathlib import Path


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
    for cid in ["A", "B", "C", "D", "E"]:
        row = data["results"].get(cid, {})
        ok = row.get("ok", False)
        details = row.get("details", "")
        lines.append(f"[{ 'PASS' if ok else 'FAIL' }] case {cid}: {details}")
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
    json_path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    txt_path.write_text(format_text_report(data), encoding="utf-8")

    print(f"archive_dir={out_dir}")
    print(f"summary_json={json_path}")
    print(f"summary_txt={txt_path}")
    print(f"copied_log={copied_log}")

    return int(data.get("exit_code", 1))


if __name__ == "__main__":
    raise SystemExit(main())
