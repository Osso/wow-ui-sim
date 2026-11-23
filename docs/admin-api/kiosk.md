# Kiosk session input (A_Admin)

`A_Admin.SetKioskSession(session)` supplies kiosk-session state to Retail and WoW Forever scenarios. This is a simulator-only setter. **INFERRED (R88fd (native evidence pending)):** no supplied session means kiosk mode and competitive mode are disabled, and `Kiosk.GetKioskLoginInfo()` returns no values.

```lua
A_Admin.SetKioskSession({
    enabled = true,
    competitive = false,
    accountName = "account",
    password = "password",
    realmAddress = "realm",
})
```

- `enabled` and `competitive` are required Booleans. They supply `Kiosk.IsEnabled()` and `Kiosk.IsCompetitiveModeEnabled()`.
- `accountName`, `password`, and `realmAddress` are optional strings. Supply all three or omit all three. When supplied, `Kiosk.GetKioskLoginInfo()` returns the three values in that order.
- Each call replaces the current kiosk-session input. `enabled = false` does not imply that other session fields are absent.

**INFERRED (R88fd (native evidence pending)):** These defaults and supplied-field rules describe simulator scenarios. Native kiosk-session creation and credential behavior are not modeled.
