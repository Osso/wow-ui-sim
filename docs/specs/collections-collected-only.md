# Collections collected-only journal tabs

`onlyShowCollectedItemsInJournal` makes the Collections journal hide a
collection tab until the account has collected at least one entry of that
collection. The three collected-only predicates the branch calls live in
`src/c_api/journal_collected.rs`; [how it works](../wiki/systems/collections-collected-only.md)
is tracked in the wiki.

## What it must do

- [x] Register `onlyShowCollectedItemsInJournal` with the pinned dump default
      `1` and account scope on PTR and WowForever. Evidence:
      `data/patch-api/source-cache/legacy-2026-10-09/1.60.1-wikitext.txt:2046`
      and `data/patch-api/sources/12.1.7-inherited-1.60.1.wikitext:1986`
      (`default=1`, `scope=Account`); the Wowless catalog carries the account
      flag at `crates/sim-data-profile/data/cvar_info_ptr.rs:1153` and
      `cvar_info_wowforever.rs:1161`. The Retail 12.1.5 catalog omits the name,
      so it stays unregistered on Retail, Wrath, Mists, Era and Anniversary.
- [x] Publish `C_MountJournal.HasDisplayableMount`,
      `C_ToyBoxInfo.HasAnyToy` and `C_TransmogCollection.HasAnyAppearance` only
      on PTR and WowForever, whose pinned
      `Blizzard_APIDocumentationGenerated` declares them at
      `MountJournalDocumentation.lua:385`, `ToyBoxInfoDocumentation.lua:21` and
      `TransmogItemsDocumentation.lua:691`. The Retail 12.1.5 pin declares none
      of them.
- [x] Each predicate answers from supplied journal state instead of a constant:
      a collected mount in `WorldState.mounts`, a collected toy in
      `WorldState.toys`, and a collected appearance in
      `WorldState.transmog_appearances`.
- [x] `CollectionsJournal_ValidateTab` uses the predicates on PTR
      (`Blizzard_Collections/Shared/Blizzard_Collections.lua:32-49`) and
      WowForever (`:28-45`): with the CVar set, a tab is visible exactly when
      its collection owns at least one entry, and every tab is visible when the
      CVar is cleared.
- [x] The Retail pin keeps every tab visible because its
      `CollectionsJournal_ValidateTab`
      (`Blizzard_Collections/Mainline/Blizzard_Collections.lua:13`) has no
      collected-only branch and reads no CVar.
- [ ] Whether `HasDisplayableMount` also requires mount usability: UNKNOWN. The
      contract reports ownership only.
- [ ] Whether the predicates consider the legacy `collected_mounts`,
      `collected_toys` and `collected_transmogs` admin sets: UNKNOWN. The
      contract reads the journal state that the matching collection getters
      already publish.

## How it works

- [Collections collected-only journal tabs](../wiki/systems/collections-collected-only.md)

## Implementation inventory

- `src/c_api/journal_collected.rs` — the three predicates and the PTR/WowForever publication gate.
- `src/cvars.rs` — `onlyShowCollectedItemsInJournal` default registration.
- `src/lua_api/globals/font_strings_collection/mount_journal.rs` — publishes `HasDisplayableMount` on `C_MountJournal`.
- `src/lua_api/globals/font_strings_collection/toy_box.rs` — publishes `HasAnyToy` on `C_ToyBoxInfo`.
- `src/lua_api/globals/missing_surface/transmog_collection.rs` — publishes `HasAnyAppearance` on `C_TransmogCollection`.

## Tests asserting this spec

`tests/test_showuipanel_toggles.rs::journal_only_show_collected_cvar_gates_tabs_on_collectible_state`
drives the pinned `CollectionsJournal_ValidateTab` with the CVar on and off,
with none and with one collected entry per collection, on Retail, PTR and
WowForever. It also reads the registered default and account flag through
`C_CVar.GetCVarInfo` on the two profiles that pin the CVar.

## Known gaps (current cycle)

- [ ] `Blizzard_FrameXMLUtil/CollectionsUtil.lua:544`
      `CollectionsUtil.IsJournalDisabled()` is the second pinned consumer of the
      same predicates and CVar. No test drives it yet.
- [ ] Native semantics and session mutability of the three predicates; an
      aso/decomp answer is pending.
- [ ] Retail 12.1.5 CVar absence rests on the pinned Wowless catalog
      (`crates/sim-data-profile/data/cvar_info_retail.rs`) and the absence of
      any Retail consumer; no Retail CVar dump is pinned.

## Out of scope

- Pet-journal collected counting: the pinned branch already uses
  `select(2, C_PetJournal.GetNumPets()) > 0`, unchanged here.
- Mount equipment availability and the Heirlooms/Warband Scenes tabs: see
  [Collections availability](collections-availability.md).
- Actual collection list filtering by the CVar; this contract covers tab
  visibility only.
