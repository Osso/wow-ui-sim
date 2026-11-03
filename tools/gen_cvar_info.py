#!/usr/bin/env python3
"""Extract native CVar flags from a pinned Wowless data snapshot.

Uses only the Python standard library. Wowless's generated addon compares
ConsoleGetAllCommands/C_CVar.GetCVarInfo results in the native client; errsv.lua
imports the captured differences. This script copies those expected flags,
including measured false values, without importing defaults or inventing flags.

The parser deliberately supports only the catalog's two-level YAML layout.
Unknown flag syntax and duplicate names fail instead of becoming false.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import urllib.request

REVISION = "69a3654e08305de6fd5ce3533faf7e5597135109"
PROFILES = {
    "retail": {
        "product": "wow",
        "last_change_revision": "94a22268b77eae35d7b272c00bf8981efe820018",
        "snapshot_version": "12.1.5.70575",
        "snapshot_hash": "d718abcc42604d8efbd35aecfa5bb75b",
    },
    "ptr": {
        "product": "wowt",
        "last_change_revision": "5dbc00c694af6c8a9ae386aa525f36d97bd3cdf4",
        "snapshot_version": "12.1.7.70493",
        "snapshot_hash": "fc3eaa61ff0ade6bf35dbab9727bc161",
    },
    "wowforever": {
        "product": "wow_classic_beta",
        "last_change_revision": "3b471c4d6eb37459cd6ed2df84db18fef4d5f1b3",
        "snapshot_version": "1.60.1.70532",
        "snapshot_hash": "ad1802f4455e427ed8b7769aab25b0c0",
    },
}
FLAG_BITS = {"account": 1, "character": 2, "locked": 4, "secure": 8, "readonly": 16}
ROOT = Path(__file__).resolve().parents[1]
BASE_URL = "https://raw.githubusercontent.com/wowless/wowless"
LICENSE_PATH = "data/cvars-info/LICENSE.wowless"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def source(path, revision, cache_dir):
    cache = cache_dir / revision / path if cache_dir else None
    if cache and cache.is_file():
        return cache.read_bytes()
    url = f"{BASE_URL}/{revision}/{path}"
    with urllib.request.urlopen(url, timeout=30) as response:
        data = response.read()
    if cache:
        cache.parent.mkdir(parents=True, exist_ok=True)
        cache.write_bytes(data)
    return data


def parse_flags(data):
    flags = {}
    current = None
    seen = set()
    for number, line in enumerate(data.decode("utf-8").splitlines(), 1):
        if not line or line.lstrip().startswith("#"):
            continue
        if not line.startswith(" "):
            match = re.fullmatch(r"([a-z0-9_.-]+):", line)
            if not match:
                raise ValueError(f"unsupported CVar key at line {number}: {line!r}")
            current = match.group(1)
            if current in flags:
                raise ValueError(f"duplicate CVar: {current}")
            flags[current] = 0
            seen = set()
            continue
        match = re.match(r"^  ([^:\s]+):\s*(.*)$", line)
        if not match:
            raise ValueError(f"unsupported CVar attribute at line {number}: {line!r}")
        key, value = match.groups()
        if key == "default":
            continue
        if key not in FLAG_BITS:
            raise ValueError(f"unsupported CVar attribute at line {number}: {line!r}")
        if current is None or key in seen or value not in ("true", "false"):
            raise ValueError(f"invalid or duplicate flag at line {number}: {line!r}")
        seen.add(key)
        if value == "true":
            flags[current] |= FLAG_BITS[key]
    if not flags:
        raise ValueError("empty CVar catalog")
    return flags


def parse_build(data):
    fields = {}
    for line in data.decode("utf-8").splitlines():
        key, value = line.split(": ", 1)
        fields[key] = value.strip("'")
    return fields


def encode_json(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")


def validate_snapshot(profile, build):
    config = PROFILES[profile]
    expected_version = config["snapshot_version"]
    version = f"{build['version']}.{build['build']}"
    if version != expected_version:
        raise ValueError(
            f"{profile}: native snapshot {version} differs from pinned {expected_version}"
        )
    if build.get("hash") != config["snapshot_hash"]:
        raise ValueError(f"{profile}: native snapshot build configuration differs from pinned")
    return version


def encode_rust_flags(profile, version, flags):
    lines = [
        f"//! Native CVar flags for {profile}, snapshot {version}.",
        f"//! Source: wowless/wowless at {REVISION}.",
        "//! Regenerate with `python tools/gen_cvar_info.py`.",
        "//! MIT-licensed data; see `data/cvars-info/LICENSE.wowless`.",
        "",
        "#[rustfmt::skip]",
        "pub static CVAR_FLAGS: phf::Map<&'static str, u8> = phf::phf_map! {",
    ]
    lines.extend(f'    "{name}" => {flags[name]}u8,' for name in sorted(flags))
    lines.append("};")
    return ("\n".join(lines) + "\n").encode("utf-8")


def generate(cache_dir):
    outputs = {}
    profiles = {}
    for profile, config in PROFILES.items():
        product = config["product"]
        changed_revision = config["last_change_revision"]
        path = f"data/products/{product}/cvars.yaml"
        build_path = f"data/products/{product}/build.yaml"
        raw = source(path, REVISION, cache_dir)
        build_raw = source(build_path, REVISION, cache_dir)
        changed_build_raw = source(build_path, changed_revision, cache_dir)
        build = parse_build(build_raw)
        version = validate_snapshot(profile, build)
        flags = parse_flags(raw)
        encoded = encode_rust_flags(profile, version, flags)
        output_path = f"crates/sim-data-profile/data/cvar_info_{profile}.rs"
        outputs[output_path] = encoded
        profiles[profile] = {
            "product": product,
            "snapshot_build": build,
            "snapshot_version": version,
            "source_url": f"{BASE_URL}/{REVISION}/{path}",
            "source_sha256": digest(raw),
            "build_source_url": f"{BASE_URL}/{REVISION}/{build_path}",
            "build_source_sha256": digest(build_raw),
            "catalog_last_change_revision": changed_revision,
            "catalog_last_change_build": parse_build(changed_build_raw),
            "catalog_last_change_build_url": f"{BASE_URL}/{changed_revision}/{build_path}",
            "catalog_last_change_build_sha256": digest(changed_build_raw),
            "entries": len(flags),
            "output_path": output_path,
            "output_sha256": digest(encoded),
        }
    capture_sources = {}
    for path in ("addon/Wowless/generated.lua", "tools/errsv.lua"):
        raw = source(path, REVISION, cache_dir)
        capture_sources[path] = {
            "url": f"{BASE_URL}/{REVISION}/{path}",
            "sha256": digest(raw),
        }
    outputs[LICENSE_PATH] = source("LICENSE", REVISION, cache_dir)
    outputs["data/cvars-info/provenance.json"] = encode_json({
        "schema": 2,
        "source_repository": "https://github.com/wowless/wowless",
        "source_revision": REVISION,
        "flag_bits": FLAG_BITS,
        "capture_sources": capture_sources,
        "capture_contract": {
            "enumeration": "ConsoleGetAllCommands",
            "query": "C_CVar.GetCVarInfo",
            "return_count": 7,
            "flag_return_positions": {
                "account": 3, "character": 4, "locked": 5, "secure": 6, "readonly": 7,
            },
            "excluded_native_names": ["cache-*", "praisethesun", "ttsusecharactersettings"],
            "false_encoding": "Absent YAML flag fields mean false in the native comparison.",
        },
        "evidence_limits": [
            "Upstream maintained native-query expectations, not decompiled client data.",
            "Snapshot versions and build configurations match immutable native-capture pins; individual flag capture receipts are not bundled.",
            "Catalog last-change builds are recorded separately from current snapshot builds.",
            "Missing names have no captured metadata; zero masks are retained for captured all-false rows.",
            "Defaults and current CVar values are deliberately excluded.",
            "No catalog is supplied for profiles without a matching pinned native snapshot.",
        ],
        "profiles": profiles,
    })
    return outputs


def write_outputs(outputs, check):
    mismatches = []
    for filename, data in outputs.items():
        path = ROOT / filename
        if path.is_file():
            current = path.read_bytes()
            if current == data:
                continue
            if filename == LICENSE_PATH:
                # Preserve an existing Windows checkout's identical notice bytes.
                if current.replace(b"\r\n", b"\n") == data.replace(b"\r\n", b"\n"):
                    continue
        if check:
            mismatches.append(filename)
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    return mismatches


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-dir", type=Path, help="optional downloaded-source cache")
    parser.add_argument("--check", action="store_true", help="compare outputs without writing")
    args = parser.parse_args()
    outputs = generate(args.cache_dir)
    mismatches = write_outputs(outputs, args.check)
    if mismatches:
        parser.exit(1, "Out of date: " + ", ".join(mismatches) + "\n")
    provenance = json.loads(outputs["data/cvars-info/provenance.json"])
    counts = ", ".join(
        f"{profile}={provenance['profiles'][profile]['entries']}" for profile in PROFILES
    )
    print(f"CVar flags: {counts}; " + ("verified" if args.check else "generated"))


if __name__ == "__main__":
    main()
