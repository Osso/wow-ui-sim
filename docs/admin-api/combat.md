# Combat State

### A_Admin.SetInCombat(inCombat)

Toggles the player's combat state.

- **inCombat** `boolean` -- `true` to enter combat, `false` to leave
- **Affects:** `InCombatLockdown()`, `UnitAffectingCombat("player")`
- **Fires:** `PLAYER_REGEN_DISABLED` when entering combat, `PLAYER_REGEN_ENABLED` when leaving
- **Example:**
```lua
A_Admin.SetInCombat(true)
print(InCombatLockdown())   -- true
A_Admin.SetInCombat(false)
print(InCombatLockdown())   -- false
```

### A_Admin.SetCasting(spellId, spellName, iconPath, duration)

Starts a simulated cast bar on the player.

- **spellId** `number` -- Spell ID for the cast
- **spellName** `string` -- Display name shown in the cast bar
- **iconPath** `string` -- Texture path for the cast bar icon
- **duration** `number` -- Cast time in seconds
- **Affects:** `UnitCastingInfo("player")`
- **Fires:** `UNIT_SPELLCAST_START`
- **Note:** The cast does not auto-complete. Call `StopCasting()` to cancel, or overwrite with another `SetCasting`.
- **Example:**
```lua
A_Admin.SetCasting(19750, "Flash of Light", "Interface\\Icons\\Spell_Holy_FlashHeal", 1.5)
local name, text, texture, startTime, endTime = UnitCastingInfo("player")
-- name="Flash of Light"
```

### A_Admin.DelayCasting(seconds)

Extend the current timed cast by finite, nonnegative seconds. Returns `true` and emits `UNIT_SPELLCAST_DELAYED` after updating the deadline and cumulative `UnitCastingInfo` delay in milliseconds. Zero is accepted and still notifies. Returns `false` when no cast exists; invalid/overflowing input errors without mutation.

### A_Admin.FailCasting(quiet = false)

Fail the current timed cast. Returns `true`, clears the old cast before callbacks, and emits `UNIT_SPELLCAST_FAILED` (or `UNIT_SPELLCAST_FAILED_QUIET`), then `UNIT_SPELLCAST_STOP`, with the old identity. Returns `false` when inactive. A callback-created replacement survives. A failed specialization cast discards its pending specialization before callbacks.

These are explicit simulator inputs, not native failure detection. Failure order and quiet behavior are model policies; GCD/cooldown state is unchanged. See [delay/failure contract](../specs/cast-delay-failure-inputs.md).

### A_Admin.StopCasting()

Cancels the current simulated cast.

- **Affects:** `UnitCastingInfo("player")` returns `nil`
- **Fires:** `UNIT_SPELLCAST_STOP`
- **Example:**
```lua
A_Admin.SetCasting(19750, "Flash of Light", "", 1.5)
A_Admin.StopCasting()
print(UnitCastingInfo("player"))    -- nil
```

### A_Admin.SetGCD(duration)

Triggers the global cooldown.

- **duration** `number` -- GCD duration in seconds (typically 1.5)
- **Affects:** `GetSpellCooldown(61304)` -- spell ID 61304 is the GCD sentinel
- **Example:**
```lua
A_Admin.SetGCD(1.5)
local start, dur, enabled = GetSpellCooldown(61304)
-- start=current_time, dur=1.5, enabled=1
```

### A_Admin.SetSpellCooldown(spellId, duration)

Sets a cooldown on a specific spell.

- **spellId** `number` -- Spell ID to put on cooldown
- **duration** `number` -- Cooldown duration in seconds
- **Affects:** `GetSpellCooldown(spellId)`
- **Example:**
```lua
A_Admin.SetSpellCooldown(31935, 15.0)   -- Avenger's Shield on 15s CD
local start, dur, enabled = GetSpellCooldown(31935)
-- dur=15.0
```
