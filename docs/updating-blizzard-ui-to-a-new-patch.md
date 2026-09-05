# Updating Blizzard UI to a New WoW Patch

When the installed WoW client updates (e.g. 12.0.5 → 12.0.7), the committed
Blizzard UI manifests and the CASC listfile go stale: file paths get renamed or
moved into flavor subdirs (`Mainline/`, `Vanilla/`, …), addons are added/removed,
and new files won't resolve because their `path → fileDataID` mapping is missing.

The fix is to re-derive the manifests from the **Gethe wow-ui-source** repo (the
authoritative per-build file list) and refresh the **community listfile** (the
authoritative `path → FDID` map). Do **not** use the community listfile as the
*file list* source — it lags new files after a patch. Use it only for FDID
resolution.

## Sources of truth

| Data | Source | Where it lives |
|------|--------|----------------|
| Per-profile file **list** (`retail.txt`, …) | Gethe branch tree | `data/blizzard-ui-files/<profile>.txt` |
| File **content** | local WoW CASC for retail/Classic; pinned Blizzard CDN ranges for PTR 12.1.5 | extracted to `~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns` |
| `path → FDID` map | community listfile + authoritative overrides | `data/wow-ui-sim-listfile.csv` (generated) |
| Extra or canonicalized paths | hand-maintained overrides | `data/listfile-overrides.csv` |

Profile → Gethe branch:

| Profile | Branch |
|---------|--------|
| retail | `live` |
| ptr | `ptr2` (pinned at `49b69918fcdc77e109813281e4f537d45ec7dcbf` for 12.1.5.69594) |
| mists | `classic` (current Classic = Mists of Pandaria) |
| era | `classic_era` |
| anniversary | `classic_anniversary` |

## Steps

Run from the repo root. Example targets the `retail` profile; substitute the
profile you're updating.

### 1. Refresh the community listfile (`path → FDID`)

```bash
curl -fSL --retry 3 \
  -o ~/.cache/asset-resolver/data/community-listfile.csv \
  https://github.com/wowdev/wow-listfile/releases/latest/download/community-listfile.csv
```

This is the semicolon-separated `fdid;path` list maintained by wowdev. It's the
only source that maps brand-new files (added this patch) to their FDIDs.

### 2. Pull the Gethe tree + regenerate the manifest(s)

`tools/gen_blizzard_ui_manifest.py` shallow-fetches the profile's Gethe branch to
`~/.cache/wow-ui-sim/wow-ui-source/<branch>` and writes the **full** file list
(no filtering) to `data/blizzard-ui-files/<profile>.txt`.

```bash
python3 tools/gen_blizzard_ui_manifest.py retail          # refresh + regenerate
python3 tools/gen_blizzard_ui_manifest.py                 # all profiles
python3 tools/gen_blizzard_ui_manifest.py retail --no-refresh   # reuse existing cache
```

For retail/Classic, sanity-check the reported version against the installed client build
(`.build.info` → active product `Version`). PTR 12.1.5 is different: its committed
`data/blizzard-ui-builds/ptr.json` pins the official `wowxptr` build/config identity
and Gethe revision because this machine has no matching local PTR install.

### 3. Regenerate the bundled limited listfile

Combines the refreshed community listfile with `data/listfile-overrides.csv` and
filters to the assets the simulator needs (driven partly by the manifests from
step 2).

```bash
python3 tools/gen_limited_listfile.py
```

The generator stores ordinary community rows with normalized lowercase paths. An
entry in `data/listfile-overrides.csv` is authoritative for its normalized path
and FDID: it replaces the source display path while preserving slash-normalized
canonical casing. Generated rows sort by normalized path, so canonical casing
does not change output ordering.

### 4. Rebuild and validate the sync

The manifests and limited listfile are `include_str!`'d at compile time, so
rebuild before syncing. Retail/Classic sync compares its active profile, CASC product,
active `.build.info` identity, and compiled manifest hash with cache provenance. PTR
12.1.5 instead compares its pinned `wowxptr` build/config, Gethe revision, manifest,
and content-index hashes; it downloads exact CDN archive ranges and verifies both BLTE
and decoded-content keys. A mismatch automatically removes only that profile's
`AddOns` cache before extraction; do not delete the cache manually.

```bash
cargo build --bin wow-cli
# Retail-only isolation test (fresh cache, non-retail flavors masked):
python3 scripts/test-retail-casc-isolation.py
```

A pass means every retail manifest entry resolves against installed retail CASC.
For PTR 12.1.5, `wow-cli casc sync-blizzard-ui` must report the complete pinned entry
count; then run `lua-errors` and capture its baseline. The cache alone is not startup
acceptance. The isolation script reports the exact list of any retail `missing or
unusable` entries.

### 5. Resolve residual misses

For each remaining miss:

- **Missing from the listfile** (`path → FDID` unknown): look up the FDID (e.g.
  on <https://wago.tools>) and add a verified `FDID;interface/addons/<path>` line to
  `data/listfile-overrides.csv`, then rerun step 3. Use an override as well when
  the generated entry must retain a canonical path casing.
- **Stale content-usability guard**: some entries have a content check in
  `src/blizzard_ui_sync/profile_cache.rs` (`retail_cache_entry_is_usable`, the
  Mists variants) and in `scripts/test-retail-casc-isolation.py`
  (`RETAIL_CONTENT_REQUIREMENTS`). If a patch legitimately changes a guarded
  file, update the guard's marker to a string present in the new file. (Example:
  12.0.7 split `RuneforgeUtil.lua`/`.xml`, dropping the `<Script>` include, so
  the guard moved to `name="RuneforgeCovenantSigilTemplate"`.)
- **Genuinely removed content**: the file is gone this patch — it will simply be
  absent from the regenerated manifest (nothing to do).

### 6. Update the profile-specific required/hardcoded lists

If a patch renames files referenced by the hardcoded lists or profile filters in
`src/blizzard_ui_sync/profile_cache.rs`, update them to the new paths and verify
that every file referenced by an active profile's manifest or TOC is included by
that profile's sync filter and required-entry set. Do not retain a stale
profile-only exclusion: retail 12.1's `Blizzard_CooldownBroadcaster_Bootstrap.lua`
was incorrectly filtered as PTR-only, allowing a completed retail cache without a
TOC-required file.

### 7. Re-capture baselines and commit

```bash
# After the runtime cache is synced, re-capture the Lua-error baseline:
wow-sim --no-addons --no-saved-vars lua-errors 2>/dev/null > docs/baselines/retail-lua-errors.json
```

Commit the manifest(s), `data/wow-ui-sim-listfile.csv`, any
`data/listfile-overrides.csv` additions, guard/list updates, and the refreshed
baseline together — they are one logical patch bump. PTR 12.1.5 also commits its
regenerated `data/blizzard-ui-builds/ptr.json`; see [PTR CDN content index](ptr-cdn-content-index.md).

## Notes

- The `~/.cache/wow-ui-sim/wow-ui-source/<branch>` tree is a plain checkout, not
  wired into the runtime; only the generator reads it.
- Gethe's `live` tree is a **superset** of retail CASC (it ships cross-flavor
  source like `_Classic.toc`). Those extra files still resolve from retail CASC
  once the listfile has their FDIDs, so the manifest stays a full, unfiltered
  branch listing.
