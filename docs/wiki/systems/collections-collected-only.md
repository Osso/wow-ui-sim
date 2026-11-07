# Collections collected-only journal tabs

`onlyShowCollectedItemsInJournal` makes the Collections journal show a tab only
when the account owns at least one entry of that collection, and disables the
journal entirely when every collection is empty. The contract is owned by
[Collections collected-only journal tabs](../../specs/collections-collected-only.md);
this page records how the simulator implements and proves it.

## Content

### CVar registration

`src/cvars.rs` registers `onlyShowCollectedItemsInJournal` with the pinned dump
default `1` and account scope for PTR and WowForever only. The pinned CVar dumps
are `data/patch-api/source-cache/legacy-2026-10-09/1.60.1-wikitext.txt:2046` and
`data/patch-api/sources/12.1.7-inherited-1.60.1.wikitext:1986`; the Wowless
catalogs carry the same account flag at
`crates/sim-data-profile/data/cvar_info_ptr.rs:1153` and
`cvar_info_wowforever.rs:1161`. The Retail 12.1.5 catalog omits the name, so the
CVar stays unregistered on Retail, Wrath, Mists, Era and Anniversary.

### Predicates

`src/c_api/journal_collected.rs` publishes three getters only on the two
profiles whose pinned `Blizzard_APIDocumentationGenerated` declares them:

| Getter | Declaration (PTR and WowForever pins) | Supplied state |
|---|---|---|
| `C_MountJournal.HasDisplayableMount` | `MountJournalDocumentation.lua:385` | `WorldState.mounts[].is_collected` |
| `C_ToyBoxInfo.HasAnyToy` | `ToyBoxInfoDocumentation.lua:21` | `WorldState.toys[].is_collected` |
| `C_TransmogCollection.HasAnyAppearance` | `TransmogItemsDocumentation.lua:691` | `WorldState.transmog_appearances[].is_collected` |

Each getter answers from that supplied journal state instead of a constant, so
an empty or partially collected fixture produces different answers. Whether
`HasDisplayableMount` also requires mount usability is UNKNOWN; the current
contract reports ownership only.

### Consumers

- `CollectionsJournal_ValidateTab`
  (`Blizzard_Collections/Shared/Blizzard_Collections.lua:18-51` on PTR,
  `:18-47` on WowForever) hides Mounts, Pets, Toys and Appearances tabs when the
  CVar is set and the matching collection is empty. The Pets tab reads
  `select(2, C_PetJournal.GetNumPets()) > 0`, which the pet-journal model already
  answers with the collected count.
- `CollectionsUtil.IsJournalDisabled()`
  (`Blizzard_FrameXMLUtil/CollectionsUtil.lua:543-565` on both profiles) returns
  true only when the CVar is set and all four collections are empty. The
  Collections micro button calls it from
  `Blizzard_MicroMenu/Mainline/MainMenuBarMicroButtons.lua:1744` (PTR) and
  `:1657` (WowForever) to disable itself and show
  `COLLECTIONS_JOUNRAL_DISABLED_TOOLTIP`.

The Retail pin has no collected-only branch: its
`CollectionsJournal_ValidateTab` (`Mainline/Blizzard_Collections.lua:13`) reads
no CVar, `IsJournalDisabled` does not exist, and none of the three predicates is
declared.

### Proof

`tests/test_showuipanel_toggles.rs` drives both original consumers:

- `journal_only_show_collected_cvar_gates_tabs_on_collectible_state` drives the
  pinned `CollectionsJournal_ValidateTab` with the CVar on and off, with none
  and with exactly one collected entry per collection, on Retail, PTR and
  WowForever. It also reads the registered default and account flag through
  `C_CVar.GetCVarInfo` on the two profiles that pin the CVar.
- `journal_collections_util_disables_journal_only_for_empty_collected_state`
  drives `CollectionsUtil.IsJournalDisabled()` on PTR and WowForever with the
  CVar on and off and the same supplied collection-count fixtures.

## Sources

- [Collections collected-only journal tabs](../../specs/collections-collected-only.md) — behavior contract and evidence citations.
- `src/c_api/journal_collected.rs` — predicate implementations and publication gate.
- `src/cvars.rs` — profile-gated CVar default registration.

## See Also

- [[lua-api]] — C API surface placement and namespace publication.
- [[client-profiles]] — per-profile feature and pinned-source selection.
- [Collections availability](../../specs/collections-availability.md) — Heirlooms and Warband Scenes tab availability.
