# Target aura private Count publication

Full Forever UI startup followed by `TargetUnit('player')` and an `OnUpdate` tick failed in unchanged `TargetFrameAuraButton.lua:75`: private `self.Count` was nil. The error's `frame=__tpl25839` dispatcher label names the running aura **container**, not the offending nested AuraButton.

## Root cause and correction

The container's acquired AuraButton had a Rust `Count` child and a public Lua `Count` field, but `GetForbiddenObjectTable(button).Count` was nil. XML parentKey publication populated the public object only; the secure private aura mixin receives the forbidden view. XML-owned children are now also published to the forbidden view when the parent is scoped for that partition. Ordinary public field assignments remain private-isolated. The child projection respects its existing scope marker.

The first successful Count assignment exposed a separate layout error (`TargetFrame.lua:569`, number compared with userdata); the later guarded secret-number ordering rilua pin addresses that distinct failure.

## Bounded integration result

`a2fd85382` and `30971450a`, with the separately pinned rilua ordering fix, pass 2/2 original-boundary regressions with zero Lua errors in `/tmp/wow-unit-frame-bug/aura-combo-green-20260925.log`: full startup → `TargetUnit('player')` → tick, and PlayerFrame `PLAYER_ENTERING_WORLD` → ComboFrame `3 → 0` with the CVar set before `OnLoad`. Final bounded local GUI evidence is in [[final-unit-frame-click-aura-proof]]: zero hook/final errors, Combo CVar `1` reports `0`, and hovering Aura ID `1` displays `Arcane Intellect`. This does not reproduce the original screenshot exactly or establish native-client conformance.

## Sources

- [Script-object environment contract](../../specs/script-object-environments.md) — partition isolation and child publication.
- `tests/click_targeting/forever_regressions.rs` — same-boundary full startup, targeting, tick, Count, and isolation regression.

## See Also

- [[ellesmereui-forever]] — related private AuraContainer boundaries.
- [[on-update-dirty]] — OnUpdate lifecycle context.
- [[secret-number-ordering]] — distinct numeric-ordering boundary required by the integrated path.
- [[final-unit-frame-click-aura-proof]] — final bounded GUI and verifier evidence.
