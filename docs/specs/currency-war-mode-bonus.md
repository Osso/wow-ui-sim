# Currency War Mode bonus flags

`C_CurrencyInfo.DoesWarModeBonusApply` queries explicitly supplied per-currency flags. Current cached Retail/PTR `CurrencyInfoDocumentation.lua` declares required numeric `currencyID`, two nullable booleans (`warModeApplies`, `limitOncePerTooltip`) and `AllowedWhenUntainted`. `QuestUtils` consumes both flags separately; its additional PvP/quest/faction gates are not this query's model.

## What it must do

- [x] Return exactly two values, preserving both independently nullable boolean inputs for the requested modeled currency.
- [x] Return `(nil, nil)` for absent modeled records without seeding normal eligibility. This is selected simulator policy, not native absence/catalog proof.
- [x] Reflect owned updates/removal on subsequent calls without changing previously returned scalar observations or other currency records.
- [x] Require numeric IDs without string coercion or lossy identity aliasing; authenticate secret inputs before validation/lookup and preserve wrappers/caller taint.
- [x] Publish the modeled query on current Retail/PTR surfaces without inferring earlier-profile backports or deriving flags from quantities, factions or global PvP state.

## How it works

- [C API boundary](../../AGENTS.md#c-api-boundary)
- [Currency projection, separate scope](currency-list-field-projection.md)

## Implementation inventory

- `tests/currency_war_mode_bonus.rs`: public absence/type/security controls plus nine nullable combinations, owned updates/removal and positive identity controls; getter projects live explicitly supplied flags.
- `src/c_api/item_spell/c_currency.rs`: Retail/PTR registration, secret-first numeric identity validation and two nullable scalar returns.
- `SimState.currency_war_mode_bonus`: empty owned map of currency IDs to two independently nullable flags.

## Tests asserting this spec

`tests/currency_war_mode_bonus.rs`. Initial five controls have authenticated RED; expanded nine cases execute0PASS/9FAIL before getter. Nine logical tests pass under both Retail/PTR (18 executions). Both normal loaded query assertion runs exit0/`[]`; owning publication sweep passes one case with145 rows91OK/54gaps. Formatting/default check exit0; retained proof linked from the [evidence packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/currency-war-mode-bonus-2026-10-12/README.md).

## Known gaps (current cycle)

- [x] Final independent bounded gate PASS: both profile GREENs, owning publication, Rust/readability checks and105 copies independently verified; four final report/receipt copies main byte-compared. No old binary rehash or unrelated case replay.

## Out of scope

Native eligibility catalogs, generated normal-session memberships, PvP desired/active state, quest eligibility, faction-grant relationships. Vendor `QuestUtils` computes first-instance status locally from repeated currency IDs; this is not a missing simulator field. These are separate caller dependencies: no mocked getter/vendor rewrite or end-to-end quest-display claim substitutes for those models. Completed currency field-projection proof remains separate; parent handoff OPEN.
