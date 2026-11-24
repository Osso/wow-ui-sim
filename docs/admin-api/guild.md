# Guild

### A_Admin.SetGuildEventsEnabled(enabled)

Sets the Boolean returned by `C_GuildInfo.AreGuildEventsEnabled()` on Retail and WoW Forever. It defaults to `false` until the host supplies availability.

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

### A_Admin.ClearGuild()

Removes the player from any guild.

- **Affects:** `GetGuildInfo("player")` returns `nil`
- **Example:**
```lua
A_Admin.ClearGuild()
print(GetGuildInfo("player"))   -- nil
```
