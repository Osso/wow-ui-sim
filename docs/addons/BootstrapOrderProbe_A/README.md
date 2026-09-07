# PTR bootstrap ordering probe

Existing A/B/C/D suite targets **12.1.5.69594 / 120105**. A/C/D are eager; B is LoadOnDemand. B and D list `Before.lua`, `Bootstrap.lua [Bootstrap]`, then `Normal.lua`. B's bootstrap initializes its own recorder if A has not run. Collector initialization is duplicated only in A and B so neither requires the other's execution; retire that duplication only if the client ordering contract makes one initializer sufficient.

## Capture

1. Stage all four directories with `python3 docs/addons/BootstrapOrderProbe_A/deploy.sh`.
2. Enable **all four** probe addons in PTR, keeping personal addons disabled; `/reload`.
3. Run `/boprobe`, then `/boprobe load` twice.
4. `/reload` to flush, then retrieve `_xptr_/WTF/Account/<ACCOUNT>/SavedVariables/BootstrapOrderProbe_A.lua` before another reload/logout overwrites the saved capture.

Only A declares `BootstrapOrderProbeDB`. An unsaved working table collects early events; A publishes it at its `ADDON_LOADED`, after SavedVariables restoration. Each new Lua session starts a fresh working table. No old-session records are merged. The final reload starts a new in-memory session but writes the completed session to disk first.

Ordered event tags identify each source file, counts identify bootstrap re-execution, and `load:before`/`load:after` bracket each explicit B load with return/error data. Every record captures build identity, B/ClickBinding/Collections loaded+finished flags, and helper types. Reject captures where `matchesExpectedBuild` is false. Passive addon-query failures are recorded under `errors`.

The suite loads **only its own B addon** on explicit slash commands. No Blizzard addon loads, existing frame changes, CVars, or EditMode mutations. The recorder uses one anonymous event frame. Personal-addon absence is user-supplied context, not an addon inventory.

`tests/pixel_rounding_probe.rs` contains grouped protocol tests: SavedVariables restoration, B bootstrap preceding A, mixed file order recording, explicit/repeated B loads, build mismatch and load errors. Tests simulate loader scheduling to validate the recorder; they do not establish native client ordering.
