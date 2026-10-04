# Navigation nearest-party token

Retail12.0.5 `C_Navigation.GetNearestPartyMemberToken` returns explicit host-selected party-token state to trusted callers and rejects addon-tainted callers. Source: [12.0.5 changes](../../data/patch-api/sources/12.0.5-api-changes.txt), prose-2026-03-25-092. Cached InGameNavigation declaration supplies HasRestrictions, no arguments and nonnil cstring output; prose supplies addon prohibition.

## What it must do

- [ ] Trusted calls return the selected host token; changing party1 to party2 changes the actual string output.
- [ ] Addon-tainted calls fail, including calls through an otherwise trusted nested function, before reading host selection.
- [ ] Denial preserves selection, addon stack taint and the enclosing trusted caller.
- [ ] INFERRED: missing host selection fails explicitly; it is neither a nil placeholder nor an inferred roster/distance target. Error wording is simulator-authored.

## How it works

- [C API boundary and VM ownership](../lua-api.md)

## Implementation inventory

- `src/c_api/c_navigation.rs`: trusted query and caller authorization.
- `src/c_api/mod.rs`: feature-scoped registration.
- `src/lua_api/state/sim_state.rs`, `state.rs`: host input and absent initialization.
- `src/lua_api/workarounds/temporary/navigation_defaults.rs`: removes Retail12.0.5 nearest-token default; other temporary entries unchanged.

## Tests asserting this spec

`tests/patch_12_0_5_navigation_aura_entry.rs`: three navigation cases. Existing default tests stop asserting nil for the replaced entrypoint. Authored only; main owns compilation, genuine RED/GREEN and acceptance.

## Known gaps (current cycle)

- [ ] Compile and run authored tests with state-only RED, then producers GREEN.
- [ ] Loaded startup and real tainted-addon invocation proof.

## Out of scope

Geometry, native nearest selection, roster membership validation, exact native error text, arbitrary extra arguments and older-profile authorization. No public secret arguments are declared for this zero-argument API; no AllowedWhenUntainted validator is introduced.
