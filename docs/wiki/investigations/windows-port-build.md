# Windows Port Build

Windows can build and start the simulator once default builds avoid the local `iced-dynamic` DLL.

## Content

The first Windows `cargo build --bin wow-sim` attempt reached MSVC linking and failed while producing `iced_dynamic.dll`:

```text
LINK : fatal error LNK1189: library limit of 65535 objects exceeded
```

`iced-dynamic` is only a re-export crate (`pub use iced::*;`) with `crate-type = ["dylib"]`. On Windows, that forces a very large Iced/WGPU DLL link and hits MSVC's object limit. The porting fix is to keep `fast-build` opt-in instead of enabling it in the default feature set. The root crate still depends on upstream `iced` through the normal `gui` feature, while Linux developers can explicitly add `--features fast-build` when they want the dynamic wrapper for faster local incremental builds.

Headless CI uses `cargo test --no-default-features`. That build must not reference GUI-only modules, optional CASC crates, or Windows GUI-only dependencies. GUI/rendering tests are gated with `#![cfg(feature = "gui")]`, the `bench_spellbook` binary requires `gui`, and the `casc_smoke` example requires `casc`.

Verification on Windows:

- `cargo build --bin wow-sim` succeeds.
- `cargo build --bin wow-cli` succeeds.
- `target\debug\wow-sim.exe --no-addons --no-saved-vars` reaches startup, creates a window, and keeps running as an interactive app.
- `target\debug\wow-sim.exe --no-addons --no-saved-vars screenshot -o windows-smoke.webp --width 640 --height 480` exits successfully and writes the screenshot.

Resource discovery is centralized in `src/paths.rs`. Callers use one resolver for:

- WoW install root: `C:\World of Warcraft`
- CASC data root: `C:\World of Warcraft\Data`
- extracted Interface art: `C:\World of Warcraft\_retail_\BlizzardInterfaceArt\Interface`
- AddOns: repo `./Interface/AddOns` first, then installed `_retail_\Interface\AddOns`
- WTF: `C:\World of Warcraft\_retail_\WTF`

The resolver also supports explicit overrides: `WOW_SIM_WOW_PATH`, `WOW_SIM_CASC_PATH`, `WOW_SIM_INTERFACE_PATH`, `WOW_SIM_ADDONS_PATH`, `WOW_SIM_WTF_PATH`, and `WOW_SIM_TEXTURES_PATH`.

Live WTF is treated as a read-only import source. `SavedVariablesManager` skips live WTF import once a simulator-local saved-variable file exists for the addon, and all saves write to simulator storage, not back to the WoW installation. The regression coverage is:

- `test_wtf_source_is_read_only_when_saving`
- `test_local_storage_takes_precedence_over_wtf_import_source`

Known remaining observations from the smoke runs:

- Cargo prints `warn: could not canonicalize path: 'C:\Users\adeia'` in this environment.
- The app still reports the existing `StoreFrame_CheckForFree` Lua startup error during `VARIABLES_LOADED`.
- Vulkan validation/Galaxy overlay layer warnings can appear during screenshot creation, but rendering still completes.
- `target\debug\wow-sim.exe --no-addons lua-errors` now logs `WTF import source (read-only): <ACCOUNT> @ <REALM>/<PLAYER>`, confirming the Windows WTF path is found without making it a write target.

## Release v0.1.33 outcome — verified 2026-10-15

At 12:19–12:20 UTC, public GitHub metadata reports [release run 38689124268](https://github.com/Osso/wow-ui-sim/actions/runs/38689124268) completed with failure: both Linux builds succeeded; both Windows jobs failed Checkout; macOS failed building binaries and Mists macOS failed packaging. Release publication was skipped. Two Linux Actions artifacts are not published Release assets. The tag ref exists, but the public Release-by-tag API returned 404. [Docker run 38689124309](https://github.com/Osso/wow-ui-sim/actions/runs/38689124309) also completed with failure; original Cargo stderr remains unavailable, not explained by the checkout fix.

Saved public API receipts: `/home/osso/.local/state/wow-ui-sim/handoff/release33-state-refresh-20261015/`. This supersedes the earlier in-progress status, not the historical job outcomes. Local portability commits are not proof of a rerun, published replacement, native checkout or current-source release. No tag, release or workflow was mutated.

## Evidence filename portability — 2026-10-15

Tracked evidence filenames contained literal `::`, incompatible with Windows checkout paths. This is separate from the historical Windows build/startup proof above; that proof does not establish checkout of the current evidence namespace.

Main's integration scope replaces literal `::` with `--` in 115 tracked evidence filenames. Raw log bytes remain unchanged. One renamed live receipt, `data/patch-api/evidence/7.0.3-session-2026-10-08/integrated/mists-c-collection-api--test-mount-journal.proof.json`, changes only its `/log` field to the renamed sibling. Its live `artifact-hashes.json` digest is refreshed; five original candidate metadata/live-reference documents receive bounded reference/checksum updates. The receipt exception means not all 115 renamed files remain byte-identical.

Historical `scope`, snapshot keys, source paths and original digests remain historical, not rewritten. Private original bytes are preserved at:

- `/home/osso/.local/state/wow-ui-sim/handoff/portable-path-candidate-next/originals`
- `/home/osso/.local/state/wow-ui-sim/handoff/portable-receipt-pointer-next0700/originals`

Independent static verification of `87499d15e` passed: 115 renamed paths, 114 identical payloads, 121 exact edits, zero lexical/casefold collisions, and 117/117 applicable live manifest links/digests. The original candidate reported the receipt pointer unresolved; the separate amendment closes it without rewriting that historical report. Report: `/home/osso/.local/state/wow-ui-sim/verification/portable-path-adoption-next/report.md`. That audit found 443/444 full-manifest matches: `pass2-five-models-2026-10-12/startup/stderr` had a pre-existing size/hash mismatch. Subsequent comparison to its private original reproduced the retained bytes using exactly three identity substitutions. The [packet correction](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/pass2-five-models-2026-10-12/README.md#retained-stderr-metadata-correction--2026-10-15) separates source provenance from retained integrity without rewriting either log; independent correction verification of `ba38287ab` passed all 132 retained entries, preserving the other 131 unchanged. Report: `/home/osso/.local/state/wow-ui-sim/verification/retention-stderr-correction-next/report.md`. The original audit remains historical. **Native Windows checkout remains unexecuted/open.** No general long-path fix, Docker stderr resolution, or release/tag/push change is claimed.

## Sources

- Private `portable-path-candidate-next/report.md` — original 115-name candidate, five metadata documents, preservation and static-check limits.
- Private `portable-receipt-pointer-next0700/report.md` — separate one-field receipt amendment and live digest refresh; prepared-only status preserved.
- Main's 2026-10-15 integration handoff — bounded integration scope recorded above; no new runtime proof.

- [Cargo manifest](../../Cargo.toml) - root feature selection for `iced`, `fast-build`, and CASC examples
- [iced-dynamic manifest](../../iced-dynamic/Cargo.toml) - dynamic re-export crate that triggered the Windows DLL link
- [path resolver](../../../src/paths.rs) - shared WoW resource discovery for CASC, Interface, AddOns, and WTF
- [saved variables](../../../src/saved_variables.rs) - live WTF import and simulator-local write behavior

## See Also

- [Wiki index](../index.md) — investigation catalog and portability checkpoint
- [[cli-commands]] - smoke-test commands used for build and screenshot verification
- [[rendering-pipeline]] - screenshot path and GPU renderer context
