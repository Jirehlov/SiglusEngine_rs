#!/usr/bin/env python3
from __future__ import annotations
import argparse
import shutil
from pathlib import Path

DEFAULT_GENERATED_FILES = (
    "failed_code_dynamic_runtime.log",
    "group_wait_pipeline_diagnostic.txt",
    "group_wait_runtime_with_failed_code.log",
    "group_wait_sample_diff.txt",
)


def prune_dirs(root: Path, keep: int) -> list[Path]:
    dirs = sorted([d for d in root.glob("*") if d.is_dir()])
    if keep < 0:
        keep = 0
    remove = dirs[:-keep] if keep and len(dirs) > keep else ([] if keep else dirs)
    for d in remove:
        shutil.rmtree(d)
    return remove


def prune_files(root: Path, names: tuple[str, ...]) -> list[Path]:
    removed = []
    for name in names:
        p = root / name
        if p.exists() and p.is_file():
            p.unlink()
            removed.append(p)
    return removed


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--frame-keep", type=int, default=1)
    ap.add_argument("--group-keep", type=int, default=1)
    ap.add_argument(
        "--prune-generated-root-files",
        action="store_true",
        help="remove top-level generated report artifacts (runtime/diagnostic/diff logs)",
    )
    args = ap.parse_args()
    root = Path(__file__).resolve().parents[1] / "reference"
    removed = []
    removed += prune_dirs(root / "frame_action_epoch_slot_reports", args.frame_keep)
    removed += prune_dirs(root / "group_wait_stock_reports", args.group_keep)
    if args.prune_generated_root_files:
        removed += prune_files(root, DEFAULT_GENERATED_FILES)
    print(f"removed={len(removed)}")
    for d in removed:
        print(d)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
