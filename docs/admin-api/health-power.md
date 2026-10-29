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
