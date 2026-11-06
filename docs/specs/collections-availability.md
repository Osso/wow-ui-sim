# Collections availability getters

`C_HeirloomInfo.HeirloomsAvailable`, `C_MountJournal.MountEquipmentAvailable` and
`C_WarbandScene.WarbandScenesAvailable` tell the Collections journal whether the
Heirloom collection, the mount-equipment subsystem and the Warband Scenes journal
exist for the active character. The contract lives in
`src/c_api/collections_availability.rs`; [how it works](../wiki/systems/collections-availability.md)
is tracked in the wiki.

## What it must do

- [x] Publish these getters only on profiles whose pinned
      `Blizzard_APIDocumentationGenerated` declares them: PTR 12.1.5 and
      WowForever 1.60.1 (`HeirloomInfoDocumentation.lua:29`,
      `MountJournalDocumentation.lua:498`, `WarbandSceneInfoDocumentation.lua:92`).
      They stay unpublished on Retail, Wrath, Mists, Era and Anniversary, whose
      pins do not declare them.
- [x] Each getter returns supplied `WorldState.collections_availability` state,
      not a constant.
- [x] `CollectionsJournal_ValidateTab(COLLECTIONS_JOURNAL_TAB_INDEX_HEIRLOOMS)`
      hides the Heirlooms tab when `heirlooms` is false and shows it when true.
- [x] `MountJournal_OnLoad` (`Blizzard_MountCollection.lua:161`) keeps
      `MountJournal.BottomLeftInset` shown and wires `MountJournal.SlotButton`
      when `mount_equipment` is true; it hides `BottomLeftInset` and leaves
      `SlotButton` unset when false.
- [x] `CollectionsJournal_ValidateTab(COLLECTIONS_JOURNAL_TAB_INDEX_WARBAND_SCENES)`
      hides the Warband Scenes tab when `warband_scenes` is false and shows it
      when true.
- [ ] Native default value and whether any getter can change during a
      session: UNKNOWN. The current defaults (`true` for all three) are
      best-guess scenario defaults for the two clients that ship these
      collections.

## How it works

- [Collections availability](../wiki/systems/collections-availability.md)

## Implementation inventory

- `src/c_api/collections_availability.rs` — state struct and both namespace registrations.
- `src/lua_api/state_types/character_world.rs` — `collections_availability` field.
- `src/lua_api/globals/missing_surface/heirloom.rs` — registers `HeirloomsAvailable` on `C_HeirloomInfo`.
- `src/lua_api/globals/missing_surface/warband_scene.rs` — registers `WarbandScenesAvailable` on `C_WarbandScene`.
- `src/lua_api/globals/font_strings_collection/mount_journal.rs` — registers `MountEquipmentAvailable` on `C_MountJournal`.

## Tests asserting this spec

- `tests/test_showuipanel_toggles.rs::collections_heirloom_tab_follows_heirlooms_available`
- `tests/test_showuipanel_toggles.rs::collections_mount_equipment_inset_follows_availability`
- `tests/test_showuipanel_toggles.rs::collections_warband_scene_tab_follows_availability`
- `tests/test_showuipanel_toggles.rs::toggle_collections_journal_opens_mounts_pets_and_toys_tabs_and_accepts_search_text` — end-to-end journal open/switch/close path.

## Known gaps (current cycle)

- [ ] Native semantics and defaults for the three getters; an aso/decomp
      answer is pending. Fold the answer into "What it must do" when it lands.

## Out of scope

- Mount-equipment mutation APIs (`ApplyMountEquipment`, applied item ID, effects
  suppression): existing stubs, unchanged by this spec.
