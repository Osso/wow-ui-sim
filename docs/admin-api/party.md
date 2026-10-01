# Party

### A_Admin.SetPartySize(n)

Sets the number of active party members, resizing the party array.

- **n** `number` -- Party size from `0` (solo) to `4`
- **Affects:** `GetNumGroupMembers()`, `UnitExists("party1")` through `UnitExists("party4")`
- **Fires:** `GROUP_ROSTER_UPDATE`
- **Note:** When growing the party, new members are initialized with default values. When shrinking, excess members are removed.
- **Example:**
```lua
A_Admin.SetPartySize(0)     -- go solo
A_Admin.SetPartySize(4)     -- full 4-member party
print(GetNumGroupMembers()) -- 4
```

### A_Admin.SetPartyMember(index, name, classIndex, level)

Configures a specific party member's identity.

- **index** `number` -- 1-based party index (1-4)
- **name** `string` -- Character name
- **classIndex** `number` -- 1-based class index (see `SetPlayerClass`)
- **level** `number` -- Character level
- **Affects:** `UnitName("party1")`, `UnitClass("party1")`, `UnitLevel("party1")`, etc.
- **Fires:** `GROUP_ROSTER_UPDATE`
- **Example:**
```lua
A_Admin.SetPartySize(2)
A_Admin.SetPartyMember(1, "Thrynn", 2, 80)      -- Paladin
A_Admin.SetPartyMember(2, "Kazzara", 1, 80)     -- Warrior
print(UnitName("party1"))   -- "Thrynn"
```

### A_Admin.SetPartyMemberConnected(index, connected)

Changes an existing active party member's connection state.

- **index** `number` -- 1-based party index; absent or nonpositive indices do nothing.
- **connected** `boolean` -- Connection state; new members start connected.
- **Affects:** `UnitIsConnected("partyN")`, `GroupHasOfflineMember()`.
- **Fires:** Synchronous `UNIT_CONNECTION("partyN", connected)` only on a changed value. Callback queries see the new state.
- **Lifecycle:** Removed members lose connection state; regrowth starts connected. Inactive retained members are not changed.
- **Evidence:** [Party connection contract and pending runtime proof](../specs/party-connection.md).

### A_Admin.SetPartyMemberHealth(index, current, max)

Sets a party member's health values.

- **index** `number` -- 1-based party index (1-4)
- **current** `number` -- Current health
- **max** `number` -- Maximum health
- **Affects:** `UnitHealth("partyN")`, `UnitHealthMax("partyN")`
- **Fires:** `UNIT_HEALTH`
- **Example:**
```lua
A_Admin.SetPartyMemberHealth(1, 60000, 120000)  -- party1 at 50% HP
```

### A_Admin.KillPartyMember(index)

Sets a party member's health to zero, marking them as dead.

- **index** `number` -- 1-based party index (1-4)
- **Affects:** `UnitHealth("partyN")` returns `0`, `UnitIsDead("partyN")` returns `true`
- **Fires:** `UNIT_HEALTH`
- **Example:**
```lua
A_Admin.KillPartyMember(2)
print(UnitIsDead("party2"))     -- true
```

### A_Admin.ResPartyMember(index)

Restores a dead party member to full health.

- **index** `number` -- 1-based party index (1-4)
- **Affects:** `UnitHealth("partyN")` returns full HP, `UnitIsDead("partyN")` returns `false`
- **Fires:** `UNIT_HEALTH`
- **Example:**
```lua
A_Admin.KillPartyMember(2)
A_Admin.ResPartyMember(2)
print(UnitIsDead("party2"))     -- false
```

### A_Admin.SetRotDamage(level)

Sets the automatic rot damage intensity applied to party members each tick.

- **level** `number` -- Intensity level:

| Level | Label | Damage Per Tick |
|-------|-------|-----------------|
| 0 | Off | None |
| 1 | Light | 1% of max HP |
| 2 | Moderate | 3% of max HP |
| 3 | Heavy | 5% of max HP |
| 4 | Brutal | 10% of max HP |

- **Note:** Matches the `ROT_DAMAGE_LEVELS` constant in `src/lua_api/game_data.rs`.
- **Example:**
```lua
A_Admin.SetRotDamage(2)     -- moderate damage for healing practice
A_Admin.SetRotDamage(0)     -- disable rot
```
