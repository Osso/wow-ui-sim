#!/usr/bin/env python3
"""Install the manual probe in the desktop PTR addon directory."""
import os
from pathlib import Path

addon_dir = Path(__file__).resolve().parent
for filename in (
    "Ptr125RemainingProbe.toc", "Core.lua", "Structures.lua",
    "Secrets.lua", "Main.lua", "Templates.xml",
):
    if not (addon_dir / filename).is_file():
        raise SystemExit(f"Missing probe file: {addon_dir / filename}")
os.execvp("scp", [
    "scp", "-r", str(addon_dir),
    "desktop:C:/World of Warcraft/_xptr_/Interface/AddOns/",
])
