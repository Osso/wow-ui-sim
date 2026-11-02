# SavedVariables persistence

## Current integration

GUI close, Ctrl-Q and Lua Quit use the same fallible
[session finalizer](session-finalization.md). Applicable synchronous logout
callbacks finish before persistence. Current failed declared inputs and
uninitialized registered owners remain protected. Failed attempts in an intact
environment can be retried after repair; a partial rebuilt VM cannot save. The
result reaches the process caller. Dropping the GUI does not save or retry.
[ReloadUI](reload-ui.md) reuses saving and reconstruction without completing the
session. Native Wrath shutdown facts do not establish other-profile behavior.

Ordinary addon saves and private simulator models share the
[serializer](../../src/saved_variables_serialize.rs).
[Native evidence](saved-variables-native-evidence.md) records the remaining
compatibility limits.

The [storage and process results](session-finalization.md#storage-and-process-results)
describe configured output, interactive completion and partial-save limits.
Byte-preservation and fresh-VM read-back checks belong to the serializer
regression tests.

## Inputs and scopes

[Runtime inputs](../runtime-inputs.md) select WTF identity and isolated storage.
WTF imports are read-only. Source paths relative to the WTF root are:

- Account: `Account/<account>/SavedVariables/<addon>.lua`.
- Character: `Account/<account>/<realm>/<character>/SavedVariables/<addon>.lua`.

Ordinary output uses `SavedVariables` beneath an explicit writable
`--session-wtf-path DIR`. Without that option, a non-empty `WOW_SIM_STORAGE_DIR`
selects the SavedVariables directory directly; otherwise it is `SavedVariables`
under the platform local-data directory's `wow-sim` root. Account data uses
`<addon>.lua`; character data uses `<realm>/<character>/<addon>.lua`.
Simulator preferences, CVars and private models use the session root. Local
addon enablement defaults to the root's `AddOns.txt`; `WOW_SIM_ADDONS_TXT`
overrides that file for both reads and saves. For the full layout,
see [persistence destinations](../runtime-inputs.md#persistence-destinations).
The resolved root is passed explicitly into the runtime and retained through
ReloadUI. The real WoW WTF import tree remains read-only.

Initial input defaults to a 2 MiB limit per file;
`WOW_SIM_MAX_SAVED_VARIABLE_BYTES=0` removes it. Oversized or unreadable input
fails. Trusted ReloadUI files bypass that external size cap.

Loading uses the fast data parser with Lua VM fallback. Imported Details geometry
retains the simulator's existing normalization. An explicit `--session-wtf-path`
restores that session's saved account and character scopes first, importing only
missing scopes. Ordinary launches retain WTF-first loading with simulator storage
as fallback. Invalid selected input remains an error; it never selects the other
source as recovery. ReloadUI prefers the files just written by the simulator.

The fast parser preserves Lua control escapes (`\a`, `\b`, `\f`, `\v`) and
normalizes escaped physical LF/CR/CRLF/LFCR to one LF, checked against the rilua
VM. Raw non-UTF-8 input falls back to the VM. These checks establish simulator
byte preservation; native-client parity remains separate.

## Load errors

[SavedVariablesManager](../../src/saved_variables.rs) tracks account and character
input attempts within each Lua environment. Repeated initialization preserves
loaded values and returns recorded failures instead of silently retrying failed
input. Explicit lifecycle recovery retries failed declared scopes at their exact
recorded source, without changing healthy edits. A previously failed source that
disappears remains an error. Optional imports without declared output do not block
unrelated saves. Missing files are verified as absent, including checking ancestors
so a regular file in a directory component is reported as an error.

Links to regular files are read; dangling links and links to non-files fail.
Unreadable, non-file, oversized or invalid Lua input returns an error. Failure in
one scope does not prevent the other scope from being attempted. ReloadUI clears
the environment's successful input-attempt state before reading simulator-owned
files; failed declared-source information survives partial-VM recovery.
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
do not provide atomic replacement across multiple files. The
[session finalizer](session-finalization.md) supplies the observable shutdown
boundary around these operations.
