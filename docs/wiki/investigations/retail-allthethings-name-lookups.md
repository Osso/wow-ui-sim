# Retail AllTheThings name lookups

**VERIFIED (Retail 12.1.5.70783 full-addon runs):** AllTheThings exposed missing simulator spell and area names; the full pinned indexes remove the original four errors. A separate AllTheThings instruction-budget error remains.

## Content

### Missing names

**VERIFIED (Retail 12.1.5.70783 SpellName DB2 export):** the selected export contains 416,204 non-empty readable spell names; the simulator now searches it after `SPELL_DB` misses. `C_Spell.GetSpellName` returns `Heart of Azeroth` for 275825, `Active` for 133137, and `Orange Path` for 251097. Unknown IDs still return no values.

**VERIFIED (Retail 12.1.5.70783 AreaTable DB2 export):** IDs 7288, 7290, and 7291 resolve to `Smuggler's Den`, `Hearthfire Tavern`, and `Stoktron Brewery`. Unknown and absent area IDs still return nil. Retail `C_Map.GetAreaInfo` reads the compact embedded name index; non-Retail profiles keep the existing small Lua fallback.

**INFERRED (same startup run and source order):** the `General - Unobtainables.lua` nil `reason` error was downstream of the locale table initialization failures. It disappeared when the missing names were supplied.

### Residual instruction budget

**VERIFIED (before/after full-addon error lists):** the baseline had 12 unique errors / 13 occurrences, including the `Default Locale.lua` and `LocalizationDB.lua` nil concatenations, the `General - Unobtainables.lua` nil reason, and the `EventRegistration.lua` `pairs(table expected)` error. After the full name indexes, the run had 8 unique errors / 11 occurrences; all four original ATT errors were absent, and ATT had four occurrences of `instruction budget exhausted for owner 'AllTheThings'` instead.

**VERIFIED (temporary simulator trace, `PLAYER_LOGIN`, frame 47350):** ATT entered the callback at 799,799 of 10,000,000 instructions, made zero `GetSpellName` and zero `GetAreaInfo` calls, then reached 10,000,000 after consuming 9,200,201 dispatch instructions. **VERIFIED (three later `ADDON_LOADED` callbacks, frame 49694):** each began and ended at 10,000,000 and added zero instructions.

**INFERRED (trace plus instruction-meter contract):** the residual budget error comes from ATT's own callback work, not from simulator name-index searches. This lane leaves the 10-million host policy unchanged. Native quota semantics remain unverified; main queued question R88bn to aso.

### Lookup performance

**VERIFIED (slot-3 `sim-data` unit test):** the first lookup measured 25,939 ns in a warmed run; lookups 2–1000 averaged 45 ns each. An earlier first-call sample measured 954,208 ns. The binary index uses binary search, returns a static name slice, and does not decode or allocate the full catalog at startup. Each hit scans and UTF-8-validates only the selected name.

**VERIFIED (slot-3 warm Forever `--no-run` rebuild):** after touching only `src/lib.rs`, the branch took 29.05 Cargo seconds and base `4fb6237635a` took 27.43 seconds. Both runs reused `sim-data` and `sim-data-profile`; only the root crate and dynamic library rebuilt. The 154-second earlier run rebuilt cold dependencies and is not the warm comparison.

## Sources

- [Generated data crate design](../design/generated-data-crates.md)
- [Generated data crate contract](../../specs/generated-data-crates.md)
- [AreaName provenance](../../../crates/sim-data-profile/data/area_names_retail.provenance.json)
- [SpellName provenance](../../../crates/sim-data/data/spell_names_retail.provenance.json)
- AllTheThings Retail consumer files: `locales/Default Locale.lua`, `db/Standard/LocalizationDB.lua`, `src/Settings/Pages/General - Unobtainables.lua`, and `src/Settings/Settings.lua`.

## See Also

- [[generated-data-crates]] — generated crate boundary and build measurements
- [[event-system]] — frame event dispatch and owner instruction budgets
