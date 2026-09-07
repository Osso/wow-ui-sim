# PTR bootstrap ordering probe

Existing A/B/C/D suite targets **12.1.5.69594 / 120105**. A/C/D are eager; B is LoadOnDemand. B and D list `Before.lua`, `Bootstrap.lua [Bootstrap]`, then `Normal.lua`. B's bootstrap initializes its own recorder if A has not run. Collector initialization is duplicated only in A and B so neither requires the other's execution; retire that duplication only if the client ordering contract makes one initializer sufficient.

## Captured bootstrap lifecycle

The completed September 7, 2026 PTR capture is stored ignored at `docs/local/private/probes/BootstrapOrderProbe-2026-09-07-durable.lua`; its 8,004-byte SavedVariables Lua SHA-256 is `5f21fe45373f1d2fbe3a645c7dd7f66b3c740925158dbc2f985039008c2af889`. All 13 records identify `12.1.5` / `69594` / `120105`, match the expected build, contain no errors, and mark the capture `complete=true`.

Startup records `A:eager → B:bootstrap → C:eager → D:before → D:bootstrap → D:after → PLAYER_LOGIN`. B's bootstrap runs once at startup with `loaded=true, finished=false`, then B returns to `false,false`. Eager D preserves literal `Before.lua → Bootstrap.lua [Bootstrap] → Normal.lua`; this fixture does not support a global bootstrap pre-pass before A.

The first explicit B load records `load:before → B:before → B:after → load:after`: the bootstrap does not run again. B is `true,false` during normal files and `true,true` after the load. The second load records only `load:before → load:after`, with successful results, and executes no B files. This establishes the probe's LoD lifecycle; it does not establish simulator loader eligibility, dependency, SavedVariables, or private-environment policy.

Only A declares `BootstrapOrderProbeDB`. Each Lua session has its own working collector. At `ADDON_LOADED`, restored load results are retained; startup-only data is replaced with the new collector. Each explicit `load:after` publishes after its return/error payload is attached, without relying on the saved table still aliasing the collector.

Two completed load attempts mark a capture `complete`, including attempts returning errors. A newer single-attempt capture is saved as `pendingCapture` beside the previous completed capture; the second attempt replaces it with the new completed capture. A partial capture is retained across reloads when no completed capture exists. Results from different sessions are never merged.

Regression tests reproduce completed-capture loss across fresh-VM save/restore/reload cycles and detached-table publication. These design flaws do not establish the cause of the original unchanged desktop file.

Ordered event tags identify each source file, counts identify bootstrap re-execution, and `load:before`/`load:after` bracket each explicit B load with return/error data. Every record captures build identity, B/ClickBinding/Collections loaded+finished flags, and helper types. Reject captures where `matchesExpectedBuild` is false. Passive addon-query failures are recorded under `errors`.

The suite loads **only its own B addon** on explicit slash commands. No Blizzard addon loads, existing frame changes, CVars, or EditMode mutations. The recorder uses one anonymous event frame. Personal-addon absence is user-supplied context, not an addon inventory.

`tests/pixel_rounding_probe.rs` contains grouped protocol tests: SavedVariables restoration, B bootstrap preceding A, mixed file order recording, explicit/repeated B loads, build mismatch and load errors. Tests simulate loader scheduling to validate the recorder; they do not establish native client ordering.
