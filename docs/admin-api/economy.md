# Economy & Items

### A_Admin.SetMoney(copper)

Sets the player's total money in copper.

- **copper** `number` -- Total copper amount. Conversion: 1 gold = 10,000 copper (10 silver per gold, 100 copper per silver).
- **Affects:** `GetMoney()`
- **Fires:** `PLAYER_MONEY`
- **Example:**
```lua
A_Admin.SetMoney(1000000)   -- 100 gold
A_Admin.SetMoney(50000)     -- 5 gold
print(GetMoney())           -- 50000

-- Build from components
local gold, silver, copper = 10, 5, 23
A_Admin.SetMoney(gold * 10000 + silver * 100 + copper)
```

### A_Admin.SetAccountCharacterCurrencyData(currencyID, sources)

Supplies account-character currency balances for transfer UI tests.

- **currencyID** `number` -- Currency ID.
- **sources** `table[]` -- Character records with `characterGUID`, `characterName`, `fullCharacterName`, and `quantity` fields. The quantity must be a nonnegative integer.
- **Affects:** `C_CurrencyInfo.FetchCurrencyDataFromAccountCharacters(currencyID)` after account data completes.

```lua
A_Admin.SetAccountCharacterCurrencyData(1792, {
    { characterGUID = "Player-2-00000002", characterName = "Source", fullCharacterName = "Source-Realm", quantity = 200 },
})
```

### A_Admin.SetAccountCurrencyTransferEnabled(enabled)

Sets whether account currency transfer is enabled by the server.

- **enabled** `boolean`
- **Affects:** `C_CurrencyInfo.CanTransferCurrency(currencyID)`

### A_Admin.SetCurrencyTransferRules(currencyID, rules)

Supplies currency-specific transfer eligibility inputs. Fields that are not supplied remain unknown.

- **currencyID** `number` -- Currency ID.
- **rules** `table` -- Optional boolean fields: `faction_key_0`, `faction_key_1`, `award_condition_met`, `quantity_cap_reached`, and `weekly_cap_reached`.
- **Affects:** `C_CurrencyInfo.CanTransferCurrency(currencyID)`

### A_Admin.SetItemLevel(ilvl)

Sets the player's displayed average item level.

- **ilvl** `number` -- Item level value
- **Affects:** `GetAverageItemLevel()` -- returns ilvl for all three values (equipped, bag, pvp)
- **Example:**
```lua
A_Admin.SetItemLevel(480)
local equipped, bag, pvp = GetAverageItemLevel()
-- equipped=480
```
