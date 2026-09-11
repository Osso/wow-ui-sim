# Player cast lifecycle event payloads

The existing action-spell, crafting, specialization-change, and timed-completion producers expose coherent cast identities through `UNIT_SPELLCAST_START`, `UNIT_SPELLCAST_STOP`, and `UNIT_SPELLCAST_SUCCEEDED`. Pinned contracts live in `data/patch-api/sources/12.1.5-register.json`; related query tuples are specified in [cast-bar ID returns](cast-bar-id-returns.md).

## What it must do

- [x] Emit exactly `(unitTarget, castGUID, spellID, castBarID)` for all three events on current retail and PTR. Both pinned revisions specify this order; PTR changes only the declared nullable cast-bar type from `number` to `UnitCastBarID`.
- [x] START callbacks from action spells, crafting, and specialization changes observe the initialized cast and the same numeric ID returned by `UnitCastingInfo` slot 10.
- [x] Timed completion clears casting before STOP, then emits SUCCEEDED with the same four payload values as START; subsequent completion checks emit nothing.
- [x] Successive allocated casts receive different modeled GUID/bar-ID pairs.
- [x] Specialization remains unchanged during these callbacks and changes through the existing post-completion specialization step. Requesting the already-active specialization emits no START.

### Simulator assumptions

The modeled GUID is `Cast-Sim-<cast_id>`, derived from the existing `u32` cast allocator; no second identity store is introduced. It is a string identifier, **not a native WoW GUID format**. Uniqueness lasts until the existing allocator wraps. These producers always have an allocated numeric bar ID even though the declared payload permits nil.

Existing state visibility and event order are preserved, not asserted as independently measured native behavior. Completion tests advance the existing cast deadline into the past and invoke the same extraction/emission functions used by the GUI tick; they do not inject synthetic spellcast events.

### Explicit self-cancellation

- [ ] On the `retail-12-1-0` API epoch and later, `SpellStopCasting()` takes the old cast and clears casting before emitting exactly `(unitTarget, castGUID, spellID, interruptedBy, castBarID)` for `UNIT_SPELLCAST_INTERRUPTED`. Both pinned revisions have this five-field order; only the last field's declared type changes on PTR.
- [ ] Resolve `interruptedBy` with the same player GUID resolver used by `UnitGUID`, not the current target or a fabricated enemy. Self-cancel attribution is a **simulator assumption**, not native actor evidence.
- [ ] Emit paired four-field `UNIT_SPELLCAST_STOP` after interruption, describing the canceled cast. Return exactly `true`; no active cast returns exactly `false` without notifications. Older API epochs retain the prior clear-and-boolean path.
- [ ] An interruption or STOP callback can create a new cast. Do not clear it after callbacks; it subsequently completes with its own identity and effects. The canceled cast cannot later succeed or apply its effects.

The chosen `INTERRUPTED` then `STOP` order follows two inspected consumers: `Blizzard_UIPanels_Game/Shared/CastingBarFrame.lua` handles interruption and clears its casting flag, while `Blizzard_UnitFrame/Mainline/UnitFrame.lua` uses STOP (not INTERRUPTED) to clear mana-cost prediction. This is a modeled notification policy, **not native event-order confirmation**. It does not change normal completion ordering or add FAILED, DELAYED, channel, or empower producers. No GCD, cooldown, or unrelated pending-specialization behavior is reset by this slice.

## How it works

- [Event dispatch](../event-system.md)
- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/spellcast_events.rs`: shared modeled identity payloads, START, and epoch-gated self-interruption/STOP dispatch.
- `src/lua_api/globals/combat_verbs.rs`: action-spell START and atomic self-cancellation producer.
- `src/lua_api/globals/unit_misc.rs`: shared unit GUID resolver used for self-cancel attribution.
- `src/lua_api/globals/missing_surface/profession_crafting.rs`: crafting START producer.
- `src/c_api/c_spec.rs`: specialization-change START producer.
- `src/iced_app/casting.rs`: timed completion state extraction and STOP/SUCCEEDED producers.

## Tests asserting this spec

- `src/iced_app/casting/tests.rs`: real producer callbacks and timed completion on both current profiles, in the existing library test target.
- `src/iced_app/casting/interrupted_tests.rs`: real timed casts, cancellation callbacks, repeat/no-completion behavior, and reentrant replacement casts through the existing completion/effect boundary.
- Focused `spellcast_payload_` tests: 3 passed per profile at `f474fb2f9`.
- Existing grouped integration filters `spell_casting::`, `test_crafting::`, and `admin_spec_talent_api::c_spec_set_specialization`: 18, 22, and 2 passed respectively per profile. No broad suites or acceptance gates run.

## Known gaps (current cycle)

- [ ] Native GUID formatting, allocator lifetime/wrap semantics, and secret/restricted-value behavior remain unverified.

## Out of scope

Channel, empower, FAILED/FAILED_QUIET, DELAYED and instant-spell producers; native interruption actor/order semantics; artifact audit credit; security enforcement. Existing spell effects and specialization application remain separate post-event steps. Self-cancellation is limited to the existing `SpellStopCasting` path.
