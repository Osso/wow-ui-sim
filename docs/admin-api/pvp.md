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

### `A_Admin.SetArenaOpponent(unit, exists[, guid, classIndex, specID])`

Retail 12.0.5+ and WoW Forever. Supplies or removes one opponent token, `arena1` through `arena5`. When `exists` is true, `guid`, `classIndex`, and `specID` are required. The GUID must be nonempty; class index must identify a known class; spec ID must be an unsigned integer. When false, all detail arguments must be nil. Invalid values raise an error. The default opponent set is empty. The setter fires the arena specialization and opponent update events. `UnitExists`, `UnitGUID`, and `UnitClass` read the supplied state. `GetArenaOpponentSpec` is declared in Retail and Forever `PvpInfoDocumentation.lua:188`. `GetNumArenaOpponents` and `GetNumArenaOpponentSpecs` are not declared in those files; CompactArenaFrame calls them at `CompactArenaFrame.lua:23-35`.

INFERRED R88g9 (native evidence pending): supplied opponent records represent the host arena roster; `specID = 0` means no known specialization; the two count globals return counts derived from the supplied roster.

### `A_Admin.SetArenaCrowdControl(unit, spellID[, startTime, duration])`

Retail 12.0.5+ and WoW Forever. Supplies the arena crowd-control removal spell for the unit's GUID and its cooldown window in `GetTime` seconds. Spell ID, start time and duration must be nonnegative; timing values must be finite. Invalid inputs and unknown units raise an error. For `arena1` through `arena5`, the setter fires `ARENA_CROWD_CONTROL_SPELL_UPDATE(unit, spellID)` so the CompactArenaFrame updates its icon and cooldown. `C_PvP.GetArenaCrowdControlInfo(unit)` returns `spellID, startTime * 1000, duration * 1000` (`0, 0` outside the window); `C_PvP.GetArenaCrowdControlDuration(unit)` reads the same window; `C_PvP.RequestCrowdControlSpell(unit)` fires the synchronous update event. A spell ID of 0 clears the unit.

INFERRED R88g9 (native evidence pending): a unit without a supplied spell makes `GetArenaCrowdControlInfo` return nothing and `RequestCrowdControlSpell` fire nothing.
