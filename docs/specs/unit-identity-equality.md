# Modeled unit identity equality

`UnitIsUnit` compares existing modeled identities, not token spelling, in `src/lua_api/globals/unit_misc.rs`. See [Lua API architecture](../wiki/systems/lua-api.md) and [GUID presence](unit-guid-presence.md).

## What it must do

- [x] Compare player, target and focus aliases symmetrically after public targeting assigns the same identity.
- [x] Distinguish different modeled identities after retargeting while preserving an unchanged focus identity.
- [x] Compare active party aliases with target/focus snapshots; stop resolving a removed party slot without erasing retained target/focus snapshots.
- [x] Return false when either token lacks a modeled identity, including two identical absent tokens or two nil arguments. Preserve existing argument conversion and boolean return shape.

These are simulator model-consistency requirements, not native-client proof. Cached retail `Blizzard_UnitFrame/Mainline/TargetFrame.lua`, `TargetFrame_OpenMenu`, uses `UnitIsUnit("target", "player")` to select the SELF menu. That is evidence of a consumer needing aliases, not a full menu integration test.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [GUID presence](unit-guid-presence.md)

## Implementation inventory

- `src/lua_api/globals/unit_misc.rs`: identity comparison through the existing modeled-GUID resolver.
- `src/lua_api/globals/targeting_verbs.rs`: existing target/focus snapshots used by public test fixtures; unchanged by this correction.

## Tests asserting this spec

`tests/unit_api.rs`, `test_unit_is_unit*`: player/target/focus symmetry, retargeting, clearing, party removal, absent identities, and retained same/different controls. Existing grouped integration target; no new Cargo target.

## Known gaps (current cycle)

`1787bd5f76007c12132fa7c4f65f5fa9fabc6f7e` reproduces four failures with two retained controls passing. Production `bfa742675ec489e1bfb3f2a42e0c4d3d4369d555` is independently GREEN: 83 `unit_api::` cases, 24 `targeting_verbs::` cases plus one nested consumer, format, check, and readability. `/tmp/cross-version-unit-identity-verification-ledger.md` records exact commands, revisions, and logs; authorized source remains unchanged through docs-only `33155da3e`.

- [x] Independent bounded GREEN verification and Rust checks complete.
- [ ] Native-client behavior, same-name distinct-identity fixtures and a live SELF-menu interaction remain unproven.

## Out of scope

Adding pet/vehicle/raid/remote identities, changing `UnitExists` or GUID generation, token normalization, coercion/error redesign, secret-value rules and all-profile/native parity. Tokens without a modeled GUID do not compare equal, even where another compatibility query reports presence; this correction does not invent identities for those unsupported domains.
