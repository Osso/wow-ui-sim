# World map interaction closure

The original WorldMap OnHide closes the WorldMap player interaction. Retail, PTR and WoW Forever declare `C_Map.CloseWorldMapInteraction` in `Blizzard_APIDocumentationGenerated/MapDocumentation.lua:30` (the Retail and Forever blocks are byte-identical). `A_Admin.OpenWorldMapInteraction()` supplies the interaction ([interactions](../admin-api/interactions.md)).

## What it must do

- [ ] Closing removes active interaction type 29 and locally dispatches `PLAYER_INTERACTION_MANAGER_FRAME_HIDE` with type 29.
- [ ] No active map interaction, or another interaction alone, leaves state unchanged and dispatches no hide event.
- [ ] A normal call returns zero values.

## How it works

- [Event system](../event-system.md)

## Tests asserting this spec

- `tests/blizzard_world_map_loads.rs::world_map_hide_closes_only_the_map_interaction`
- `tests/world_map_interaction_close.rs` (Retail and Forever): closing the original world map ends the supplied interaction and leaves a mailbox interaction open.

## Out of scope

- Native interaction identity, callback storage and conditional immersive events not represented in the current simulator model.
