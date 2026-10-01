# Party connection transitions

Bounded retained 12.0.5 prose row 182 motivates party connect/disconnect handling. `A_Admin.SetPartyMemberConnected(index, bool)` is a new simulator input contract, not a native WoW API. Shared party storage and admin behavior apply across client profiles, like the existing party API; the patch motivation does not justify profile-specific storage. [Audit context](../wiki/investigations/patch-12-0-5-api-audit.md).

**Cached source evidence, not native execution:** `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:4057-4065` declares `UNIT_CONNECTION`, `SynchronousEvent = true`, and exactly two nonnil payload fields: `unitTarget` (`UnitTokenVariant`) and `isConnected` (`bool`). The payload tuple is documented, not unknown. Retained prose row 182 supplies motivation, not an executed behavioral proof.

**Simulator contract/inference:** one-based party-member input, initially connected members, change-only emission, mutation before synchronous callbacks, absent valid index no-op, aggregate offline queries, discarded removed-member state, and environment isolation are intended model choices. They are not native-verified transition semantics.

## What it must do

- [x] Expose shared `A_Admin.SetPartyMemberConnected(index, bool)` for existing one-based party members. An absent valid index must succeed without creating a member, changing existing members, or emitting `UNIT_CONNECTION`.
- [x] New party members start connected. `UnitIsConnected("partyN")` reads the current member's connection state; `GroupHasOfflineMember()` is true exactly when at least one current party member is disconnected, and false for an empty party.
- [x] Both disconnect and reconnect dispatch an actual `UNIT_CONNECTION` listener before the setter returns, with exactly `(unitTarget, isConnected)`. Queries inside that callback observe post-transition state. Repeating the current value emits no event.
- [x] Reconnecting one member leaves the aggregate offline while another member remains disconnected. Shrinking/removing the roster discards removed connection state; regrowth starts connected.
- [x] Connection state and listener dispatch stay isolated per `WowLuaEnv`, without profile-specific party storage.

## How it works

- [Lua API and simulator state](../lua-api.md).
- [Event dispatch](../event-system.md).
- [Admin party API](../admin-api/party.md).

## Implementation inventory

- `src/lua_api/game_data.rs`: shared `PartyMember.connected` bool; every constructor initializes it to true.
- `src/lua_api/state/sim_state.rs`: existing per-environment party roster.
- `src/lua_api/globals/admin.rs`: active shared registration of `SetPartyMemberConnected`; `admin_party_target_helpers.rs` mutates only active existing members, releases the state borrow, then synchronously dispatches changed edges. Integer/bool decoding uses the same `FromStack` boundary as existing admin inputs; nonpositive indices no-op. Obsolete `admin_api/units.rs` receives only its constructor field, not duplicate registration or behavior.
- `src/lua_api/globals/group_queries.rs` and `src/lua_api/globals/group_queries_relationships.rs`: aggregate offline query reads active members; party-token connectivity reads the same field. Other unit tokens, including existing raid aliases, retain their previous behavior.
- `tests/party_connection.rs`: eight grouped fixtures discovered by the existing generated `integration` harness; no separate Cargo target or input scaffolding.

## Tests asserting this spec

`tests/party_connection.rs` covers both synchronous edges with exact two-field payload and callback query snapshots; repeated inputs; absent valid index; two offline members and partial reconnect; shrink/regrowth; full removal/regrowth; and two-environment isolation. Two additional GREEN controls cover inactive retained members and unchanged non-party queries, including the shared lookup's raid aliases. Every fixture first requires the new setter to be callable. Calls must succeed directly, without `pcall` rejection checks that could mistake a missing method for valid rejection. Callback observations are asserted outside the callback, so swallowed handler errors cannot stand in for proof.

`tests/admin_party_api.rs::test_group_has_offline_member_defaults_false` remains unchanged: connected default members correctly yield false.

Proof ledger, 2026-10-01: tests/spec revision `03ebe69723ad7e01e54b05b261c75b6e19df77d7`; parent default compile (`cargo test --test integration --no-run --message-format=json`) exits 0 in 97.987s. Actual grouped runtime RED is 0/6, each failing the callable-setter assertion. Saved artifacts: `/tmp/patch-12.0.5-batch14-red-{build.json,build.log,build-result.json,run.json,run.log}`. Superseding bounded development GREEN at implementation `527cb2f57e992777ca5b4b75392fa0c9a8758da0`: `/tmp/patch-12.0.5-batch14-green-runs.json` records eight party connection fixtures, 28 admin-party controls and 19 admin-event controls, all passing (55 total); referenced `green-run-{0,1,2}.log` files contain the matching results. Integration binary SHA-256: `b30c011863771cea9c0376c364e3028e45642cb359bce2b4f4cc6722799dab16`. These prove bounded state/input/query/synchronous event behavior, not native transition semantics. Saved same-revision startup `/tmp/patch-12.0.5-batch14-green-startup-run.json` records exit 0; `green-startup.json` contains `[]`, with `green-startup.log` retained: zero startup Lua errors. Parent startup evidence records 7.849781865s (7.85s), binary SHA-256 `7fe44e2b00677b9a411a1e841beb5e36102dd0a8d6fb4bb6c989c7cf166ca771`; retained log reports CLEAN, 0 unique/0 occurrences. Startup is parent-owned saved evidence, not independently verified by the party report.

Independent bounded report `/tmp/patch-12.0.5-party-connection-independent-proof.md` records PASS for reused 55/55 runtime results, implementation wiring and changed-code readability. Its default `cargo fmt --check` exits 0 in 28.289s and `cargo check` exits 0 in 33.555s without warnings/errors at captured `527cb2f57` source/config scope (before/after snapshots identical). Final inspected HEAD `110cf15ae` changes only wiki index relative to implementation; concurrent unrelated `tests/forever_auto_roll.rs` changes invalidate whole-checkout formatter freshness, not party formatting. Subsequent unrelated tests are outside these recorded gates. Durable captures: `/tmp/patch-12.0.5-party-connection-independent/{before.json,after.json,results.json,final-scope.json}`. No blanket current-worktree formatting, all-test compilation, native, networking, all-profile execution, full-row, or whole-page completion claim.

## Known gaps (current cycle)

- [x] Independent bounded acceptance and snapshot-scoped default Rust gates recorded for implementation `527cb2f57`; subsequent unrelated tests and the concurrent `tests/forever_auto_roll.rs` edit are excluded, not current whole-worktree acceptance.

## Out of scope

Party-category expansion, networking, raid connection behavior, changes to player/target/focus connection semantics, new storage by profile, and native probes. Roster-size events retain their existing contract; these fixtures do not prescribe additional connection events for roster removal.
