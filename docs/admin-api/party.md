# Party

GUI, screenshot, dump-tree, and run-tests start with four demo party members and the player (`GetNumGroupMembers()` returns 5). Set `WOW_SIM_NO_PARTY=1` to start solo. `SetPartySize` retains existing slots when it activates or resizes the roster.

Health setters dispatch `UNIT_HEALTH` after updating state, then `UNIT_MAXHEALTH` if the maximum changed. Native order was observed in the 4.x build 15595 party stats path (aso); other update paths are unverified.

### A_Admin.SetPartyPolicyFlags(toRaid, toParty, inJailersTower, walkIn)

Retail and WoW Forever: replace four supplied boolean observations, initially false. `AllowedToDoPartyConversion(toRaid)` selects the corresponding direction; `IsPartyInJailersTower()` and `IsPartyWalkIn()` read the other flags. Non-boolean input is rejected before mutation. This is service input, not a conversion, group transfer or walk-in operation. It emits no event.

VERIFIED declarations: both profiles `PartyInfoDocumentation.lua:11–24,466–483`. Both select `Blizzard_CompactRaidFrames/[Family]/Blizzard_CompactRaidFrameManager.lua` and XML; the leave-instance button's normal OnUpdate takes its text from `IsPartyWalkIn()`. Native policy derivation and event timing are unknown.

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
print(GetNumGroupMembers()) -- 5 (four party members and the player)
```

### A_Admin.SetPartyMember(index, name, classIndex, level)

Configures a specific party member's identity.

- **index** `number` -- 1-based party index (1-4)
- **name** `string` -- Character name
- **classIndex** `number` -- 1-based class index (see `SetPlayerClass`)
- **level** `number` -- Character level
- **Affects:** `UnitName("party1")`, `UnitClass("party1")`, `UnitLevel("party1")`, etc. Changing identity clears that slot's buffs and debuffs and fires `UNIT_AURA` after the change.
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
- **Fires:** `UNIT_HEALTH`, then `UNIT_MAXHEALTH` if max changed, with `partyN` as the unit.
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

### A_Admin.SetLFGRoleCheck(category, lfgID, slots, members, isBattleground)

Starts an LFG role check. Retail and PTR only.

- **category** `number` -- `LE_LFG_CATEGORY_*` value, at least 1
- **lfgID**, **slots**, **members** `number` -- non-negative integers
- **isBattleground** `boolean`
- **Affects:** `GetLFGRoleUpdate()` returns `true, slots, members, category, lfgID, isBattleground`; Blizzard `GetLFGMode(category, lfgID)` returns `"rolecheck"`
- **Fires:** nothing

### A_Admin.ClearLFGRoleCheck()

Ends the role check. `GetLFGRoleUpdate()` then returns `false, 0, 0, 0, 0, false` (INFERRED R88g9, native evidence pending: the tuple the simulator reported before this model).

### A_Admin.ShowLfgExpandSearchPrompt(key, data)

Supplies the server's expand-search prompt (Retail and WoW Forever). `key` is a non-negative integer; `data` is an opaque string. It stores the prompt and fires `SHOW_LFG_EXPAND_SEARCH_PROMPT`, so the original `LFGEventFrame` shows the `LFG_QUEUE_EXPAND` popup.

- **Affects:** `C_LFGInfo.ConfirmLfgExpandSearch()` (the popup's accept action) clears a prompt with a nonzero key ([LFG expand confirmation](../specs/lfg-expand-confirmation.md)).

## Loot threshold and opt-out (Retail)

`GetLootThreshold()` reads the group-loot quality threshold (initially 2,
Uncommon). `SetLootThreshold(quality)` changes it only for the leader of a group
supplied with `A_Admin.SetPartySize`/`A_Admin.SetPartyLeader`, only to 2–4, and
fires `PARTY_LOOT_METHOD_CHANGED` when the value changes.
`SetOptOutOfLoot(1|nil)` stores the opt-out flag read by `GetOptOutOfLoot()`.
INFERRED (R-ag10-16). Implementation: `src/lua_api/globals/real/loot_method.rs`.

## Chat and combat logging (Retail)

`LoggingChat([on])` and `LoggingCombat([on])` back the `/chatlog` and
`/combatlog` slash commands: with an argument they set the toggle, and they
return the resulting state. Both start off and no log file is written.
INFERRED (R-ag10-17). Implementation: `src/lua_api/globals/real/session_logging.rs`.
