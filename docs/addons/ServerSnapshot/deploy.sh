#!/usr/bin/env python3
"""Install only ServerSnapshot's tracked Lua/TOC files into an existing AddOns root."""

import argparse
import os
from pathlib import Path
import shutil
import tempfile


def install_file(source: Path, destination: Path) -> None:
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{source.name}.", dir=destination.parent
    )
    os.close(descriptor)
    temporary = Path(temporary_name)
    try:
        shutil.copy2(source, temporary)
        temporary.replace(destination)
    finally:
        temporary.unlink(missing_ok=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "addons_root", type=Path, help="Existing WoW Interface/AddOns directory"
    )
    root = parser.parse_args().addons_root.resolve()
    if not root.is_dir():
        parser.error(f"AddOns directory does not exist: {root}")
    source = Path(__file__).resolve().parent
    destination = root / "ServerSnapshot"
    destination.mkdir(exist_ok=True)
    for name in ("ServerSnapshot.lua", "ServerSnapshot.toc"):
        install_file(source / name, destination / name)
        print(destination / name)
    print("Reload or log out of WoW to write a fresh ServerSnapshotDB bag capture.")


if __name__ == "__main__":
    main()
