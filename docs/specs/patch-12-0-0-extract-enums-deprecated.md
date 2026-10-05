# Retail 12.0.0 extract enum and deprecated API proof

Bounded proof for B01, B07 and B03 in the [54-row scout slice](../../data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-extract-scout.md). The [investigation](../wiki/investigations/patch-12-0-0-api-audit.md#enum-and-deprecated-extract-proof) owns root causes; the [per-source report](../../data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-enums-deprecated-report.md) owns outcomes and command/revision evidence. Coverage ledgers remain unchanged.

## What it must do

- [x] Freeze all 32 enum rows by source ID, with cached declaration file:line, file hash, explicit member values and Meta. Check full current tables, numeric types, raw/ordinary lookup, removed Recraft and renamed old-key absence after real cached Game preload.
- [x] Distinguish six superseded historical contracts from current publication defects. Observe B07 seed conflicts at the actual loaded-environment boundary before changing producers.
- [x] Require native and cached raw/ordinary absence for all 21 deprecated occurrences under current Retail; any frozen republished direct alias must match its loaded cached assignment and successor function identity exactly. None of these legacy names is republished by the current cache.
- [x] Gate simulator legacy publication from the cumulative `retail-12-0-0` epoch; preserve existing earlier/non-retail paths. Cached loading and post-load simulator restoration must not reintroduce removed APIs.
- [x] Exercise all 20 listed successors with concrete fixtures and require the exact six pending-result gap set, detecting both new and stale gaps. Fourteen successor fixtures currently pass; this is not full successor completion.
- [ ] Execute historical exact-12.0.0 native and enum controls. Master-base compilation currently blocks this verification.

## How it works

- [Investigation and root causes](../wiki/investigations/patch-12-0-0-api-audit.md#enum-and-deprecated-extract-proof).
- [Existing direct alias attribution](../wiki/investigations/plain-global-publication.md).

## Implementation inventory

- `tests/patch_12_0_0_enums.rs`, `tests/data/patch_12_0_0_enums.json`: complete current enum projections and explicit historical supersession.
- `tests/patch_12_0_0_deprecated.rs`, `tests/data/patch_12_0_0_deprecated.json`: native/cached retirement, conditional exact alias identity, bounded successor results and pending reasons.
- `src/c_api/patch_retired_members.rs`: five deprecated namespace keys excluded from automatic fabrication.
- `src/lua_api/globals/{register.rs,real/mod.rs,cooldown_probes.rs,spell_state_probes.rs}`: legacy tab/cooldown/usability publication gates.
- `src/lua_api/globals/{missing_surface/item_socket_info.rs,missing_surface/mythic_plus.rs,quest_surface/register.rs}`: legacy global/late namespace registration gates; successor behavior unchanged.
- `src/lua_api/workarounds/temporary/{legacy_spell_globals.rs,inventory_query_defaults.rs}`: pre-12.0.0 wrappers cannot restore retired globals under Retail.

## Tests asserting this spec

- `patch_12_0_0_enums::patch_12_0_0_enum_publication`: standalone integration filter, 32 row projections.
- `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors`: single-filter `prefork_full_ui`, 21 retirements and exact six successor gaps.
- `patch_12_0_0_deprecated::patch_12_0_0_deprecated_native_retirement`: historical epoch integration control, currently compilation-blocked.
- Existing 12.1.0 enum and deprecated wrapper controls: pass individually. Existing default lib/deprecated filters retain the same failures seen at master base.

## Known gaps (current cycle)

- [ ] Six successor results remain unproved; exact IDs, observed defects and ownership exclusions live in the [per-source report](../../data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-enums-deprecated-report.md#outcomes), not duplicated here.
- [ ] Historical execution remains blocked by the master-reproduced `on_update.rs:61` reference to `private_aura_sounds`, gated behind 12.0.5.
- [ ] First patch that renumbered historical Tooltip members cannot be established from inspected registers; verified 12.1.0 cache supersedes the historical values without an invented earlier date.

## Out of scope

- Concurrent namespace successor/event/CVar producers and the publication-sweep gap fixture: separate workstream ownership.
- Coverage-ledger edits, vendor/cache changes, native WoW parity, enum consumer domains and Classic runtime verification: not authorized proof scope.
