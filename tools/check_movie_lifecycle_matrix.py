#!/usr/bin/env python3
"""Static regression guard for object movie terminal-state lifecycle matrix."""

from __future__ import annotations

from pathlib import Path
import json
import sys


def must_contain(text: str, needle: str, ctx: str) -> None:
    if needle not in text:
        raise AssertionError(f"missing needle in {ctx}: {needle}")


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    post_render = (root / "src" / "gui" / "host_stage_object_cmd_post_render.rs").read_text(encoding="utf-8")
    cmd = (root / "src" / "gui" / "host_stage_object_cmd.rs").read_text(encoding="utf-8")
    reset = (root / "src" / "gui" / "host_stage_object_movie_reset.rs").read_text(encoding="utf-8")

    # Matrix: transition/event -> whether terminal failure/interrupted must be cleared.
    expected = {
        "ObjectStarted": True,
        "ObjectFinished": True,
        "FREE_INIT": True,
        "RESET_FOR_CREATE": True,
        "RESUME_MOVIE": True,
        "CREATE_MOVIE_FAMILY": True,
        "CHECK_MOVIE_FAILED_CODE_MAP": True,
    }

    checks = {
        "ObjectStarted": "MoviePlaybackEvent::ObjectStarted",
        "ObjectFinished": "MoviePlaybackEvent::ObjectFinished",
        "FREE_INIT": "ELM_OBJECT_FREE",
        "RESET_FOR_CREATE": "reset_object_runtime_state_for_create",
        "RESUME_MOVIE": "ELM_OBJECT_RESUME_MOVIE",
        "CREATE_MOVIE_FAMILY": "ELM_OBJECT_CREATE_MOVIE_WAIT_KEY",
        "CHECK_MOVIE_FAILED_CODE_MAP": "-(10 + info.category_code())",
    }

    for k, needle in checks.items():
        source = cmd if k in ("ObjectStarted", "ObjectFinished", "CHECK_MOVIE_FAILED_CODE_MAP") else (reset if k == "RESET_FOR_CREATE" else post_render)
        must_contain(source, needle, k)

    # Stronger assertion: required clear helper call sites.
    required_clear_call_sites = [
        (cmd, "ObjectStarted", "self.clear_movie_terminal_state(stage, index);"),
        (cmd, "ObjectFinished", "self.clear_movie_terminal_state(stage, index);"),
        (reset, "RESET_FOR_CREATE", "self.clear_movie_terminal_state(plane, object_index);"),
        (post_render, "FREE_INIT", "self.clear_movie_terminal_state(plane, object_index);"),
        (post_render, "RESUME_MOVIE", "self.clear_movie_terminal_state(plane, object_index);"),
        (post_render, "CREATE_MOVIE_FAMILY", "self.clear_movie_terminal_state(plane, object_index);"),
    ]
    for src, ctx, needle in required_clear_call_sites:
        must_contain(src, needle, ctx)

    out = {
        "ok": True,
        "matrix": expected,
        "notes": [
            "free/init/create/resume paths clear terminal failure/interrupted state",
            "check_movie failed code mapping remains -(10 + category_code)",
        ],
    }
    print(json.dumps(out, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssertionError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, ensure_ascii=False, indent=2))
        raise SystemExit(1)
