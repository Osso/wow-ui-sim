# Quest accept confirmation

Bounded Retail 12.0.5 `QUEST_ACCEPT_CONFIRM` input tied to the existing pending quest offer. Producer lives beside `A_Admin.OpenQuestNpc` in `src/lua_api/globals/admin_quests.rs`; targeted GREEN remains pending. See [event dispatch](../event-system.md) and [Lua API](../wiki/systems/lua-api.md).

## What it must do

- [ ] Under `profile-retail` + `retail-12-0-5`, expose `A_Admin.RequestQuestAcceptConfirmation(name, questTitle)`, with two explicit string inputs. Derive event `questID` exclusively from current `pending_quest_offer`; do not invent a sharer name or substitute the gossip title.
- [ ] With a pending offer, synchronously publish exactly `(name:string, questTitle:string, questID:number)` to real frame listeners before returning. Preserve pending offer, selected quest and quest log until an acceptance/close action occurs. Existing selected quest must be queryable during the callback; acceptance inside that callback must consume the matching pending offer.
- [ ] Selecting an available quest alone still sets the offer and emits `QUEST_DETAIL`, not `QUEST_ACCEPT_CONFIRM` or an acceptance transition.
- [ ] `ConfirmAcceptQuest()` consumes the offer, adds that ID once to the existing log, and queues existing `QUEST_ACCEPTED` without re-prompting. Calling it again with no offer remains inert.
- [ ] Preserve established simulator `CloseQuestFrame()` semantics: clear pending offer, queue `QUEST_FINISHED`, leave log unchanged and emit no acceptance or new prompt. **Chosen simulator policy:** confirmation input with no pending offer is a silent no-op, both initially and after close; no event or state mutation. This is not native-verified default behavior.

### Grounding and policy limits

Inspected profile cache on 2026-10-01 under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`:

- `Blizzard_APIDocumentationGenerated/QuestOfferDocumentation.lua:64-74` declares `SynchronousEvent = true`; payload names/types are `name:cstring`, `questTitle:cstring`, `questID:number`, all nonnil. This is cached source evidence for the added third argument, not a live native probe.
- `Blizzard_Game/Mainline/EventRouting.lua:83` forwards the event to `GameEvent.HandleQuestAcceptConfirm`. `Mainline/EventImplementation.lua:331-338` consumes only the first two arguments, queries `C_QuestLog.GetNumQuestLogEntries()`, then shows `QUEST_ACCEPT` or `QUEST_ACCEPT_LOG_FULL`. No third-argument query/validation behavior is established by this consumer.
- `Blizzard_StaticPopup_Game/GameDialogDefs.lua:1582-1605` defines YES callbacks invoking `ConfirmAcceptQuest()`. Neither confirmation popup defines `OnCancel`; a native decline transition or pending-state effect cannot be inferred from the NO button alone. Fixtures cover the separate established simulator close action, not popup cancellation.
- `src/lua_api/globals/missing_surface/gossip_info.rs:101-121` selects an existing available row, sets `pending_quest_offer` and `selected_quest_log_id`, and dispatches `QUEST_DETAIL`. `src/lua_api/globals/quest_verbs.rs:67-97` establishes acceptance, deduplication, absent-offer no-op and close behavior in the current simulator.

The new admin name/signature and absent-offer policy are explicit simulator design choices. Two string inputs avoid a redundant caller-supplied quest ID and any invented mismatch-validation contract. Existing pending ID is the only association; this scope does not introduce a second confirmation record, acceptance gating, or payload defaults. Malformed-input/native validation semantics remain unspecified.

## How it works

- [Existing event queue and synchronous dispatch](../event-system.md)
- [Existing Lua API subsystem](../wiki/systems/lua-api.md)

## Implementation inventory

- `tests/quest_accept_confirmation.rs`: five grouped fixtures, gated by Retail profile and 12.0.5 epoch; no separate Cargo target.
- `src/lua_api/globals/admin_quests.rs`: gated explicit confirmation input; decodes two strings through `FromStack`, reads only the pending offer ID, releases the state borrow, and roots event strings across synchronous listeners. No offer is a no-op; no new storage or transition.
- `src/lua_api/globals/missing_surface/gossip_info.rs`: existing available-quest selection producer; unchanged.
- `src/lua_api/globals/quest_verbs.rs`: existing acceptance/close transitions; unchanged.

## Tests asserting this spec

`tests/quest_accept_confirmation.rs` exercises actual `A_Admin.OpenQuestNpc` → `C_GossipInfo.SelectAvailableQuest` ordering, then the explicit admin input. Listeners assert exactly three arguments, concrete values/types and selected-quest visibility. Explicit confirmation title differs from seeded gossip title. Rust observes actual `pending_quest_offer`, `quest_log` and queued lifecycle events; no query globals, handlers or event dispatchers are replaced. Acceptance inside the prompt callback checks pending-state availability at the real dispatch boundary.

**Proof ledger:** actual parent RED at `81421d0574b6af3c3fc289216a6e910e4ee3806e` on 2026-10-01: 1 PASS (selection control), 4 FAIL at missing `A_Admin.RequestQuestAcceptConfirmation`; saved `/tmp/patch-12.0.5-batch16-red-run.log` and `.json` identify the integration binary and hash. Producer follows this RED; scoped `rustfmt` only, no new build/check/test, delegation or push. Parent owns targeted GREEN and final gates; requirement boxes remain unverified. Unrelated bin-test compilation failure at `enable_state.rs:225` (missing `AddonMetadata.addon_dir`) is outside this scope.

## Known gaps (current cycle)

- [x] Parent compiled integration artifact and ran actual targeted RED: selection passed; four confirmation fixtures stopped at missing producer.
- [x] Implement bounded producer after valid RED.
- [ ] Parent execute targeted GREEN and required final gates.

## Out of scope

Native payload validation/defaults, popup cancellation/decline behavior, quest-sharing transport, full-log rejection policy, rich quest-log projection, generic event injection, source-row completion/accounting and changes to other supported profiles. Producer must be gated to the Retail profile/12.0.5 epoch; existing historical/non-Retail selection and acceptance behavior must remain unchanged. Fixtures assert simulator model transitions, not loaded Blizzard popup integration or native parity.
