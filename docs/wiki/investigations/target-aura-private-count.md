# Target aura private Count publication

Full Forever UI startup followed by `TargetUnit('player')` and an `OnUpdate` tick failed in unchanged `TargetFrameAuraButton.lua:75`: private `self.Count` was nil. The error's `frame=__tpl_25839` label names the running aura **container**, not the failing button.

## Root cause and correction

The container's acquired AuraButton had a Rust `Count` child and a public Lua `Count` field, but `GetForbiddenObjectTable(button).Count` was nil. XML parentKey publication populated the public object only; the secure private aura mixin receives the forbidden view. XML-owned children are now also published to the forbidden view when the parent is scoped for that partition. Ordinary public field assignments remain private-isolated. The child projection respects its existing scope marker.

The first successful Count assignment exposed a separate layout error (`TargetFrame.lua:569`, number compared with userdata); the later guarded secret-number ordering rilua pin addresses that distinct failure. Full-startup aura tick proof after both changes must be reported independently from Count root-cause proof.

## Sources

- [Script-object environment contract](../../specs/script-object-environments.md) — partition isolation and child publication.
- `tests/click_targeting/forever_regressions.rs` — same-boundary full startup, targeting, tick, Count, and isolation regression.

## See Also

- [[ellesmereui-forever]] — related private AuraContainer boundaries.
- [[on-update-dirty]] — OnUpdate lifecycle context.
