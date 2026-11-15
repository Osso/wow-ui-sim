# Empty color override query

`C_ColorOverrides.GetColorOverrideInfo(overrideType)` reports no override until
an override registry is modeled. This temporary simulator policy preserves the
original ColorManager default-color path. See [Lua API architecture](../lua-api.md).

## What it must do

- [x] Return exactly one nil for an override type when no override registry exists.
- [x] Original Retail `ColorManager.GetColorDataForBagItemQuality` returns its
  default bag color for a quality with an entry in `ITEM_QUALITY_OVERRIDES`.
- [x] Original CharacterFrame opening retains equipment icons and player title.

Cached `Blizzard_APIDocumentationGenerated/ColorOverridesDocumentation.lua`
declares the required `ColorOverride` argument and nullable `ColorOverrideInfo`
result. Nil is this simulator's empty-state policy, not native default proof.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/workarounds/temporary/color_defaults.rs`: temporary empty query;
  retire when a color override registry supplies the query through `c_api`.

## Tests asserting this spec

- `tests/blizzard_ui_unit.rs::original_color_manager_bag_quality_uses_default_without_override`
- Existing `character_panel_equipment_slots_match_inventory_or_background_textures`
  and `character_panel_title_text_matches_player_name` preservation cases.

## Known gaps (current cycle)

- [ ] R88v.v1–v3 verify the native six-member `C_ColorOverrides` model, GlobalColor defaults, and `playerColorOverrides` persistence. That full stateful model is deferred to a follow-up; keep the four absent members absent and do not add partial handlers in this lane. The reload/logout chain and exact persisted-string encoding remain unverified here.

Original pre-postload consumer RED at test `eab97a0f9` against the sealed
`8c3f5646d` Retail library reaches ColorManager.lua:95 (missing query).
Normal Retail library/binary `019ac27ff` passes the new consumer and both
unchanged CharacterFrame cases (three unique logical cases); separate startup
returns `[]`, exit 0. Original CVar stat/hash remains unchanged. Exact private
receipts: `handoff/wrath-fixes-redgreen/color-retail-green-preservation-epoch.txt`
and `color-retail-green-startup-comparison.json` under the simulator state root.
The limited fixture still reports unrelated load diagnostics; these are not a
clean full-fixture or native-parity claim. The rilua unwind bug remains unfixed.

## Out of scope

Override mutation, populated override records, general color registry fidelity,
native parity, and the separately reported rilua error-unwind defect.
