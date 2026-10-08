"""Extend a patch audit's validation receipts to cover registers merged after it was written.

Usage: python3 extend_receipts.py <worktree> <evidence-dir> <prefix> <note>
Adds missing register-reproduction rows (byte-verified regeneration with recorded flags),
missing per-sweep results files (by running the prefork sweep with its out_env) and
missing/refreshed sweep-summary rows. Refreshes the page's own results file too.
"""
import filecmp, glob, hashlib, json, os, re, subprocess, sys

W, E, PREFIX, NOTE = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
SRC = f"{W}/data/patch-api/sources"


def key(patch):
    return [int(t) for t in patch.split(".")]


def dump(path, value):
    with open(path, "w") as fh:
        json.dump(value, fh, indent=2, ensure_ascii=False)
        fh.write("\n")


def patches():
    return sorted((os.path.basename(p).removesuffix("-wikitext-register.json")
                   for p in glob.glob(f"{SRC}/*-wikitext-register.json")), key=key)


def provenance(pt):
    prov = json.load(open(f"{SRC}/{pt}-api-changes.provenance.json"))
    return prov.get("generator_flags", []), str(prov.get("revid") or prov["wikitext"]["revid"])


def extend_reproduction():
    path = f"{E}/{PREFIX}-register-reproduction.json"
    if not os.path.exists(path):
        return
    rows = json.load(open(path))
    have = {r["patch"] for r in rows}
    for pt in patches():
        if pt in have:
            continue
        flags, rev = provenance(pt)
        out = f"{W}/target/patch-api-audit-registers/{pt}-wikitext-register.json"
        os.makedirs(os.path.dirname(out), exist_ok=True)
        cmd = ["python3", f"{W}/tools/gen_patch_wikitext_register.py", pt,
               f"{SRC}/{pt}-api-changes.wikitext", rev, out, *flags]
        subprocess.run(cmd, check=True, cwd=W)
        reg = f"{SRC}/{pt}-wikitext-register.json"
        row = dict(rows[0])
        row.update({"patch": pt, "byte_identical": filecmp.cmp(out, reg, shallow=False),
                    "sha256": hashlib.sha256(open(reg, "rb").read()).hexdigest()})
        for k in row:
            if "flag" in k and k != "flags_basis":
                row[k] = flags
        if "flags_basis" in row:
            row["flags_basis"] = NOTE
        if "command" in row:
            row["command"] = cmd
        if "exit" in row:
            row["exit"] = 0
        rows.append(row)
        print("receipt", pt, row["byte_identical"])
    dump(path, sorted(rows, key=lambda r: key(r["patch"])))


def run_sweep(pt):
    stem = "patch_" + pt.replace(".", "_") + "_publication_sweep"
    src = open(f"{W}/tests/{stem}.rs").read()
    out_env = re.search(r'out_env: "([A-Z0-9_]+)"', src).group(1)
    out = f"{E}/{stem}-results.json"
    env = dict(os.environ, **{out_env: out})
    r = subprocess.run(["cargo", "test", "--test", "prefork_full_ui", "--", stem],
                       cwd=W, env=env, capture_output=True, text=True)
    assert "1 passed; 0 failed" in r.stdout + r.stderr, f"{stem} failed"
    return json.load(open(out))


def extend_summary(own):
    path = f"{E}/{PREFIX}-sweep-summary.json"
    if not os.path.exists(path):
        return
    rows = {r["patch"]: r for r in json.load(open(path))}
    for pt in patches():
        if pt in rows and pt != own:
            continue
        res = run_sweep(pt)
        gaps = sum(not v["ok"] for v in res.values())
        rows[pt] = {"patch": pt, "rows": len(res), "ok": len(res) - gaps, "gaps": gaps,
                    "result": "pass"}
        print("summary", pt, rows[pt])
    dump(path, sorted(rows.values(), key=lambda r: key(r["patch"])))


if __name__ == "__main__":
    extend_reproduction()
    extend_summary(sys.argv[5] if len(sys.argv) > 5 else None)
