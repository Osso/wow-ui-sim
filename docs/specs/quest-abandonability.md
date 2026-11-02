# Retail quest abandonability

`C_QuestLog.CanAbandonQuest` reads supplied quest records and classification state for the original quest map.

## What it must do

- [ ] Require a signed numeric quest ID; accept numeric strings and truncate finite fractions.
- [ ] Return false for nonpositive IDs or records not ready. Ready records need not be in the quest log.
- [ ] Return false when the supplied classification has flag 256; absent classification does not block a ready record.
- [ ] When record flag 2048 is supplied, use the host-supplied conditional result; missing conditional state is an error, not a guessed result.
- [ ] Quest log entries are treated as ready simulator records. Additional ready IDs may be supplied independently.

## Best-guess basis

Retail `QuestLogDocumentation.lua:46` declares the API. Retail build 70688 local static evidence establishes the record and flag decisions. The boolean publication path is emulated, not observed in a running client. Local `Blizzard_UIPanels_Game/Mainline/QuestMapFrame.lua:1084` consumes the result.

## How it works

- [Quest log state](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_quest_log.rs` registers and evaluates the Retail query.
- `src/lua_api/sim_substates/quest_log.rs` stores supplied record and classification inputs.

## Tests asserting this spec

- `tests/objective_tracker_tree.rs::objective_tracker_frame_layout_is_locked` exercises the original quest-map event path.

## Known gaps (current cycle)

- [ ] Native 2048 conditional inputs and transitions, protected-value behavior and exact errors remain unknown.
- [ ] Legacy global equivalence is not established.

## Out of scope

Quest data network requests and native cache lifecycle are not modeled.
