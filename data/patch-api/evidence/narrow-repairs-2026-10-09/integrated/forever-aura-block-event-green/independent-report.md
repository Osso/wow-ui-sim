# Independent event-repair verification — 2026-10-10

**OVERALL: PASS for bounded finite-event repair; full stock-startup acceptance PENDING.**

Read/followed `verify` and `rust-readability` skills. Artifact-only, read-only review of repository and existing main-owned receipts. No builds, tests, checks, runtime execution, operations, or delegation performed. Only this requested report written.

## Source scope and wiring

- Repair commit: `d5dcc8b5953ef1e62a49f58845919077167a4a0a`. Sole Rust implementation change is addition of `"UNIT_AURA_BLOCK_LIST_CLEARED"` at `src/event/valid_events.rs:61` in sorted, feature-gated `FOREVER_REGISTERABLE_EVENTS`.
- `[EXIST] PASS`: implementation and `tests/wowforever_finite_constants.rs` present and read.
- `[SUBSTANTIVE] PASS`: literal finite-list addition, not a shim, placeholder, permissive validator, or replacement function.
- `[WIRED] PASS`: Forever validator uses this list's binary search (`valid_events.rs:75`); `src/lua_api/frame/methods/text_attribute_event/events.rs:103–104` calls the public validator. RegisterEvent reaches it through checked insertion; RegisterUnitEvent calls it before listener insertion (line 48).
- `[ANTI-PATTERN] PASS`: added implementation line and three new tests contain no TODO/FIXME/HACK/XXX, suppressions, empty handlers, or commented-out implementation. Manual Rust-readability audit found no changed-code violations. Single literal adds no function complexity, nesting, hidden side effects, or new parameters; new behavioral test bodies remain short and concrete. No metric/check commands run.
- Event already appears in `PATCH_12_1_REGISTERABLE_EVENTS` (`valid_events.rs:200`). It is **not** in `NON_REGISTERABLE_EVENTS`, whose unchanged contents are only `COMBAT_LOG_EVENT` and `COMBAT_LOG_EVENT_UNFILTERED` (`valid_events.rs:291`). Default tables and validator functions are unchanged. Default behavior preservation is established by source isolation, not by a newly executed default runtime test.
- Cached Forever `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:631–638` declares this literal, `SynchronousEvent = true`, and one non-nil `unitTarget: UnitTokenVariant`. Declaration is not native firing or secrecy evidence.

## Authentic RED and preserved blocked history

RED epoch: `forever-aura-block-event-red-current/20261010T190445Z`; submission revision `20e808b945188ce2b952f3dd81f2aa6547bc8587`.

- `compile-result.json`: `exit: 0`, `source_equal: true`; cargo stream errors empty and compiler `build-finished.success: true`.
- `cleared-event.stdout`: **1 passed, 2 failed**, 8331 filtered out; execution exit **101**.
- Both positive tests fail specifically on unknown declared name `UNIT_AURA_BLOCK_LIST_CLEARED`: RegisterEvent at test line 115; RegisterUnitEvent at line 147. Unrelated-unknown-name rejection control passes.
- Earlier epoch `20261010T190145Z/outcome.json` remains `status: resource-blocked`, `build_performed: false`. It is preserved and is not labeled behavioral RED.

## GREEN receipts and provenance

GREEN epoch: `20261010T191909Z`; main submission revision `ef55bf873ce7c9bcca3d88a3d690d447edf7a897`, not the repair commit itself. Relevant file hashes match repair commit exactly.

| Evidence | Result |
|---|---|
| Forever integration compilation | exit 0; compiler success; stream errors empty; source equality true |
| Existing module execution `wowforever_finite_constants::` | exit 0; **18 passed, 0 failed**, 8316 filtered out; expected/reached 18; artifact unchanged |
| Main-owned `cargo fmt --check` receipt | exit 0; source equality true; empty stdout/stderr |
| Main-owned default `cargo check --offline --locked -j 12` receipt | exit 0; source equality true |
| Main-owned Forever check with `--no-default-features --features sound,gui,casc,client-wowforever` | exit 0; source equality true |

All receipts available; no waiting or reruns needed. Both check logs and compile stderr retain six `iced-wgpu-patched/Cargo.toml` deprecated hyphenated Clippy manifest-key warnings; successful exit does **not** mean warning-free.

Read compiler artifact messages, compile receipts, execution receipts, full test output, and check output. Each compile emits exactly one integration executable artifact, with `client-wowforever` and expected feature set; each compiler reports successful completion. Independently recomputed sealed-binary SHA-256 matches recorded artifact:

- RED: `4a6e87d83ae076b89b678ca1478b9e8da8db97f507cd61b87c1d2b0ddc1362dd`.
- GREEN: `413a24349e80958509045f0138dcb7989011695fb6d69a0d5340c8de30276c06`.

Both epochs' 3853-file source-before/source-after inventories match internally. RED-to-GREEN inventory changes **only** `src/event/valid_events.rs`. Current inventoried source matches GREEN with zero mismatches. Relevant git blobs independently match inventory hashes:

- Tests, RED/repair/GREEN: `1c97e9a1064969e694f649bf65611e7e1ea7d23a605f563ae5d5ded5e9b9c355`.
- Validator RED: `e0d9361c9b4d66e3d6e2f168f9d473b0a33daaad77ee1f96baf60dc67efbd5b6`.
- Validator repair/GREEN: `8f7d35c6c324bff7a09c741fb8153441d31acc51c6aae49681c76a5929396b35`.

Compared original module at `20e808b94^`: all **15 original test bodies unchanged**, with exactly three added tests. The complete test file is identical between authentic RED, repair commit, and GREEN. New positives were not weakened to accept registration errors. They assert one-argument injected delivery, player/target delivery, player-only filtering, unregistration suppression, and empty Lua-error collection. Separate negative control checks both methods reject unrelated unknown names without retaining registration.

## Acceptance boundary

**PASS:** exact finite Forever registration repair and injected simulator behavior, three unchanged new cases plus 15 unchanged old module controls, formatting, default and Forever compile checks.

**PENDING / separate scope:** whole stock baseline repair acceptance (19 unique errors / 36 occurrences); these module receipts do not close it. No native event production, native payload/firing parity, payload secrecy, or whole-startup acceptance claim.

Provenance exclusions remain those recorded by main: external dependency/source-to-artifact provenance, untracked/index files, inherited environment, and runtime assets outside fixture inputs. Current docs/untracked changes are outside the 3853-file inventory and do not expand this report's proof scope.
