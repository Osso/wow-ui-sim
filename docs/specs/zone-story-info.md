# Zone story lookup

`C_QuestLog.GetZoneStoryInfo` supplies the achievement and story-map pair used by the original zone-story header and tooltip on Retail and WoW Forever. The simulator returns only relationships supplied through `A_Admin`; native relationship data remains pending R88el.

## What it must do

- [x] On Retail and WoW Forever, return the supplied `(achievementID, storyMapID)` pair for a matching `uiMapID`; return no values when no relation was supplied. This is an INFERRED simulator contract (R88el (native evidence pending)).
- [x] Accept explicit positive integer IDs through `A_Admin.SetZoneStoryInfo(uiMapID, achievementID, storyMapID)`.
- [ ] Establish native default and relationship data from native evidence.
- [ ] Preserve the existing PTR default and registrations on other profiles.

## How it works

- [C API architecture](../wiki/systems/c-api.md) (implementation flow).

## Implementation inventory

- `src/c_api/c_quest_log.rs` — Retail/Forever query and admin input.
- `src/lua_api/state/sim_state.rs` — supplied map-story relations.
- `src/lua_api/state.rs` — empty initial state.
- `src/lua_api/workarounds/temporary/zone_story_default.rs` — PTR-only default.

## Tests asserting this spec

- `tests/zone_story_consumer.rs` — supplied relations through the original Blizzard header and tooltip.

## Known gaps (current cycle)

- [ ] Native evidence and authored map-story data remain unavailable.
- [ ] In the Forever headless integration fixture, the original tooltip consumes title and achievement progress but cannot finish its display because frame geometry is unresolved; Retail completes the tooltip path.

## Out of scope

- Generating achievement/map relations from unsupported catalogs.
- Changing PTR behavior or synthesizing default relationships.
