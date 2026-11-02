# Retail currency transfer eligibility

`IsAccountTransferableCurrency` returns true only for an existing currency with a strictly positive transfer percentage. The initial percentages come from the generated CurrencyTypes data pinned at 12.1.0.69497; WM64 infers the source column name as `WarbondTransferPercentage`. Zero, missing and NaN values return false.

`CanTransferCurrency` returns a boolean and an optional result. In order, it checks the supplied server control, transfer-in-progress state, context presence, percentage and category flags, faction keys, award condition, quantity and weekly caps, and a positive quantity on a different character in the account cache. Success returns true and nil. The query does not request data or dispatch an event.

The server control, category and currency flags, faction keys, award result, effective cap results, current GUID and account-character quantities are supplied state. Missing inputs needed for a decision produce an explicit not-modeled error. The server control has no assumed default. Currency and transaction loading do not gate this query. The host supplies account cache data and controls response timing.

WM64 inspected Retail 12.1.5 build 70688. The public names and complete semantics of the faction keys, award-condition evaluation, effective cap overrides and live server transitions remain unknown. Supplied effective results are not a native evaluator for those systems. The pinned currency data predates the inspected client build.
