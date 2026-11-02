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
| `WOW_SIM_WTF_CHARACTER` | Selected character for imported scopes |
| `WOW_INSTALL_PATH` | Installation used for game content |
| `WOW_PRODUCT` | CASC product, subject to the compiled client profile |
| `WOW_SIM_NO_ERROR_DIALOG` | Suppress fatal-error dialogs when present |

Existing class/race/spec inputs are described in the [selection contract](specs/cli-class-spec.md).

## Persistence destinations

`--session-wtf-path DIR` selects one writable directory for session state. When
omitted, the root remains the platform local-data directory's `wow-sim` folder.
The option has no environment-variable fallback. Both the supplied and default
directory use the same explicit runtime constructor before CVars, TTS and other
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

`WOW_SIM_ADDONS_TXT` can select a separate local addon-enable file without
switching the rest of the session state. When set, it takes precedence over
`<session-root>/AddOns.txt` for both reads and saves. Other stores continue to use
the resolved session root.

The root is reused through the existing ReloadUI path. Existing store formats and
relative layouts are retained; this setting does not introduce native `Config.wtf`
serialization. `WOW_SIM_WTF_PATH` remains a separate read-only import source, with
the selected account/realm/character determining imported scopes. The writable
root must be separate from the real WoW WTF tree.

```powershell
$env:WOW_SIM_WTF_PATH = 'C:\Games\World of Warcraft\_retail_\WTF'
& '.\target\debug\wow-sim.exe' --session-wtf-path 'C:\Sessions\Example\WTF'
```

The [persistence regression tests](../tests/runtime_persistence_paths.rs) cover
isolated stores, save and reload, separate addon selection, and unchanged
imported data.

Static source/asset caches remain shared, and missing content may require network
access. See the [source setup workflow](updating-blizzard-ui-to-a-new-patch.md).

The [SavedVariables contract](specs/saved-variables.md) describes scope loading,
serialization and load-error handling. Existing shutdown behavior is unchanged.

## Executable selection and diagnostics

The compiled `client-*` feature fixes the profile. `WOW_PRODUCT=wowf` cannot
turn a Retail binary into Forever, and package `--version` does not identify it.

Use `lua-errors`, `dump-tree`, `screenshot` or `--exec-lua` to inspect results;
see the [CLI reference](wiki/reference/cli-commands.md). A zero exit does not
prove correct rendering or zero Lua errors. Screenshots and interactive viewing
require a GUI build.

For build instructions, see the [native development guide](../README.md#native-development).
