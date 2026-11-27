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

### A_Admin.SetGuildBankMoney(copper)

Sets the copper amount returned by `GetGuildBankMoney()`, then fires `GUILDBANK_UPDATE_MONEY`. The original `GuildBankMoneyFrame` (`Blizzard_GuildBankUI/Mainline/Blizzard_GuildBankUI.xml:534-541`, `MoneyFrame.lua:158-165`) shows the new amount.

Retail and WoW Forever default to 0 copper. Mists retains its prior 123456-copper scenario default.

The event and profile defaults are INFERRED R88g9 (native evidence pending).

### A_Admin.SetGuildBankTabCost(copper)

Supplies a nonnegative tab cost returned by `GetGuildBankTabCost()`. The default is 0.

The setter and default are INFERRED R88g9 (native evidence pending).

### A_Admin.SetGuildBankRankPermissions(rank, moneyPerDay, tabRights, tabStacksPerDay)

Supplies one guild rank's bank permissions and selects that rank. `rank` is 1–10. `moneyPerDay` is copper; negative values represent unlimited allowance. Each of `tabRights` and `tabStacksPerDay` must contain six numbers. Rights use bit 1 for view, bit 2 for deposit and bit 4 for edit-tab-info; stack limits must be nonnegative. A zero money limit means the player cannot withdraw guild-bank money. The bank UI updates when this input changes.

With no selected permission row, `GetGuildBankWithdrawMoney()` and `GetGuildBankWithdrawGoldLimit()` return 0, and `CanWithdrawGuildBankMoney()` and `CanEditGuildBankTabInfo()` return false. The selected-rank projection and empty defaults are INFERRED R88g9 (native evidence pending).

### A_Admin.OpenGuildBank()

Retail and WoW Forever only. Starts the `Enum.PlayerInteractionType.GuildBanker` interaction and fires `PLAYER_INTERACTION_MANAGER_FRAME_SHOW`. The original `Blizzard_GuildBankUI_Bootstrap.lua` registration loads `Blizzard_GuildBankUI` and shows `GuildBankFrame`.

### Legacy guild-bank getters with no supplied state

Retail and WoW Forever return these empty-state values: `GetCurrentGuildBankTab()` returns 1; `GetNumGuildBankTabs()` returns 0; `GetGuildBankTabInfo()` returns no values; `GetGuildBankTabCost()` returns 0; `GetGuildBankText(tab)` returns an empty string; `GetGuildBankItemInfo()` returns no values; `GetGuildBankItemLink()` returns nil; `GetGuildBankMoney()` returns 0; `GetGuildBankWithdrawMoney()` and `GetGuildBankWithdrawGoldLimit()` return 0; `CanWithdrawGuildBankMoney()` and `CanEditGuildBankTabInfo()` return false. `QueryGuildBankTab()` selects the requested tab. The corresponding setters and money operations return no values. `PickupGuildBankItem()` has no supplied item to pick up.

Mists keeps its previous money default (123456 copper), unlimited withdraw-money value (-1), and true `CanWithdrawGuildBankMoney()` / `CanEditGuildBankTabInfo()` defaults. Its `GetGuildBankWithdrawGoldLimit()` remains 0. Values and behavior beyond the documented inputs are INFERRED R88g9 (native evidence pending).

- **Example:**
```lua
A_Admin.SetGuildBankMoney(1234567)
A_Admin.OpenGuildBank()
print(GuildBankMoneyFrameGoldButton:GetText()) -- "123"
```

### A_Admin.ClearGuild()

Removes the player from any guild.

- **Affects:** `GetGuildInfo("player")` returns `nil`
- **Example:**
```lua
A_Admin.ClearGuild()
print(GetGuildInfo("player"))   -- nil
```
