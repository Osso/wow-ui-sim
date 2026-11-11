# Group member role counts

`GetGroupMemberCounts` supplies group counts to the selected original `PartyUtil.lua` consumer. Retail behavior is based on R44-Q1, inspected from Retail 12.1.5 build 70688. This evidence does not establish parity for other profiles.

## What it must do

- [ ] On Retail, return exactly one table. With no selected roster, all four role counts are zero.
- [ ] Argument 1 selects container one or two when its converted integer is 1 or 2. Otherwise, select container two when present, then container one. Do not assign party/raid names to these containers without evidence.
- [ ] If argument 1 is a table, update and return that same table. Preserve its other fields.
- [ ] Scan no more than 40 slots and stop at the first absent member. Do not exclude the player separately.
- [ ] Skip members whose flags contain bit 16 from both role and class counts.
- [ ] Classify role masks A and B in priority order: tank if `A & 2 != 0` or `B & 2 != 0`, `B & 8 == 0`, and `A & 8 != 0`; healer if `A & 4 != 0`; damager if `A & 8 != 0` and `(B & 10) != 2`; otherwise `NOROLE`.
- [ ] Resolve class from the supplied resolved class when present. Otherwise use cached class when it is positive and in range, or class 1. Skip a resolved class below zero or above the class range. Class zero contributes internally but has no published class key.
- [ ] Publish class counts under class tokens, using available class rows only.
- [ ] Keep the selected Retail behavior separate from profiles whose native parity is not established.

## How it works

- See the [C API and Lua API boundaries](../wiki/index.md).

## Implementation inventory

- `src/lua_api/globals/real/group_role_counts.rs` — Retail roster count projection and the existing non-Retail simulator projection.
- `src/lua_api/state_types/group_member_counts.rs` — host-supplied Retail roster input.
- `src/lua_api/state/sim_state.rs` — stores that input on `SimState`.

## Tests asserting this spec

- `tests/group_role_counts.rs::group_role_counts_retail_uses_native_roster_fields` — selected original `PartyUtil.lua`, both containers, default selection, table reuse, role priority, skip flag, class resolution, null termination, and 40-slot cap.
- `tests/group_role_counts.rs::group_role_counts_nonretail_keeps_simulator_party_projection` — selected original `PartyUtil.lua` with the existing non-Retail simulator input model; it is not native parity proof.
- `tests/admin_pvp_guild_api.rs` — no-group role total.

## Known gaps (current cycle)

- [ ] The native semantic names of the two containers and role masks A/B are unknown.
- [ ] The full producer and server-response lifecycle for roster inputs is not modeled.
- [ ] PTR and other-profile native parity is unknown; their current simulator role projection remains unchanged.

## Out of scope

- Do not infer a group event contract from the getter or numeric error-report calls.
- Do not derive the two native role masks from assigned simulator roles.
