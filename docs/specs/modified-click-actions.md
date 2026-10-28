# Modified-click actions

Modified-click action bindings select the action used by Blizzard item-click handling.

## What it must do

- [ ] With BUTTON1 down, Shift selects CHATLINK and Control selects DRESSUP in the Retail `HandleModifiedItemClick` consumer.
- [ ] Action bindings can be read and changed through `GetModifiedClick` and `SetModifiedClick`.
- [ ] Defaults come from the selected profile's Blizzard `ModifiedClick` definitions, not invented values.

Retail's standard definitions are `Blizzard_FrameXML/Bindings_Standard.xml:1776-1797`; CHATLINK is `SHIFT-BUTTON1` and DRESSUP is `CTRL-BUTTON1` at lines 1782–1783. The selected `Blizzard_ItemButton_Mainline.toc:5-9` loads `Mainline/ItemButtonTemplate.xml:3`, which loads the consumer at `Mainline/ItemButtonTemplate.lua:367-400`. That consumer checks CHATLINK before DRESSUP.

## How it works

- See [Lua API](../wiki/systems/lua-api.md).

## Implementation inventory

- `src/lua_api/globals/real/modifier_keys.rs` — modifier queries.
- `src/lua_api/workarounds/temporary/modified_click_defaults.rs` — existing temporary binding state.

## Tests asserting this spec

- `tests/blizzard_item_button_loads.rs` — selected Retail consumer routing.

## Known gaps (current cycle)

- [ ] Native semantics for extra held modifiers, NONE, unknown or nil action, button matching outside a click, and Get/Set return values are UNKNOWN pending native evidence.

## Out of scope

- Full input binding persistence and unrelated input behavior.
