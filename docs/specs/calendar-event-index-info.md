# Calendar event identity lookup

`C_Calendar.GetEventIndexInfo` exposes simulator-owned event-ID-to-position records. User selected generated test data in normal simulator sessions and location inputs as filters. [Dataset](../../data/calendar/demo-event-indices.json) uses numeric **simulator fixture IDs**, not native Calendar data. Current generated Calendar documentation declares required `eventID: CalendarEventID`, optional numeric `monthOffset`/`monthDay`, and a nilable DTO with `offsetMonths`, `monthDay`, `eventIndex`; it does not establish concrete native ID representation or hint outcomes. [Source boundary](../wiki/investigations/patch-8-1-0-api-audit.md#current-retail-calendar-query-contract--2026-10-10).

## What it must do

- [x] Seed normal simulator sessions with demo IDs910001/910002/910003 and positions `(0,15,1)`, `(0,15,2)`, `(1,4,1)` from the committed dataset.
- [x] Return one DTO for a configured numeric simulator ID; return one nil for no configured identity. Nil-on-miss is simulator policy, not native verification.
- [x] Accept omitted/nil location inputs; each supplied monthOffset/monthDay must match the stored offsetMonths/monthDay or the result is nil. User-selected filtering policy.
- [x] Return independent result snapshots; caller table mutation cannot change the owned records. Backing-state changes affect later lookups without adding a Lua calendar editor API.
- [x] Validate required numeric simulator IDs and optional numeric location inputs without lossy ID aliasing or silently coercing wrong types. Numeric ID representation is fixture-model policy, not native typedef proof.
- [x] Apply the documented `AllowedWhenUntainted` input policy using the VM's existing authentication behavior; secret wrappers/caller taint remain intact.

## How it works

- [C API boundary](../../AGENTS.md#c-api-boundary)
- [Current handoff](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md)

## Implementation inventory

- `data/calendar/demo-event-indices.json`: generated fixture source.
- `src/c_api/c_calendar.rs`: modeled DTO/query/registration.
- `SimState.calendar_event_indices`: owned positions, seeded from the demo dataset in normal defaults.

## Tests asserting this spec

- `tests/calendar_event_index_info.rs`: public Lua defaults/filtering/absence/snapshots/security, owned-state update/removal and cleanup restoration.
- Test-stage `a4159ce53` executed authentic RED: nine failures; first positive fixture query returned no DTO. This does not prove later branches. Producer `fa66dd878` focused execution reports 11 passed; normal loaded demo/publication assertions and Lua-error smoke report exit0/`[]`. Independent GREEN verification PASS; [retained evidence](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/calendar-event-index-2026-10-11/README.md). Independent publication verification PASS: one exact case, 145 classified identities (89 OK / 56 current known gaps). Bounded lookup acceptance verified; parent full handoff remains open.

## Verification

- [x] Independent GREEN verification PASS: 11 behavioral cases, loaded normal-session assertions and required Rust checks.
- [x] Separate [PTR behavioral execution](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/calendar-event-index-2026-10-11/ptr-2026-10-12/README.md) reaches all eleven cases at the applicable f314 artifact, including secret-input controls; not a current-HEAD build or native/editor/catalog claim.
- [x] Loaded normal-session publication assertion and owning prefork sweep verified; only the current event-index gap row removed. Frozen historical accounting preserved; [publication proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/calendar-event-index-2026-10-11/publication/publication-independent-report.md).

## Out of scope

Full Calendar event catalog, selected/open-event state, navigation/editor/invitation lifecycle, real dates, native IDs/ordering/secrecy fidelity and historical8.1 signature claims. These are distinct unestablished domains; this identity query has no selected-event hidden input. No vendor changes or synthetic native provenance.
