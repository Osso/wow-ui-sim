"""Inspect the linked PTR test binary, then run only native ICU wrapper tests."""

import json
import os
from pathlib import Path
import re
import subprocess
import sys

FILTER = "intl_native::tests::"


def read_test_binary(build_log):
    executables = set()
    for line in build_log.read_text(encoding="utf-8-sig").splitlines():
        event = json.loads(line)
        if (
            event.get("reason") == "compiler-artifact"
            and event.get("target", {}).get("name") == "wow_ui_sim"
            and event.get("profile", {}).get("test")
            and event.get("executable")
        ):
            executables.add(event["executable"])
    if len(executables) != 1:
        raise RuntimeError(f"Expected one wow_ui_sim test binary, found {executables}")
    executable = Path(executables.pop()).resolve(strict=True)
    return executable


def run_capture(argv, timeout):
    result = subprocess.run(
        [str(arg) for arg in argv],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=timeout,
        check=False,
    )
    print(result.stdout, end="", flush=True)
    return result


def find_dumpbin():
    vswhere = Path(os.environ["ProgramFiles(x86)"]) / "Microsoft Visual Studio/Installer/vswhere.exe"
    result = run_capture(
        [vswhere, "-latest", "-products", "*", "-requires",
         "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-find",
         "VC/Tools/MSVC/*/bin/Hostx64/x64/dumpbin.exe"],
        30,
    )
    result.check_returncode()
    paths = result.stdout.strip().splitlines()
    if not paths:
        raise RuntimeError("Visual Studio dumpbin.exe was not found")
    return Path(paths[0]).resolve(strict=True)


def inspect_linkage(executable):
    if sys.platform == "win32":
        command = [find_dumpbin(), "/DEPENDENTS", executable]
    elif sys.platform == "darwin":
        command = ["otool", "-L", executable]
    elif sys.platform.startswith("linux"):
        command = ["ldd", executable]
    else:
        raise RuntimeError(f"Unsupported linkage probe platform: {sys.platform}")
    result = run_capture(command, 30)
    Path("ptr-icu-linkage.txt").write_text(result.stdout, encoding="utf-8")
    result.check_returncode()
    if sys.platform == "win32":
        if re.search(r"\bicu[^\s]*\.dll\b", result.stdout, re.IGNORECASE):
            raise RuntimeError("ICU DLL import found; Windows must link static ICU")
    else:
        if "libicu" not in result.stdout or "not found" in result.stdout:
            raise RuntimeError("Expected resolvable shared ICU imports")


def run_tests(executable):
    listing = run_capture([executable, FILTER, "--list"], 30)
    listing.check_returncode()
    tests = [line for line in listing.stdout.splitlines() if line.endswith(": test")]
    if not tests:
        raise RuntimeError(f"No native tests matched {FILTER!r}; refusing a zero-test pass")
    result = run_capture([executable, FILTER, "--nocapture", "--test-threads=1"], 90)
    Path("ptr-icu-tests.txt").write_text(listing.stdout + result.stdout, encoding="utf-8")
    result.check_returncode()


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: run_ptr_icu_tests.py <cargo-build-jsonl>")
    executable = read_test_binary(Path(sys.argv[1]))
    inspect_linkage(executable)
    run_tests(executable)


if __name__ == "__main__":
    main()
