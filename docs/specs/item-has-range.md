# C_Item.ItemHasRange (Retail)

## What it must do

`C_Item.ItemHasRange(itemInfo)` returns one Boolean. A missing or unsupported argument raises an error.

1. Without a present player (no context), it returns false. VERIFIED (WM43).
2. The spell is the item's use spell, chosen the same way as `C_Item.GetItemSpell`: the first nonzero spell with TriggerType 0 in LegacySlotIndex order, otherwise the equipped item's type-7 enchantment spell. No spell returns false. VERIFIED (WM43, WM43 follow-up).
3. If SpellMisc `Attributes_0` has bit 4 or 1024, the range is minimum 0, maximum 100, and RangeIndex is not read. VERIFIED (WM43).
4. Otherwise SpellRange `RangeMin`/`RangeMax` use index 1 when the spell classifies as helpful, else index 0. VERIFIED (WM43).
5. Classification:
   - The first SpellTargetRestrictions row for the current difficulty is used. When no row matches and the difficulty has a Difficulty row, the lookup moves to its `FallbackDifficultyID`.
   - The selected row's `Targets` bit 256 gives helpful, then bit 128 gives harmful, then bit 2 gives neither. Otherwise the effect scans decide.
   - The harmful scan precedes the helpful scan. EffectAttributes bit 4096, or an implicit target in {2, 6, 15, 16, 24, 28, 53, 54, 93, 104, 129, 134}, is harmful evidence. An implicit target in {1, 3, 4, 5, 20, 21, 27, 29, 30, 31, 33, 34, 35, 45, 56, 57, 58, 59, 118, 119, 121, 128, 132, 133, 136} is helpful evidence.
   - VERIFIED (WM43 follow-up). Column names are INFERRED.
6. The result is true when the absolute selected minimum or maximum exceeds 1/4,194,304. VERIFIED (WM43).

The current difficulty is `GetInstanceInfo`'s difficultyID (`world.instance_difficulty`); INFERRED. The first restriction row for a difficulty is taken in CSV row order; INFERRED.

## Data

`wow-cli generate items` writes `crates/sim-data/data/item_spell_ranges.rs` from the 70391 CSVs:

- SpellMisc and difficulty-0 SpellEffect rows for spells named by ItemEffect or type-7 enchantments
- all SpellRange rows
- SpellTargetRestrictions rows for those spells, in CSV order
- Difficulty fallbacks

Generation fails if a fallback chain does not terminate, or if a referenced spell has a nonzero Node effect.

In this data, Sky Chain (184586) is the only item whose result depends on the index. Its spell 346772 classifies as neither on range 318, so index 0 gives (3, 10) and the result is true.

## Known gaps

- Node route: the cached predicate and node descriptors are not modelled. No 70391 item-referenced spell has a nonzero Node, and generation fails if one appears.
- SpellRange Flags 1 (melee substitution) and 2 (reach correction) are not applied; stored arrays are used. INFERRED: every flagged 70391 row stores a nonzero maximum, so the Boolean result does not change.
- Only DifficultyID 0 SpellMisc and SpellEffect rows are read; native difficulty-priority selection is UNKNOWN.
- The eight inline item-description effect references and bonus-list (ItemBonus type 23) insertions are not modelled.
- A spell without a SpellMisc row, or with a RangeIndex missing from SpellRange, returns false. INFERRED; the native lookup failure is UNKNOWN.
