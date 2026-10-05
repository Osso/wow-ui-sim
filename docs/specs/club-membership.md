# Club membership

`src/c_api/` owns opaque member identity and community membership management. The cached retail `ClubDocumentation.lua` defines string-like opaque member selectors, privilege-gated management and synchronous member events. [Implementation](../wiki/systems/club-membership.md).

## What it must do

- [ ] Return stable opaque IDs from member lists, member info and message authors; reject numeric member selectors.
- [ ] Update roles/notes and remove members before synchronous callbacks observe state.
- [ ] Preserve required/unique owner, enforce assignable and kickable role lists.
- [ ] Store pending invitations separately from members; honor invite, revoke and query privileges.
- [ ] Deny restricted/uninitialized mutations without state changes or events.
- [ ] Accept explicit host member changes and publish ADDED, PRESENCE_UPDATED, REMOVED, ROLE_UPDATED and UPDATED with the same ID used by getters.

## How it works

- [Club model and inferred policies](../wiki/systems/club-membership.md).

## Implementation inventory

- `src/c_api/club_model.rs`: club/member/invitation state and role policy.
- `src/c_api/club_members.rs`: Lua getters and management/event boundary.
- `src/c_api/c_club.rs`: registration, guild streams/messages and host inputs.
- `src/lua_api/state{,/sim_state}.rs`: stores club state.

## Tests asserting this spec

- `tests/club_membership.rs`: opaque identity, creation and management behavior.
- `tests/c_club_probes.rs`: guild-club getters and message behavior.
- `tests/p1207_gap_closures.rs`: BattleTag requests use returned member IDs.

## Known gaps (current cycle)

- [ ] Finish host inputs and concrete management/event proof.
- [ ] Run required regression and startup checks.

## Out of scope

- Live community service/network transport, persistence and native server validation are absent.
- Coverage JSON reconciliation and unrelated API work are excluded by task authorization.
