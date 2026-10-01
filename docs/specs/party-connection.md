# Party connection transitions

Bounded retained 12.0.5 prose row 182 motivates party connect/disconnect handling. `A_Admin.SetPartyMemberConnected(index, bool)` is a new simulator input contract, not a native WoW API. Shared party storage and admin behavior apply across client profiles, like the existing party API; the patch motivation does not justify profile-specific storage. [Audit context](../wiki/investigations/patch-12-0-5-api-audit.md).

**Cached source evidence, not native execution:** `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:4057-4065` declares `UNIT_CONNECTION`, `SynchronousEvent = true`, and exactly two nonnil payload fields: `unitTarget` (`UnitTokenVariant`) and `isConnected` (`bool`). The payload tuple is documented, not unknown. Retained prose row 182 supplies motivation, not an executed behavioral proof.

**Simulator contract/inference:** one-based party-member input, initially connected members, change-only emission, mutation before synchronous callbacks, absent valid index no-op, aggregate offline queries, discarded removed-member state, and environment isolation are intended model choices. They are not native-verified transition semantics.

## What it must do

- [ ] Expose shared `A_Admin.SetPartyMemberConnected(index, bool)` for existing one-based party members. An absent valid index must succeed without creating a member, changing existing members, or emitting `UNIT_CONNECTION`.
- [ ] New party members start connected. `UnitIsConnected("partyN")` reads the current member's connection state; `GroupHasOfflineMember()` is true exactly when at least one current party member is disconnected, and false for an empty party.
- [ ] Both disconnect and reconnect dispatch an actual `UNIT_CONNECTION` listener before the setter returns, with exactly `(unitTarget, isConnected)`. Queries inside that callback observe post-transition state. Repeating the current value emits no event.
- [ ] Reconnecting one member leaves the aggregate offline while another member remains disconnected. Shrinking/removing the roster discards removed connection state; regrowth starts connected.
- [ ] Connection state and listener dispatch stay isolated per `WowLuaEnv`, without profile-specific party storage.

## How it works

- [Lua API and simulator state](../lua-api.md).
- [Event dispatch](../event-system.md).
- [Admin party API](../admin-api/party.md).

## Implementation inventory

- `src/lua_api/game_data.rs`: existing `PartyMember` has no connection field; intended shared model location.
- `src/lua_api/state/sim_state.rs`: existing per-environment party roster.
- `src/lua_api/globals/admin.rs` and `src/lua_api/globals/admin_api/units.rs`: existing party admin registration/setters; new connection input remains unimplemented.
- `src/lua_api/globals/group_queries.rs` and `src/lua_api/globals/group_queries_relationships.rs`: existing aggregate and unit connection queries currently assume connected members.
- `tests/party_connection.rs`: six grouped fixtures discovered by the existing generated `integration` harness; no separate Cargo target or input scaffolding.

## Tests asserting this spec

`tests/party_connection.rs` covers both synchronous edges with exact two-field payload and callback query snapshots; repeated inputs; absent valid index; two offline members and partial reconnect; shrink/regrowth; full removal/regrowth; and two-environment isolation. Every fixture first requires the new setter to be callable. Calls must succeed directly, without `pcall` rejection checks that could mistake a missing method for valid rejection. Callback observations are asserted outside the callback, so swallowed handler errors cannot stand in for proof.

`tests/admin_party_api.rs::test_group_has_offline_member_defaults_false` remains unchanged: connected default members correctly yield false.

Proof ledger, 2026-10-01: tests/spec only; no test, build, or check execution in this change. Actual RED is pending parent execution, followed by production implementation and GREEN. Suggested bounded parent invocation: `cargo test --test integration party_connection::`. No native, cross-profile execution, full-row, or whole-page completion credit.

## Known gaps (current cycle)

- [ ] Parent records actual RED; implements shared connection input/model/query transitions; records GREEN and required final gates.

## Out of scope

Party-category expansion, networking, raid connection behavior, player/target/focus connection semantics, new storage by profile, native probes, and production edits in this tests/spec-only change. Roster-size events retain their existing contract; these fixtures do not prescribe additional connection events for roster removal.
