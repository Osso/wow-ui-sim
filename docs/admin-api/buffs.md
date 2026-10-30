# Buffs & Auras

### A_Admin.AddBuff(spellId, name, icon, duration, stacks, caster)

Adds a buff to the player's aura list.

- **spellId** `number` -- Spell ID for the aura
- **name** `string` -- Display name
- **icon** `number` -- File data ID for the aura icon texture
- **duration** `number` -- Duration in seconds (0 = permanent/no expiry)
- **stacks** `number` -- Stack count (`applications` in aura data)
- **caster** `string?` -- Stored source unit token; defaults to `player`. `player` and `pet` set `isFromPlayerOrPlayerPet`. Player-aura queries currently project `sourceUnit` as `player` regardless of the stored token; native behavior is pending investigation.
- **Affects:** `UnitBuff("player", index)`, `UnitAura("player", index)`, `GetPlayerAuraBySpellID(spellId)`
- **Fires:** `UNIT_AURA`
- **Example:**
```lua
-- Add Power Word: Fortitude (permanent raid buff)
A_Admin.AddBuff(21562, "Power Word: Fortitude", 135987, 0, 0)

-- Add a 30-second 5-stack buff
A_Admin.AddBuff(12345, "Test Buff", 136001, 30.0, 5)

local name, icon, count = UnitBuff("player", 1)
```

### A_Admin.AddDebuff(spellId, name, icon, duration, stacks, dispelType)

Adds a debuff (harmful aura) to the player's aura list.

- **spellId** `number` -- Spell ID for the aura
- **name** `string` -- Display name
- **icon** `string` -- File data ID for the aura icon texture
- **duration** `number` -- Duration in seconds (0 = permanent/no expiry)
- **stacks** `number` -- Stack count (`applications` in aura data)
- **dispelType** `string?` -- Optional dispel type: `"Magic"`, `"Curse"`, `"Disease"`, `"Poison"`, or `"Bleed"`. Omit for a non-dispellable debuff (`dispelName` will be nil).
- **Affects:** `UnitDebuff("player", index)`, `C_UnitAuras.GetAuraDataByIndex("player", index, "HARMFUL")`; dispellable debuffs render the dispel-icon border atlas (e.g. `ui-debuff-border-magic-icon`) in the player DebuffFrame
- **Fires:** `UNIT_AURA` (with `isFullUpdate = true`)
- **Example:**
```lua
-- Magic debuff: blue border + dispel swirl in DebuffFrame
A_Admin.AddDebuff(589, "Shadow Word: Pain", "136207", 30, 1, "Magic")

-- Non-dispellable debuff: plain border
A_Admin.AddDebuff(770, "Faerie Fire", "136033", 40, 1)
```

### A_Admin.RemoveBuff(spellId)

Removes the buff with the given spell ID from the player's aura list.

- **spellId** `number` -- Spell ID of the buff to remove
- **Affects:** `UnitBuff("player")` and `GetPlayerAuraBySpellID(spellId)` no longer return this aura
- **Fires:** `UNIT_AURA`
- **Example:**
```lua
A_Admin.RemoveBuff(21562)   -- remove Power Word: Fortitude
```

### A_Admin.ClearBuffs()

Removes all buffs from the player's aura list.

- **Affects:** `UnitBuff("player", 1)` returns `nil`
- **Fires:** `UNIT_AURA`
- **Example:**
```lua
A_Admin.ClearBuffs()
print(UnitBuff("player", 1))    -- nil
```
