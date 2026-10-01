# Modeled unit identity equality

`UnitIsUnit` compares existing modeled identities, not token spelling, in `src/lua_api/globals/unit_misc.rs`. See [Lua API architecture](../wiki/systems/lua-api.md) and [GUID presence](unit-guid-presence.md).

## What it must do

- [x] Compare player, target and focus aliases symmetrically after public targeting assigns the same identity.
- [x] Distinguish different modeled identities after retargeting while preserving an unchanged focus identity.
- [x] Compare active party aliases with target/focus snapshots; stop resolving a removed party slot without erasing retained target/focus snapshots.
- [x] Return false for permitted comparisons when either token lacks a modeled identity, including two identical absent base/group tokens or two nil arguments. Older profiles preserve the existing argument conversion and boolean return shape.

### March 31, 2026 / 12.0.5 permissions

The final March 31 statements in [retained patch notes](../../data/patch-api/sources/12.0.5-api-changes.txt) supersede the earlier PTR 2/3 target-of-target secret-result rule: "All other comparisons are disallowed and return nil." The simulator implements denial as **zero return values**, per the bounded task contract; the notes alone do not establish return arity.

- [ ] With `retail-12-0-5`, permit either base token: `player`, `pet`, `vehicle`, `mouseover`, `target`, `softenemy`, `softfriend`, `softinteract`, `focus`, `none`, `npc`, `questnpc`, regardless of the other token.
- [ ] Otherwise permit either canonical `party1..4`, `raid1..40`, `partypet1..4`, or `raidpet1..40` against a noncompound, non-nameplate counterpart. Deny all remaining pairs with zero return values, even identical tokens.
- [ ] Permitted pairs retain modeled GUID equality; permission never creates missing identities. Older profiles do not gain this permission gate or secret-token conversion.
- [ ] Accept only VM-authenticated secret **string** tokens from an untainted caller, using the VM's existing guard. Reject tainted secret inputs before permission checks; do not clear taint or decode arbitrary userdata.
- [ ] **Inferred missing/nil policy:** retain one plain `false` whenever either argument is missing/nil, including a nonbase counterpart. Ordinary invalid argument types still error. Explicit empty strings are tokens, not missing arguments.

Token spelling is case-sensitive and unnormalized. Canonical numeric bounds/no-leading-zero spelling and the classification of `nameplate*` plus target-suffixed compounds are inferred lexical policy, not native-probe evidence. Plain unknown noncompound counterparts may compare against canonical group tokens but cannot acquire a modeled identity.

These are simulator model-consistency requirements, not native-client proof. Cached retail `Blizzard_UnitFrame/Mainline/TargetFrame.lua`, `TargetFrame_OpenMenu`, uses `UnitIsUnit("target", "player")` to select the SELF menu. That is evidence of a consumer needing aliases, not a full menu integration test.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [GUID presence](unit-guid-presence.md)

## Implementation inventory

- `src/lua_api/globals/unit_misc.rs`: identity comparison through the existing modeled-GUID resolver.
- `src/lua_api/globals/targeting_verbs.rs`: existing target/focus snapshots used by public test fixtures; unchanged by this correction.

## Tests asserting this spec

`tests/unit_api.rs`, `test_unit_is_unit*`: player/target/focus symmetry, retargeting, clearing, party removal, absent identities, and retained same/different controls. `tests/unit_comparison_permissions.rs`: symmetric permission matrix, canonical token limits, zero-return denial, missing/type controls, secure typed tokens and taint preservation, plus older-profile controls. Both use the existing grouped integration target; no new Cargo target.

## Known gaps (current cycle)

`1787bd5f76007c12132fa7c4f65f5fa9fabc6f7e` reproduces four failures with two retained controls passing. Production `bfa742675ec489e1bfb3f2a42e0c4d3d4369d555` is independently GREEN: 83 `unit_api::` cases, 24 `targeting_verbs::` cases plus one nested consumer, format, check, and readability. `/tmp/cross-version-unit-identity-verification-ledger.md` records exact commands, revisions, and logs; authorized source remains unchanged through docs-only `33155da3e`.

Historical GUID-equality proof above does not cover the new permission gate. Tests at `9a50d8a5cc20d0adf0b7c529d237fc043ef57532` are RED: 6 cases, 3 expected failures (restricted counterpart denial, nonbase denial, secret-string conversion); retained log `/tmp/patch-12.0.5-batch4-unit-permissions-red.log`. Post-implementation batched GREEN and Rust/readability gates are parent-owned and pending; additional typed-secret rejection/arity assertions await their first execution.

- [x] Historical bounded GUID-equality GREEN verification and Rust checks complete.
- [ ] Permission matrix, retained unit identity controls, and older-profile GREEN verification.
- [ ] Native-client behavior, same-name distinct-identity fixtures and a live SELF-menu interaction remain unproven.

## Out of scope

Adding pet/vehicle/raid/remote identities, changing `UnitExists` or GUID generation, token normalization, general coercion/error redesign, other secret-value API rules and all-profile/native parity. Tokens without a modeled GUID do not compare equal, even where another compatibility query reports presence; this correction does not invent identities for those unsupported domains.
