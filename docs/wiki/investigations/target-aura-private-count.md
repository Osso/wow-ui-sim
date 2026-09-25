# Target aura private Count publication

Full Forever UI startup followed by `TargetUnit('player')` and an `OnUpdate` tick failed in unchanged `TargetFrameAuraButton.lua:75`: private `self.Count` was nil. The error's `frame=__tpl25839` dispatcher label names the running aura **container**, not the offending nested AuraButton.

## Root cause and correction

The container's acquired AuraButton had a Rust `Count` child and a public Lua `Count` field, but `GetForbiddenObjectTable(button).Count` was nil. XML parentKey publication populated the public object only; the secure private aura mixin receives the forbidden view. XML-owned children are now also published to the forbidden view when the parent is scoped for that partition. Ordinary public field assignments remain private-isolated. The child projection respects its existing scope marker.

The first successful Count assignment exposed a separate layout error (`TargetFrame.lua:569`, number compared with userdata); the later guarded secret-number ordering rilua pin addresses that distinct failure.

## Private method-origin correction

`eafd75a76` corrects a separate projection-factory taint boundary. Runtime provenance for AuraContainer `#61195` (DandersFrames) and `#76243` (BetterBlizzFrames) shows the generated script handler and private `OnUpdate` clean, while the private `Update` wrapper carries the addon taint at `@shared-bootstrap:150`; the underlying Blizzard method is clean. The pre-fix probe reports `trustedread=false`, `capturedread=false`, and `addonread=false`: an addon caller tainted both private-projection creation and bound-method lookup.

The correction creates the proxy metatable and its bound-method adapters through the native secure-call factory only. It does not clear taint at invocation: a private call retains the active caller and the underlying method retains its own addon origin. This is simulator taint behavior, not native private-object security conformance.

## Verification boundary

`/tmp/retail-regression/private-method-origin-red.*` is the pre-fix RED for the three false origin reads. `/tmp/retail-regression/wrapper-after.*` is also pre-fix: one unique Lua error across 22 occurrences (20 DandersFrames, 2 BetterBlizzFrames). No rebuilt full startup, reduction, or GREEN result exists for `eafd75a76`; do not treat the committed focused regression as runtime confirmation.

## Bounded integration result

`a2fd85382` and `30971450a`, with the separately pinned rilua ordering fix, pass 2/2 original-boundary regressions with zero Lua errors in `/tmp/wow-unit-frame-bug/aura-combo-green-20260925.log`: full startup → `TargetUnit('player')` → tick, and PlayerFrame `PLAYER_ENTERING_WORLD` → ComboFrame `3 → 0` with the CVar set before `OnLoad`. Final bounded local GUI evidence is in [[final-unit-frame-click-aura-proof]]: zero hook/final errors, Combo CVar `1` reports `0`, and hovering Aura ID `1` displays `Arcane Intellect`. This does not reproduce the original screenshot exactly or establish native-client conformance.

## Sources

- [Script-object environment contract](../../specs/script-object-environments.md) — partition isolation and child publication.
- `tests/click_targeting/forever_regressions.rs` — same-boundary full startup, targeting, tick, Count, and isolation regression.
- `tests/userdata_proxy.rs` — committed method-origin regression; no test run is recorded here.

## See Also

- [[ellesmereui-forever]] — related private AuraContainer boundaries.
- [[on-update-dirty]] — OnUpdate lifecycle context.
- [[secret-number-ordering]] — distinct numeric-ordering boundary required by the integrated path.
- [[final-unit-frame-click-aura-proof]] — final bounded GUI and verifier evidence.
