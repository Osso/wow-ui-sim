#!/usr/bin/env python3
"""Stage the read-only PTR probe on the configured desktop."""
import os
from pathlib import Path

addon_dir = Path(__file__).resolve().parent
for filename in ("PixelRoundingProbe.toc", "PixelRoundingProbe.lua"):
    if not (addon_dir / filename).is_file():
        raise SystemExit(f"Missing probe file: {addon_dir / filename}")
os.execvp("scp", [
    "scp", "-r", str(addon_dir),
    "desktop:C:/World of Warcraft/_xptr_/Interface/AddOns/",
])
