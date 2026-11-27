# LFG expand-search confirmation (Retail and WoW Forever)

`C_LFGInfo.ConfirmLfgExpandSearch` acts on a host-supplied cached prompt record. The original LFG popup binds its accept action to this API.

## What it must do

- [ ] For a supplied nonzero prompt key, clear the key and supplied prompt data, even when no connection exists.
- [ ] Do not change the queued dungeon model or invent a queue expansion.

## Best-guess basis

`LFGInfoDocumentation.lua:86` declares the function; the Retail and Forever blocks are byte-identical. `A_Admin.ShowLfgExpandSearchPrompt(key, data)` supplies the prompt and fires `SHOW_LFG_EXPAND_SEARCH_PROMPT`. `Blizzard_GroupFinder/Shared/LFGFrame.lua:92` binds the original accept callback. Retail build 70688 static evidence shows the nonzero-key decision and clearing; actual transport and runtime transitions are not observed.

## How it works

- [Event system](../event-system.md)

## Implementation inventory

- `src/c_api/c_lfg_info.rs` registers and consumes the prompt.
- `src/lua_api/state/sim_state.rs` stores the host-supplied prompt.

## Tests asserting this spec

- `tests/patch_9_0_1_publication_sweep.rs::lfg_expand_prompt_accept_clears_supplied_state` calls the original popup binding with two supplied prompts.
- `tests/lfg_expand_search_prompt.rs` accepts the original Retail popup with nonzero and zero keys. Forever excludes Blizzard_GroupFinder (`## ExcludeLoadGameType: camelot`), so its case calls the API directly with the same inputs.

## Known gaps (current cycle)

- [ ] Confirmation transport, connection state, event delivery and the behavior when not queued are unknown.
- [ ] A zero-key prompt is not equated with an unqueued client; its remaining behavior is unmodeled.

## Out of scope

Actual matchmaking and server-side expansion are not simulated.
