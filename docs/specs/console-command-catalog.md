# Console command catalog

`ConsoleGetAllCommands` and `C_Console.GetAllCommands` expose the simulator's supported CVar names through `src/c_api/c_console.rs`. This is a bounded modeled catalog, not the complete native console command list.

## What it must do

- [ ] Return fresh arrays of CVar records through both global and namespace queries, containing supported built-in and runtime-registered names with no case-insensitive duplicates.
- [ ] Observe later CVar registrations without modifying prior query results; result mutations must not affect subsequent queries or other environments.
- [ ] Preserve CVar values and overrides during enumeration.
- [ ] Use the existing catalog's deterministic case-insensitive name ordering. This is simulator policy, not native ordering evidence.
- [ ] Each record contains `command`, `commandType = Enum.ConsoleCommandType.Cvar`, `category = Enum.ConsoleCategory.None`, and empty `help`, `scriptContents`, and `scriptParameters`. Empty metadata follows Wowless's CVar-only precedent; it does not claim native metadata fidelity.

## How it works

- [Lua API architecture](../lua-api.md)
- [Forever addon investigation](../wiki/investigations/forever-addon-comparison.md)

## Implementation inventory

- `src/c_api/c_console.rs`: builds records from existing CVar storage.
- `src/cvars.rs`: authoritative supported-name catalog, including profile removals.
- `src/lua_api/globals/register.rs`: registers the legacy global.
- `src/lua_api/globals/missing_surface/small_namespaces.rs`: connects the existing namespace method to the same implementation.

## Tests asserting this spec

- `tests/console_commands.rs`: built-in catalog, metadata, ordering, registration lifecycle, snapshots, overrides and environment isolation.

## Known gaps (current cycle)

- [ ] Validate unchanged Datamine startup and a bounded real interaction after console and numeric registration fixes.

## Out of scope

Non-CVar commands, native category/help metadata, command execution, full client catalog completeness and native-client conformance. Cached Forever `ConsoleDocumentation.lua` establishes the global and record shape; local Wowless `data/impl/ConsoleGetAllCommands.lua` supplies the CVar-only precedent, not native verification.
