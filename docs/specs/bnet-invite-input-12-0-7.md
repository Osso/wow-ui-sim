# Retail 12.0.7 Battle.net invitation input

B02, `prose-undated-021` and `global api-C_BattleNet-InviteFriend-027` in the [retained 12.0.7 source](../../data/patch-api/sources/12.0.7-api-changes.txt) define this bounded authoring slice. Current generated declarations may postdate 12.0.7; declarations are not native behavior evidence. Bounded tests pass on the default cumulative retail build; strict historical and older-epoch builds were not run. See [Lua API architecture](../lua-api.md).

## What it must do

- [x] Accept numeric gameAccountID, return zero results, capture the last accepted request per environment, replacing it live. **INFERRED** this capture represents an invitation command input only, not a friendship or successful service operation.
- [x] Never append a fake offline friend or fabricate account metadata from gameAccountID; queryable friend count remains unchanged after requests and repeats.
- [x] **INFERRED** require finite positive integral i32 ID with no string coercion. Missing/nil, empty/BattleTag strings, fractions, overflow, nonfinite or wrong types error without changing the last request.
- [x] Authenticate arg1 and EVERY supplied extra with VM unwrap_secret before validation/mutation; untainted secret number succeeds, tainted secrets in any position fail before invalid public arg1 validation.
- [x] Preserve caller taint and rooted wrapper secrecy/identity through GC; environments and last-request state remain independent. **INFERRED** ignored public extras.
- [x] Load unchanged cached Deprecated_BattleNet.lua in a CVar-enabled test environment; its BNInviteFriend forwards numeric ID and zero returns. Runtime producers do not install the wrapper or enable its CVar.

## How it works

- [Lua API architecture](../lua-api.md)
- [C API signature audit](../c-api-signature-audit.md)

## Implementation inventory

- `src/c_api/c_battle_net_invite.rs` — authenticated numeric request capture.
- `src/c_api/c_battle_net.rs` — gated producer delegation; older-feature string simulation retained.
- `src/lua_api/state/sim_state.rs, src/lua_api/state.rs` — None-default request field.

## Tests asserting this spec

`tests/patch_12_0_7_b01_b04.rs` — ONE module in the auto-generated integration harness; first-line retail-12-0-7 cfg. Tests prefixed B02 assert the bounded values and policies above. Default-build B01/B02/B03 tests passed in this integration round.

## Known gaps (current cycle)

- [ ] Prove service-level party invitation behavior or retain only partial scope; request capture cannot prove delivery, rejection or events.
- [ ] Historical 12.0.7 declaration and end-to-end invitation delivery remain unverified.

## Out of scope

Battle.net network service, accepted friendships, party delivery, failures, events and wrapper availability. Offline friend insertion is a different operation from this cached numeric declaration and earns no credit.
