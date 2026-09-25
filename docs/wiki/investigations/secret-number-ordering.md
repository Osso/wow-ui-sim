# Secret-number ordering

Rilua pin `9ad8720b10ef7ed7b855fe741a6368c631af0dc2`, published from `osso/host-secret-bool`, supplies a narrow VM ordering boundary required by Forever target-aura layout. This is inferred simulator compatibility policy, not native-verified secret-value semantics.

## Root cause

`TargetFrameAuraFlowLayoutMixin:OnLayoutComplete` stores `secretwrap(lineCount)`. Unchanged Blizzard `TargetFrameMixin:ShouldAnchorSpellBarToAuraContainer` subsequently compares that wrapped value with public zero or two. Before the pin, the comparison reached the ordinary number/userdata error boundary.

## Contract

Guarded VM numeric ordering supports secret-wrapped numbers against public or secret-wrapped numbers through `<`, `<=`, `>`, and `>=`. The guard rejects inspection from tainted callers, including a tainted ancestor that reaches untainted code. Public numeric ordering remains unchanged; nonnumeric wrappers remain invalid for ordering; secret arithmetic and equality remain opaque.

## Evidence and limits

Independent rilua verification records 21/21 checks plus `cargo fmt` and `cargo check` in `/tmp/wow-unit-frame-bug/rilua-verify-ledger-20260925.json`; the inherited `strlen` warning remains outside this slice. Simulator integration is bounded 2/2 with zero Lua errors in `/tmp/wow-unit-frame-bug/aura-combo-green-20260925.log`: full startup → `TargetUnit('player')` → tick, and the original PlayerFrame `PLAYER_ENTERING_WORLD` → ComboFrame `3 → 0` CVar pre-`OnLoad` regression. Final local GUI evidence observes that CVar reporting `0`, with zero hook/final errors; see [[final-unit-frame-click-aura-proof]]. No native-client conformance or original screenshot-exact reproduction is claimed. The target-aura path is the demonstrated consumer; this page does not cover the separate Count root cause.

## Sources

- [secret-number ordering spec](../../specs/secret-number-ordering.md) — pinned revision, consumer path, contract, and bounded acceptance
- [Cargo manifest](../../../Cargo.toml) — simulator dependency pin

## See Also

- [[ellesmereui-forever]] — related Forever aura-secret display investigation
- [[lua51-unknown-escapes]] — another published rilua compatibility boundary
- [[target-aura-private-count]] — separate private-Count root cause on the integrated path
- [[final-unit-frame-click-aura-proof]] — final bounded GUI and verifier evidence
