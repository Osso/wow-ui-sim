#!/usr/bin/env python3
"""Install only the full-suite controller; no simulator build or restart."""

from pathlib import Path
import shutil

source = Path(__file__).resolve().parent / "tools/full_suite.py"
target = Path.home() / "bin/full-suite"
target.parent.mkdir(parents=True, exist_ok=True)
shutil.copyfile(source, target)
target.chmod(0o755)
print(target)
