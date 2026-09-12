# Events

### A_Admin.FireEvent(event, ...)

Fires a WoW game event directly to all registered listeners.

- **event** `string` -- Event name (e.g., `"ZONE_CHANGED_NEW_AREA"`)
- **...** -- Optional event arguments passed to `OnEvent` handlers
- **Example:**
```lua
-- Fire a simple event
A_Admin.FireEvent("ZONE_CHANGED_NEW_AREA")

-- Fire an event with arguments
A_Admin.FireEvent("ADDON_LOADED", "MyAddon")
A_Admin.FireEvent("CHAT_MSG_SAY", "Hello world", "Arthas", "", "", "Arthas")
A_Admin.FireEvent("UNIT_HEALTH", "player")
```

**Note:** `A_Admin.FireEvent` is a namespaced alias for the internal `FireEvent` global, making it clear in test scripts that this is a simulator-only call with no real WoW equivalent.

### A_Admin.SimulateBossKill(encounterID, encounterName, difficultyID, groupSize[, encounterUnitStatus])

Fires simulator encounter completion events. On retail 12.0.7 and later, `ENCOUNTER_END` receives a sixth `encounterUnitStatus` argument; earlier epochs retain five event arguments.

- **encounterUnitStatus** optional dense array of `{ creatureID, creatureName, remainingHealthPercent }` records.
- Omitted or `nil` supplies a fresh empty list. Records are copied before listeners run.
- `creatureID` must be a positive integer; `creatureName` a string; `remainingHealthPercent` a finite number from 0 through 100. Invalid input rejects before either event fires.
- This is explicit simulator input, not native boss tracking, timing, or security behavior. See [encounter-end unit status](../specs/encounter-end-unit-status.md).
