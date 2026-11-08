# Friend info by name

Current cached Retail/PTR `FriendListDocumentation.lua` declares `C_FriendList.GetFriendInfo(name: cstring)` with `AllowedWhenUntainted`. Unchanged Mainline `GameDialogDefs.lua` uses it to prefill `SET_FRIENDNOTE` from `info.notes`. Producer `be9c734a4` commits the exact Retail/PTR query over environment-owned `SocialFriend` records with the shared unchanged simulator DTO; the current alternate-name provider is removed, indexed/earlier providers retained, and C_Social unchanged. [Independent bounded Retail/PTR proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/friend-info-exact-2026-10-12/README.md) accepts the query and unchanged popup, not native friend-system parity.

## What it must do

- [x] Publish the exact current-profile query and project supplied friend name/note from the owning environment; live note changes must be visible without leaking across environments.
- [x] Return one info table for a matching supplied name and one nil for a miss. Exact string matching and miss arity preserve existing simulator lookup policy, not independently established native normalization or arity.
- [x] Retain the existing strict string-input policy. Authenticate wrapped values before validation: untainted secret names work, tainted secret arguments fail without clearing caller taint, ordinary names remain usable by tainted callers.
- [x] Load the whole unchanged current-profile popup/FriendsFrame dependency closure and exercise real `StaticPopup_Show("SET_FRIENDNOTE", name)`: the actual Mainline handler must prefill the real edit box and reflect a changed supplied note on reopening, without recorded Lua errors.
- [x] Preserve existing indexed/earlier-provider behavior while replacing the current non-native alternate-name provider; do not add an alternate-name fallback. Earlier providers/C_Social retain inspected source-equivalent behavior; new indexed/state runtime proof is Retail/PTR only.

## How it works

- [Lua API architecture](../lua-api.md)
- [Historical/current API investigation](../wiki/investigations/patch-8-1-0-api-audit.md)

## Implementation inventory

- `src/lua_api/state_types/social.rs`: existing supplied `SocialFriend` name/note and other represented fields.
- `src/c_api/c_friend_list.rs`: exact current query, VM-authenticated string input and shared existing simulator projection.
- `src/lua_api/globals/missing_surface/friend_list.rs`: indexed/earlier-provider registration; current alternate-name provider replaced.
- `tests/friend_info_exact.rs`: focused current-profile behavioral and real public-popup fixtures.

## Tests asserting this spec

- `tests/friend_info_exact.rs`, registered by `tests/integration.rs`: exact query, live environment-owned state, miss/type/security controls and actual popup prefill/reopen.
- `tests/friend_list_who.rs` and `tests/system_api_seeded.rs`: preserved indexed/name state assertions using the profile's provider; no runtime fallback.

## Known gaps (current cycle)

- [x] Initial pre-producer RED executed and authenticated: four cases0PASS/4FAIL; three stop at absent exact query, popup stops at an overbroad diagnostic-readiness assertion. Downstream assertions remain unreached.
- [x] Corrected pre-producer RED independently authenticated: four cases0PASS/4FAIL. Whole closure and Mainline provenance assertions reach unchanged `OnShow`; nil `info` causes the actual note-prefill failure. Reopen remains unreached. Optional nil-probe observations remain logged, not replaced with fabricated globals.
- [x] Independent current gate:12 newer passing executions (Retail4+PTR8), plus4 unchanged Retail passes reused;16 scope executions/eight names, not16 new runs. Normal291/283 addons exit0/`[]`/CLEAN0; fmt/default check0, nine source hashes unchanged; source/readability and fixture-delta proofs admitted separately. Six inherited dependency warnings remain. First producer4PASS/1FAIL, helper diagnostic, parser failure and resource refusals remain retained.
- [x] Independent packet retention/link/privacy PASS:258 byte-exact copies/3,816,250 bytes,24 links including2 anchors,zero privacy blockers. Private gate reports excluded from recursive copies; original behavioral failures remain preserved.
- [ ] Separate helper/native parity, ordinary numeric/userdata/malformed-UTF8 runtime coverage, all-profile runtime and parent full-handoff acceptance remain outside this bounded proof.

## Out of scope

This bounded caller fix does not establish native catalogs, name normalization/coercion, complete native `FriendInfo` fields, note mutation APIs, network friend lifecycle or new earlier-profile contracts. Those remain separate parent-handoff gaps; no new normal-session seeding follows from fixtures. Existing synthetic records are simulator inputs, not native facts.

## Current bounded GUI publication — 2026-10-12

[New90-second WoW API publication only; not Pi/LLM provider diagnostics](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/live-retail-gui-providers-2026-10-12/README.md) · [Independent publication gate; logged event, not pixels; intentional124 expiry, not assertion failure or service repair](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/live-retail-gui-providers-2026-10-12/live-provider-report.md). [Owned-process/socket cleanup, existing display preserved;15 copies/39,405 bytes retention independently2905 PASS at5e1b7bab4; native/visual/authenticated-remote/user acceptance OPEN](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/live-retail-gui-providers-2026-10-12/README.md).

