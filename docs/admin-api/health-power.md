# Health & Power

Health setters dispatch `UNIT_HEALTH` after updating state, then `UNIT_MAXHEALTH` if the maximum changed. Native order was observed in the 4.x build 15595 party stats path (aso); other update paths are unverified.

### A_Admin.SetPlayerHealth(current, max)

Sets the player's current and maximum health.

- **current** `number` -- Current health value
- **max** `number` -- Maximum health value
- **Affects:** `UnitHealth("player")`, `UnitHealthMax("player")`, `UnitIsDead("player")`
- **Fires:** `UNIT_HEALTH("player")`, then `UNIT_MAXHEALTH("player")` if max changed.
- **Example:**
```lua
A_Admin.SetPlayerHealth(50000, 100000)
print(UnitHealth("player"), UnitHealthMax("player"))    -- 50000, 100000

-- Simulate player death
A_Admin.SetPlayerHealth(0, 100000)
print(UnitIsDead("player"))     -- true
```

### A_Admin.SetPlayerPower(current, max [, powerType])

Sets the player's current and maximum power/resource.

- **current** `number` -- Current power value
- **max** `number` -- Maximum power value
- **powerType** `number` (optional) -- Power type index (defaults to the class primary resource):

| Value | Power Type |
|-------|------------|
| 0 | Mana |
| 1 | Rage |
| 2 | Focus |
| 3 | Energy |
| 4 | Combo Points |
| 6 | Runic Power |

- **Affects:** `UnitPower("player")`, `UnitPowerMax("player")`
- **Fires:** `UNIT_POWER_UPDATE`
- **Example:**
```lua
A_Admin.SetPlayerPower(50000, 100000, 0)    -- 50% mana
A_Admin.SetPlayerPower(0, 100, 1)           -- 0 rage
```

### A_Admin.SetTargetHealth(current, max)

Sets the current target's health values.

- **current** `number` -- Current health value
- **max** `number` -- Maximum health value
- **Affects:** `UnitHealth("target")`, `UnitHealthMax("target")`
- **Fires:** `UNIT_HEALTH("target")`, then `UNIT_MAXHEALTH("target")` if max changed.
- **Note:** Requires an active target. Has no effect if no target is set.
- **Example:**
```lua
A_Admin.SetTarget("Hogger", 11, 1, true)
A_Admin.SetTargetHealth(22000, 45000)   -- target at ~50% HP
```

### A_Admin.SetReleaseTimeRemaining(seconds)

Supplies the spirit-release timer that the server starts at death (Retail, PTR and WoW Forever).

- **seconds** `number` -- positive seconds until the automatic release, `-1` for a death without a release timer, or `0` to clear the timer.
- **Affects:** `GetReleaseTimeRemaining()`. The Blizzard `PLAYER_DEAD` handler shows the DEATH dialog only for a positive or `-1` value.
- **INFERRED R88g9 (native evidence pending):** a positive timer counts down on the simulation clock, rounds up to whole seconds and stops at 0. Without a supplied timer the result is 0.
- **Example:**
```lua
A_Admin.SetReleaseTimeRemaining(360)
A_Admin.FireEvent("PLAYER_DEAD")    -- the original handler shows the DEATH dialog
```

### A_Admin.SetNoReleaseAura(seconds)

Supplies a cannot-release effect on the player (Retail, PTR and WoW Forever).

- **seconds** `number` -- positive seconds until the effect ends, `-1` for an until-cancelled effect, or `0` to clear it.
- **Affects:** `HasNoReleaseAura()` returns `hasCannotReleaseEffect`, `longestDuration` and `hasUntilCancelledDuration`. The DEATH dialog `OnUpdate` disables its release button and shows the whole seconds of a timed effect.
- **INFERRED R88g9 (native evidence pending):** a timed effect counts down on the simulation clock and ends at 0; an until-cancelled effect reports a duration of 0.

### A_Admin.AddSelfResurrectOption(name, optionType, id, canUse, isLimited, priority)

Appends one `SelfResurrectOption` record (Retail, PTR and WoW Forever). `optionType` is an `Enum.SelfResurrectOptionType` value.

- **Affects:** `C_DeathInfo.GetSelfResurrectOptions()` returns the records in supply order. The DEATH dialog shows button 2 (and button 3 during an encounter that limits resurrections) and labels it from the first option after its usable-first sort.
- **INFERRED R88g9 (native evidence pending):** without a supplied option the call returns nothing.

### A_Admin.ClearSelfResurrectOptions()

Removes all supplied self-resurrect options.

### A_Admin.AddDeathRecap(recapID, defaultEligible, maxHealth[, currentHP, amount])

Appends one death recap. With `currentHP` the recap carries one event (`currentHP` and the optional `amount`). `defaultEligible` marks the recap that `C_DeathRecap` calls without a `recapID` select.

- **Affects:** `C_DeathRecap.HasRecapEvents`, `GetRecapEvents`, `GetRecapLink`, `GetRecapMaxHealth` (Retail, PTR and WoW Forever), and the DEATH dialog recap button.

### A_Admin.ClearDeathRecaps()

Removes all supplied death recaps.

### A_Admin.SetCorpseMapPosition(uiMapID, x, y)

Supplies the player's corpse position on one UI map (Retail, PTR, WoW Forever and Mists). `uiMapID` must be positive; `x` and `y` are normalized map coordinates in `[0, 1]`. Invalid values raise an error.

- **Affects:** `C_DeathInfo.GetCorpseMapPosition(uiMapID)` returns a `Vector2DMixin` for that map while the player is dead (`health <= 0`); other maps return nil. The WorldMap `DeathMapDataProvider` shows a `CorpsePinTemplate` pin at the position.
- **INFERRED R88g9 (native evidence pending):** supplying a position also marks the corpse as available; `RetrieveCorpse()` clears that flag.
- **Example:**
```lua
A_Admin.SetPlayerHealth(0, 100)
A_Admin.SetCorpseMapPosition(84, 0.625, 0.375)
OpenWorldMap(84)    -- the original provider adds the corpse pin
```

### A_Admin.ClearCorpse()

Removes the supplied corpse and all its map positions.
