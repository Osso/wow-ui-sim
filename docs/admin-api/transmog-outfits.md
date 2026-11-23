# Transmog outfit test input

Forever scenarios can set outfit capacity before creating records through the real `C_TransmogOutfitInfo.AddNewOutfit` API:

```lua
A_Admin.SetUnlockedOutfitSlots(2, 2)
C_TransmogOutfitInfo.AddNewOutfit("Copper", 123)
C_TransmogOutfitInfo.AddNewOutfit("Silver", 456)
```

`SetUnlockedOutfitSlots(source, count)` replaces one capacity value. `source` must be `0`, `1`, or `2`; `count` must be a non-negative integer that fits `u32`. This changes only the simulator's unlocked capacity input. It does not create outfits or emit events. It is simulator-only and does not model native unlock policy.

See the [Forever spellbook contract](../specs/forever-camelot-spellbook.md) for the selected Player Spells consumer proof.
