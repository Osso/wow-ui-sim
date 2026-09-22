# Console command catalog

`ConsoleGetAllCommands` and `C_Console.GetAllCommands` expose the simulator's supported CVar names through `src/c_api/c_console.rs`. This is a bounded modeled catalog, not the complete native console command list.

## What it must do

- [x] Return fresh arrays of CVar records through both global and namespace queries, containing supported built-in and runtime-registered names with no case-insensitive duplicates.
- [x] Observe later CVar registrations without modifying prior query results; result mutations must not affect subsequent queries or other environments.
- [x] Preserve CVar values and overrides during enumeration.
- [x] Use the existing catalog's deterministic case-insensitive name ordering. This is simulator policy, not native ordering evidence.
- [x] Each record contains `command`, `commandType = Enum.ConsoleCommandType.Cvar`, `category = Enum.ConsoleCategory.None`, and empty `help`, `scriptContents`, and `scriptParameters`. Empty metadata follows Wowless's CVar-only precedent; it does not claim native metadata fidelity.

## How it works

- [Lua API architecture](../lua-api.md)
- [Forever addon investigation](../wiki/investigations/forever-addon-comparison.md)

## Implementation inventory

- `src/c_api/c_console.rs`: builds records from existing CVar storage.
- `src/cvars.rs`: authoritative supported-name catalog, including profile removals.
- `src/lua_api/globals/register.rs`: registers the legacy global.
- `src/lua_api/globals/missing_surface/small_namespaces.rs`: connects the existing namespace method to the same implementation.

## Tests asserting this spec

- `tests/console_commands.rs`: built-in catalog, metadata, ordering, registration lifecycle, snapshots, overrides and environment isolation (2/2 in `/tmp/forever-addon-audit/console-tests-green-bm5izhjq/ledger.json`).

## Known gaps (current cycle)

- [x] Validate unchanged Datamine root startup: 863 byte-identical files load with zero warnings and trailing `[]`; Data and Maps remain LoadOnDemand (`/tmp/forever-addon-runtime/datamine-catalog-startup-b4hktcva/ledger.json`).
- [ ] Resolve the observed nil enum input and nil `tagToActor` errors exposed by the real chat-dispatched `/dm ui` open/close attempt before crediting a bounded interaction; no missing-function cause is established (`/tmp/forever-addon-audit/verify-datamine-ui-failure-ledger.json`).

## Out of scope

Non-CVar commands, native category/help metadata, command execution, full client catalog completeness and native-client conformance. Cached Forever `ConsoleDocumentation.lua` establishes the global and record shape; local Wowless `data/impl/ConsoleGetAllCommands.lua` supplies the CVar-only precedent, not native verification.
