# RegisterUnitEvent unit arguments (R88aj)

`Frame:RegisterUnitEvent(event, ...)` unit-argument handling in `src/lua_api/frame/methods/text_attribute_event/`. The native answer is `pr46-49-decomp-answers.md` (untracked decomp handoff), section `R88aj`. The unit-argument rules are identical in the three decoded builds, Mists 5.5.4.70631, Retail 12.1.5.70688 and Forever 1.60.1.70707; Retail and Forever add secret-value, forbidden-aspect and post-return checks, which are not modelled. This supersedes the two-unit bounded model in [frame unit-event filter](frame-unit-event-filter.md).

## What it must do

- [x] **VERIFIED (R88aj aj1/aj3):** no Lua table support. The handler reads at most four unit arguments (Lua positions after the event) and stops at the first one that is not a string or number; valid units read before it stay in the filter. A table as the first unit gives zero units.
- [x] **VERIFIED (R88aj aj1/aj2):** zero valid units register the event with an empty filter, the same as `RegisterEvent`, without an error.
- [x] **VERIFIED (R88aj aj2):** an unknown token (for example `""`, `"0"`, `"pettarget"`, `"playerpet"`) is skipped without error but uses one of the four positions. Unit tokens are case-insensitive.
- [x] **VERIFIED (R88aj aj4):** each successful call returns exactly one boolean: true when the frame was not registered, or when the call changed the unit filter (position by position); false when the filter is identical, including an empty filter registered again with zero units.
- [x] **VERIFIED (R88aj aj5):** a second call replaces the filter (no merge). `RegisterEvent` after `RegisterUnitEvent` clears the filter and returns true.
- [x] **PRESERVATION:** `UnregisterEvent` removes the registration and its filter. This is the pre-existing simulator behaviour, already covered on all profiles by `tests/unit_event_second_unit.rs:61-66`. VERIFIED natively only for Mists 70631 (R88aj aj5); UNKNOWN for Retail 70688 and Forever 70707 (not decoded).
- [x] **VERIFIED consumer path:** with `CAAEnabled=1`, the original Mists `CombatAudioAlertManagerMixin:RegisterForUnitHealth` (`Blizzard_CombatAudioAlertManager.lua:438`) registers `UNIT_HEALTH` from its unit table with an empty filter when player or target health alerts are on, and leaves it unregistered when both are off.
- [x] **INFERRED (R88aj):** an empty filter delivers the event for all units (dispatch not decoded). Existing dispatch behavior; not asserted.
- [x] **INFERRED (R88aj applicability):** the scan applies to every profile. Forever's cache is the decoded 70707 build; the Mists 70268, Retail, PTR and Wrath caches are not decoded builds.

## How it works

- [Event system](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/unit_event.rs` — the four-position scan.
- `src/lua_api/frame/methods/text_attribute_event/events.rs` — `RegisterUnitEvent` return and `RegisterEvent` filter clearing.
- `src/widget/registry/mod.rs` — filter replacement and change detection; `src/widget/frame.rs` stores a unit list per event.
- `src/lua_api/script_helpers/event_dispatch.rs` — first-payload match against any stored unit.

## Tests asserting this spec

- `tests/register_unit_event_units.rs` — return values for table, mixed, nil-stopped, unknown-token, case, order, five-unit, zero-unit, RegisterEvent and UnregisterEvent sequences. The test has no profile gate. On PTR, Wrath, Era and Anniversary it applies only by INFERRED applicability (no decoded build of those client lines). Era and Anniversary are not built, and Wrath is compile-only, so the test has run on Retail, PTR, Forever and Mists.
- `tests/mists_r88aj.rs` — original Mists `RefreshEvents` → `RegisterForUnitHealth` table registration for player, target, both and neither.

## Known gaps (current cycle)

- [ ] **VERIFIED, not modelled:** the usage error for a missing or non-string event (`Usage: ("event", "unit1" [,"unit2", "unit3", "unit4"])` with the `%s:%s(): %s` prefix) and the unit-event set check; the simulator keeps its previous event validation.
- [ ] **VERIFIED for Mists, not modelled:** the exact known-token list (`boss6`–`boss8` are unknown in Mists 70631; the simulator accepts any numbered `boss` token).
- [ ] **VERIFIED, not modelled:** deferred registration during event dispatch, the tainted zero-unit `COMBAT_LOG_*`/`MINIMAP_PING` false return, secret-value arguments, and Retail/Forever forbidden-aspect and post-return routines.
- [ ] **UNKNOWN (R88aj):** Retail and Forever known-token and unit-event lists.
- [ ] **VERIFIED (R88aj aj1), not modelled:** event names are not case-sensitive natively; the simulator matches event names case-sensitively.
