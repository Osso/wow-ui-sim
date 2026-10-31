# Updating Blizzard UI to a New WoW Patch

When the selected WoW target updates (e.g. 12.0.5 → 12.0.7), the committed
Blizzard UI manifests and the CASC listfile may go stale: file paths get renamed or
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
| Per-profile file **list** (`retail.txt`, …) | Gethe branch tree; exact pinned revision for Wrath | `data/blizzard-ui-files/<profile>.txt` |
| File **content** | pinned Blizzard CDN ranges for Retail and PTR; local WoW CASC for other profiles except Wrath; pinned Gethe snapshot for Wrath | `~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns`, plus sibling `FrameXML` for Wrath |
| `path → FDID` map | community listfile + authoritative overrides | `data/wow-ui-sim-listfile.csv` (generated) |
| Extra or canonicalized paths | hand-maintained overrides | `data/listfile-overrides.csv` |

Profile → Gethe branch:

| Profile | Branch |
|---------|--------|
| retail | `live` (immutable revision in `data/blizzard-ui-builds/retail.json`) |
| ptr | `ptr` (immutable revision in `data/blizzard-ui-builds/ptr.json`; [selected identity](ptr-cdn-content-index.md#selected-identity-and-regeneration-evidence)) |
| mists | `classic` (current Classic = Mists of Pandaria) |
| era | `classic_era` |
| anniversary | `classic_anniversary` |
| wowforever | `forever` (current selected target 1.60.1.70532; distinct runtime profile, cache, and manifest) |

## Retail selected target — October 31, 2026

Retail sync pins `wow` **12.1.5.70575** in `data/blizzard-ui-builds/retail.json`. Its build, CDN and install keys and all 4,026 file ranges come from Blizzard metadata; runtime content comes only from the Blizzard CDN. The installed client build does not select or invalidate this cache. Sync verifies each cached content key and retains matching files, including a verified restored cache with older provenance. It writes pinned provenance after all files match.

To retarget, regenerate the manifest and CDN index for the same immutable build, verify every downloaded range and decoded content key, and update the pin and manifest together. Do not use Gethe content as a runtime fallback.

## Historical Retail target — October 16, 2026

Retail's selected installed product is `wow` **12.1.5.70448**, active build key
`88d5e8ef05ec5a8446cdec93d142d7c6`. Gethe `live` revision
`b9be9379719047977fca4dba1bb1d9988e8840b3` reports that exact version;
its full manifest contains 4,026 paths (+34/-49 from the previous manifest).
The refreshed limited listfile maps every entry and contains 146,699 rows
(+0/-17). Keep installed identity, source identity and runtime acceptance separate.

When another gate uses the shared Retail cache, **never sync it in place**.
`blizzard_ui_sync::default_cache_addons_path()` uses `dirs::cache_dir()`, so set
an absolute private `XDG_CACHE_HOME` on both the sync and every subsequent test
or startup process. Runtime files then live beneath
`$XDG_CACHE_HOME/wow-ui-sim/blizzard-ui/retail/AddOns`. Validate its provenance's
product, version, full build key and compiled manifest SHA256. Keep the original
CVar-copy helper and child-only `XDG_DATA_HOME`; cache isolation does not replace
CVar isolation. No changes to the shared cache or vendor Lua are required.

The root install `.build.info` identifies installed PTR (`wowxptr`) 12.1.5.70340,
not 70448; `_ptr_/.build.info` is absent. This Retail update does not move PTR's
manifest, installed target, source pin or feature closure.

Retail 70448 also requires its [separate GlobalStrings table](specs/client-profiles.md).
The committed CSV/provenance contains 26,069 authenticated unencrypted shipped
rows; 65 unreadable encrypted-section rows are omitted explicitly, not filled
with fabricated text. Its WDC5 layout remains `D40F6D96`, with three sections.
The existing multi-section DB2 reader and multiline-safe repo PHF generator are
used; the old shared and Forever string tables remain unchanged. The
[bounded Retail acceptance](specs/client-profiles.md#retail-70448-bounded-execution)
records clean isolated startup, all 265 requested preservation cases and executed
12.1.5 behavior tests after rebase onto `9c0251fca`. No Retail baseline JSON
previously existed; no other profile's baseline is rewritten.

## Matching the installed Forever build

### Current selected target — October 25, 2026

The selected non-beta product is `wowf` **1.60.1.70532**, build key
`2582289d81e380163c7a13472df4cb81`; its build config identifies
`WOW-70532patch1.60.1_Retail` / `build-uid = wowf`.
After desktop Scan and Repair, read-only latest-generation index inspection
selects live root and encoding entries. Syncthing `wow-install` reached idle
with needFiles/needBytes zero; all 16 newest desktop index names and sizes
match locally. Independent local BLTE header/chunk verification and decoded
MD5 checks pass for root `6e2657526d320facacb6f3dfff62d8a2` and encoding
`7a54801f1a74da76cef250d8dd12351e`; the metadata/index/archive snapshot
remained unchanged. Verified October 25, 2026, 01:18 UTC.

`src/asset_resolver_config.rs` now selects `wowf` for Forever; its product
regression expects `wowf`. Historical beta fixtures remain unchanged.
The new selector's Rust execution/check proof, lane-private offline cache sync,
provenance, baseline and startup are pending a released build slot after
club-unread-integrate and S49. No runtime acceptance is claimed.
The manifest and baseline remain pending their valid capture/co-commit.
No CDN, borrowed keyring/root, fabricated data or decompiled source is used.

### Historical beta selection — October 24, 2026

User decision `forever_target_build_20261024` starts a new acceptance epoch for
installed `wow_classic_beta` **1.60.1.70532**, active build key
`ad1802f4455e427ed8b7769aab25b0c0`. Read-only `.product.db` decoding selects
that exact record; its build config independently identifies
`WOW-70532patch1.60.1_Beta` and hashes to the selected build key. This proves
installed metadata/config identity, not complete or playable content.

Gethe `forever` revision `ca8368f2f2380cbe696854f7f2646387c278a61b`
reports `1.60.1.70532`. The unmodified generator produces the full **4,486-path**
manifest (+55/-3 from 70374); all 4,486 copied source blobs match the Git tree.
The refreshed community listfile and explicit-source limited generator produce
146,753 rows (+55/-1), with no unmapped Forever manifest entry.
Forever has no profile-only sync exclusion or hardcoded required-entry list.

Runtime acquisition/provenance, baseline recapture and all-target warning proof
are pending an authorized build slot and main's asset-resolver absent-keyring-row
fix. Do not add a synthetic `.build.info` row or change the dependency pin in
this source lane. The 70374 receipts below remain historical; neither their
successful acquisition nor any older clean/failing capture establishes 70532
startup acceptance. The manifest and new baseline must be committed together
after the supported sync and byte-exact capture.

<a id="current-selected-target--october-15-2026"></a>

### Historical selected target — October 15, 2026

The installed-product reader reports
`product="wow_classic_beta" version="1.60.1.70374"`, exit 0.
Gethe commit `735496de78ebd30ce816a0637982c556987f258c` has matching build
`70374`; its manifest has 4,434 paths versus the historical 4,398, with 47
added and 11 removed. This is the selected target, not a claim about branch
recency or completed runtime acquisition.

Community-listfile refresh, limited-listfile regeneration, the matching Forever
build and supported local/CDN sync completed: all 4,434 paths were acquired.
However, `Blizzard_LoadLocale/LoadLocale.lua` matched the `ptPT` root variant,
not `enUS`: both pinned local and simulator CDN selection lost locale identity.
The matched native local probe now establishes enUS residency: FDID `5612369`
is indexed, decodes to 138 bytes, matches the enUS content hash, and reports
`hasenUS=true`, `hasptPT=false`. The earlier missing result selected the wrong
variant, not missing local enUS content. See the owning
[locale contract and proof matrix](specs/casc-loading.md#locale-checkpoint--october-15-2026):
candidate selector 5/5 GREEN versus actual pinned resolver 0/5 RED.
The integrated source pins asset-resolver's published enUS fix while retaining
its TACT-aware local decoder, and qualifies simulator CDN/path selection.
Normal binaries compiled at `9a6ea924e` plus dirty 70374 manifest/listfile/BuildInfo metadata completed the supported corrected-enUS sync: 4,434 extracted. Exact selected-root content-key validation matched all 4,434 files, with no missing or mismatched entries; the old cache matched 4,433 with one wrong-ptPT LoadLocale file. See the linked proof matrix for implementation versus pending Rust-test coverage.

The actual corrected-cache headless capture is the [current failing baseline](baselines/README.md#forever-lua-errors), not clean startup. Independent saved locale/runtime receipt verification PASS authenticates acquisition and the failing capture; separate source proof does not establish clean startup or integrated Rust-test GREEN. No native, all-profile or complete encrypted-content acceptance is established.
Preserve historical receipts below; do not relabel old 69977 captures as 70374 proof.

### Historical selected target — September 24, 2026

On September 24, 2026, the installed `wow_classic_beta` build was `1.60.1.69977`
(`3bd89ce2721f7c75e7525dc83741076f`). Gethe commit
`c6e89983189e4f626f549204a23c2d2bea93080a` has that exact `version.txt`;
the branch tip already reports `70009` and must not replace this selected target.
Prepare the source cache from the matching commit, then use the generator's
`wowforever --no-refresh` mode. Its 4,398-file manifest is unchanged from `69913`;
the source diff changes only three Mainline GlueXML files. The runtime cache
still requires the normal CASC sync and new provenance, not a manual source copy.
At that checkpoint, the simulator's temporary `GetBuildInfo()` default and
profile regression tracked build `69977`; historical captures and atlas-data
provenance retain their original build labels.

## Wrath: separate pinned source route

With `client-wrath` selected, run the same `wow-cli casc sync-blizzard-ui` command. Its [pinned source route](wiki/systems/client-profiles.md#runtime-cache) is separate from Mists and Forever; do not replace either snapshot to prepare Wrath.

The [owning Wrath source contract](specs/client-profiles.md#separate-pinned-wrath-source-restoration) records component proof and remaining full-profile, consumer and native acceptance gates. Component tests do not establish full FrameXML startup; retain the scrolling workaround.

The CASC/listfile procedure below applies to CASC-backed profiles, not Wrath source acquisition.

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

For non-pinned Classic products, sanity-check the reported version and active build key against the requested `.product.db` record. The resolver selects the exact product code through cascette's shared parser (`currentVersionStr` field 7, `activeBuildKey` field 14, optional `activeInstallKey` field 16); it does not use `.build.info` as authority or fallback. This is simulator policy, not a native-launcher or complete-content guarantee. Retail and PTR use their committed build and content indexes instead of the installed product identity.

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
does not change output ordering. Every valid override FDID is included in the
subset even when its path is absent from community data, literal scans, atlas
entries, and Blizzard UI manifests.

The Forever `INV_SideTab_{Currency,Honor_Alliance,Reputation2,Stats}_c60`
icon overrides (FDIDs 8197078, 8197097, 8197103, 8197104) were verified
against the active Forever CASC root's name hashes. That root uses uppercase
backslash paths with direct Jenkins96 ordering; control paths resolved to
FDIDs 134400 and 2447783. This identifies names, not extraction availability:
the first three content keys were locally indexed, while Stats was not.

On September 20, 2026, refreshing the community listfile and regenerating this
file added 592 `wowforever` path-to-FDID rows. Every one of the 4,398 paths in
`data/blizzard-ui-files/wowforever.txt` then mapped, and the rebuilt
`client-wowforever` sync extracted all 4,398 files from local `wow_classic_beta`
CASC. On September 25, `692e36936`/`4eb32befa` made the same selected product's
`.product.db` record authoritative for installed identity, so its absent `.build.info`
row no longer requires repair or Battle.net. This establishes source acquisition and
identity selection only; it does not establish startup, full content, or Blizzard UI
compatibility.

### 4. Rebuild and validate the sync

The manifests and limited listfile are `include_str!`'d at compile time, so
rebuild before syncing. Non-pinned sync compares installed identity and manifest hash with cache provenance. Retail and PTR use pinned build/config and content-index hashes and verify CDN BLTE and decoded-content keys. Retail preserves each matching cached file when provenance differs; PTR invalidates its cache on a provenance mismatch.

```bash
cargo build --bin wow-cli --features tools
# Use a private XDG_CACHE_HOME for source-sync tests.
```

For Retail and PTR, `wow-cli casc sync-blizzard-ui` must report the complete pinned entry count; then run `lua-errors` and capture its baseline. The cache alone is not startup acceptance.

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

Commit each manifest/listfile data regeneration separately with added/removed
counts and exact installed/source identity. Commit producer fixes with their
regressions and owning specs, then document executed startup/preservation proof.
Refresh an existing profile baseline only from the matching isolated runtime,
not from source acquisition alone. PTR 12.1.7 also commits its
regenerated `data/blizzard-ui-builds/ptr.json`; see [PTR CDN content index](ptr-cdn-content-index.md).

## Notes

- Branch-based `~/.cache/wow-ui-sim/wow-ui-source/<branch>` checkouts feed the generator, not runtime loading. Wrath’s separate validated snapshot also feeds its runtime-cache sync; the loader still reads the profile runtime cache.
- Gethe's `live` tree is a **superset** of retail CASC (it ships cross-flavor
  source like `_Classic.toc`). Those extra files still resolve from retail CASC
  once the listfile has their FDIDs, so the manifest stays a full, unfiltered
  branch listing.

## Classic Era/Anniversary GlobalStrings

Both Vanilla profiles select `data/global_strings_classic.rs`: all 19,624 enUS rows from installed `wow_classic_era` **1.15.9.70003**, build key `2417084b8b041ee120d02d4adf31b0bc`. This is a shared selected Vanilla dataset, not a new build-string pin or a Retail/Forever fallback. [Owning contract](specs/classic-global-strings.md).

Local archive provenance: GlobalStrings DB2 FDID **1394440**, content key `5426329b56962158a807759e84de041b`, encoding key `fa1e3b831a0e2a3ffa078b2d1c426370`; decoded DB2 SHA256 `fc1f1a3b41a683c5a3b4b2533840689f501bda3dae3b655e45dec7e49881466f`. CSV SHA256 `3de553113c4cb19c50bec10a233d331917078643e819a61830ff03c32672f2aa`; generated Rust SHA256 `e263e5ed7477ab586ac076a9a3f2b9e2a63d61c3a2f6fae56d2f1d3749c0c668`. The extraction receipt validates installed product/config identity, enUS root selection, local journal/archive location and BLTE/content hashes. Schema metadata `WOWSTATIC_1_15_9_68185` is not the installed build version.

Exact generation command from the private extraction handoff (no install read needed to regenerate from its CSV):

```text
python3 -B /home/osso/.local/state/wow-ui-sim/handoff/era-globalstrings/generate_classic.py --repo /home/osso/Projects/wow/wow-ui-sim --source /home/osso/.local/state/wow-ui-sim/handoff/era-globalstrings/GlobalStrings.csv --output /home/osso/.local/state/wow-ui-sim/handoff/era-globalstrings/generated/data/global_strings_classic.rs
```

The inspectable wrapper reuses only `tools/gen_forever_global_strings.py`'s CSV/PHF generator algorithm (SHA256 `60987fa51e14b0ddaee1f0d80beee7f13542ec54abcd46b908a1016eb0f20e62`), replacing provenance comments, **not Forever data**. It preserves all three multiline rows; the legacy line-oriented Rust generator is unsuitable. Handoff `era-globalstrings/README.md`, `.provenance.json`, `validation.json` and sealed CSV own the full offline reproduction chain.

`OPTION_TOOLTIP_HD_TEXTURES` is absent from this shipped base DB2. Do not substitute the distinct `HD_TEXTURES_BUTTON_TOOLTIP`, copy a foreign build's row, or relax consumers. Record any resulting error as remaining. Hotfixes and native global-publication behavior are not established by base DB2 extraction.
