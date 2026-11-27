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

### `A_Admin.SetArenaCrowdControl(unit, spellID[, startTime, duration])`

Retail 12.0.5+ and WoW Forever. Supplies the arena crowd-control removal spell for the unit's GUID and its cooldown window in `GetTime` seconds. `C_PvP.GetArenaCrowdControlInfo(unit)` returns `spellID, startTime * 1000, duration * 1000` (`0, 0` outside the window); `C_PvP.GetArenaCrowdControlDuration(unit)` reads the same window; `C_PvP.RequestCrowdControlSpell(unit)` fires the synchronous `ARENA_CROWD_CONTROL_SPELL_UPDATE(unit, spellID)`. A spell ID of 0 clears the unit. An unknown unit raises an error.

INFERRED R88g9 (native evidence pending): a unit without a supplied spell makes `GetArenaCrowdControlInfo` return nothing and `RequestCrowdControlSpell` fire nothing.
