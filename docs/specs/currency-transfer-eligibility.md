# Retail currency transfer eligibility

`IsAccountTransferableCurrency` returns true only for an existing currency with a strictly positive transfer percentage. Percentages, currency flags and category flags come from generated CurrencyTypes and CurrencyCategory data for 12.1.5.70391; WM64 infers the percentage column name as `WarbondTransferPercentage`. Zero, missing and NaN values return false.

`CanTransferCurrency` returns a boolean and an optional result. In order, it checks the supplied server control, transfer-in-progress state, context presence, percentage and category flags, faction keys, award condition, quantity and weekly caps, and a positive quantity on a different character in the account cache. Success returns true and nil. The query does not request data or dispatch an event.

The shared server control defaults to disabled, matching WM66's verified cold-client zero fill; its ordinary-session value is unknown and may be supplied. Transfer-in-progress defaults to idle. In-world context defaults to present, inferred as the active local player. Currency and transaction loading do not gate this query. The host supplies account cache data and controls response timing.

When needed for a decision, missing faction-key values, award-condition results, effective quantity or weekly cap results, and the current character GUID produce explicit not-modeled errors. WM64 leaves the public faction-key names, condition internals and effective cap overrides unknown; WM66 leaves protected context guards and login/server timing unknown. Account-character quantities are supplied in the account cache.

WM64 inspected Retail 12.1.5 build 70688. The public names and complete semantics of the faction keys, award-condition evaluation, effective cap overrides and live server transitions remain unknown. Supplied effective results are not a native evaluator for those systems. The generated currency data predates the inspected 70688 client build.
