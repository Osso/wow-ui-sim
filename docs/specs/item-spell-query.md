# Item spell query

## What it must do

- All seven simulator profiles expose `C_Item.GetItemSpell(itemInfo)` through one query handler and the existing item-ID/name parser. Wrath, Mists, Era, Anniversary and Forever also expose the legacy `GetItemSpell` global through that handler, following `ClientProfile::has_legacy_item_globals`. Unresolved numeric/string selectors return zero values; other argument types fail.
- Each profile supplies its own complete item-effect mapping and spell names. Retail examines every association, rejects duplicate or invalid legacy slot indices, then selects the first positive trigger-zero spell in slot order. Other profiles preserve the prior catalog policy: consider positive trigger-zero candidates and select the lowest legacy slot, then effect ID. Non-use effects do not invalidate those candidates. Generation must retain passive effects, nonpositive spell IDs, invalid/duplicate slots and associations without names rather than preselecting one use spell.
- When Retail has no selected use spell, the supplied equipped item's enchantment can provide its first type-seven spell. A selected spell without a name returns zero values without falling through to the enchantment. Other profiles do not acquire the Retail enchantment fallback.
- Cursor-grid eligibility, Retail range/usability queries and Retail artifact-power classification read the same profile-specific effect mapping. An unnamed selected spell remains available to consumers that need its ID.
- `ChatFrameUtil.InsertLink` can use the public query result to choose the `/use` macro branch for an item with a named spell and the `/equip` branch otherwise.

## Profile data and provenance

Compile-time-selected catalogs live in `crates/sim-data-profile/data/item_effects_<profile>.rs`. Each catalog contains:

- `ITEM_EFFECTS`: `(item_id, effect_id, signed_spell_id, trigger, legacy_slot)` for every resolvable item/effect association.
- `ITEM_EFFECT_METADATA`: sparse `(effect_id, specialization_id, player_condition_id)` rows, emitted only when an available metadata field is nonzero.
- `ITEM_SPELL_NAMES`: names keyed by spell ID.
- `HAS_SPECIALIZATION_IDS` and `HAS_PLAYER_CONDITION_IDS`: whether the input schema supplies each metadata column. An omitted sparse row means zero only for available fields; Wrath's absent `PlayerConditionID` column remains unavailable.

The sparse representation preserves all fields of the previous seven-column mapping without repeating mostly zero specialization/condition values on every association. It does not add other DB2 fields such as charges or cooldowns. Unresolved `(item_id, effect_id)` pairs remain in provenance instead of receiving invented effect values.

The pinned CSV builds are Retail 12.1.5.70575, PTR 12.1.7.70493, Wrath 3.4.3.54261, Mists 5.5.4.70268, Era and Anniversary 1.15.9.70003, and Forever 1.60.1.70532. Era and Anniversary retain separate outputs and provenance for their shared Vanilla source build. Forever's item data remains 70532 despite its 70707 UI pin. Wrath's DB2 snapshot is separate from its original 3.3.5 UI source.

Retail additionally preserves every association value and spell-name spelling from the [historical mapping](https://github.com/Osso/wow-ui-sim/blob/d1196bd7dd0e25de561d342055f92c6f51ab0dfb/crates/sim-data/data/item_effects.rs). The generator merges that immutable, hash-checked input with all associations from the pinned CSVs. Historical values win for existing associations and names, while new associations remain. This retains item `285065` and the existing name bytes without a second runtime catalog or supplement lookup. The historical source is an input to generation; the old shared mapping is no longer compiled.

`wow-cli generate items --profile PROFILE --build VERSION [--csv-dir DIR] [--retail-baseline PATH]` writes the selected catalog and `data/item-effects/<profile>-provenance.json`. The optional Retail baseline path supplies a local copy of the same immutable input and must pass its canonical-LF hash check. Other profiles reject that option. Retail, PTR and Forever can omit `--build` to use the current UI pin; the other four Classic profiles require an explicit item-data build. Omitting `--profile` runs the common-data generator, which no longer owns item-effect output.

Retail, PTR and Forever join `ItemEffect`, `ItemXItemEffect` and `SpellName` CSVs. The other four Classic profiles use `ItemEffect.ParentItemID` and `SpellName`. Provenance records source URLs, hashes and byte counts, association sources, resolved/output counts, sparse metadata columns, unresolved associations, missing names and the generation command. Retail also records the immutable historical source and merge policy.

## Native evidence and known gaps

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:948` declares the input and outputs. WM34-A native evidence for Retail 12.1.5/70688 identifies selection and lookup; zero-value missing-name publication is emulated, not live-observed, and successful return arity is inferred. Non-Retail candidate ordering preserves the prior catalog's behavior as an inferred compatibility policy; it is not established native parity.

- Modified-item effect collections, bag-held object enchantments, all thirteen enchantment slots, object resolution order and embedded effect keys are not fully modeled. The current object path covers the equipped record's single supplied enchantment.
- Multiple-use ordering and mixed passive/use behavior remain native-unverified beyond the bounded evidence. Preserving all effect rows makes those rules revisable without regenerating preselected answers.
- Eligibility changes, live loading and full protected-wrapper return behavior remain unverified.
- Wrath's 3.4.3 DB2 snapshot does not establish exact original 3.3.5 item-association parity.
- Modern Classic UI sources condition their deprecated global alias on `loadDeprecationFallbacks`. The simulator preserves its established legacy-global policy; that CVar's native alias lifetime is not modeled here.

## Verification requirements

- Select each of the seven profile features independently. Check sorted effect rows, sorted unique name/metadata keys, nonzero sparse metadata, source-column availability and profile-specific associations.
- Verify that core rows and sparse metadata reconstruct every preserved historical Retail tuple and name. Keep passive, negative/zero and unnamed effects, and account for unresolved source associations in provenance.
- Exercise both API spellings on the five legacy profiles. Retain input forms, exact two-value versus zero-value arity, invalid-argument rejection and cross-profile exclusion checks.
- Preserve Retail's raw slot validation, existing results, added associations and equipped-enchantment precedence. Verify unnamed selections still feed ID consumers without taking the enchantment fallback.
- Preserve non-Retail candidate ordering without rejecting a use spell because a non-use effect shares its slot. Forever item `239695` must return exactly `"Learning", 55884` through both API spellings; the same duplicate-slot arrangement must still be rejected under Retail rules.
