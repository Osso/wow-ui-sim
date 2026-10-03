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

## Known gaps (current cycle)

- [ ] Staged only; compilation, RED/GREEN and independent verification remain unrun.

## Out of scope

- Automatic spell secrecy, instantness or caster classification. Host input is explicit; trusting it is **INFERRED** simulator policy, not native parity.
- Completing a real cast or applying its effects. Existing GUI completion producer is player-only and unchanged; this slice adds a nonplayer-capable host producer, not an NPC combat engine.
- SecretWhenUnitSpellCastRestricted payload wrapping/taint. Delivery eligibility does not prove the cached declaration's payload secrecy contract.
- Generic manually injected events (`fire_event_with_args`, admin events, loader dispatch). They lack instant/secrecy metadata and are not interpreted as cast simulation; no fallback metadata is guessed.
