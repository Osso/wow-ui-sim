# Script-object environment crossings

Retail-family secure Blizzard functions use a distinct object-table partition for frame-private state. Public addon code must not obtain private mixin methods by overriding similarly named public fields. [XML mixin bindings](../xml-template-system.md) and the native AuraContainer frame provider exercise this boundary.

## What it must do

- [ ] Preserve native frame identity in the forbidden partition, so native parent arguments recognize the same frame while public and forbidden Lua tables remain distinct.
- [ ] Convert direct frame arguments and results when Lua closures cross between the public and secure environments. Ordinary tables and nested data are not recursively rewritten.
- [ ] Keep explicit native `GetObjectTable` results public for outbound addon initializers.
- [ ] Intern forbidden views and make repeated `GetForbiddenObjectTable` projection idempotent.
- [ ] Run AuraContainer provider creation and inbound child ownership validation without publishing private methods or invoking public overrides of private methods.

## How it works

- `rilua` optional environment-transfer hook, pinned to `f1eab545d3536cec8a64a3e1491ee9e06dd1b091`.
- `src/lua_api/script_object_transfer.rs` registers a host conversion callback after secure-environment creation. Canonical registry mappings identify partitions, not user-controlled backlink fields alone.
- `src/lua_api/env_init/shared_bootstrap.lua` constructs and interns forbidden tables.

## Implementation inventory

- `src/lua_api/script_object_transfer.rs` — direct frame-reference conversion.
- `src/lua_api/env_init/mod.rs`, `src/lua_api/mod.rs` — retail-family installation.
- `src/lua_api/env_init/shared_bootstrap.lua` — partition identity and projection.
- `Cargo.toml`, `Cargo.lock` — published runtime dependency pin.

## Tests asserting this spec

- `tests/userdata_proxy.rs` — native parent acceptance, spoof rejection, real provider acquisition, initializer ownership, and private/public isolation.
- Runtime environment-transfer tests cover arguments, varargs, multiple results, tail returns, native callbacks, reentry and error propagation.

## Known gaps

- [ ] Simulator integration verification pending.
- Coroutine yield/resume conversion and recursive conversion of table contents are not implemented by this facility.
- Conditional aura-secrecy access enforcement is separate from partition conversion and remains unmodeled.

## Out of scope

Vendor Lua changes, private-method stubs in the public table, or equating distinct Lua tables through a universal equality override.
