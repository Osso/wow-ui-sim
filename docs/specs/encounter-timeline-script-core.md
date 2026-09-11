# EncounterTimeline script-event core

PTR `C_EncounterTimeline` owns script events rather than the former Flash of Light demo. The source boundary is Gethe `a89e9d0ceb7f6cd31e8fc5ca7df1a338ac0b1b58` → `49b69918fcdc77e109813281e4f537d45ec7dcbf`, `EncounterTimelineDocumentation.lua` and `EncounterTimelineConstantsDocumentation.lua`. Changed request/info fields are also recorded in the [pinned register](../../data/patch-api/sources/12.1.5-register.json).

## What it must do

- [x] `AddScriptEvent` returns nonzero IDs unique within an environment; IDs are not reused after removal. Validation completes before state changes.
- [x] Preserve all request fields. Default `maxQueueDuration=0`, `overrideName=""`, omitted icons to zero, severity Medium, paused false. Return all ten info fields in independent tables. Nonempty override names are byte-preserved; otherwise use the existing spell catalogue, with empty name for unknown spells (simulator policy).
- [x] Support cancel, finish, pause, resume, cancel-all, info/state/list/count, current time, elapsed/remaining, timer, active/any/paused queries, and nonblocked script events.
- [x] Advance the event clock through the normal simulator OnUpdate tick, not a test-only clock or mutable Lua field. Pause freezes elapsed time; resume continues remaining time.
- [x] Commit state before dispatching added/state/removed notifications. Callbacks can query or mutate events without holding a SimState borrow. Cancel-all operates on its original ID set, so events created by callbacks survive.
- [x] Retain terminal data until at least one subsequent game tick has run its OnUpdate/OnPostUpdate callbacks. Remove before notifying `ENCOUNTER_TIMELINE_EVENT_REMOVED`; ID queries then return no result.
- [x] Return ordinary LuaDurationObject wrappers sharing an event-owned clock. Existing wrappers follow pauses/resumes and freeze after termination/removal; changing one wrapper's duration fields does not mutate events or another wrapper.
- [x] Earlier retail retains its actual seeded demo and 12.5-second placeholder timer. PTR excludes that demo entirely.

## Modeled choices and boundaries

The timeline's clock starts at zero and accumulates the real runtime OnUpdate delta, matching animation simulation rather than the wall-clock `GetTime` epoch. Duration wrappers track `[0, duration]` using private read-only provider clocks; normal duration clock lookup observes provider metamethods. Finish forces elapsed to duration; cancellation freezes current elapsed. Invalid argument shapes error; unknown IDs return no values. Repeated or terminal transitions are no-ops. Added and state notifications dispatch synchronously in deterministic ID order; exact native uniqueness/coalescing is not claimed.

Automatic completion occurs when active elapsed reaches duration at tick start. Terminal events remain in lists/counts until post-update removal on a later tick. Script events are never blocked or approximate. Feature available/enabled are true in this core model, not modeled user settings. Request IDs/icon masks require nonnegative `u32` integers; durations require finite nonnegative numbers and severity is Low/Medium/High. These input policies are not native coercion evidence.

`maxQueueDuration` is preserved as request/info metadata only. No queued hold, track placement, sorted filtering, highlighting, EditMode, or view/layout APIs are implemented by this slice. Deferred track/filter/EditMode/view methods are explicitly withheld from the generic missing-method fallback. Full Blizzard timeline-view integration requires the parent-owned track slice. Timer elapsed/remaining consumers can operate independently of those tracks. No security, secret-value, taint, or native timing equivalence is claimed.

## How it works

- [Event dispatch](../event-system.md)
- [Duration core](duration-core.md)

## Implementation inventory

- `src/c_api/c_encounter_timeline.rs`: registration, public mutations, event dispatch and tick boundaries.
- `src/c_api/c_encounter_timeline/`: owned event state, input validation, query snapshots and duration clock adapters.
- `src/lua_api/on_update.rs`: runtime lifecycle progression/removal hooks.
- `src/lua_api/globals/lua_duration_object/core.rs`: clock-provider lookup shared with manual clocks.
- `src/lua_api/workarounds/temporary/encounter_state.rs`: earlier-profile demo only; encounter customization remains separate.

## Tests asserting this spec

- `tests/encounter_timeline_script.rs`: public API lifecycle, callbacks, clock progression, retained timers, validation, isolation and earlier-retail baseline.
- `tests/duration_core.rs`: existing duration/manual-clock behavior.
- `tests/system_api_seeded.rs`: unchanged earlier-profile seeded timeline expectations; PTR uses the new lifecycle tests.
- `startup_globals::test_patch_12_0_7_safe_global_bridges`: existing encounter color/customization consumer.

### Proof and candidate audit rows

No artifact statuses are changed by this slice. Nine changed occurrences have bounded candidate proof: `GetCurrentTime`, `GetEventTimeElapsed`, `GetEventTimeRemaining`; `EncounterTimelineEventInfo` and its `duration`/`maxQueueDuration`; `EncounterTimelineScriptEventRequest` and its `duration`/`maxQueueDuration`. Queue-duration credit is field preservation only, not queued-hold behavior. The producer APIs themselves are unchanged between the pinned revisions and supply behavioral evidence for those structures. Earlier-retail tests preserve the existing demo, not native base conformance.

Commands (each profile run separately):

```text
cargo test --test integration --offline --no-default-features --features sound,gui,client-<profile> -- encounter_timeline_script:: duration_core:: system_api_seeded::test_c_encounter_timeline --nocapture
cargo test --lib --offline --no-default-features --features sound,gui,client-<profile> -- test_patch_12_0_7_safe_global_bridges installs_timeline_and_event_customization_state --nocapture
```

The PTR script tests load the existing Blizzard event-frame/settings files to exercise timer elapsed/remaining consumption without loading track-dependent views. No vendor source is modified. Logs: `/tmp/encounter-script-consumers-{ptr,retail}.log`, `/tmp/encounter-script-existing-{ptr,retail}.log`; RED: `/tmp/encounter-script-red-fixed.log`. Check/readability, full UI integration, and artifact validation remain parent-owned.

## Known gaps (current cycle)

- [x] Focused proof at `39b037a91`: six PTR script-core tests plus three existing duration tests; retail preserves the demo with five integration tests; existing customization tests pass (one PTR, two retail).
- [ ] Parent owns track/filter/EditMode/queued-hold integration and final check/readability/artifact gates.
- [ ] Native transition/coalescing, secrecy, unknown-spell and validation semantics remain unverified.

## Out of scope

Encounter-driven gameplay producers, visual track assignment, queue holds, filtering/sorting, EditMode, user settings, protected/restricted access and native secret enforcement belong to later slices. No alternate demo data remains on PTR.
