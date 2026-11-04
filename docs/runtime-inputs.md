# Supplied runtime inputs and isolated persistence

The supplied session directory, or the platform default when omitted, is resolved
once before startup changes the working directory to the executable directory.
Relative session paths use the launch directory. Use absolute paths for other
external resource inputs. Real WoW WTF inputs are read-only; session state goes
to a separate writable simulator WTF directory.

## Inputs

| Input | Purpose |
| --- | --- |
| `--addons-path DIR` | Exclusive third-party addon root; repeat for multiple roots |
| `--session-wtf-path DIR` | Writable directory for simulator session state |
| `WOW_SIM_ADDONS_TXT` | Optional local addon-enable file, overriding session `AddOns.txt` for reads and saves |
| `WOW_SIM_WTF_PATH` | Read-only real WoW WTF import directory |
| `WOW_SIM_WTF_ACCOUNT` | Selected account |
| `WOW_SIM_WTF_REALM` | Selected realm |
| `WOW_SIM_WTF_CHARACTER` | Exact imported character; enables addon-visible identity seeding |
| `WOW_INSTALL_PATH` | Installation used for game content |
| `WOW_PRODUCT` | CASC product, subject to the compiled client profile |
| `WOW_SIM_NO_ERROR_DIALOG` | Suppress fatal-error dialogs when present |

`--class`, `--race` and `--spec` seed the initial model before addons load, then
are reapplied after startup; see the [selection contract](specs/cli-class-spec.md).
Reload retains the current class, race and specialization, including later GUI
or scripted changes.

A nonempty character selection supplies name and realm before addons load and is
reapplied during ReloadUI. Forever separates `First-Surname`; other profiles keep
the character component intact. Missing explicit root/account/realm/character
directories fail startup instead of selecting another identity. Omitted
account/realm values retain discovery. `--no-saved-vars` disables persistence while
still allowing identity from the read-only source. See the
[identity contract](specs/character-identity.md) for query behavior and test limits.

## Windows installation discovery

Explicit `WOW_INSTALL_PATH` and `WOW_DATA_PATH` take precedence. Next, the
resolver reads `ProgramData/Battle.net/Agent/product.db` on Windows drives and
selects a `wow` family entry for the active `WOW_PRODUCT`. Its recorded root
must contain `Data/data` and the recorded client flavor directory. Existing
common installation paths are checked last.

The install root's `.product.db` can contain a wrapped database or a bare product
record. A missing-product error lists the installed product codes. For example,
`found: wowf` means that a Retail executable cannot use that Forever-only install;
use the matching client build.

## Persistence destinations

`--session-wtf-path DIR` selects one writable directory for session state. When
omitted, the root remains the platform local-data directory's `wow-sim` folder.
The session root itself has no environment-variable fallback. Both the supplied
and default directory use the same runtime constructor before CVars, TTS and other
stores initialize.
Individual stores derive their paths from this root:

| Relative path | Contents |
| --- | --- |
| `SavedVariables/<addon>.lua` | Account SavedVariables |
| `SavedVariables/<realm>/<character>/<addon>.lua` | Character SavedVariables |
| `config.json` | Simulator preferences |
| `cvars.json` | Persisted CVars |
| `cvars/<profile>/SavedVariables/` | Private simulator models, including TTS preferences |
| `AddOns.txt` | Local addon enablement unless `WOW_SIM_ADDONS_TXT` is set |

For SavedVariables only, an explicit `--session-wtf-path` takes precedence.
When the flag is absent, a non-empty `WOW_SIM_STORAGE_DIR` selects the
SavedVariables directory directly. Otherwise, the default path above applies.
Other stores continue to use the session root.

`WOW_SIM_ADDONS_TXT` can select a separate local addon-enable file without
switching the rest of the session state. When set, it takes precedence over
`<session-root>/AddOns.txt` for both reads and saves. Other stores continue to use
the resolved session root.

The root is reused through ReloadUI and finalization. Existing store formats and
relative layouts are retained; this setting does not introduce native `Config.wtf`
serialization. `WOW_SIM_WTF_PATH` remains a separate read-only import source, with
the selected account/realm/character determining imported scopes. The writable
root must be separate from the real WoW WTF tree.

An explicit `--session-wtf-path` restores existing simulator SavedVariables before
considering the WTF import, independently for each addon's account and character
scope. A missing saved scope imports its original WTF file; an invalid saved file
reports a load error instead of silently resetting it from the import. Reusing
the same session directory therefore keeps edits on later launches. Omitting the
option retains the ordinary import-first behavior. ReloadUI continues to restore
the files it just saved.

```powershell
$env:WOW_SIM_WTF_PATH = 'C:\Games\World of Warcraft\_retail_\WTF'
& '.\target\debug\wow-sim.exe' --session-wtf-path 'C:\Sessions\Example\WTF'
```

The [persistence regression tests](../tests/runtime_persistence_paths.rs) check
stored JSON and Lua values, account/character reload and finalization, TTS models,
Wrath layouts, explicit-session precedence over legacy output overrides, and
unchanged import bytes. The [storage precedence test](../tests/storage_env_precedence.rs)
covers `WOW_SIM_STORAGE_DIR` when the flag is absent, explicit-flag precedence,
and the default destination.
Current session-finalization verification and its source-cache limits are recorded
in the [session contract](specs/session-finalization.md#verification).

Static source/asset caches remain shared, and missing content may require network
access. See the [source setup workflow](updating-blizzard-ui-to-a-new-patch.md).

The [SavedVariables contract](specs/saved-variables.md) describes scope loading,
serialization and load-error handling. GUI close and Quit use the fallible
[session finalizer](specs/session-finalization.md); reload saves through `save_all`
and reconstructs the UI without completing the session.

`--session-wtf-path` selects output storage for both finalization and ReloadUI.
A normal successful interactive exit follows finalization; failure or abrupt
termination does not establish a completed save. A failed ReloadUI attempt keeps
recovery controls available; after repair another explicit request can retry.
An incomplete reconstructed VM cannot write SavedVariables. See
[storage and process results](specs/session-finalization.md#storage-and-process-results).

## Executable selection and diagnostics

The compiled `client-*` feature fixes the profile. `WOW_PRODUCT=wowf` cannot
turn a Retail binary into Forever, and package `--version` does not identify it.

Use `lua-errors`, `dump-tree`, `screenshot` or `--exec-lua` to inspect results;
see the [CLI reference](wiki/reference/cli-commands.md). A zero exit does not
prove correct rendering or zero Lua errors. Screenshots and interactive viewing
require a GUI build.

For build instructions, see the [native development guide](../README.md#native-development).
