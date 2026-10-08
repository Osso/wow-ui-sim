# Patch 7.3.0 API audit

Page **553643** refetched 2026-10-08 and pinned to revision **5335723**, timestamp 2017-10-26T17:46:34Z. All three table publications pass; one real sound-alias bug fixed. Eleven occurrence IDs accounted. No new retirements or shims.

## Source and accounting

The ten-line summary names SOUNDKIT, C_ArtifactRelicForgeUI and C_Console. It also announces Blizzard_Console, the Table Inspector and the PlaySound input transition. Default extraction retains every statement, including prose surrounding inventory names.

Existing inventory flags require template references, nested API sections or caption tables; none fits bare `New global table:` / `New API tables:` summaries. Separate `parse_legacy_summary_tables` function and additive `--legacy-summary-tables` option retain exactly those names under New. A behavioral fixture excludes addon/debug prose and similarly shaped lines outside New. No extractor change was needed.

[Register](../../../data/patch-api/sources/7.3.0-wikitext-register.json): three inventory IDs. [Ledger](../../../data/patch-api/sources/7.3.0-page-coverage.json): eleven IDs, including eight extract rows. Three publication rows, four bounded prose rows, three editorial metadata rows and one pending prose row. [Per-line scout](../../../data/patch-api/evidence/7.3.0-session-2026-10-08/p730-extract-scout.json) preserves literal raw/extract coordinates and individual reasons. Counts derive from files.

## Concrete coverage matrix

| Boundary | Handled | Missing/problematic | Proof |
|---|---|---|---|
| Table publications | SOUNDKIT, C_ArtifactRelicForgeUI, C_Console raw/lookup tables | No unspecified member/signature/security parity inferred | Three-row cached prefork sweep; zero gaps |
| SOUNDKIT / PlaySound | Concrete ID 861; subsequent ID 839; old string name rejected without replacing request state | Complete historical key/name similarity lacks a source catalog; audio fidelity and optional argument/return parity not claimed | Bare namespace/global and actual cached Blizzard alias tests |
| Table Inspector | Actual vendor window focuses root `{p730Field=17, child={leaf=23}}`, selects child, navigates backward, closes | Exact `/tinspect` slash routing/consent not proved by direct window entry point | Cached Blizzard_DebugTools lifecycle case; no vendor overrides |
| Blizzard_Console | Current addon load/frame/helpers/history tail behavior | Historical 2017 implementation parity not claimed | Six existing targeted prefork cases |
| Source preservation | All earlier inputs unchanged; 43 integrated registers reproduce | Three inherited extracts remain nonreproducible | Reproduction receipts, exact original-input hashes |

[Problematic contracts](../../../data/patch-api/evidence/7.3.0-session-2026-10-08/p730-problematic-contracts.json) retain exact limitations. The `/tinspect` row stays audit-pending despite bounded window-lifecycle credit. No consent shim was added. Table existence is not used to fabricate unspecified artifact-forge/console behavior.

## Sound alias root cause and model

Blizzard `Blizzard_SharedXML/Mainline/Sound.lua` assigns `PlaySound = C_Sound.PlaySound`. The legacy global previously recorded numeric requests, but `sound_driver_defaults.rs` supplied a no-op namespace member. Full UI therefore lost request recording and string-name rejection. The prefork regression reproduced `None` instead of `Some(861)`; inspector and publication cases already passed.

Moved the existing numeric request implementation to `src/c_api/c_sound.rs`, registered it on C_Sound and reused it from the legacy global. Removed only the replaced PlaySound stubs. Existing 12.1 options code moved unchanged to `c_sound/options.rs` with its epoch gate retained; basic sound requests remain shared across profiles. The model records the requested sound-kit ID and forwards it to an existing audio sink, if present. No Blizzard Lua changed. Bare and cached tests cover the distinct pre-/post-addon-load boundaries.

## Retirement scans

**No new retirements.** The source contains no removal/rename statements or member retirement candidates. Removing global/namespace stubs here replaces behavior, not the public APIs.

[Complete scans](../../../data/patch-api/evidence/7.3.0-session-2026-10-08/p730-scans.json) use `/usr/bin/grep -R -n -w -F` for all named identities and PlaySound in cached retail AddOns, excluding `*Documentation*`, and src/tests. Bare-name scans include `pcall(Name, ...)` and `and Name then` without syntax filtering; stdout/stderr/exits are retained untruncated. Cached SOUNDKIT, artifact namespace and PlaySound consumers are preserved. C_Console has no cached qualified-name hits, but existing simulator console consumers remain; no namespace retirement is inferred from absence.

