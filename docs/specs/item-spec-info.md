# Item specialization list

Retail `C_Item.GetItemSpecInfo(itemInfo)` uses the existing item-specialization band cache. It returns zero values when no record can be constructed, `noTraitsAndNoSpecializations` is set, or `itemFlag27` is set. Otherwise it returns a fresh table containing a stable filtered subsequence, without sorting or deduplication. VERIFIED locally (WM48); return serialization is EMULATED in bounded offline probes.

Exclude an existing specialization whose OrderIndex is 4. Otherwise, a present modified item record with Bonding other than 1 admits the specialization, including missing specialization rows. Otherwise require a specialization row matching the player class, or class 14 when CharacterlessLogin is nonzero. The field names OrderIndex and Bonding are INFERRED (WM48).

The item ID and inferred player-level selector use the existing WM42 cache and bands. Modified-item bonus IDs, modifiers and context are not modeled. Unconditional equality of the selector source with `DoesItemContainSpec` is UNKNOWN. Data is generated from 70391 CSVs; the native evidence is build 70688. The copied itemFlag27 bit is `ItemSparse.Flags_0 & 134217728`; accepted specialization overrides skip the copy. VERIFIED (WM58).
