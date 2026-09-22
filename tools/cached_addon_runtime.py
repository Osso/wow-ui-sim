"""Stage cached addons and describe bounded probes; never execute or download them."""

import hashlib
import json
import stat
import zipfile
from pathlib import Path, PurePosixPath, PureWindowsPath

BASE_ADDONS = frozenset({"Admin", "SimCommands", "TestFramework", "Blizzard_FrameXML"})
OBSERVER_PREFIX = "AUDIT_ADDON\t"
OBSERVER_DONE = "AUDIT_DONE"


def select_cached_projects(index):
    """Keep every indexed project, explicitly blocking missing Forever archives."""
    rows = []
    for project in index["projects"]:
        archive = project.get("archives", {}).get("forever")
        ready = archive is not None and archive.get("status") == "downloaded"
        rows.append(
            {
                "slug": project["slug"],
                "projectId": project["projectId"],
                "status": "ready" if ready else "blocked",
                "archive": dict(archive) if ready else None,
                "reason": None if ready else "No successfully cached Forever archive",
            }
        )
    return rows


def verify_archive(archive_path, expected_sha256):
    """Read and authenticate the cached archive before any staging writes."""
    content = Path(archive_path).read_bytes()
    actual = hashlib.sha256(content).hexdigest()
    if actual != expected_sha256:
        raise ValueError(
            f"SHA-256 mismatch for {archive_path}: expected {expected_sha256}, got {actual}"
        )
    return content


def checked_member_path(info):
    name = info.filename.replace("\\", "/")
    path = PurePosixPath(name)
    if path.is_absolute() or PureWindowsPath(name).drive or ".." in path.parts:
        raise ValueError(f"Unsafe archive member: {info.filename!r}")
    if not path.parts:
        raise ValueError(f"Empty archive member: {info.filename!r}")
    if stat.S_ISLNK(info.external_attr >> 16):
        raise ValueError(f"Archive symlink rejected: {info.filename!r}")
    return Path(*path.parts)


def read_package_files(content):
    import io

    files = {}
    directories = set()
    with zipfile.ZipFile(io.BytesIO(content)) as archive:
        for info in archive.infolist():
            path = checked_member_path(info)
            if info.filename.replace("\\", "/").endswith("/"):
                directories.add(path)
                continue
            data = archive.read(info)
            if path in files and files[path] != data:
                raise ValueError(f"Archive content conflict: {path}")
            files[path] = data
            directories.update(parent for parent in path.parents if parent != Path("."))
    if directories.intersection(files):
        raise ValueError("Archive file/directory conflict")
    roots = sorted({path.parts[0] for path in files if len(path.parts) > 1})
    if not roots:
        raise ValueError("Archive has no package root directories")
    return files, roots


def build_observer(roots):
    """Observe both loaded flags; no LoadAddOn, slash commands, or assertions."""
    # Decimal escapes keep arbitrary UTF-8 names valid Lua 5.1 string literals.
    names = ", ".join(
        '"' + "".join(f"\\{byte:03d}" for byte in name.encode()) + '"' for name in roots
    )
    return f"""for _, name in ipairs({{{names}}}) do
    local loadingOrLoaded, fullyLoaded = C_AddOns.IsAddOnLoaded(name)
    local onDemand = C_AddOns.IsAddOnLoadOnDemand(name)
    local _, _, _, _, reason = C_AddOns.GetAddOnInfo(name)
    print("AUDIT_ADDON", name, loadingOrLoaded == true, fullyLoaded == true, onDemand == true, reason or "")
end
print("AUDIT_DONE")
"""


def reject_symlink_path(path):
    for component in (path, *path.parents):
        if component.is_symlink():
            raise ValueError(f"Staging symlink rejected: {component}")


def validate_writes(files, directories):
    """Validate the complete write set before creating or replacing any content."""
    for directory in directories:
        reject_symlink_path(directory)
        if directory.exists() and not directory.is_dir():
            raise ValueError(f"Staging directory conflict: {directory}")
    for path, content in files.items():
        reject_symlink_path(path)
        if path.exists() and (not path.is_file() or path.read_bytes() != content):
            raise ValueError(f"Staging content conflict: {path}")


