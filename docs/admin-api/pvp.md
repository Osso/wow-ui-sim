# PvP

### A_Admin.SetPvpTalentTooltipRecord(talentID, spellID, overridesSpellID, levelRequired, categoryPositionMask)

Replaces one Retail tooltip catalog row with validated simulator input. IDs and level are signed 32-bit integers where documented by the producer; spell IDs and the category mask are unsigned. This setter does not invent eligibility, grant, or inspect outcomes.

### A_Admin.SetPvpTalentTooltipResolution(spellID, resolvedSpellID)

Sets or clears one Retail PvP tooltip spell-resolution override. Both IDs are unsigned 32-bit integers; nil clears the override. An absent override preserves the original spell ID.

### A_Admin.SetPvPEnabled(enabled)

Toggles the player's PvP flag.

- **enabled** `boolean`
- **Affects:** `UnitIsPVP("player")`
- **Fires:** `UPDATE_FACTION`
- **Example:**
```lua
A_Admin.SetPvPEnabled(true)
print(UnitIsPVP("player"))  -- true
```

### A_Admin.SetHonorLevel(level)

Sets the player's honor level.

- **level** `number` -- Honor level (1-500)
- **Affects:** `UnitHonorLevel("player")`
- **Example:**
```lua
A_Admin.SetHonorLevel(50)
print(UnitHonorLevel("player"))     -- 50
```
