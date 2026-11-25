# Guild

### A_Admin.SetGuildEventsEnabled(enabled)

Sets the Boolean returned by `C_GuildInfo.AreGuildEventsEnabled()` on Retail and WoW Forever. It defaults to `false` until the host supplies availability.

This default is INFERRED R88fq (native evidence pending).

### A_Admin.SetGuildPreferredPlaySettingsFeatures(canChangeLocale, canChangeDatacenterLocality)

Sets the two Boolean returns from `C_GuildInfo.GetPreferredPlaySettingsFeatures()` on Retail and WoW Forever. Both default to `false` until the host supplies capabilities.

This default is INFERRED R88fq (native evidence pending).

### A_Admin.SetGuildReputationEnabled(enabled)

Sets the Boolean returned by `C_GuildInfo.IsGuildReputationEnabled()` on Retail and WoW Forever. It defaults to `false` until the host supplies availability.

This default is INFERRED R88fq (native evidence pending).

### A_Admin.SetGuildRenameStatus(status)

Supplies the status record returned asynchronously by `C_GuildInfo.RequestRenameStatus()` on Retail and WoW Forever. The next request emits `GUILD_RENAME_STATUS_UPDATE` on a later timer tick. If no status is supplied, the request emits no reply.

`status` uses the `GuildRenameStatus` fields: `isNameChangeEnabled`, `isPlayerGuildMaster`, `refundEligibleEndTime`, `nextRenameTime`, `renamePrice`, `refundAmount`, `currentGuildMoney`, `result`, `oldGuildName`, `reservedName`, and `reservedNameExpirationTime`.

- **Example:**
```lua
A_Admin.SetGuildRenameStatus({
    isNameChangeEnabled = true,
    isPlayerGuildMaster = true,
    refundEligibleEndTime = 0,
    nextRenameTime = 0,
    renamePrice = 500,
    refundAmount = 0,
    currentGuildMoney = 1000,
    result = Enum.GuildErrorType.Success,
    oldGuildName = "Previous",
    reservedName = "",
    reservedNameExpirationTime = 0,
})
```

### A_Admin.SetGuildInfo(name, rank, numMembers)

Configures the player's guild membership.

- **name** `string` -- Guild name
- **rank** `string` -- Player's rank title within the guild
- **numMembers** `number` -- Total guild member count
- **Affects:** `GetGuildInfo("player")` -- returns name, rank, realm; `GetNumGuildMembers()`
- **Example:**
```lua
A_Admin.SetGuildInfo("Knights of the Ebon Blade", "Champion", 42)
local name, rank = GetGuildInfo("player")
-- name="Knights of the Ebon Blade", rank="Champion"
```

### A_Admin.SetGuildPerks(spellID, level, ...)

Supplies the ordered guild-perk list used by the Retail and WoW Forever Communities Guild Perks panel. Pass one or more `(spellID, level)` pairs; call with no arguments to clear the list. `GetGuildPerkInfo(index)` resolves the supplied spell name and icon from the spell catalog and returns the spell ID. The perk list starts empty until the host supplies it.

This getter shape and empty default are INFERRED R88g3 (native evidence pending), based on the original `Blizzard_Communities/GuildPerks.lua` consumer.

- **Example:**
```lua
A_Admin.SetGuildInfo("Knights of the Ebon Blade", "Champion", 42)
A_Admin.SetGuildPerks(83944, 1, 83940, 2)
```

### A_Admin.ClearGuild()

Removes the player from any guild.

- **Affects:** `GetGuildInfo("player")` returns `nil`
- **Example:**
```lua
A_Admin.ClearGuild()
print(GetGuildInfo("player"))   -- nil
```
