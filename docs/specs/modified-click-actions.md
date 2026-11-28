# Modified-click actions

Modified-click action bindings select the action used by Blizzard item-click handling.

## What it must do

- [ ] With BUTTON1 down, Shift selects CHATLINK and Control selects DRESSUP in the Retail `HandleModifiedItemClick` consumer.
- [ ] Action bindings can be read and changed through `GetModifiedClick` and `SetModifiedClick`.
- [ ] Defaults come from the selected profile's Blizzard `ModifiedClick` definitions, not invented values. Retail and Forever keep the setting in Rust; other profiles retain the legacy Lua state.

**VERIFIED:** the loader selects `Blizzard_FrameXML/Bindings_Standard.xml` for Retail and PTR; PTR declares SELFCAST=ALT, FOCUSCAST=NONE, and AUTOLOOTTOGGLE=SHIFT at lines 1776–1781. Retail's standard definitions at lines 1776–1797 include CHATLINK=`SHIFT-BUTTON1` and DRESSUP=`CTRL-BUTTON1` at lines 1782–1783. The selected `Blizzard_ItemButton_Mainline.toc:5-9` loads `Mainline/ItemButtonTemplate.xml:3`, which loads the consumer at `Mainline/ItemButtonTemplate.lua:367-400`. That consumer checks CHATLINK before DRESSUP.

## How it works

- See [Lua API](../wiki/systems/lua-api.md).

## Implementation inventory

- `src/lua_api/globals/real/modifier_keys.rs` — modifier queries.
- `src/lua_api/state/sim_state.rs` — Retail and Forever modified-click state, initialized from selected XML by `src/loader/addon.rs`.
- `src/lua_api/workarounds/temporary/modified_click_defaults.rs` — legacy-only binding state.

## Tests asserting this spec

- `tests/blizzard_item_button_loads.rs` — selected Retail consumer routing.
- `tests/keybindings.rs::modified_clicks_round_trip_through_runtime_surface` — selected Retail/PTR defaults after `Blizzard_FrameXML` loads.
- `tests/startup_api_stubs.rs::startup_client_time_camera_and_channel_globals_are_callable` — Retail/PTR SELFCAST default after `Blizzard_FrameXML` loads.

**VERIFIED:** Retail `Bindings_Standard.xml:1779` and Forever `Bindings_Camelot.xml:1537` define `AUTOLOOTTOGGLE=SHIFT`. Both selected `Blizzard_SettingsDefinitions_Frame.toc` files have `AllowLoad: Game` and load `Controls.xml`, which selects `Controls.lua:65`; `Blizzard_Settings_Shared/Blizzard_Setting.lua:366–375` reads and writes the binding through the setting. The typed string setting behavior is inferred from the shared consumer; native edge-case behavior remains unknown.

## Known gaps (current cycle)

- [ ] Native semantics for extra held modifiers, NONE, unknown or nil action, button matching outside a click, and Get/Set return values are UNKNOWN pending native evidence.

## Out of scope

- Full input binding persistence and unrelated input behavior.
