# Spell range queries

Retail `SpellDocumentation.lua:879-892` and Forever `SpellDocumentation.lua:948-961` declare `C_Spell.IsSpellInRange` with a nilable result. The optional target defaults to `target`. A missing catalog spell or nonexistent target returns nil; otherwise the simulator uses its existing unit reachability model. The nil cases are user-approved, not native evidence; see [C_Spell.IsSpellInRange](https://warcraft.wiki.gg/wiki/API_C_Spell.IsSpellInRange).

Legacy `IsSpellInRange` accepts a known spell name or a spellbook slot and book type, and returns 1 or 0 for an eligible visible unit and nil for invalid lookup or invisible/missing units. The 1/0/nil result shape was observed on WoW 3.3.5a build 12340; the name and nil policies on these profiles are user-approved, not native evidence. See [IsSpellInRange](https://warcraft.wiki.gg/wiki/API_IsSpellInRange). The existing model does not provide per-spell distance or independent visibility observations.
