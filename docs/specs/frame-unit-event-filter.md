# Frame unit-event filter

`Frame:RegisterUnitEvent(event, unit1[, unit2])` must retain both supplied valid literal unit tokens for frame-script delivery. Registration lives under `src/lua_api/frame/methods/text_attribute_event/`; see the [event system](../event-system.md) for dispatch architecture. This bounded repair is approved for `player` and `target`; it is not native-client proof. The [retained 5.0.4 source](../../data/patch-api/sources/5.0.4-api-changes.txt) specifies `unit1[, unit2]`; no unlimited-argument or native query-return contract is inferred.

## What it must do

- [x] Match the first event payload against either registered literal token (OR), delivering the original payload to `OnEvent`. With `"player", "target"`, both player and target health events must arrive; an unmatched token must not arrive.
- [x] Preserve single-unit filtering: player-only registration suppresses target events and target-only registration suppresses player events.
- [x] Re-registering with one unit replaces the two-unit filter; the former second unit no longer matches.
- [x] `UnregisterEvent` and `UnregisterAllEvents` remove the complete filter and prevent delivery for both units.
- [x] Keep existing `IsEventRegistered` first-unit reporting and invalid-first-unit policy unchanged; neither behavior is newly established native parity.
- [x] Keep exact query return arity unchanged for the bounded current model. [Independent Retail CLI proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/unit2-query-arity/final-report.md) passes 5/5 states with exactly two values: fresh, pair registration, single-target replacement, unregister, and invalid-first policy. Marker plus trailing `[]` and exit 0 establish execution; no other-profile arity or native query-parity claim.

## How it works

- [Event system](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/events.rs` — first/second registration and unchanged first-unit query.
- `src/lua_api/frame/methods/text_attribute_event/unit_event.rs` — existing literal-token validation and registered-unit query helper.
- `src/widget/frame.rs` and `src/widget/registry/mod.rs` — first string plus optional second string per event; registration index and lifecycle.
- `src/lua_api/script_helpers/event_dispatch.rs` — first-payload matching against either stored token.
- `src/widget/frame_size.rs` — storage estimate includes both token strings.

## Tests asserting this spec

- `tests/unit_event_second_unit.rs::second_unit_filter_receives_modeled_player_and_target_health_events` — modeled player healing and target damage, single-unit controls, original first payload, and existing first-unit query control.
- `tests/unit_event_second_unit.rs::reregistering_and_unregistering_clear_the_second_unit_filter` — unmatched-token suppression, one-unit replacement, unregister, and unregister-all controls.

## Accepted bounded proof

[Authentic RED](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/unit2-red/final-report.md) records 1 FAIL/1 PASS: missing second-unit target damage delivery. [Independent GREEN](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/unit2-green/final-report.md) accepts 119/119 cases across seven profiles and 49 commands, including both unit2 cases and first-unit/invalid-first query controls per profile. The separate Retail query receipt closes bounded current-model exact arity (5/5 states), not native query parity. Native query behavior, aliases, catalog identity, Mists startup and full handoff remain OPEN.

## Known gaps (current cycle)

No remaining gap in the approved player/target repair and bounded query-preservation slice. Broader native contracts remain separate.

## Out of scope

Generic unit aliases, invalid-second-unit behavior, third-argument contracts, changes to invalid-first-unit policy or query arity, and native-client parity claims. These exceed the approved valid `player`/`target` second-unit repair.
