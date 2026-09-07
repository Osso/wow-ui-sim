# PTR bootstrap ordering probe

Existing A/B/C/D suite targets **12.1.5.69594 / 120105**. A/C/D are eager; B is LoadOnDemand. B and D list `Before.lua`, `Bootstrap.lua [Bootstrap]`, then `Normal.lua`. B's bootstrap initializes its own recorder if A has not run. Collector initialization is duplicated only in A and B so neither requires the other's execution; retire that duplication only if the client ordering contract makes one initializer sufficient.

## Captured startup result

The September 7, 2026 PTR capture is stored ignored at `docs/local/private/probes/BootstrapOrderProbe-2026-09-07.lua`; raw SavedVariables SHA-256 `7f8fe830a9937bb7d4b1989b3097661cf3c8492bd11607f2cc8d3a3b1a8d34b1` matched the desktop copy. All nine records identify `12.1.5` / `69594` / `120105` and have no errors.

It records `A:eager → B:bootstrap → C:eager → D:before → D:bootstrap → D:after → PLAYER_LOGIN → snapshot → snapshot`. Thus B's LoD bootstrap runs at startup, but B returns to `loaded=false, finished=false`; eager D preserves the literal file order around its `[Bootstrap]` entry. It does not support a global pre-pass before A.

The capture lacks the requested `/boprobe load` calls, so B explicit-load ordering and bootstrap re-execution remain unknown.

## Capture explicit B loading

1. Enable **all four** probe addons in PTR, keeping personal addons disabled; `/reload`.
2. Run `/boprobe load` twice.
3. `/reload` to flush, then retrieve `_xptr_/WTF/Account/<ACCOUNT>/SavedVariables/BootstrapOrderProbe_A.lua`. Later reloads without commands preserve the capture.

Only A declares `BootstrapOrderProbeDB`. Each Lua session has its own working collector. At `ADDON_LOADED`, restored load results are retained; startup-only data is replaced with the new collector. Each explicit `load:after` publishes after its return/error payload is attached, without relying on the saved table still aliasing the collector.

Two completed load attempts mark a capture `complete`, including attempts returning errors. A newer single-attempt capture is saved as `pendingCapture` beside the previous completed capture; the second attempt replaces it with the new completed capture. A partial capture is retained across reloads when no completed capture exists. Results from different sessions are never merged.

Regression tests reproduce completed-capture loss across fresh-VM save/restore/reload cycles and detached-table publication. These design flaws do not establish the cause of the original unchanged desktop file.

Ordered event tags identify each source file, counts identify bootstrap re-execution, and `load:before`/`load:after` bracket each explicit B load with return/error data. Every record captures build identity, B/ClickBinding/Collections loaded+finished flags, and helper types. Reject captures where `matchesExpectedBuild` is false. Passive addon-query failures are recorded under `errors`.

The suite loads **only its own B addon** on explicit slash commands. No Blizzard addon loads, existing frame changes, CVars, or EditMode mutations. The recorder uses one anonymous event frame. Personal-addon absence is user-supplied context, not an addon inventory.

`tests/pixel_rounding_probe.rs` contains grouped protocol tests: SavedVariables restoration, B bootstrap preceding A, mixed file order recording, explicit/repeated B loads, build mismatch and load errors. Tests simulate loader scheduling to validate the recorder; they do not establish native client ordering.
