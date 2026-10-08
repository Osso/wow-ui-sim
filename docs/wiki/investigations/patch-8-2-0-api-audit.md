# Patch 8.2.0 API page audit

Warcraft Wiki page **533235**, current revision **5142162** (2021-04-23T01:19:12Z), fetched 2026-10-08. Pinned raw source, revision response, successful HTTP headers and failed initial fetch/browser attempts are retained. Audit against current retail, not reconstructed historical WoW. Branch `p820-page`; runtime proof revision `e0d115f55`.

## Source and parser boundary

[Provenance](../../../data/patch-api/sources/8.2.0-api-changes.provenance.json) records opt-in `--legacy-api-tables` for generator and extractor. Both previous default behaviors remain unchanged. Older source puts its global inventory under `API/Changes`, repeats widget tables for handlers, and repeats console tables under CVars. Heading-only extraction would lose globals and misclassify second-table columns. Caption-based parsing handles each table independently, preserves original source line IDs and numerical headers, and identifies commands and widget handlers separately. Standalone reference metadata is retained rather than dropped.

Register contains **193 inventory occurrences**, including six Checkout handlers and five console commands. Every header count agrees with its own table. [Ledger](../../../data/patch-api/sources/8.2.0-page-coverage.json) accounts for **223 unique IDs**: 193 inventory, 24 extract and six comparison-caption contexts. Statuses: 107 partial-development-green, 33 bounded-coverage, 63 audit-pending, 20 metadata-only. Ten substantive prose contracts remain pending; source/build/TOC links carry no runtime credit. [Extract scout](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-extract-scout.json) maps every retained statement to literal raw/extract lines.

## Concrete coverage matrix

| Boundary | Handled | Still missing / disabled | Proof |
|---|---|---|---|
| Publication/absence | 140 OK / 193 inventory IDs | 53 exact reviewed failures | Full-UI prefork raw/ordinary lookup, HasScript, event registration and console/CVar queries |
| Voice ducking | Normalized `SetMasterVolumeScale` updates existing voice state; getter round-trips; output volume unchanged | Native invalid-range and secret-argument parity unproven | Concrete 0, .37, 1, .62 writes; rejected inputs leave state unchanged; bare and cached UI |
| Retirement | One obsolete widget query absent repeatedly on retail | No retirement of any current consumer | Whole-word untruncated grep, bare/cached lookup and Mists preservation |
| Probe correctness | Colon-qualified handler owner parsed correctly | Five Checkout handlers still rejected by HasScript | Frame:OnShow behavioral RED/GREEN; Checkout:OnEscapePressed receives publication-only credit |
| Prose | All ten contracts retained literally | Path sandbox/fonts/audio, protected report/panels, nameplate family, tooltip textures, widget migration and fstack output | Exact pending reasons; no historical behavior parity asserted |

Initial discovery: **62 gaps**. State-backed voice setter, bounded retirement and corrected Checkout:OnEscapePressed probe close three publication failures. Handler support is not a modeled Checkout event lifecycle. [Per-ID review](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-gap-review.json) records all 53 exact literal source lines, expectations, observations and reasons. [Complete caller scout](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-gap-caller-scout.json) retains all src/tests hits, not just samples.

Remaining gaps require Battle.net friend favorites; club streams/finder settings and caches; essence deactivation permission policy; journal previews; selected upgrade/inspect/mount-equipment state; match scoreboard/rewards/classification; disenchant metadata; visualization records; channel join requests; movement gates; POI/splash eligibility; host commands/window updates; Checkout callback/clipboard lifecycle; tabard asset producers; and an unspecified historical CVar default. Temporary mount-equipment no-op is not promoted as a solution. Tabard 3D model rendering remains intentionally unsupported. Exact per-member reasons are in the review, not replaced by namespace autostubs or guessed constants.

## Retirement evidence

`C_UIWidgetManager.GetTextureWithStateVisualizationInfo`: **zero qualified cached hits, zero bare cached hits, zero pre-change src/tests hits**. Added separate `RETIRED_8_2_0_MEMBERS` under the existing retail-epoch retirement gate. Classic profiles do not load that module. Bare and repeated lookup no longer fabricates the member; unchanged cached Blizzard full UI produces no extra Lua errors. Mists existing ordinary lookup remains callable.

