# Patch 8.0.1 API audit

Warcraft Wiki page **149302**, current revision **1463362** (2023-07-20T21:26:53Z), refetched 2026-10-08. [Raw source](../../../data/patch-api/sources/8.0.1-api-changes.wikitext), [provenance](../../../data/patch-api/sources/8.0.1-api-changes.provenance.json), [HTTP response/receipts](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/p801-fetch-attempts.json). BFA pre-patch page contains 293 lines, mostly event lists; no additional linked-page content is silently inferred.

## Complete source accounting

Opt-in `--bfa-prepatch` handles top-level New namespaces, nested member bullets, prose removals (excluding replacement links after “Use”), plain named removals, and four-level Added/Removed event sections. Distinct from 8.1.5's `--legacy-api-bullets`; both behaviors survive rebase. Namespace roots and repeated glyph occurrences are retained. No numerical source headers exist to reconcile. Extract preserves every change/removal statement, documentation qualification and external reference field without expanding links.

[Register](../../../data/patch-api/sources/8.0.1-wikitext-register.json): **269 inventory occurrences**. [Ledger](../../../data/patch-api/sources/8.0.1-page-coverage.json): **295 unique IDs**, adding 26 extract rows. Statuses: 151 partial-development-green, 103 bounded-coverage, 28 audit-pending, 13 metadata-only. [Extract scout](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/p801-extract-scout.json) maps each retained line to literal raw/extract coordinates. Counts derive from files, not validator constants.

## Concrete coverage matrix

| Boundary | Handled | Missing/problematic | Proof |
|---|---|---|---|
| Inventory | 251/269 current full-UI observations OK | 18 exact failures; raw namespace lookup is not fabricated-callable lookup | Prefork shared publication classifier; exact known-gap equality |
| World-position projection | Explicit map/continent rectangles; normalized affine result and GetXY; override, updates, reversed axes, edges, miss, overlap, degenerate/nonfinite and isolation | No real geography snapshots, native overlapping-map hierarchy selection or secret-argument parity | Two integration cases with concrete world coordinates; current raw function also observed in full UI |
| Event/aura input changes | Unknown event rejected without registration; UNIT_POWER_UPDATE accepted; string aura index rejected; actual AuraUtil missing-name query returns nil | Exact error wording, populated aura-name ordering/filter parity, power-change payload/lifecycle | One cached full-UI prefork case; no Blizzard Lua patch |
| Retirements | Existing absent/deprecated identities observed; all explicit removal/migration candidates scanned | GLYPH_ADDED/REMOVED retained for existing simulator producers/tests; no destructive retirement | 98 unique identities, 392 untruncated whole-word scans; master/8.1.0/8.1.5 register snapshots |
| Prose | Three bounded rows: event rejection, aura index input and three named absent globals | Ten substantive prose contracts remain pending | Exact per-line notes in ledger/scout; no blanket historical parity claim |

Initial discovery found **20 failures**. One closes through the new world-rectangle model. `C_ChatInfo.ReportPlayer` closes through the **merged 8.1.5 removal**, not new reporting behavior. Remaining **18 inventory failures**: eight chat members, five map metadata members, two glyph identities repeated twice, and SPELL_TEXT_UPDATE (superseded by the 11.0.0 removal but still registered).

[Per-ID review](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/p801-gap-review.json) retains literal statements, observations, expectations and individual reasons. Chat gaps require report eligibility, indexed roster snapshots, a prefix-interest registry with current result/security/capacity policy, channel classification and retail sender/logged-delivery semantics. Existing outbound-intent senders are explicitly Forever-only; their profile gate is not bypassed. Map gaps require bounty associations, art-help placement, display metadata, point-highlight visuals and dungeon floor catalogs, not guesses from hierarchy/art pixels. [Caller scout](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/p801-gap-caller-scout.json) retains complete src/tests and cached hits.

Pending prose: full chat migration; combat-log no-loadout/current-event timing; power event producer/payload; twenty joined-channel capacity; Vignettes namespace replacement; unnamed “all map API” removals; current-map/WorldMapFrame transition; party-member map positions; encounter-journal instance composition; aggregate glyph removal. Publication-only credit does not resolve these contracts.

## Backing model, not shim

`src/c_api/map_world_coordinates.rs` implements `C_Map.GetMapPosFromWorldPos` against environment-local `SimState.map_world_rects`, keyed by uiMapID. Each `MapWorldRect` carries continent identity and world-space extents. Only known maps with explicit finite, nondegenerate rectangles can produce output. Missing/outside/ambiguous inputs return no values. Rectangles begin empty; no native map geometry is fabricated from texture dimensions. Existing forward `GetWorldPosFromMapPos` art-based approximation remains outside this change and is **not** used as a native round-trip oracle.

Fixture: continent 42, map 2248, rectangle x=[-200,600], y=[100,500], world point (0,400) → normalized (.25,.75). Changing right extent to 200 changes x to .5. A second overlapping map requires explicit override; reversed x extent maps (100,400) to .25. Independent environments have no snapshot. No registration-only placeholder or vendor monkey-patch supplies this behavior. [Spec](../../specs/patch-8-0-1-publication-sweep.md).

