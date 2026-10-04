# Aura entry instance identifiers

Retail12.0.5 entry transitions invalidate prior host-modeled aura IDs while retaining aura content. Source: [12.0.5 changes](../../data/patch-api/sources/12.0.5-api-changes.txt), prose-2026-03-31-150. Host supplies replacement IDs; randomness and distribution remain host-owned and unproven. This bounded transition does not earn prose-2026-04-10-202 cooldown-viewer tooltip credit.

## What it must do

- [ ] INFERRED boundary: each ENCOUNTER_START, CHALLENGE_MODE_START or PVP_MATCH_ACTIVE delivery consumes one explicit host replacement batch before dispatching that event to consumers.
- [ ] Replace player stored aura IDs and each party member's buff/debuff IDs together; preserve names, spell IDs, timing, stacks, polarity and source identity.
- [ ] Instance, index, slot and spell queries observe new IDs; old instance queries miss. Repeated entries cannot reuse IDs retired by prior entry transitions.
- [ ] Non-entry events and ordinary OnUpdate preserve current IDs; an explicitly added post-entry aura remains queryable.
- [ ] INFERRED host protocol: missing input for live auras, wrong count, nonpositive IDs, duplicate replacements and live/retired reuse fail before mutation or event delivery. Batch input remains available after rejection.

## How it works

- [Event dispatch](../event-system.md)
- [Aura API surface](../lua-api.md)

## Implementation inventory

- `src/c_api/aura_entry_ids.rs`: host batch and retired-ID state.
- `src/c_api/aura_entry.rs`: shared validation and atomic rekey.
- `src/c_api/mod.rs`, `src/lua_api/state.rs`, `state/sim_state.rs`: feature gate and state ownership.
- `src/lua_api/env_events.rs`, `globals/state_backed_queries.rs`, `loader_env.rs`: entry dispatch boundaries.

## Tests asserting this spec

`tests/patch_12_0_5_navigation_aura_entry.rs`: five aura cases cover all three entry kinds, repeated entry, real event consumers, ordinary updates, invalid batches, party payload, post-entry insertion and host/admin/global/loader dispatch. Authored only; no executed proof here.

## Development proof and independent bounded acceptance — 2026-10-03 (aura-entry-instance-ids)

Commit `fcdcf43c6`. RED: 0 PASS / 8 FAIL module. GREEN: 8/8 module. This section supersedes wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun), [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b99-verify-auras-nav.md) SHA256 `c810ea3b9a7e3468a21bae71c13d18bedfe71c13c51e4438eb1f8a280ef11b20`. Review rejected the first version for dropping entry events; fixed in the follow-up commit. Rekey happens only on host input. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-03-31-150 partial-development-green under capability `aura-entry-instance-ids`; **131 capabilities/362 IDs; 28 pending /269 bounded /30 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Main must run genuine state-only RED, producer GREEN and required verification.
- [ ] Row150 remains a bounded entry freshness/payload model, not native randomness parity or universal aura storage parity.
- [ ] Row202 NOT EARNED: loaded Blizzard cooldown-viewer UNIT_AURA refresh and tooltip consumer/event ordering absent. No UNIT_AURA notification is synthesized by this slice.

## Out of scope

Target's constant fixture auras, native RNG, replacement allocation outside entry, stale per-ID blocked/provider bindings, private-aura rendering, old profiles and loaded cooldown-viewer behavior. Host must stage IDs for nonempty entry; missing input fails instead of selecting a fallback allocator. Zero live auras with no staged batch need no transition.
