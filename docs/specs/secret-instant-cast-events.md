# Secret instant-cast completion events

Bounded retail 12.0.5 host-input producer for `prose-2026-04-10-193`, sourced from [API change text](../../data/patch-api/sources/12.0.5-api-changes.txt), line 193. This covers event eligibility, not automatic world/casting simulation or secret payload production.

## What it must do

- [ ] Suppress UNIT_SPELLCAST_SUCCEEDED before any frame or global callback when the explicit completion is instant, its spell is secret, and its caster is not the player.
- [ ] Deliver the player exception, public instant casts, and noninstant casts through the existing synchronous event dispatcher.
- [ ] Use explicit host caster identity, including a token alias of the player; do not mistake pet/vehicle or group membership for the player exception.
- [ ] Preserve unit token, cast GUID, spell ID, and optional cast-bar ID in the delivered four-argument payload.
- [ ] Classification changes affect the next completion without leaking between environments.

## How it works

- [Event dispatcher](../event-system.md)

## Implementation inventory

- `src/lua_api/cast_success.rs`: typed host completion snapshot and synchronous producer.
- `src/lua_api/mod.rs`: feature-gated type export.
- `src/lua_api/env_events.rs`: existing frame/global dispatch, unchanged.

## Tests asserting this spec

- `tests/cast_events_identity.rs`: eight-policy-combination matrix, player alias, pet negative control, all listener families, payload and host-input transition.

## Development proof and independent bounded acceptance — 2026-10-03

Commit `a4cce2db1`. RED: 0 PASS / 4 FAIL cast cases. GREEN: 4/4 cast cases inside a 404/404 run with control suites; `cargo fmt --check` exit0; startup `lua-errors` `[]`. This section supersedes any wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun): **ACCEPT WITH QUALIFICATIONS**, [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b98-verify-events-commands.md) SHA256 `72c3dbad682b576a045792d498f279ba54e7ef7b6094d5364da3d8a5817b4ac3`. Helper-level only: nothing in the runtime produces a non-player cast through it. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-04-10-193 partial-development-green under capability `secret-instant-cast-events`; **119 capabilities/362 IDs; 42 pending /262 bounded /23 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Staged only; compilation, RED/GREEN and independent verification remain unrun.

## Out of scope

- Automatic spell secrecy, instantness or caster classification. Host input is explicit; trusting it is **INFERRED** simulator policy, not native parity.
- Completing a real cast or applying its effects. Existing GUI completion producer is player-only and unchanged; this slice adds a nonplayer-capable host producer, not an NPC combat engine.
- SecretWhenUnitSpellCastRestricted payload wrapping/taint. Delivery eligibility does not prove the cached declaration's payload secrecy contract.
- Generic manually injected events (`fire_event_with_args`, admin events, loader dispatch). They lack instant/secrecy metadata and are not interpreted as cast simulation; no fallback metadata is guessed.
