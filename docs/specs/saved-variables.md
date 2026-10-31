# SavedVariables persistence

## Current integration

Production GUI close saves through `SavedVariablesManager::save_all` in
[App::drop](../../src/iced_app/app.rs); errors are logged. [ReloadUI](reload-ui.md)
saves before retiring the VM and loads the new simulator files during reconstruction.
A reliable, fallible finish operation that includes all logout callbacks remains
unfinished; native Wrath shutdown facts do not establish other-profile behavior.

Ordinary addon saves and private simulator models share the
[serializer](../../src/saved_variables_serialize.rs).
[Native evidence](saved-variables-native-evidence.md) records the remaining
compatibility limits.

## Inputs and scopes

WTF imports are read-only. Source paths relative to the WTF root are:

- Account: `Account/<account>/SavedVariables/<addon>.lua`.
- Character: `Account/<account>/<realm>/<character>/SavedVariables/<addon>.lua`.

Ordinary output uses `WOW_SIM_STORAGE_DIR`, or the platform data directory's
`wow-sim/SavedVariables`: `<addon>.lua` for account data and
`<realm>/<character>/<addon>.lua` for character data. Initial input defaults to a
2 MiB limit per file; `WOW_SIM_MAX_SAVED_VARIABLE_BYTES=0` removes it. Oversized
or unreadable input fails. Trusted ReloadUI files bypass that external size cap.

Loading uses the fast data parser with Lua VM fallback. Imported Details geometry
retains the simulator's existing normalization. Missing WTF scopes can fall back
to simulator storage. ReloadUI prefers the files just written by the simulator.

The fast parser preserves Lua control escapes (`\a`, `\b`, `\f`, `\v`) and
normalizes escaped physical LF/CR/CRLF/LFCR to one LF, checked against the rilua
VM. Raw non-UTF-8 input falls back to the VM. These checks establish simulator
byte preservation; native-client parity remains separate.

## Load errors

[SavedVariablesManager](../../src/saved_variables.rs) tracks account and character
input attempts within each Lua environment. Repeated initialization preserves
loaded values and returns recorded failures instead of silently retrying failed
input. Missing files are verified as absent, including checking ancestors so a
regular file in a directory component is reported as an error.

Links to regular files are read; dangling links and links to non-files fail.
Unreadable, non-file, oversized or invalid Lua input returns an error. Failure in
one scope does not prevent the other scope from being attempted. ReloadUI clears
the environment's input-attempt state before reading simulator-owned files.
The `C_AddOns` runtime load-on-demand path currently lacks the persistence manager;
see [ReloadUI](reload-ui.md).

## Serialization

The [serializer](../../src/saved_variables_serialize.rs) supports explicit nil
assignments, booleans, finite f64 values, exact byte strings and raw nested tables.
Keys may be byte strings, finite numbers or booleans. Sparse slots and added/deleted
entries survive. Three-digit decimal escapes preserve arbitrary bytes; decimal
numbers round-trip finite values, including signed zero. Repeated acyclic tables
become independent copies. Cycles, unsupported values/keys, invalid identifiers or
references, non-finite numbers and nesting beyond 128 tables fail explicitly.
Metatables are not serialized and lookup metamethods are not invoked. Serializer
and filesystem errors propagate through `save_addon`/`save_all`. These operations
do not provide atomic replacement across multiple files or an observable finish
operation for production shutdown.
