#!/usr/bin/env python3
"""Verify failed-code mapping consistency across create/check/query(iapp selector 14) paths."""

from __future__ import annotations

import json
from pathlib import Path


def must(text: str, needle: str, tag: str) -> None:
    if needle not in text:
        raise AssertionError(f"[{tag}] missing: {needle}")


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    cmd = (root / "src" / "gui" / "host_stage_object_cmd.rs").read_text(encoding="utf-8")
    impl_stage = (root / "src" / "gui" / "host_impl_stage_object.rs").read_text(encoding="utf-8")
    iapp = (root / "src" / "gui" / "host_impl_stage_object_iapp.rs").read_text(encoding="utf-8")

    # canonical failed-code mapping in check_movie path
    must(cmd, "fn object_check_movie_failed_code", "cmd")
    must(cmd, "-(10 + info.category_code())", "cmd")

    # check_movie route must consume canonical mapping
    must(impl_stage, "MovieWaitState::Failed => self.object_check_movie_failed_code(plane, obj_index)", "on_object_get")

    # iapp selector-14 must exist and use same helper
    must(iapp, "CheckMovieFailedCode", "iapp enum")
    must(iapp, "14 => Self::CheckMovieFailedCode", "iapp selector")
    must(iapp, "Self::CheckMovieFailedCode => (-15..=-11).contains(&value) || value == -1", "iapp domain")
    must(impl_stage, "IappMovieQuerySelector::CheckMovieFailedCode => self.object_check_movie_failed_code(plane, obj_index)", "iapp impl")

    print(json.dumps({
        "ok": True,
        "checks": [
            "object_check_movie_failed_code canonical mapping present",
            "check_movie failed branch uses canonical mapping",
            "iapp selector 14 exists and uses same mapping helper",
            "selector 14 domain remains -11..-15 or -1",
        ],
    }, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssertionError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, ensure_ascii=False, indent=2))
        raise SystemExit(1)
