# Player Identity

## A_Admin.SetPlayerName(name)

Sets the player character's name.

- **name** `string` -- The character name to display
- **Affects:** `UnitName("player")`, `GetUnitName("player")`
- **Example:**
```lua
A_Admin.SetPlayerName("Arthas")
print(UnitName("player"))  -- "Arthas"
```

## A_Admin.SetPlayerClass(classIndex)

Sets the player's class by 1-based index.

- **classIndex** `number` -- 1-based class index:

| Index | Class | File Token |
|-------|-------|------------|
| 1 | Warrior | WARRIOR |
| 2 | Paladin | PALADIN |
| 3 | Hunter | HUNTER |
| 4 | Rogue | ROGUE |
| 5 | Priest | PRIEST |
| 6 | Death Knight | DEATHKNIGHT |
| 7 | Shaman | SHAMAN |
| 8 | Mage | MAGE |
| 9 | Warlock | WARLOCK |
| 10 | Monk | MONK |
| 11 | Druid | DRUID |
| 12 | Demon Hunter | DEMONHUNTER |
| 13 | Evoker | EVOKER |

- **Affects:** `UnitClass("player")`, `UnitClassBase("player")`
- **Example:**
```lua
A_Admin.SetPlayerClass(2)   -- Paladin
local name, file, idx = UnitClass("player")
-- name="Paladin", file="PALADIN", idx=2
```

## A_Admin.SetPlayerRace(raceIndex)

Sets the player's race by 0-based index.

- **raceIndex** `number` -- 0-based race index:

| Index | Race | Faction |
|-------|------|---------|
| 0 | Human | Alliance |
| 1 | Orc | Horde |
| 2 | Dwarf | Alliance |
| 3 | Night Elf | Alliance |
| 4 | Undead | Horde |
| 5 | Tauren | Horde |
| 6 | Gnome | Alliance |
| 7 | Troll | Horde |
| 8 | Blood Elf | Horde |
| 9 | Draenei | Alliance |
| 10 | Worgen | Alliance |
| 11 | Goblin | Horde |
| 12 | Pandaren | Neutral |
| 13 | Dracthyr | Neutral |
| 14 | Earthen | Neutral |

- **Affects:** `UnitRace("player")`, `UnitFactionGroup("player")`
- **Example:**
```lua
A_Admin.SetPlayerRace(8)    -- Blood Elf
local race, raceFile = UnitRace("player")
local faction, factionFile = UnitFactionGroup("player")
-- race="Blood Elf", faction="Horde"
```

## A_Admin.SetPlayerLevel(level)

Sets the player's character level.

- **level** `number` -- Character level (typically 1-80)
- **Affects:** `UnitLevel("player")`, `UnitEffectiveLevel("player")`
- **Example:**
```lua
A_Admin.SetPlayerLevel(60)
print(UnitLevel("player"))  -- 60
```

## A_Admin.SetPlayerSex(sex)

Sets the player's displayed sex for localization purposes.

- **sex** `number` -- Sex identifier: `1` = unknown, `2` = male, `3` = female
- **Affects:** `UnitSex("player")`
- **Example:**
```lua
A_Admin.SetPlayerSex(3)     -- female
print(UnitSex("player"))    -- 3
```

## A_Admin.SetAccountType(account)

Supplies one account-type snapshot for the Retail and WoW Forever profiles. **INFERRED (R88fw; native evidence is pending):** the five account queries and `GetRestrictedAccountData()` read this same snapshot.

- **account** `table` -- All fields are required:
  - `trial` `boolean`
  - `veteranTrial` `boolean`
  - `restricted` `boolean`
  - `expansionTrial` `boolean`
  - `expansionTrialRemaining` `number` -- Remaining expansion-trial seconds; must be finite and nonnegative
  - `restrictedLevel` `number` -- Level limit; must be nonnegative
  - `restrictedMoney` `number` -- Copper limit; must be nonnegative
  - `restrictedProfession` `number` -- Profession-skill limit; must be nonnegative
- **Affects:** `IsTrialAccount()`, `IsVeteranTrialAccount()`, `IsRestrictedAccount()`, `IsExpansionTrial()`, `GetExpansionTrialInfo()`, `GetRestrictedAccountData()`
- **Example:**
```lua
A_Admin.SetAccountType({
    trial = true,
    veteranTrial = false,
    restricted = true,
    expansionTrial = true,
    expansionTrialRemaining = 3600,
    restrictedLevel = 20,
    restrictedMoney = 100000,
    restrictedProfession = 100,
})
local isTrial, remaining = GetExpansionTrialInfo()
local level, money, profession = GetRestrictedAccountData()
```
