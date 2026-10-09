# Wrath Classic 3.4.2 source accounting

Account for frozen Warcraft Wiki page `355030` / revision `3422177`, TOC `30402`, without treating retail changes as Wrath proof. [Literal ledger](../../data/patch-api/sources/3.4.2-page-coverage.json); [audit](../wiki/investigations/patch-3-4-2-api-audit.md).

## What it must do

- [x] Validate response identity and both manifest hashes before owned source copying; replay the exact frozen source and recorded plaintext flags.
- [x] Preserve all literal inventory occurrences, eight numerical headers, CVar default/scope/category/description fields, and every nonblank source line; derive counts from serialized inputs.
- [x] Reject missing/altered inventory or prose, invented signatures or publication/behavior credit, wrong profiles and foreign successors.
- [x] Keep supported Wrath profile / configured interface separate from historical TOC, documentation-only cache observation, runtime and native proof.
- [x] Replay sealed historical own inputs independently of current tools, caches, global registers or original Git objects; reject serialized source and proof-log tampering.

## How it works

- [Coverage matrix and historical replay](../wiki/investigations/patch-3-4-2-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/3.4.2-api-changes.{wikitext,txt}` — literal source and non-inventory plaintext.
- `data/patch-api/sources/3.4.2-source-inventory.json` — immutable separate Wrath source inventory, consumed directly by the current factory sweep without generator changes.
- `data/patch-api/sources/3.4.2-page-coverage.json` — inventory/prose/context ledger and explicit limits.
- `data/patch-api/evidence/3.4.2-session-2026-10-09/` — pinned responses, historical tools/profile/cache observation, validator, own fixtures and seals.

## Tests asserting this spec

`data/patch-api/evidence/3.4.2-session-2026-10-09/test_source_accounting.py` checks serialized accounting and mutation rejection. `validate.py` replays sealed historical inputs. Neither command loads the simulator or establishes native parity. Exact commands/revisions/results live in `source-proof.json` in the same directory.

## Current bare-factory measurement

- [x] Read all 155 existing source occurrences directly; preserve the source audit and 23 historical seals unchanged.
- [x] Match `wrath-classic` only to `ClientProfile::Wrath`; use standalone `WowLuaEnv::new`, without publisher files, native probes or full Game initialization.
- [x] Record every observation under simulator interface 38001 (3.3.5-era architecture), distinct from source TOC 30402; require the exact 83 publication-mismatch IDs.
- [x] Keep CVar value/default observations separate from publication: 14 added registrations absent; 11 published defaults match and `TargetAutoLock` observes `0` versus literal `1`.
- [x] Keep same-client history only: foreign registers cannot supersede Wrath; actual 3.4.3 has zero explicit rows and supplies no supersession.

`tests/patch_3_4_2_factory.rs` uses the inventory directly. Current observations, ledger, proof commands and logs live in `data/patch-api/evidence/3.4.2-factory-2026-10-09/`, separate from historical source proof. `data/patch-api/3.4.2-factory-known-gaps.json` pins only publication/absence mismatches, not native compatibility failures or CVar default equality.

## Known gaps (current cycle)

- [ ] 83 factory publication/absence mismatches; one additional published CVar default mismatch. The 35 source CVar records retain metadata, but persistence and subsystem effects remain unmeasured.
- [ ] Two summary claims remain UNPROVEN: unspecified retail 10.1.0 subset and Settings panel adoption.
- [ ] Generator remains unchanged; shared test classifier now accepts `wrath-classic`. Coordinator owns integration, broader gates and any publisher-loaded measurement after 3.4.3 integration.
- [ ] No explicit call signatures, event payloads or detailed widget/API state transitions supplied; linked contracts deliberately unexpanded.

## Out of scope

Other page edits, linked API reconstruction, retail/Cata supersession, runtime retirements, shims/fallbacks, vendor/Wowless/cache mutation, broad publication/build/check/smoke/readability/final gates, push/merge/delegation. Actual 3.4.3 Wrath successor is retained as reference only; its empty inventory cannot supersede any named member or establish parity. Main integrates after 3.4.3. Factory publication proof is not loaded Blizzard UI, native Wrath Classic parity or justification for production API changes.
