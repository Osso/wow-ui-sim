# Console command catalog

`ConsoleGetAllCommands` and `C_Console.GetAllCommands` expose the simulator's supported CVar names and epoch-scoped console commands through `src/c_api/c_console.rs`. This is a bounded modeled catalog, not the complete native console command list.

## What it must do

- [x] Return fresh arrays of CVar records through both global and namespace queries, containing supported built-in and runtime-registered names with no case-insensitive duplicates.
- [x] Observe later CVar registrations without modifying prior query results; result mutations must not affect subsequent queries or other environments.
- [x] Preserve CVar values and overrides during enumeration.
- [x] Use the existing catalog's deterministic case-insensitive name ordering. This is simulator policy, not native ordering evidence.
- [x] Each record contains `command`, `commandType`, `category = Enum.ConsoleCategory.None`, and empty `help`, `scriptContents`, and `scriptParameters`. Absent native metadata uses these INFERRED defaults; it does not claim native metadata fidelity.
- [x] CVars use `Enum.ConsoleCommandType.Cvar` (cached docs spell the enum key `Cvar`, not `CVar`). Retail epochs from 12.0.7 expose `fetchBleepProxies` and `MemUsageStackTrace` as `Command`; earlier epochs omit them. GetCVar/default remain nil for those commands.
- [x] Shared publication sweeps probe command rows by exact name and Command type, requiring presence for additions and absence for removals. Commands do not receive CVar-default comparisons.

## Evidence and inferred limits

Cached retail `ConsoleDocumentation.lua` defines all six required fields and command types Cvar=0, Command=1, Macro=2, Script=3. Cached `Blizzard_Console_AutoComplete.lua` uses names, types, categories and help for completion; CVar values are queried only for Cvar records. The 12.0.7 wikitext register adds the two commands; 12.0.5 and 12.1.0 contain no command rows. INFERRED: additions persist in later retail epochs, remain absent in unmeasured non-retail profiles, and use stable case-insensitive ordering and empty metadata. Command execution is not modeled.

The baseline empty-table test was stale since `2c78bff73` introduced state-backed CVar enumeration; it failed at master base `2a7915d14` with 1,653 records. Replace the assertion with observable runtime registration, record shape/type and command/CVar separation.

## How it works

- [Lua API architecture](../lua-api.md)
- [Forever addon investigation](../wiki/investigations/forever-addon-comparison.md)

## Implementation inventory

- `src/c_api/c_console.rs`: builds records from existing CVar storage plus epoch-scoped native command names.
- `src/cvars.rs`: authoritative supported-name catalog, including profile removals.
- `src/lua_api/globals/register.rs`: registers the legacy global.
- `src/lua_api/globals/missing_surface/small_namespaces.rs`: connects the existing namespace method to the same implementation.

## Tests asserting this spec

- `tests/console_commands.rs`: built-in catalog, metadata, ordering, registration lifecycle, snapshots, overrides and environment isolation (2/2 in `/tmp/forever-addon-audit/console-tests-green-bm5izhjq/ledger.json`).

## Known gaps (current cycle)

- [ ] Validate unchanged Datamine root startup. The prior 863-member `[]` observation is invalidated: diagnostic trace recovered and silently discarded `__Datamine_45253` after `Missing FileName or FilePath` (`/tmp/forever-addon-runtime/datamine-recovery-trace-2do7026r/stderr`).
- [ ] Resolve the observed nil enum input and nil `tagToActor` errors exposed by the real chat-dispatched `/dm ui` open/close attempt before crediting a bounded interaction; no missing-function cause is established (`/tmp/forever-addon-audit/verify-datamine-ui-failure-ledger.json`).

## Out of scope

Commands beyond the two documented 12.0.7 additions, macro/script records, native category/help metadata, command execution, full client catalog completeness and native-client conformance. Cached Forever `ConsoleDocumentation.lua` establishes the global and record shape; local Wowless `data/impl/ConsoleGetAllCommands.lua` supplies the CVar-only precedent, not native verification.
