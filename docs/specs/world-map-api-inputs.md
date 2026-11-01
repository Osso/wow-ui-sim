# Retail world-map API inputs

Retail's pinned `Blizzard_APIDocumentationGenerated/MinimapDocumentation.lua:28` and `QuestLogDocumentation.lua:125,692` declare the queries consumed by `Blizzard_WorldMapTemplates.lua:112,724` and `Blizzard_UIPanels_Game/Mainline/WorldMapBountyBoard.lua:160`.

## What it must do

- [ ] Project initialized minimap tracking defaults with complete bit-mask containment, including combined filters.
- [ ] Query map bounties from supplied simulator state and distinguish missing map resolution from an empty resolved list.
- [ ] Scan the current quest log for eligible entries with native-observed flag masks.
- [ ] Exercise the original Retail world-map consumers against distinct input states.

## Best-guess basis

Native build 70688 evidence WM1–WM3 reports VERIFIED local decisions and EMULATED result publication. The Lua-visible connection is a best guess; no live client transition was observed. Map/bounty selections are supplied scenario state, not a native catalog.

## How it works

- [Bounty-set associations](bounty-set-map-relations.md)
- [Minimap filter default](minimap-filter-default.md)

## Implementation inventory

Pending.

## Tests asserting this spec

`tests/test_keybindings_panels_detail.rs`: original filter, bounty board, and threat frame cases.

## Known gaps (current cycle)

- [ ] Real-client publication and runtime transitions are unobserved; inferred defaults and results are not live-client proof.
- [ ] Native bounty selection and map ancestry rules are not established by the provided evidence.
- [ ] Quest flag domain meanings and producers are unknown; do not infer them from their bit values.

## Out of scope

Publication sweeps, C_Widget, other lanes and historical client profiles.
