# Retail LFG expand-search confirmation

`C_LFGInfo.ConfirmLfgExpandSearch` acts on a host-supplied cached prompt record. The original LFG popup binds its accept action to this API.

## What it must do

- [ ] For a supplied nonzero prompt key, clear the key and supplied prompt data, even when no connection exists.
- [ ] Do not change the queued dungeon model or invent a queue expansion.

## Best-guess basis

`LFGInfoDocumentation.lua:86` declares the function. `Blizzard_GroupFinder/Shared/LFGFrame.lua:92` binds the original accept callback. Retail build 70688 static evidence shows the nonzero-key decision and clearing; actual transport and runtime transitions are not observed.

## How it works

- [Event system](../event-system.md)

## Implementation inventory

- `src/c_api/c_lfg_info.rs` registers and consumes the prompt.
- `src/lua_api/state/sim_state.rs` stores the host-supplied prompt.

## Tests asserting this spec

- `tests/patch_9_0_1_publication_sweep.rs::lfg_expand_prompt_accept_clears_supplied_state` calls the original popup binding with two supplied prompts.

## Known gaps (current cycle)

- [ ] Confirmation transport, connection state, event delivery and the behavior when not queued are unknown.
- [ ] A zero-key prompt is not equated with an unqueued client; its remaining behavior is unmodeled.

## Out of scope

Actual matchmaking and server-side expansion are not simulated.