def stage_package(archive_path, expected_sha256, isolated_root, repo_addons):
    """Hash-check and stage one package plus its isolated observer/environment."""
    content = verify_archive(archive_path, expected_sha256)
    package_files, roots = read_package_files(content)
    root = Path(isolated_root).absolute()
    addons = root / "Interface" / "AddOns"
    files = {addons / path: data for path, data in package_files.items()}
    states = {
        path.name: path.name in BASE_ADDONS
        for path in Path(repo_addons).iterdir()
        if path.is_dir()
    }
    states.update({name: True for name in roots})
    files[root / "AddOns.txt"] = "".join(
        f"{name}: {'enabled' if enabled else 'disabled'}\n"
        for name, enabled in sorted(states.items())
    ).encode()
    files[root / "load-observer.lua"] = build_observer(roots).encode()
    directories = {root / "fake-install", root / "wtf"}
    for path in files:
        directories.update(path.parents)
    validate_writes(files, directories)
    for directory in sorted(directories, key=lambda path: len(path.parts)):
        directory.mkdir(exist_ok=True)
    for path, data in files.items():
        if not path.exists():
            path.write_bytes(data)
    return {"root": str(root), "roots": roots, "archiveSha256": expected_sha256}


def build_argv(staged, simulator, timeout=60):
    """Return an argv list for the caller's runner, never launch a process."""
    if type(timeout) is not int or not 1 <= timeout <= 90:
        raise ValueError("timeout must be an integer between 1 and 90 seconds")
    root = Path(staged["root"])
    return [
        "env",
        "-u",
        "WOW_SIM_NO_ADDONS",
        f"WOW_SIM_ADDONS_PATH={root}/Interface/AddOns",
        f"WOW_SIM_ADDONS_TXT={root}/AddOns.txt",
        f"WOW_INSTALL_PATH={root}/fake-install",
        f"WOW_DATA_PATH={root}/fake-install",
        f"WOW_SIM_WTF_PATH={root}/wtf",
        "WOW_SIM_CASC=0",
        "timeout",
        str(timeout),
        str(Path(simulator).absolute()),
        "--no-saved-vars",
        "--exec-lua",
        f"@{root}/load-observer.lua",
        "lua-errors",
    ]


def split_error_json(stdout):
    lines = stdout.splitlines()
    for index in range(len(lines) - 1, -1, -1):
        if not lines[index].lstrip().startswith("["):
            continue
        try:
            errors = json.loads("\n".join(lines[index:]))
        except json.JSONDecodeError:
            continue
        if isinstance(errors, list):
            return lines[:index], errors
    raise ValueError("Missing or malformed trailing Lua-error JSON array")


def parse_observations(lines, expected_roots):
    observations = []
    completed = False
    for line in lines:
        if line == OBSERVER_DONE:
            if completed:
                raise ValueError("Duplicate observer completion marker")
            completed = True
        elif line.startswith(OBSERVER_PREFIX):
            fields = line.split("\t")
            if (
                completed
                or len(fields) != 6
                or any(flag not in ("true", "false") for flag in fields[2:5])
            ):
                raise ValueError("Malformed or out-of-order addon observation")
            observations.append(
                {
                    "name": fields[1],
                    "loadingOrLoaded": fields[2] == "true",
                    "fullyLoaded": fields[3] == "true",
                    "loadOnDemand": fields[4] == "true",
                    "reason": fields[5],
                }
            )
    names = [item["name"] for item in observations]
    if (
        not completed
        or sorted(names) != sorted(expected_roots)
        or len(set(names)) != len(names)
    ):
        raise ValueError("Incomplete or duplicate package-root observations")
    return observations


def parse_result(exit_code, stdout, expected_roots):
    """Classify startup evidence only; clean startup is not addon compatibility."""
    result = {
        "status": "incomplete",
        "exitCode": exit_code,
        "observations": [],
        "errors": None,
    }
    try:
        lines, result["errors"] = split_error_json(stdout)
        result["observations"] = parse_observations(lines, expected_roots)
    except ValueError as error:
        result["reason"] = str(error)
        if exit_code != 0:
            result["status"] = "failed"
        return result
    if exit_code != 0 or result["errors"]:
        result["status"] = "failed"
    elif not any(item["fullyLoaded"] for item in result["observations"]) or any(
        not item["fullyLoaded"] and not item["loadOnDemand"]
        for item in result["observations"]
    ):
        result["status"] = "unloaded"
    else:
        result["status"] = "clean-startup"
    return result
