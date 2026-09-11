# Encounter Timeline tracks, filters, views, and previews

The PTR timeline uses the script lifecycle model plus deterministic track placement. Contracts come from pinned generated EncounterTimeline documentation; numerical layout choices below are simulator assumptions, not native values. See [script lifecycle](encounter-timeline-script-core.md).

## What it must do

- [ ] After countdown, keep an active event for `maxQueueDuration` in the Queued track; public remaining time and retained duration timers stay at zero. Pause freezes hold time; finish/cancel and the existing delayed-removal boundary remain available.
- [ ] Publish track info structures for Queued, Short, Medium, Long, and Indeterminate. Return `(track, optional one-based sort index)` for event placement.
- [ ] Model Queued as Sorted with capacity 3, Short as Linear 0–15 seconds, Medium as Linear 15–60 seconds, Long as Sorted above 60 seconds with capacity 3, and Indeterminate as Hidden. Overflow and paused events are hidden, not blocked. Sorted entries use remaining time then ID.
- [ ] Return fresh track tables; getters, filtering, visibility, callbacks, and rendering consumers use the same placement state. Intro/gap values are zero. Long has unbounded duration. Indeterminate has infinite bounds to exclude it from the vendor duration-to-track search; hidden layout ignores its duration.
- [ ] `GetSortedEventList` consumes positional optional count/duration and default-true terminal/hidden exclusions. Apply filters before count truncation; maximum duration means current remaining time in this model.
- [ ] Highlight visible active events once per lifecycle at remaining time <=5 seconds. Emit color/highlight notifications after consistent state is available.
- [ ] Support None/Timeline/Bars view values 0/1/2. None hides events. Timeline/Bars share track metadata; view changes notify deactivation, layout, activation, and state after committing the new view. Reentrant changes invalidate stale notifications.
- [ ] Edit Mode owns three synthetic named slots, with durations 8/35/90 seconds and hold 30. `AddEditModeEvents` refreshes those slots without growing the list and returns 30; script events are unaffected. Cancel targets previews only and retains the core terminal-removal boundary. Existing caller timers perform refresh; no scheduler loop is introduced.
- [ ] Color results are fresh ColorMixin values derived from severity/highlight. Icon slots select event mask bits, use standard role atlases for role bits and the event icon for other bits, and clear unused slots. Asset/color policy and security aspects remain unverified.
- [ ] Preserve earlier-retail demo behavior and exercise actual Blizzard TrackLayout/TimelineView consumers without vendor patches or fake API globals.

## How it works

- [Timeline script core](encounter-timeline-script-core.md)
- [Event system](../event-system.md)

## Implementation inventory

- `src/c_api/c_encounter_timeline/model.rs`: event source, clocks, queue holds, positions, and view state.
- `src/c_api/c_encounter_timeline/layout.rs`: deterministic placement, capacity, and highlight policy.
- `src/c_api/c_encounter_timeline/{tracks,filter,view,preview,visuals,notifications}.rs`: public queries, preview producer, visual data, and post-mutation callbacks.
- `src/lua_api/globals/enum_data/combat_system.rs`: PTR view enum values; older profile values unchanged.

## Tests asserting this spec

- `tests/encounter_timeline_tracks.rs`: public producer/tick/filter/preview/view/reentrancy behavior.
- `tests/encounter_timeline_script.rs`: prior lifecycle/timer and retail preservation coverage.

## Known gaps (current cycle)

- [ ] Native layout capacities, thresholds, hidden/overflow policy, sorting ties, colors, icon assets, preview fixtures, and event ordering are unverified simulator policies.
- [ ] Secret/protected/forbidden enforcement and encounter-owned events are not modeled by this slice.
- [ ] Full consumer proof and targeted regression results must be recorded after execution.

## Out of scope

Vendor patches, encounter gameplay producers, native layout/security conformance, animation/rendering redesign, audit artifact credit, final verification gates, deployment, and publishing remain parent-owned or separate work.