[Removal scans](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-removal-consumers.json) cover **all 21 source removals**, including qualified and bare names in the cached retail AddOns tree excluding `*Documentation*`, and complete `src/`/`tests/` caller scans. Scanner is `/usr/bin/grep -w`, not rg. Bare-name scans include indirect `pcall(Name, ...)` and `and Name then` references. [After-change scans](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-whole-callers-after.json) retain new owned declarations/tests. [Decisions](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-retirement-decisions.json) distinguish the single changed member from already absent/superseded/deprecated identities. No current cached consumer was retired. Blizzard deprecation wrappers remain untouched.

## Verification commands

All commands use `/home/osso/.worktrees/wow-ui-sim-p820-page`; Cargo uses `CARGO_TARGET_DIR=/home/osso/.cache/wow-ui-sim-targets/p820-page`. Long command stdout/stderr is retained in logs; no polling, agents, push, merge or full integration suite.

| Command | Result |
|---|---|
| `cargo test --test prefork_full_ui -- publication_sweep` | 38/38: all 37 sweeps plus factory regression, 8,170 inventory rows |
| `cargo test --test integration patch_8_2_0_ -- --nocapture` | 3/3 bare volume/retirement/handler cases |
| `cargo test --test prefork_full_ui -- patch_8_2_0_cached_` | 1/1 cached volume/retirement; unchanged Lua error count |
| `cargo test --test integration c_voice_chat_probes:: -- --nocapture` | 25/25 existing voice regressions |
| `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_8_2_0_mists_ -- --nocapture` | 1/1 legacy preservation |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Exit 0, zero non-vendor warnings |
| `cargo check` | Exit 0, zero non-vendor warnings |
| `cargo fmt`, `cargo fmt --check` | Exit 0 |
| `python3 tools/test_extract_patch_non_inventory.py` | 28/28 |
| `python3 tools/test_gen_patch_wikitext_register.py` | 20/20 |
| Generator per-source commands with recorded or explicitly inferred flags | 37/37 byte-identical registers |
| Extractor `--patch <patch> --text-only --check` with recorded or explicitly inferred flags | 34/37; three inherited failures unchanged |
| `P820_SWEEP_REGISTER=<negative> cargo test --test prefork_full_ui -- patch_8_2_0_publication_sweep` | Expected exit 1: exactly AZERITE_ESSENCE_ACTIVATED adds one gap, 59 → 60; zero resolved IDs |
| `cargo build --bin wow-sim`, then `timeout 90 <target>/debug/wow-sim --no-addons --no-saved-vars lua-errors` | Both exit 0; startup `[]` |
| `python3 data/patch-api/evidence/8.2.0-session-2026-10-08/validate.py` | PASS; counts derived from files |

[Proof ledger](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-proof.json) indexes exact commands/revisions/scopes/exits/log hashes, including expected discovery/behavior REDs and a separately labeled fixed import compilation failure. Failed initial extractor fixture is corrected to retain its editorial Widgets heading; it is not claimed passing. [Negative receipt](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-negative-result.json), [sweep summary](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-sweep-summary.json), [readability](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-readability.md). Metric CLI unavailable; changed Rust manually audited. Inherited iced manifest deprecations remain unsuppressed.

## Preservation and unfinished boundaries

All **188 prior source/register/ledger inputs** remain byte-identical; **74 prior extraction-mode outcomes** remain unchanged. [Register reproduction](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-register-reproduction.json) contains every exact generator command. [Saved-extract reproduction](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-saved-extract-reproduction.json) records every check command, stdout and error. Existing **12.0.5, 12.0.7 and 12.1.0** extract failures predate this branch and are not silently rewritten; validator derives exceptions from the completed 8.3.0 receipts.

**8.2.5 merged in `127aa3724`**: `later_registers` includes its real register (integration commit `9009f9c79`). Four ClubFinder applicant-list members, `C_Commentator.GetElapsedMs` and `C_PvP.GetMatchPVPStatIDs` are superseded by its removals: 59 → 53 gaps, with bounded absence coverage rather than historical behavior credit. [Supersession IDs](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-later-gap-closures.json). No p825-page worktree touched. Host has no WoW install; CASC-dependent contracts are recorded, not claimed tested. Wrath/Era/Anniversary not executed; existing profile gates exclude them from retail-only changes.

## Sources

- [Pinned wikitext](../../../data/patch-api/sources/8.2.0-api-changes.wikitext), [fetch response](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-fetch.json) and [HTTP receipt](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/p820-fetch-attempts.json).
- [Spec](../../specs/patch-8-2-0-publication-sweep.md), [dynamic validator](../../../data/patch-api/evidence/8.2.0-session-2026-10-08/validate.py).
- [Binding handoff](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[patch-8-3-0-api-audit]] — accounting, preservation and proof template.
- [[patch-api-audit-manifest]] — patch source/register chronology.
