# PTR Training Grounds classification

`C_PvP.IsTrainingGroundsArena` and `C_PvP.IsTrainingGroundsBG` classify LFG dungeon catalog entries. The pinned [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json) adds both functions with a required numeric `lfgDungeonsID` and one non-nil boolean result. Catalog state is exposed through the [C API boundary](../lua-api.md).

## What it must do

- [ ] PTR publishes both queries; earlier retail keeps them absent before and after bootstrap, including namespace fallback lookup.
- [ ] Explicit Arena classification returns true only from the arena query; Battleground returns true only from the BG query.
- [ ] Unclassified and unknown IDs return false from both queries (simulator policy).
- [ ] Queries observe catalog classification changes immediately and do not share state between environments.
- [ ] Default catalog entries are unclassified. Synthetic test IDs do not represent native Training Grounds IDs.
- [ ] IDs must be finite integral Lua numbers representable by the catalog's signed 32-bit ID; malformed inputs error without coercion. This is simulator validation, not a claim about native ID limits.
- [ ] Existing LFG and PvP queries remain unchanged.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_pvp.rs`: classification enum, PTR queries, earlier-profile absence, numeric validation.
- `src/lua_api/state_types/collections.rs`: optional classification on each `LfdDungeonInfo`.
- `src/lua_api/state/defaults/lfg.rs`: default catalog entries remain unclassified.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_training_grounds.rs`: synthetic catalog classification, live changes, isolation, validation, profile surface.
- `tests/c_lfg_info_probes.rs`: existing LFG query regressions.
- `tests/pvp_probes.rs`: existing PvP query regressions.

## Known gaps (current cycle)

- [ ] Authoritative native Training Grounds dungeon IDs and classifications are unavailable.
- [ ] Native unknown-ID, coercion, and validation behavior is unverified.
- [ ] `SecretArguments = AllowedWhenUntainted` is not enforced or proven by ordinary catalog tests.

## Out of scope

No match simulation, queue transitions, new admin API, or inferred classifications from dungeon names/types. Runtime fixtures can update the existing catalog directly. Native catalog, security, taint, and secret-value conformance require separate evidence.
