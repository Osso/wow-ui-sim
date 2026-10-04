# Retail 12.0.7 delve entrance title

B04 title row `global api-C_DelvesUI-GetDelveEntranceTitleString-029` defines this bounded slice. Generated declarations may postdate 12.0.7 and are not native behavior evidence. Title tests pass on the default cumulative retail build. See [Lua API architecture](../lua-api.md).

## What it must do

- [x] Read exact host `Option<String>` live, returning one string or one nil; support replacement, explicit empty string, removal and environment isolation without a fabricated location.
- [x] A nil title permits the cached picker's `title or DELVE_LABEL` expression to use its own localized label; no simulator fallback is added.
- [x] **INFERRED** public output; ignore undeclared secret/public extras without authentication, validation or caller-taint changes.

## How it works

- [Lua API architecture](../lua-api.md)
- [C API signature audit](../c-api-signature-audit.md)

## Implementation inventory

- `src/c_api/c_delves_entrance.rs`: independent optional title query.
- `src/lua_api/globals/missing_surface/delves_ui.rs`: epoch-gated title delegate; older constant title retained.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: None-default title input.

## Tests asserting this spec

- `tests/patch_12_0_7_b01_b04.rs`: two title tests, exact values, isolation and ignored secret extras.
- `src/loader/tests/wow_api_globals/startup_globals.rs`: default nil title; old string assertion retained under negative epoch gate.
- `tests/delves_ui.rs`: existing world-tier and other entrance controls remain unchanged.

## Known gaps (current cycle)

- [ ] Historical declarations, native output secrecy and full cached picker interaction remain unverified.
- [ ] Older title branch remains unexecuted in this round.

## Out of scope

Row `global api-C_DelvesUI-GetWorldTierDifficultyForActivePlayer-030`: proposed empty-default difficulty errors conflict with cached InstanceDifficulty/DifficultyUtil callers. No candidate state, producer, tests or runtime initialization is integrated. Existing world-tier behavior remains unchanged pending native/default evidence or explicit initialization authorization.

Localization, location/service acquisition and full delve selection are outside this host-input contract.
