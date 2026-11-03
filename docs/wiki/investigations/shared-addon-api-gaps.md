# Shared addon API gaps

Item-spell queries require profile-specific associations without discarding the
other effects and metadata used by related queries. One full mapping per profile
now supplies the modern item namespace, Classic legacy wrappers, cursor selection
and Retail effect consumers.

## Content

### Item-associated spells

`C_Item.GetItemSpell` uses one handler and the existing item-ID/name parser on all
seven profiles. Wrath, Mists, Era, Anniversary and Forever also expose
`GetItemSpell` under `ClientProfile::has_legacy_item_globals`. A named use spell
returns its name and ID; unresolved numeric/string selectors or missing named
selections return no values. Other argument types fail.

The runtime selector applies profile-specific rules to the same raw representation.
Retail validates every association's legacy slot, rejects duplicate or invalid
slots, and chooses the first positive trigger-zero spell in slot order. Other
profiles preserve the prior catalog's candidate ordering: positive trigger-zero
spells ordered by legacy slot, then effect ID. Non-use effects do not invalidate
those candidates.
An unnamed selected spell retains its ID for cursor, range and usability queries.
An equipped item's enchantment remains a Retail-only fallback when no use spell
is selected; a selected spell without a name does not fall through to it.
Artifact-power classification also reads the same profile mapping and can still
inspect non-use effects. That API remains Retail-only.

Forever item `239695` has activated spell `55884` and trigger-six spell `1229308`
in legacy slot 1. Candidate-only selection retains `"Learning", 55884` through
both API spellings. Applying Retail's whole-association duplicate-slot rejection
to this profile would drop that result. The selector regression therefore covers
the Forever result and Retail's continued rejection of the same slot arrangement.

### Full mapping and sparse metadata

The previous raw mapping held item ID, effect ID, spell ID, trigger, legacy slot,
specialization ID and condition ID. Profile catalogs retain those fields so that
selection can be revised and other effect consumers remain supported; preselecting
one named activated spell per item would discard that information. The mapping
now lives in `sim-data-profile`, replacing the shared runtime mapping.

Each profile has a flat `ITEM_EFFECTS` table containing
`(item_id, effect_id, signed_spell_id, trigger, legacy_slot)`, a spell-ID-keyed
`ITEM_SPELL_NAMES` table, and sparse `ITEM_EFFECT_METADATA` rows keyed by effect ID.
Metadata rows contain specialization and condition values only when at least one
available value is nonzero. These fields are sparse enough to keep outside the
core rows; splitting the core associations further would add a lookup while
deduplicating very few effects.

| Profile | CSV build | Sparse metadata rows |
|---------|-----------|---------------------:|
| Retail | 12.1.5.70575 | 80 |
| PTR | 12.1.7.70493 | 80 |
| Wrath | 3.4.3.54261 | 0 |
| Mists | 5.5.4.70268 | 0 |
| Era | 1.15.9.70003 | 1 |
| Anniversary | 1.15.9.70003 | 1 |
| Forever | 1.60.1.70532 | 3 |

`HAS_SPECIALIZATION_IDS` and `HAS_PLAYER_CONDITION_IDS` record source-column
availability. A missing sparse row supplies zero only for available fields.
Wrath's input lacks `PlayerConditionID`; its zero placeholder is not a captured
condition value. The migration preserves the previous mapping's fields without
adding unrelated DB2 fields such as charges and cooldowns.

Passive effects, nonpositive spell IDs, invalid/duplicate slots and associations
without names remain in the raw tables. Unresolved item/effect references remain
explicit provenance pairs, with no invented core fields: PTR has one such pair
and Forever has eight in the pinned sources. Names and numeric IDs/links retain
their existing parser boundary: names use the compact item catalog, while IDs and
links can address the larger effect mapping.

### Retail preservation and generation

Retail retains all 60,638 historical association tuples and 36,875 names, then
adds seven raw associations for five new items from its pinned CSV export. The
result is one 60,645-row profile mapping, including item `285065` and the original
non-breaking-space spelling in spell `1262474`'s name. The generator's historical
input is pinned by commit, Git blob and canonical-LF SHA-256; historical
association values and name spelling take precedence over the CSV source.
No historical table or supplement participates in runtime lookup.

