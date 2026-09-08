#!/usr/bin/env python3
"""Install the read-only probe through the configured desktop SSH alias."""
import os
from pathlib import Path

addon_dir = Path(__file__).resolve().parent
for filename in ("UnitFrameLayerProbe.toc", "UnitFrameLayerProbe.lua", "Controls.lua"):
    if not (addon_dir / filename).is_file():
        raise SystemExit(f"Missing probe file: {addon_dir / filename}")
os.execvp("scp", [
    "scp", "-r", str(addon_dir),
    "desktop:C:/World of Warcraft/_retail_/Interface/AddOns/",
])
