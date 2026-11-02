# Explicit ping cooldown interval

`C_Ping.GetCooldownInfo()` projects the ping manager's stored cooldown pair from `src/c_api/c_ping_cooldown.rs`. WM51-F1 identifies it as the same pair that a Retail 12.1.5 send stores when it crosses the burst limit and clears at expiry; see [secure ping sends](ping-secure-publication.md#sends-and-pending-hit-test-information). Hosts can still supply the pair directly. Getter and utility-table registration are implemented; bounded Retail GREEN passes. Tests target `client-retail`/`client-ptr`; backing state uses their shared `retail-12-0-0` feature floor.

Accepted draft cached Retail evidence: `Blizzard_APIDocumentationGenerated/PingManagerDocumentation.lua` declares no arguments and nonnil `PingCooldownInfo`, without secret/restriction annotations on this getter. `PingManagerSharedDocumentation.lua` requires numeric nonnil `startTimeMs`/`endTimeMs`. `Blizzard_PingUI/Blizzard_PingUI.lua:127–141,194–205` compares `GetTime()*1000 < endTimeMs` and requests `(endTimeMs/1000)-GetTime()` seconds. Sources are under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns`; documentation and consumer lines were locally re-read on 2026-10-12, not freshly fetched or native-observed. Fresh DTO allocation and inactive `(0,0)` are accepted simulator assumptions; PTR/native parity is unverified.

## What it must do

- [x] Return exactly one fresh nonnil table with numeric `startTimeMs` and `endTimeMs`, projecting explicit values unchanged as absolute milliseconds, not seconds/elapsed/remaining duration.
- [x] Default each environment to `(0,0)`; explicit replacement/reset changes the next read while older DTOs remain snapshots.
- [x] Caller mutation cannot alter other DTOs, future reads or backing state; simultaneous environments remain independent.
- [x] Allow genuinely tainted zero-argument calls; no secret-argument policy from adjacent ping operations may be imported.
- [x] Actual cached vendor OnEnter selects normal cursor/no timer for zero state at clock 12, error cursor/3.5-second request for `(12000,15500)`, normal cursor/cleared listener after direct callback, and no new timer at equality clock 15.5. Getter state remains unchanged.

## How it works

- [Lua API overview](../lua-api.md).
- [Runtime architecture](../wiki/systems/lua-api.md).

## Implementation inventory

- `src/c_api/c_ping_secure/burst.rs`: public `PingCooldownInterval { start_time_ms: u32, end_time_ms: u32 }` in cached milliseconds, held as `PingBurst.interval` in `SimState.ping_burst`; zero default.
- `src/c_api/c_ping_cooldown.rs`: `pub(crate) register(state)` preserves the existing `C_Ping` namespace and installs the exact Rust `GetCooldownInfo` function. Getter copies the pair, drops the state borrow before allocation, roots a fresh result table before setting its two fields, and returns one DTO. No argument or security checks.
- `src/c_api/mod.rs`: model re-export and getter registration alongside existing ping utility tables under the shared modern feature floor.

## Tests asserting this spec

Current bounded proof: **6/6 PASS**, zero selected failures, fixture failures, skips or timeouts. Corrected active-cache Ping consumer reaches actual assertions; callback delivery is a direct spy, not scheduler proof. [Integrated current proof](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#five-producer-and-receiver-source-checkpoint--2026-10-12) owns aggregate gates and retention status.

[tests/ping_cooldown_interval.rs](../../tests/ping_cooldown_interval.rs): six cases for default/tainted reads, supplied milliseconds, fresh DTO/mutation, live replacement/reset, environment isolation, and bounded actual vendor consumer. Main reports Genuine76 RED: five public-getter behavioral failures plus one known consumer-fixture cache-path failure; the fixture failure is not getter RED. Main corrected the source-cache fixture path at `346590852`. No tests were run for this scoped implementation; Bounded Retail GREEN passes.

Vendor test reads the active profile cache without fallback and executes the unchanged Lua definition file, not XML/startup. Only clock/cursor/timer sinks are controlled; getter and vendor methods remain real. Direct callback delivery proves requested delay and closure effects, not real scheduler behavior. No tooltip operation exists on the chosen path. Active PTR cache routing is an integration adaptation, not PTR proof.

## Known gaps (current cycle)

- [x] Main reports five genuine public-getter RED failures; the separate consumer cache-path failure is not behavioral RED.
- [x] Implement the getter projection and namespace-local registration after reported RED; no interval producer exists or is added.
- [x] Current bounded Retail GREEN: 6/6 tests, exit0.
- [ ] Native default/fresh allocation, output security and PTR consumer equivalence remain unverified.

## Out of scope

Charge/refill generation, timer policy, automatic expiry/reset, events/callback dispatch, secret arguments, malformed interval validation, extra fields, full startup, other profiles, vendor/config/WTF edits.
