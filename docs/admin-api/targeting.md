# Targeting

### A_Admin.SetTarget(name, level, classIndex, isEnemy)

Creates a custom target unit.

- **name** `string` -- Display name for the target
- **level** `number` -- Unit level
- **classIndex** `number` -- 1-based class index (see `SetPlayerClass`)
- **isEnemy** `boolean` -- `true` for hostile NPC/player, `false` for friendly
- **Affects:** `UnitExists("target")`, `UnitName("target")`, `UnitLevel("target")`, `UnitClass("target")`, `UnitIsEnemy("player", "target")`
- **Fires:** `PLAYER_TARGET_CHANGED`
- **Example:**
```lua
A_Admin.SetTarget("Hogger", 11, 1, true)    -- hostile Warrior-class mob
A_Admin.SetTarget("Jaina", 70, 8, false)    -- friendly Mage
```

### A_Admin.SetTargetInteractable(interactable)

Sets the current target's interaction state for `UnitIsInteractable("target")`.
Requires a current target and does not fire an event.

- **interactable** `boolean`
- **Affects:** `UnitIsInteractable("target")`
- **Example:**
```lua
A_Admin.SetTarget("Talker", 60, 1, false)
A_Admin.SetTargetInteractable(true)
assert(UnitIsInteractable("target"))
```

### A_Admin.ClearTarget()

Removes the current target.

- **Affects:** `UnitExists("target")` returns `false`
- **Fires:** `PLAYER_TARGET_CHANGED`
- **Example:**
```lua
A_Admin.ClearTarget()
print(UnitExists("target"))     -- false
```

### A_Admin.SetFocus(name, level, classIndex, isEnemy)

Creates a custom focus target unit. Same parameters as `SetTarget`.

- **name** `string` -- Display name
- **level** `number` -- Unit level
- **classIndex** `number` -- 1-based class index
- **isEnemy** `boolean` -- Hostile or friendly
- **Affects:** `UnitExists("focus")`, `UnitName("focus")`, `UnitLevel("focus")`
- **Fires:** `PLAYER_FOCUS_CHANGED`
- **Example:**
```lua
A_Admin.SetFocus("Thrall", 80, 7, false)    -- friendly Shaman focus
```

### A_Admin.ClearFocus()

Removes the current focus target.

- **Affects:** `UnitExists("focus")` returns `false`
- **Fires:** `PLAYER_FOCUS_CHANGED`
- **Example:**
```lua
A_Admin.ClearFocus()
print(UnitExists("focus"))      -- false
```
