# Encounter Timeline tracks, filters, views, and previews

The PTR timeline uses the script lifecycle model plus deterministic track placement. Contracts come from pinned generated EncounterTimeline documentation; numerical layout choices below are simulator assumptions, not native values. See [script lifecycle](encounter-timeline-script-core.md).

## What it must do

- [x] After countdown, keep an active event for `maxQueueDuration` in the Queued track; public remaining time and retained duration timers stay at zero. Pause freezes hold time; finish/cancel and the existing delayed-removal boundary remain available.
- [x] Publish track info structures for Queued, Short, Medium, Long, and Indeterminate. Return `(track, optional one-based sort index)` for event placement.
- [x] Model Queued as Sorted with capacity 3, Short as Linear 0–15 seconds, Medium as Linear 15–60 seconds, Long as Sorted above 60 seconds with capacity 3, and Indeterminate as Hidden. Overflow and paused events are hidden, not blocked. Sorted entries use remaining time then ID.
- [x] Return fresh track tables; getters, filtering, visibility, callbacks, and rendering consumers use the same placement state. Intro/gap values are zero. Long has unbounded duration. Indeterminate has infinite bounds to exclude it from the vendor duration-to-track search; hidden layout ignores its duration.
- [x] `GetSortedEventList` consumes positional optional count/duration and default-true terminal/hidden exclusions. Apply filters before count truncation; maximum duration means current remaining time in this model.
- [x] Highlight visible active events once per lifecycle at remaining time <=5 seconds. Emit color/highlight notifications after consistent state is available.
- [x] Support None/Timeline/Bars view values 0/1/2. None hides events. Timeline/Bars share track metadata; view changes notify deactivation, layout, activation for non-None views, and state after committing the new view. None emits no activation because the real Blizzard consumer has no corresponding view frame. Reentrant changes invalidate stale notifications.
- [x] Edit Mode owns three synthetic named slots, with durations 8/35/90 seconds and hold 30. `AddEditModeEvents` refreshes live slots without accumulation and returns 30; script events are unaffected. Cancel targets previews only and retains the core terminal-removal boundary. A refresh before canceled slots are removed creates new live IDs and leaves old terminal records until the next tick; terminal IDs never reactivate. Existing caller timers perform refresh; no scheduler loop is introduced.
- [x] Color results are fresh ColorMixin values derived from severity/highlight. Icon slots select event mask bits, use standard role atlases for role bits and the event icon for other bits, and clear unused slots. Asset/color policy and security aspects remain unverified.
- [x] Preserve earlier-retail demo behavior and exercise actual Blizzard TrackLayout/TimelineView consumers without vendor patches or fake API globals.

## How it works

- [Timeline script core](encounter-timeline-script-core.md)
- [Event system](../event-system.md)

## Implementation inventory

- `src/c_api/c_encounter_timeline/model.rs`: event source, clocks, queue holds, positions, and view state.
- `src/c_api/c_encounter_timeline/layout.rs`: deterministic placement, capacity, and highlight policy.
- `src/c_api/c_encounter_timeline/{tracks,filter,view,preview,visuals,notifications}.rs`: public queries, preview producer, visual data, and post-mutation callbacks.
- `src/lua_api/globals/missing_surface/encounter_events.rs`: preserves earlier-retail color-component compatibility without overwriting the modeled PTR ColorMixin query.

## Tests asserting this spec

- `tests/encounter_timeline_tracks.rs`: public producer/tick/filter/preview/view/reentrancy behavior.
- `tests/encounter_timeline_view.rs`: actual Blizzard TrackLayout, TrackView/TimerView, Edit Mode timer loop, and texture-slot consumers. The timer test advances the existing pending timer deadline; it does not replace the callback/API.
- `tests/encounter_timeline_script.rs`: prior lifecycle/timer and retail preservation coverage.

### Focused development proof

At `ec9f06834`, the PTR grouped run passed 18 integration cases plus one existing library bridge case. Earlier-retail proof at `88461edd0` passed five integration and two library cases; subsequent production changes were confined to the PTR-gated timeline module, so that proof remains applicable. Tests exercise actual Blizzard TrackLayout, TrackView/TimerView activation, callback-visible state/color, texture slots, and the original Edit Mode `C_Timer.NewTimer` refresh path. No vendor globals or API replacements are installed by the fixtures. Existing dispatch can suppress handler errors; tests therefore also assert observable consumer state and inspect captured timeline Lua errors.

```text
cargo test --lib --test integration --offline --no-default-features --features sound,gui,client-<profile> -- encounter_timeline_tracks:: encounter_timeline_view:: encounter_timeline_script:: duration_core:: system_api_seeded::test_c_encounter_timeline test_patch_12_0_7_safe_global_bridges installs_timeline_and_event_customization_state --nocapture
```

Profiles ran separately. Logs: `/tmp/encounter-tracks-ec9f06834-ptr.log`, `/tmp/encounter-tracks-88461edd0-retail.log`; ledger: `/tmp/encounter-tracks-proof-ledger.json`. Initial RED: `/tmp/encounter-tracks-red.log`; terminal-preview RED: `/tmp/encounter-preview-terminal-red.log`. Root `cargo fmt` ran before source commits.

### Audit boundary

The pinned target documents every newly registered public helper used by this model, including `GetViewType`; no extra public helper is credited. The audit credits only the eleven changed rows represented here plus the prior nine core rows. Fixed thresholds, capacities, hidden/overflow placement, duration filtering, sorting ties, colors, icon assets, preview fixtures, callback ordering, and all native/security semantics are simulator policy or unverified. Queue-hold credit is modeled behavior only, not native placement conformance.

## Known gaps (current cycle)

- [ ] Native layout capacities, thresholds, hidden/overflow policy, sorting ties, colors, icon assets, preview fixtures, and event ordering are unverified simulator policies.
- [ ] Secret/protected/forbidden enforcement and encounter-owned events are not modeled by this slice.
- [ ] Parent owns final combined check/readability/smoke gates; this slice's committed source/test review now backs the audit artifact credit.

## Out of scope

Vendor patches, encounter gameplay producers, native layout/security conformance, animation/rendering redesign, audit artifact credit, final verification gates, deployment, and publishing remain parent-owned or separate work.
