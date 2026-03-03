#!/usr/bin/env python3
from __future__ import annotations
import argparse
from pathlib import Path

PREFIXES = ("vm.group_wait.route ", "vm.group_wait.stock ", "vm.group_wait.result ")

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", type=Path, required=True)
    ap.add_argument("--output", type=Path, required=True)
    args = ap.parse_args()
    lines = args.input.read_text(encoding="utf-8", errors="replace").splitlines()
    picked = [ln for ln in lines if any(ln.startswith(p) for p in PREFIXES)]
    args.output.write_text("\n".join(picked) + ("\n" if picked else ""), encoding="utf-8")
    print(f"picked={len(picked)}")
    return 0 if picked else 1

if __name__ == "__main__":
    raise SystemExit(main())
