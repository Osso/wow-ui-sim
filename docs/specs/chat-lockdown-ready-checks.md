# Chat lockdown ready checks

Bounded Retail 12.0.5 ready-check restriction contract for the explicit input in [chat messaging lockdown](chat-messaging-lockdown.md). Existing public producers live in `src/c_api/c_party_info.rs` and `src/lua_api/globals/group_verbs.rs`; the shared producers now reject explicit lockdown before mutations or dispatch. See [event system](../event-system.md) for dispatch architecture.

Retained source [`prose-2026-03-31-169`](../../data/patch-api/sources/12.0.5-register.json), source line 169, states: “The recent restrictions to countdown, ready check, ping and loot method APIs have been loosened to only apply when in chat messaging lockdown rather than in all combat.” This establishes the restriction axis; it does not establish native error behavior or exact ready-check event payloads.

## What it must do

- [ ] With explicit `chat_messaging_lockdown = true`, reject `C_PartyInfo.DoReadyCheck()` and `C_PartyInfo.ConfirmReadyCheck(bool)` before changing either `ready_check.active` or `ready_check.response`, or dispatching any `READY_CHECK`, `READY_CHECK_CONFIRM`, or `READY_CHECK_FINISHED` event. Cover combat false and true, fresh/inactive state, and preexisting active/response state.
- [ ] With explicit lockdown false, preserve existing ready-check behavior independently of combat: start sets active true and clears response; confirmation sets active false and stores either true or false. Clearing lockdown after rejection permits subsequent operations in the same environment without changing combat.
- [ ] Preserve existing synchronous observable events: start emits one `READY_CHECK` with zero arguments; confirmation emits `READY_CHECK_CONFIRM("player", response)` then `READY_CHECK_FINISHED()` exactly once each. Real frame handlers observe the updated queried status and active time-left state. These payload/timing expectations characterize existing simulator behavior, not native parity.
- [ ] Apply the same restriction and unlock recovery to existing legacy `ReadyCheck()`, including fresh and preexisting active/response fixtures; no alias bypass.
- [ ] **SIMULATOR INFERENCE:** report a blocked call as an explicit runtime error detectable by `pcall`. Native blocked-error convention is **UNKNOWN**; no native name/message parity is asserted. Tests require failure and an error value, not a hardcoded full error string.

The test module is gated by both `profile-retail` and `retail-12-0-5` (12.0.5+ epoch), matching the explicit input. No combat-derived restriction, new input, Lua setter, security change, or profile expansion is authorized.

## How it works

- [Chat messaging lockdown input/predicate contract](chat-messaging-lockdown.md)
- [Event system](../event-system.md)
- [Lua API environment/state architecture](../lua-api.md)

## Implementation inventory

- `tests/chat_lockdown_ready_checks.rs` — six grouped autodiscovered fixtures in the existing integration target; no Cargo target added.
- `src/c_api/c_party_info.rs` — existing namespaced start/confirm producers; unchanged here.
- `src/lua_api/globals/group_verbs.rs` — shared start/confirm producers call the guard first under `retail-12-0-5`; existing legacy `ReadyCheck()` funnels through start. Unlocked mutations, synchronous events and payloads remain unchanged; older profiles retain existing behavior.
- `src/c_api/c_chat_info.rs` — C API-owned shared guard reads only `SimState.chat_messaging_lockdown` and returns a contextual runtime error when true. Rejection/error wording is inferred simulator policy, not native characterization.
- `src/c_api/mod.rs` — exposes the existing epoch-gated module within the crate for shared producer calls; module gate unchanged.
- `src/lua_api/state/support_types.rs` — existing `ReadyCheckState` with `active` and `response`; unchanged here.
- Input `f777027be` and predicate `18b09cbf9` are prerequisites owned by the linked predicate contract.

Inspection finds `ReadyCheck()` registered and calling shared `start_ready_check`. No legacy `ConfirmReadyCheck` or `DoReadyCheck` global is registered by the existing ready-check producers; those names are namespaced only. No invented legacy-confirm fixture or alias is added.

## Tests asserting this spec

`tests/chat_lockdown_ready_checks.rs` contains six tests:

| Filter suffix | Contract |
| --- | --- |
| `public_start_blocks_fresh_state_on_both_combat_axes` | Fresh blocked start; both combat axes; unchanged state and zero events. |
| `public_confirm_blocks_inactive_state_for_both_responses_and_combat_axes` | Inactive blocked confirmation; both responses and combat axes; unchanged state and zero events. |
| `public_start_preserves_active_response_then_recovers_after_unlock` | Active true/false response preserved, then start/confirm recover on both combat axes. |
| `public_confirm_preserves_active_response_then_recovers_after_unlock` | Active opposite response preserved, then confirmation recovers on both combat axes. |
| `unlocked_public_lifecycle_ignores_combat_for_both_responses` | Lockdown false lifecycle, exact counts/order/payloads and handler-visible state on both combat axes. |
| `legacy_ready_check_cannot_bypass_lockdown_and_recovers_after_unlock` | Existing alias blocked from fresh/active response inputs, then recovers on both combat axes. |

Parent filter: `cargo test --test integration chat_lockdown_ready_checks::`. Predicate control filter: `cargo test --test integration chat_messaging_lockdown::`. Existing legacy control: `cargo test --test integration group_verbs::ready_check_fires_event`.

**Proof ledger:** parent artifacts `/tmp/patch-12.0.5-batch31-red-build-result.json` and `-red-run.log` bind compiled RED to tests/spec `8649fe072`: compile exit 0 in 162.49s; six selected tests, one unlocked lifecycle PASS and five FAIL at actual locked-state mutations. This producer slice adds guards and formatting only; no builds/checks/tests run here. Bounded GREEN and independent verification remain parent-owned. All requirements remain unchecked pending acceptance; no source-row or whole-audit acceptance credit.

## Known gaps (current cycle)

- [ ] Bounded GREEN for the six ready-check fixtures, predicate and existing group controls after the guarded producer change.
- [ ] Independent acceptance of state preservation, real event silence/payloads, unlock recovery and both combat axes.

## Out of scope

- **Countdown, ping, loot:** countdown API restrictions, ping API restrictions and loot-method API restrictions in the same retained prose row remain unenforced/unproved by this slice; ready-check coverage cannot close that whole row.
- **Messages/channels:** chat/addon message sending and channel join/leave/mutation restrictions remain open; these fixtures make no claims about them.
- **Native producer:** lockdown activation, reset, ordering, encounter/M+/PvP inputs and native blocked-error convention remain unknown/unverified; explicit Rust bool input is not a native producer.
- **Macros:** macro evaluation/execution restrictions remain open; no macro path is exercised.
- Other profiles, vendor behavior, broader security/taint semantics, startup acceptance and whole-audit closure remain unchanged/unclaimed.
