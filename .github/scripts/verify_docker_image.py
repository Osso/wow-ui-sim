"""Verify the final headless image offline, with two actually executed addon tests."""

import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import uuid


def run_container(image, arguments, fixture=None, qemu=None):
    name = "wow-ui-sim-proof-" + uuid.uuid4().hex
    command = ["docker", "run", "--rm", "--name", name, "--network", "none",
               "-e", "WOW_SIM_CASC=0", "-e", "WOW_SIM_NO_ADDONS=1"]
    if qemu is not None:
        # QEMU must not be PID 1: Linux otherwise ignores its fatal self-signal.
        command += ["--init", "--ulimit", "core=0",
                    "--mount", f"type=bind,src={qemu},dst=/qemu-x86_64-static,readonly",
                    "--entrypoint", "/qemu-x86_64-static"]
        arguments = ["-cpu", "Nehalem", "/app/wow-sim", *arguments]
    if fixture is not None:
        command += ["-v", f"{fixture}:/app/Interface/AddOns/PublicationSmoke:ro"]
    try:
        result = subprocess.run(command + [image, *arguments], capture_output=True,
                                text=True, timeout=90)
        print(result.stdout, end="")
        print(result.stderr, end="", file=sys.stderr)
        result.check_returncode()
        return result
    finally:
        # A timed-out Docker client does not necessarily stop its container.
        subprocess.run(["docker", "rm", "-f", name], capture_output=True, timeout=30)


def verify(image, qemu):
    # Fail before publication when a builder's native CPU leaks into the image.
    run_container(image, ["--version"], qemu=qemu)
    run_container(image, ["--version"])
    startup = run_container(image, ["--no-addons", "--no-saved-vars", "lua-errors"])
    if json.loads(startup.stdout) != []:
        raise RuntimeError("Final image has Blizzard startup Lua errors")
    with tempfile.TemporaryDirectory(prefix="wow-publication-smoke-") as directory:
        fixture = Path(directory)
        (fixture / "tests").mkdir()
        (fixture / "PublicationSmoke.toc").write_text(
            "## Interface: 120105\n## Title: PublicationSmoke\n", encoding="utf-8")
        (fixture / "tests/smoke.lua").write_text("""
test("Retail API epoch", function()
    local _, _, _, interface = GetBuildInfo()
    assertEquals(120105, interface)
end)
test("frame API is usable", function()
    local frame = CreateFrame("Frame")
    frame:SetSize(37, 19)
    assertEquals(37, frame:GetWidth())
    assertEquals(19, frame:GetHeight())
end)
""", encoding="utf-8")
        for emulator in (qemu, None):
            result = run_container(image, ["--no-addons", "--no-saved-vars",
                                           "run-tests", "PublicationSmoke"], fixture, emulator)
            output = re.sub(r"\x1b\[[0-9;]*m", "", result.stdout + result.stderr)
            if not re.search(r"(?m)^2 tests, 2 passed, 0 failed$", output):
                raise RuntimeError("Expected exactly two executed, passing addon tests")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image")
    parser.add_argument("--qemu", type=Path, default=Path("/usr/bin/qemu-x86_64"))
    arguments = parser.parse_args()
    qemu = arguments.qemu.resolve(strict=True)
    verify(arguments.image, qemu)
