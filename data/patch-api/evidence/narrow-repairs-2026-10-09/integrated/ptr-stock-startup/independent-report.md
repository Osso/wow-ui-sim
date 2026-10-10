# Independent PTR stock startup receipt gate

**PASS — bounded completed startup capture. Errorless/full-profile/native-client acceptance NOT established.**

Evidence directory: `20261010T184101Z/`. Read-only receipt/source inspection; no build, test, check, simulator rerun, operational action, or delegation. Receipts completed within the permitted wait (one bounded wait, maximum 150 seconds). Only this requested report was written. Runtime/vendor/character payloads omitted.

## Source, artifact, profile and toolchain

- `submission.json`: revision `b048251f38b2d53e33b3365590cd227d40fc07d0`; `/usr/bin/cargo build --bin wow-sim --no-default-features --features sound,gui,casc,client-ptr --offline --locked --message-format=json --timings -j 12`. Normal dev build, optimized + debuginfo; compiler artifact `fresh=false` means compiled, not a globally clean build.
- Independently compared `source-before.json` and `source-after.json`: **3,853 entries each, identical**; current covered files also match. Includes `data/blizzard-ui-files/ptr.txt`. Scope excludes external/path dependencies, untracked inputs and inherited environment; recorded revision is not complete source provenance.
- Exactly one executable `wow-sim` compiler-artifact record; executable/features match `artifact.json`. Effective features include `client-ptr`, `retail-12-1-5`, `gui`, `casc`, `sound`, `rodio`; no `default`, `client-retail`, `profile-retail` or other client selector. `src/client_profile.rs:169-174` selects `ClientProfile::Ptr`; `Cargo.toml:152` maps PTR to the 12.1.5 epoch.
- Independently hashed sealed executable: SHA-256 `68a834b307bbd1a31af8e84880d8eecee02aeecf4fada526a93583eb6a72ff34`, matches `artifact.json`; runtime receipt reports artifact unchanged. This binds the sealed executable and receipt, not every external dependency/build environment input.
- `toolchain/cargo.json`: exit **0**, Cargo **1.99.0**, commit `5f94df478`, 2026-08-27. `toolchain/rustc.json`: exit **0**, rustc **1.99.0**, commit `b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, 2026-09-28; host `x86_64-unknown-linux-gnu`, LLVM **23.1.1**, Arch package `1:1.99.0-1`. Version-command receipts, not proof against inherited compiler overrides.

## Cache boundary

`cache-before.json` / `cache-after.json` cover `/home/osso/.cache/wow-ui-sim/blizzard-ui/ptr/AddOns`: **4,027 entries each, identical**. Independently rehashed current files: match after receipt; current file set also matches. Worker after-snapshot checks the original file list, so its equality alone does not prove no transient additions. Current inventory supplies only the present-time addition check.

`src/blizzard_ui_sync.rs:41-61` derives the profile cache from `dirs::cache_dir()` and active profile, checks completion marker and PTR identity; `src/client_profile.rs:307-322` uses the cached/default profile directory. Logs do not contain the exact absolute PTR cache path. Inherited cache-directory environment was not sealed; snapshots establish this specified cache's stability, not independently observed open-file provenance for every runtime input.

## Completed capture and diagnostics

- `compile-result.json`: build exit **0**, source equal, **0** stream-drain errors. Full compiler JSON stream inspected: **0 compiler-message diagnostics**. `compile.stderr`: **6 individual manifest deprecation warnings**, plus **1 aggregate warning-summary line**; not seven distinct warnings. Build finished in **2m 19s**.
- `runtime-submission.json`: executes the sealed binary through `timeout 90`, with `--no-addons --no-saved-vars`, stock `dump-tree`, and completion-marker Lua. Sound feature is compiled in but runtime audio disabled by `WOW_SIM_NO_SOUND=1`; actual log contains `Sound disabled`. `WOW_SIM_LOG_HANDLER_TIMINGS` explicitly removed.
- `runtime-result.json` / `outcome.json`: runtime exit **0**, completion marker **true**, status **CAPTURED**. Independently found **1** `[PtrStockStartupComplete]` marker in stderr, **0** in stdout. Marker proves the post-settle exec-Lua point was reached; exit receipt proves command returned. It is not an assertion about every later callback succeeding.
- Entire startup stdout (**1,082 bytes, 16 lines**) and stderr (**17,952 bytes, 284 lines**) inspected. **0 emitted `Lua error:` records**, **0 suppression-summary records**, **0 warning/WARN diagnostic lines**, **0 error diagnostic lines**; no failed/missing/unable/panic or warning-symbol indications. These are captured-output counts, not an exact in-memory Lua-error-state count. This dump command emits no final `Lua errors: N unique, M occurrence(s)` state summary. `src/lua_api/script_helpers.rs:568-614` separates state recording from emission, including never/first-only policies; therefore total unique/occurrence state counts remain **unproven**, not zero.
- File-budget success/error and handler-budget-error records: **0 emitted each**, independently confirmed. Timing opt-in was off; file diagnostics are epoch-scoped. These zeros are **not evidence of zero Lua errors** or zero quota consumption.

## Limits

Third-party addon and SavedVariables loading disabled by recorded argv; this does not seal all addon files or prohibit WTF-backed EditMode cache use. `src/bin/wow_sim/startup.rs:236-243,281-286` configures/loads the separate EditMode cache when ordinary SavedVariables are absent. No claim of cold startup, GUI rendering, native WoW parity, all-profile compatibility, zero stored Lua errors, or warning-free build. Exit 0 is successful capture, not errorlessness.
