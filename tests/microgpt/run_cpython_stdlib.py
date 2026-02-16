#!/usr/bin/env python3
"""Run microgpt.py on CPython while forcing Tython stdlib modules."""

from __future__ import annotations

import argparse
import importlib.util
import runpy
import sys
from pathlib import Path


def preload_module(name: str, path: Path) -> None:
    spec = importlib.util.spec_from_file_location(name, str(path))
    if spec is None or spec.loader is None:
        raise RuntimeError(f"failed to load module spec for {name} from {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    sys.modules[name] = module


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check-imports",
        action="store_true",
        help="only verify that stdlib/math.py and stdlib/random.py are being used",
    )
    args = parser.parse_args()

    script_dir = Path(__file__).resolve().parent
    repo_root = script_dir.parent.parent
    math_path = repo_root / "stdlib" / "math.py"
    random_path = repo_root / "stdlib" / "random.py"

    preload_module("math", math_path)
    preload_module("random", random_path)

    if args.check_imports:
        import math  # noqa: PLC0415
        import random  # noqa: PLC0415

        print("math file:", getattr(math, "__file__", "<builtin>"))
        print("random file:", getattr(random, "__file__", "<builtin>"))
        return 0

    # microgpt.py expects input.txt in the current working directory.
    prev_cwd = Path.cwd()
    try:
        import os

        os.chdir(script_dir)
        runpy.run_path("microgpt.py", run_name="__main__")
    finally:
        os.chdir(prev_cwd)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
