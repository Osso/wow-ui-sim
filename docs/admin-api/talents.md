# Specialization & Talents

### A_Admin.SetSpecSelectionEnabled(enabled)

Sets the scenario input for `C_SpecializationInfo.IsSpecSelectionEnabled` on PTR and WoWForever. The scenario defaults to true; this does not describe native eligibility.

### Wrath scenario inputs

`A_Admin.SetWrathGlyphSocket(group, index, enabled, glyphType, glyphSpellID, icon)` supplies one socket in a one-based talent group. The socket index is 1 through 6; the last three values may be nil. `A_Admin.SetWrathUnspentTalentPoints(group, points)` supplies a group's unspent points. These setters are available only on Wrath. Unsupplied sockets have no record and unsupplied points are zero; these are simulator scenario defaults, not native initial values. No inspected or pet talent groups are supplied; their point queries return zero as a scenario default. Setters do not fire events.

### A_Admin.SetSpec(specIndex)

Sets the player's active specialization.

- **specIndex** `number` -- 1-based spec index within the player's class
- **Affects:** `GetSpecialization()`, `GetSpecializationInfo()` (WoWForever uses the `C_SpecializationInfo` equivalents).
- **Fires:** `PLAYER_SPECIALIZATION_CHANGED` with `"player"` after the state changes, on Retail, PTR, Mists, Era, Anniversary, and WoWForever. Wrath does not declare this event and does not fire it.
- Setting the current index again is a simulator no-op and fires no event.
- **Example:**
```lua
-- For a Paladin: 1=Holy, 2=Protection, 3=Retribution
A_Admin.SetSpec(3)
print(GetSpecialization())  -- 3
local id, name = GetSpecializationInfo(3)
-- name="Retribution"
```

### A_Admin.SetTalentRank(nodeId, rank)

Sets the purchased rank on a talent tree node.

- **nodeId** `number` -- Trait node ID from the talent database
- **rank** `number` -- Number of ranks purchased (0 = unlearned)
- **Affects:** `C_Traits.GetNodeInfo(configID, nodeId).currentRank`
- **Fires:** `TRAIT_CONFIG_UPDATED`
- **Example:**
```lua
A_Admin.SetTalentRank(87800, 1)     -- learn a specific talent node
A_Admin.SetTalentRank(87800, 0)     -- unlearn it
```

### A_Admin.SetTalentSelection(nodeId, entryId)

Selects which entry is active for a choice node in the talent tree.

- **nodeId** `number` -- Trait node ID
- **entryId** `number` -- Entry ID of the selected choice
- **Affects:** `C_Traits.GetNodeInfo(configID, nodeId).activeEntry`
- **Fires:** `TRAIT_CONFIG_UPDATED`
- **Example:**
```lua
-- Select the right-side talent on a choice node
A_Admin.SetTalentSelection(88100, 88101)
```

### A_Admin.ResetTalents()

Clears all talent node ranks and selections, returning the tree to an unspent state.

- **Affects:** All `C_Traits.GetNodeInfo()` calls return rank 0 and no selection
- **Fires:** `TRAIT_CONFIG_UPDATED`
- **Example:**
```lua
A_Admin.ResetTalents()
```
