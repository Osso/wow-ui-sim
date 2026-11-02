# Interactive session finalization

`WowLuaEnv::finalize_session` finishes Lua shutdown callbacks and saves registered
SavedVariables through the existing persistence manager and serializer. Its
successful completion result is retained for the session. Failed attempts can be
retried while the current environment remains usable.

## Completion boundary

Title-bar close, Ctrl-Q and Lua `Quit`/`QuitGame`/`ForceQuit` enter one GUI finish
path after the active Lua call returns. They validate retained input failures,
run applicable synchronous shutdown callbacks, then call `save_all`. Disabled
persistence stays disabled. Successful finalization runs once; later requests
return the same outcome without rerunning callbacks or saving again.

Current initialization failures protect the affected addon's declared output;
historical diagnostics do not permanently invalidate the process. A failed
enabled owner with registered output must initialize successfully or be disabled
before finalization can succeed. Failed owners' data is never saved. Optional
EditMode/ServerSnapshot imports, undeclared input scopes, nil probes and missing
API observations remain diagnostic for unrelated SavedVariables.

Essential startup callback completion is tracked separately from GUI timer and
OnUpdate settling. Quit from PLAYER_LOGIN or an initial timer can finalize
without waiting for that settling budget. Closing before essential initialization
skips registered output and reports an initialization error; a process with no
registered output can close normally. Actual unowned/shared initialization
failures still protect the incomplete environment.

Ordinary event delivery finishes other listeners after a callback error. A failed
shutdown or save attempt returns an error and leaves an intact environment
retryable after repair. Successful finalization still runs only once.

The GUI records `[wow-sim] Session finalization succeeded` or
`[wow-sim] Session finalization failed: ...` and returns its result through
`run_iced_ui`. The executable returns a nonzero status on failure. An unexpected
runtime exit without finalization also returns an error. No destructor writes or
retries persistence. The existing Lua IPC can request `Quit()`; its command reply
acknowledges execution, while final completion is established by the process result.
Both IPC queues stop when Quit is requested, including between inspector and Lua
commands. The active Lua call finishes; later independent update stages do not
run. Composite mouse clicks skip remaining stages after an exit request and
still return their command response. App update guards and the runner share
one completion result.

## Profile events and reload

Forever uses the researched, unmeasured sequence of conditional argless
`PLAYER_LEAVING_WORLD`, argless `PLAYER_LOGOUT`, then
`ADDONS_UNLOADING(closingClient)`. The boolean is true for finalization and false
for ReloadUI. Wrath retains its existing authenticated shutdown sequence and
protected-action window. Other profiles retain their existing event policy;
this change does not infer their native shutdown chronology. See
[native evidence](saved-variables-native-evidence.md).

ReloadUI still saves before retiring the VM, reconstructs through the original
launch loader and reads simulator-owned saves instead of stale imported WTF.
Player name and class use the existing preservation and startup paths. Reload
is not session completion. Each explicit request is consumed
once, so a failed attempt does not retry on every GUI tick.

After repair, an intact environment can retry a failed save or ReloadUI. Failed
declared inputs are reread from their recorded source with the existing size and
source rules; disappearance remains an error rather than enabling stale fallback.
Healthy loaded scopes keep their current edits. Optional unregistered imports do
not participate in this output validation.

Recovery reload saves healthy registered owners while excluding uninitialized
owners and owners whose shutdown callbacks failed. An unowned shutdown failure
skips all writes for that reload. The new VM then loads through the existing
startup path. Lua and finalized runtime-loader diagnostic collections survive
reconstruction; host errors are returned to callers and kept in the GUI log.
Current owner failures clear only on successful initialization or replacement of
the environment.

A structural rebuild failure leaves an incomplete VM that cannot save or run
ordinary GUI callbacks. Another explicit ReloadUI discards that VM without saving
it and reconstructs from valid inputs and prior storage. Exact failed-input source
information survives this recovery. The GUI keeps recovery controls available
and reports the failure in its log rather than closing automatically. These
recovery safeguards are simulator policy; native shutdown event order is unchanged.

Wrath recovery restores a previous placed-frame layout when one exists. If that
recovery skipped geometry saving and no prior layout exists, restoration is a
no-op. Normal reload still requires the layout it just wrote, and existing invalid
or unreadable layout files retain their errors.