## Retirement evidence

**No new retirements.** [Scans](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/p801-removal-scans.json) cover every unique source removal plus four migrated chat globals and the Vignettes namespace/UNIT_POWER identity. Scanner: `/usr/bin/grep -R -n -w -F`, cached retail AddOns excluding `*Documentation*`; qualified and bare terms separately. Complete src/tests scans use bare names, so `pcall(Name, ...)` and `and Name then` are included without syntactic filtering. stdout/stderr/exits and exact argv are saved without truncation.

[Decisions](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/p801-retirement-decisions.json) preserve any current cached consumer. GLYPH_ADDED and GLYPH_REMOVED each have zero cached hits but eight src/tests hits: `glyph_state.rs` still dispatches them and `c_glyph_globals` tests consume them. Removing registration would break that modeled lifecycle; mismatch remains explicit. Other removed inventory identities already satisfy absence/deprecation/supersession observations. No new global or namespace gate was required. Pinned master, p810-page and p815-page register scans retain every matching re-addition and revision; no other worktree was modified.

## Verification

Cargo target: `/home/osso/.cache/wow-ui-sim-targets/p801-page`. All commands executed from the owned worktree with explicit cwd. No agents/model CLIs, push, merge, full integration suite or synchronous polling.

| Command | Result |
|---|---|
| `cargo test --test prefork_full_ui -- publication_sweep` | 41/41; 40 page sweeps plus factory regression |
| `cargo test --test prefork_full_ui -- patch_8_0_1_cached_` | 1/1 input semantics |
| `cargo test --test integration patch_8_0_1_ -- --nocapture` | 2/2 world-rectangle cases |
| `cargo test --test integration c_map_probes:: -- --nocapture` | 28/28 |
| `cargo test --test integration c_map_api:: -- --nocapture` | 51/51 |
| `python3 -B tools/test_extract_patch_non_inventory.py` | 32/32 |
| `python3 -B tools/test_gen_patch_wikitext_register.py` | 23/23 |
| Per-source register regeneration | 40/40 byte-identical |
| Per-source extract `--text-only --check` | 37/40; inherited 12.0.5, 12.0.7, 12.1.0 failures unchanged |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Exit 0; zero non-vendor warnings |
| `cargo fmt --check`, `cargo check` | Exit 0; zero non-vendor warnings |
| Negative register injection | Expected exit 1; exactly one new failure, 18 → 19, no resolved IDs |
| `cargo build --bin wow-sim`, then `timeout 90 <target>/debug/wow-sim --no-addons --no-saved-vars lua-errors` | Exit 0; startup `[]`, zero Lua errors |

[Proof receipts](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/p801-proof.json) pin commands, code revisions/scopes, results and log hashes. Initial discovery/behavior REDs, the fixed mid-file Rust doc-comment parse error, and the stale ReportPlayer fixture after 8.1.5 integration remain labeled historical failures. No failing run is represented as green. Inherited iced manifest deprecations remain unsuppressed. Changed Rust was manually audited for readability; no suppression or runtime fallback added.

## Portable evidence and integration

[Validator](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/validate.py) is read-only, derives counts and historical register scope from pinned Git/artifact inputs, and imposes no absolute cwd/target equality. Mutable shared inputs use Git snapshots; later earlier-page additions do not retroactively enlarge this proof. Rebase onto repaired master `ee7aebea7` retained both opt-in parsers and restored the already-truncated base wiki before updates. Full repaired index/log contents were preserved and checked before commits.

Validator **PASS** locally and in a separate temporary clone, invoked from the original checkout. [Portability receipt](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/p801-validator-portability.json) also proves a simulated later register/known-gap closure does not expand historical scope, and protected source tampering fails. Temporary clone was removed. The original p815-page ref disappeared after its merge; its failed lookup is retained as invalid intermediate evidence, and the accepted register scan pins merged endpoint `4f21aa7dade78c22accac3e703dd3ef4e5ae8831`.

8.1.5 is now a real `later_registers` input; 8.1.0's first-position integration placeholder remains. Integrator must add its register, reconcile any exact superseded gaps, and refresh affected proof without treating later events/fields as native historical parity. Host has no WoW install; no CASC-dependent visual proof is claimed. Remaining 18 inventory gaps, ten prose contracts, real geography/secret parity and three inherited extract failures are explicit unfinished boundaries.

## Sources

- [Pinned source/provenance](../../../data/patch-api/sources/8.0.1-api-changes.provenance.json).
- [Ledger](../../../data/patch-api/sources/8.0.1-page-coverage.json), [source scout](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/p801-extract-scout.json), [gap review](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/p801-gap-review.json).
- [Cached native map signature](../../../data/patch-api/evidence/8.0.1-session-2026-10-08/p801-map-doc-documentation-grep.txt).
- [Per-page integration procedure](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md).

## See Also

- [[patch-8-1-5-api-audit]] — merged reporting retirement supersession and separate bullet parser.
- [[patch-8-2-0-api-audit]] — discovery/retirement/evidence template.
- [[patch-audit-validator-portability]] — historical scope and checkout independence.
