# CurrencyTransfer hidden EditBox startup notification

The Retail CurrencyTransfer startup Usage error came from simulator `OnTextChanged` dispatch while the original transfer menu was hidden and had no selected currency. **VERIFIED (simulator source):** the current implementation keeps changed EditBox notifications pending while the EditBox or an ancestor is hidden. The API behavior stays strict. This visibility rule is **INFERRED — R88g8 (native evidence pending)**; it is not a verified native contract.

## Content

### Observed sequence

- **VERIFIED (Retail and WoW Forever startup captures before the EditBox change):** `Blizzard_TokenUI/Blizzard_CurrencyTransfer.lua:488` reported the required-currency Usage error, and `Blizzard_ScriptErrors` recorded its echo.
- **VERIFIED (Retail and WoW Forever cached UI files):** the TokenUI TOCs do not declare `LoadOnDemand`; `Blizzard_CurrencyTransfer.xml` creates `CurrencyTransferMenu` hidden, and its amount input has no XML text value. The amount selector's `OnLoad` calls `Reset`, which calls `SetNumber(0)`.
- **VERIFIED (Retail cached Lua):** the menu's `OnLoad` initializes visuals. `OnCurrencyTransferRequested` sets the currency before `ShowUIPanel`; `OnShow` calls `FullRefresh`, which selects the highest-quantity source.
- **VERIFIED (simulator source):** XML frame finalization creates children and fires lifecycle scripts before returning. The loader flushes pending EditBox text changes after the complete top-level frame, so it dispatched the hidden input's change after `OnLoad` but before a later request could select a currency.
- **VERIFIED (simulator and required API contract):** the hidden input had no selected currency at that point, and `GetMaxTransferableAmountFromQuantity` rejects a nil currency ID with a Usage error. No C API behavior change is valid.

### Evidence limits and model choice

- **UNKNOWN (native):** the initial native text value of this XML EditBox and whether a hidden EditBox receives or delays `OnTextChanged`. The absent XML text value does not establish the native constructor default.
- **INFERRED — R88g8 (native evidence pending):** a changed hidden EditBox should retain its pending notification until a host boundary after it and its ancestors become visible. The original consumer sets the currency before showing the menu; its visible `FullRefresh` selects a source when roster data is ready. This avoids dispatch during uninitialized hidden construction without weakening the required API contract.
- **VERIFIED (test source):** `hidden_currency_transfer_input_waits_for_menu_visibility` loads the original UI, checks the hidden/unselected state, checks positive maxima for currency IDs 1560 and 1792, and rejects the startup Usage record. Passing test evidence is pending. The original menu-consumer test separately checks the selected-source amount and cost behavior.

The rule applies to EditBoxes generally. It changes notification timing only; text state and pending origin remain intact. Revisit the inference if native evidence becomes available.

## Sources

- [EditBox text position and selection contract](../../specs/editbox-text-position-selection.md)
- [EditBox notification implementation](../../../src/lua_api/editbox_update.rs)
- [Original TokenUI startup and consumer tests](../../../tests/blizzard_token_ui_loads.rs)
- [Currency API argument tests](../../../tests/currency_transfer_arguments.rs)
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_TokenUI/Blizzard_CurrencyTransfer.lua`
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_TokenUI/Blizzard_CurrencyTransfer.xml`
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_TokenUI/Blizzard_TokenUI.toc`

## See Also

- [[event-system]] — EditBox script dispatch and host boundaries
- [[lua-api]] — public frame and C API surfaces
