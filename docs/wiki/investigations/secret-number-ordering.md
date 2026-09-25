# Secret-number ordering

Rilua pin `9ad8720b10ef7ed7b855fe741a6368c631af0dc2`, published from `osso/host-secret-bool`, supplies a narrow VM ordering boundary required by Forever target-aura layout. This is inferred simulator compatibility policy, not native-verified secret-value semantics.

## Root cause

`TargetFrameAuraFlowLayoutMixin:OnLayoutComplete` stores `secretwrap(lineCount)`. Unchanged Blizzard `TargetFrameMixin:ShouldAnchorSpellBarToAuraContainer` subsequently compares that wrapped value with public zero or two. Before the pin, the comparison reached the ordinary number/userdata error boundary.

## Contract

Guarded VM numeric ordering supports secret-wrapped numbers against public or secret-wrapped numbers through `<`, `<=`, `>`, and `>=`. The guard rejects inspection from tainted callers, including a tainted ancestor that reaches untainted code. Public numeric ordering remains unchanged; nonnumeric wrappers remain invalid for ordering; secret arithmetic and equality remain opaque.

## Evidence and limits

The dependency publication reports its focused table-security proof, but independent dependency verification and simulator integration remain pending. There is no clean tick, GUI, or native-client conformance claim. The target-aura path is the demonstrated consumer; this page does not cover the separate Count specification.

## Sources

- [secret-number ordering spec](../../specs/secret-number-ordering.md) — pinned revision, consumer path, contract, and pending acceptance
- [Cargo manifest](../../../Cargo.toml) — simulator dependency pin

## See Also

- [[ellesmereui-forever]] — related Forever aura-secret display investigation
- [[lua51-unknown-escapes]] — another published rilua compatibility boundary
