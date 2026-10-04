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

## Development proof and independent bounded acceptance — 2026-10-03 (navigation-nearest-party-token)

Commit `fcdcf43c6`. RED: 0 PASS / 8 FAIL module. GREEN: 8/8 module. This section supersedes wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun), [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b99-verify-auras-nav.md) SHA256 `c810ea3b9a7e3468a21bae71c13d18bedfe71c13c51e4438eb1f8a280ef11b20`. Host selection has no runtime producer; cached SuperTrackedFrame would error if party tracking became active without one. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-03-25-092 bounded-coverage under capability `navigation-nearest-party-token`; **131 capabilities/362 IDs; 28 pending /269 bounded /30 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Compile and run authored tests with state-only RED, then producers GREEN.
- [ ] Loaded startup and real tainted-addon invocation proof.

## Out of scope

Geometry, native nearest selection, roster membership validation, exact native error text, arbitrary extra arguments and older-profile authorization. No public secret arguments are declared for this zero-argument API; no AllowedWhenUntainted validator is introduced.
