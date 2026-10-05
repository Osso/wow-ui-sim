# Console command registry

`src/c_api/c_console.rs` rebuilds fresh records for namespace and legacy queries from `CVarStorage::all_keys()` plus retail epoch-scoped command names. Contract and inferred metadata policies live in [console command catalog](../../specs/console-command-catalog.md).

## Data flow

Runtime registration updates CVar storage; subsequent queries enumerate it without mutating prior results. Command names stay separate from CVars. Shared publication sweeps use exact names and `Enum.ConsoleCommandType.Command` instead of querying CVar values for command rows. Added rows require presence; removed rows require absence.

## Root cause

`test_c_console_get_all_commands_empty` retained the empty-stub expectation after `2c78bff73` exposed live CVar records. Master base `2a7915d14` reproduced 1,653 records versus expected zero. The replacement tests check live registration, documented record shape and command/CVar separation.

## Sources

- [Contract and tests](../../specs/console-command-catalog.md)
- [Registry implementation](../../../src/c_api/c_console.rs)
- [Shared sweep](../../../tests/common/publication_sweep.rs)
- Cached retail `Blizzard_APIDocumentationGenerated/ConsoleDocumentation.lua` and `Blizzard_Console/Blizzard_Console_AutoComplete.lua` — record schema and unchanged consumers

## See Also

- [Forever addon investigation](../investigations/forever-addon-comparison.md) — original CVar-only registry
- [12.0.7 API audit](../investigations/patch-12-0-7-api-audit.md) — command publication gaps