## Storage and process results

Saving uses the existing `save_all`/`save_addon` operations and the `SavedVariables`
directory beneath an explicit writable `--session-wtf-path DIR` root.
Without that option, a non-empty `WOW_SIM_STORAGE_DIR` selects the SavedVariables
directory directly; otherwise it is `SavedVariables` beneath the platform
local-data directory's `wow-sim` root. Other stores continue to use the session
root. See [persistence destinations](../runtime-inputs.md#persistence-destinations).
ReloadUI saves to and reloads from the same storage. Real WoW WTF imports remain
read-only, and disabled SavedVariables persistence produces no addon saves.
Ordinary launches load configured WTF imports before simulator storage. An
explicit `--session-wtf-path` restores saved session scopes first and imports
only missing scopes; invalid selected input remains an error.

A normal interactive exit with status 0 follows successful finalization. A
finalization error returns a nonzero status. Abrupt termination does not establish
a completed save. Diagnostic command exit statuses describe those commands and
do not establish interactive finalization.

Writes are per file rather than atomic across all addons. A failed save may leave
partial files or files written by an earlier reload; finalization does not roll
back storage. Temporary world-departure/protected-action state is restored when
an attempt fails before reconstruction, without undoing addon callback mutations.
Regression fixtures cover serializer byte preservation and fresh-VM read-back.

## Verification

Native Windows verification on 2026-11-02 used Cargo/rustc 1.99.1, the committed
locked dependencies, installed static ICU, and `sound,gui,casc` with the indicated
client profile. The focused cases check returned errors, disk contents and Lua
read-back, not only process status:

| Cases | Forever | Wrath |
| --- | ---: | ---: |
| Lifecycle and recovery | 15 | 19 |
| SavedVariables input selection and preservation | 17 | 17 |
| GUI completion and recovery | 11 | 11 |
| Runtime addon failures | 5 | 5 |
| IPC ordering and exit boundaries | 9 | 9 |
| VM lifetime | 1 | 1 |
| Startup addon failure/recovery | 6 | 6 |
| Persistence integration | 3 | 3 |
| Total passing cases | 67 | 71 |

The default Retail library and GUI binary also pass `cargo check --locked
--offline --lib --bin wow-sim`. The builds retain existing profile-specific unused
imports; no zero-warning or Retail runtime result is claimed.

The failed-owner startup fixture observes restored values at `ADDON_LOADED`,
preserving Wrath's existing code-before-SavedVariables order. The six compiled
startup identity tests include distinct writer/reader processes for explicit
session restoration and isolated persistence-enabled/disabled failure cases.
Those full-startup identity cases remain unexecuted here: the installed Forever
cache is 1.60.1.70634 against the 1.60.1.70532 pin, and the Wrath runtime cache is
absent. They are separate from the passing temporary-addon cases above.

Two bounded real Forever GUI processes used the explicit UI-version-mismatch
option. With persistence disabled, `Quit()` returned exit 0, logged successful
finalization and created no SavedVariables. With persistence enabled, it returned
exit 1 and created no SavedVariables because the selected
`Blizzard_SettingsDefinitions_Shared` TOC references a missing `Preset.lua`.
That newer-cache topology is absent from the committed manifest; loader, profile
selection and manifest behavior are unchanged. See the
[source topology limitation](../updating-blizzard-ui-to-a-new-patch.md).
Successful enabled-persistence GUI save/relaunch remains unverified.

Both GUI runs retained the original persisted CVar bytes, size and timestamps,
and the sealed binary and cache provenance/completion hashes. Changed Rust files
pass a focused `rustfmt --check`; the workspace-wide formatter aborted with an
allocation error. Full-suite, seven-profile and measured native chronology
coverage are not claimed.

## Sources

- [Lifecycle](../../src/lua_api/env_lifecycle.rs) — callback/save boundary and retained outcome.
- [GUI owner](../../src/iced_app/app.rs) — shared completion result.
- [GUI runner](../../src/iced_app/mod.rs) — result propagation to the executable.
- [Persistence](saved-variables.md) and [ReloadUI](reload-ui.md) — reused storage and reconstruction.
