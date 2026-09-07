#!/usr/bin/env python3
"""Stage all four probe addons in the PTR install; never run the client."""
import os
from pathlib import Path

root = Path(__file__).resolve().parent.parent
addons = [root / f"BootstrapOrderProbe_{letter}" for letter in "ABCD"]
for addon in addons:
    if not (addon / f"{addon.name}.toc").is_file():
        raise SystemExit(f"Missing probe TOC: {addon}")
os.execvp("scp", ["scp", "-r", *(str(addon) for addon in addons),
                  "desktop:C:/World of Warcraft/_xptr_/Interface/AddOns/"])