[Later-register snapshots](../../../data/patch-api/evidence/7.3.0-session-2026-10-08/p730-later-register-scan.json) read master `af7a101e2` and p732-page `ac94f5a4a` Git blobs only. No other worktree changed. The historical snapshot used a 7.3.2 placeholder. Commit `045259220` replaces it with the merged register from master `85c2acb2d`; the integrated own sweep still has zero gaps. No supersession closure or new replacement exception is needed.

## Historical verification

Owned target: `/home/osso/.cache/wow-ui-sim-targets/p730-page`. Every command uses the explicit owned cwd. Long Cargo commands launch asynchronously with full logs; no polling/wait loop, full integration suite, agents/model CLIs, push or merge.

| Command/filter | Result |
|---|---|
| `cargo test --test prefork_full_ui -- publication_sweep` | 43/43: 42 pages plus factory regression |
| `cargo test --test prefork_full_ui -- patch_7_3_0` | 3/3: sound transition, inspector lifecycle, table publications |
| Prefork `blizzard_console_` / `blizzard_debug_tools_` / `blizzard_deprecated_sound_` | 6/6, 11/11, 4/4 |
| Integration `patch_7_3_0_sound_model::` / `utility_api::test_sound_` / `soundkit_ig_inventory_rotate_character::` | 1/1, 2/2, 2/2 |
| Integration `patch_12_1_0_struct_shapes::patch_12_1_0_play_sound` | 1/1 options regression |
| Lib `sound_driver_defaults::tests::` | 2/2 |
| Python extractor / generator / validator fixtures | 33/33, 25/25, 8/8 |
| Every retained register / extract | 42/42 registers; 39/42 extracts; inherited 12.0.5, 12.0.7, 12.1.0 failures unchanged |
| Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Exit 0; zero non-vendor warnings |
| `cargo fmt --check` | Exit 0 |
| Built wow-sim; `timeout 90 ... --no-addons --no-saved-vars lua-errors` | Exit 0; startup `[]` |
| Negative table substitution | Expected exit 1; exactly one failure, 0 → 1; no resolved IDs |
| Every validator in a second checkout | 19/19 pass, including this page |

[Proof ledger](../../../data/patch-api/evidence/7.3.0-session-2026-10-08/p730-proof-ledger.md) pins commands, revisions, source scopes, exits and log hashes. Initial unsupported `u32` eval conversion, real sound-model RED, empty options-filter selection and pre-fix Mists/format receipts are explicitly non-acceptance evidence. Corrected nonempty options filter passes. Vendor iced manifest deprecations remain unsuppressed. No non-vendor warnings remain. Rust changes were manually audited for readability; short named registration/model paths, unchanged options behavior, no suppressions.

## Historical validator

[Validator](../../../data/patch-api/evidence/7.3.0-session-2026-10-08/validate.py) passes, is read-only, uses `historical_registers` / historical sweep paths at the recorded runtime revision, derives counts from files, and has no absolute cwd/target equality gate. Accounting is pinned to its Git revision. New earlier-page audits do not enlarge historical proof scope; complete Git history remains required. Retained hashes reject arbitrary evidence/source changes.

## Integrated refresh

Rebased onto master `85c2acb2d`, including the completed 7.3.2 audit. Fresh proof uses [integrated evidence](../../../data/patch-api/evidence/7.3.0-session-2026-10-08/integrated/) and fixed historical register/sweep scope at `b13975944`. Original sealed receipts remain unchanged. All 43 registers and 40/43 extracts reproduce with their recorded/inherited flags; the same three inherited extract failures remain. The shared receipt-extension tool adds the real 7.3.2 register and two-row sweep. Fresh negative-control, all-sweeps, caller, sound/menu/popup/panel, Mists and branch/master startup checks run separately; their final results are recorded in the integrated command ledger.

## Sources

- [Pinned source/provenance](../../../data/patch-api/sources/7.3.0-api-changes.provenance.json).
- [Evidence directory](../../../data/patch-api/evidence/7.3.0-session-2026-10-08/).
- [Spec](../../specs/patch-7-3-0-publication-sweep.md).

## See Also

- [[patch-8-0-1-api-audit]] — prior-page template and reproduction recipes.
- [[patch-audit-validator-portability]] — fixed historical scope and input-drift policy.

[Portability receipt](../../../data/patch-api/evidence/7.3.0-session-2026-10-08/p730-validator-portability.json) proves a separate temporary clone passes while invoked from the original cwd; adding future audit register/sweep files does not enlarge the historical scope. Source-byte tampering fails before restoration. All nineteen cloned validators pass, retained evidence is unchanged, and the temporary clone is removed. No existing other worktree was touched.