`wow-cli generate items --profile PROFILE --build VERSION [--csv-dir DIR]`
generates the full mapping and `data/item-effects/<profile>-provenance.json`.
Retail can use `--retail-baseline PATH` for an offline copy of the same
hash-checked historical input. Omitting `--profile` runs common-data generation
without regenerating the moved mapping. Retail, PTR and Forever can omit the
build to follow their UI pin; Wrath, Mists, Era and Anniversary require an explicit
data build. Retail, PTR and Forever use `ItemXItemEffect`; the other four Classic
profiles use `ItemEffect.ParentItemID`.

Provenance records CSV URLs, hashes and byte counts, association source, metadata
availability, resolved/output counts, unresolved pairs and missing names. Retail
also records the historical input and merge policy. Era and Anniversary retain
separate outputs for their shared Vanilla source build. Forever's item snapshot
remains 70532 despite its 70707 UI pin; Wrath's 3.4.3 data remains distinct from
its original 3.3.5 UI source.

### Native behavior boundaries

The selector preserves the existing Retail slot rules. Non-Retail candidate
ordering preserves the prior catalog policy as inferred compatibility behavior;
it is not established native parity.
Mixed passive/use behavior is also native-unverified: the original trinket
consumer documents no native result for Hex Lord's Dooming Idol (`270169`), while
the pinned data has a passive slot-zero spell and an activated slot-one spell.
That observation does not establish a general first-slot rule. Keeping all effect
rows allows later selector corrections without regenerating preselected answers.

Pinned Wrath `ChatFrame.lua` uses the legacy global for macro-link insertion.
The cached consumer regression checks active TOC/XML selection and executes that
unchanged file: Hearthstone inserts `/use`, while passive Neltharion's Tear inserts
`/equip`. An absent cache produces an explicit skip. Modern Classic sources
conditionally alias the global through `loadDeprecationFallbacks`; the simulator
retains its established legacy-global policy without modeling that alias lifetime.

## Sources

- [Query contract](../../specs/item-spell-query.md) — public behavior, native
  evidence limits and verification requirements.
- [Runtime selection](../../../src/c_api/item_spell/item_effects.rs) and
  [artifact classification](../../../src/c_api/item_spell/item_classifiers.rs) —
  use-spell selection, unnamed IDs and non-use effects.
- [Item generator](../../../src/bin/wow_cli/gen_items.rs) and
  [profile generation](../../../src/bin/wow_cli/gen_items/item_effects.rs) —
  raw joins, sparse metadata and preservation policy.
- [Retail provenance](../../../data/item-effects/retail-provenance.json) and
  sibling profile records — source pins, hashes, counts and unresolved pairs.
- [Historical Retail mapping](https://github.com/Osso/wow-ui-sim/blob/d1196bd7dd0e25de561d342055f92c6f51ab0dfb/crates/sim-data/data/item_effects.rs)
  — immutable data input retained during generation.
- [Catalog checks](../../../crates/sim-data-profile/src/item_effect_tests.rs)
  and [Lua query tests](../../../tests/item_use_spell.rs) — data preservation,
  profile selection, exact arity and cached Wrath consumer coverage.
- [Pinned Wrath ChatFrame](https://github.com/Gethe/wow-ui-source/blob/c4e0255fc574598428ee1c25f160ab949798bc98/FrameXML/ChatFrame.lua#L3514)
  and [Classic legacy alias](https://github.com/Gethe/wow-ui-source/blob/e5d9b3a22c3f0f6f41ebac0e454f4969594261cf/Interface/AddOns/Blizzard_DeprecatedItemScript/Deprecated_ItemScript.lua#L18)
  — original consumers and conditional aliasing.

## See Also

- [[generated-data-crates]] — compile-time profile data boundary.
- [[consumer-selection-and-red-proof]] — runtime selection and proof limits.
