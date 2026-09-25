# Secret-number ordering

## What it must do

- [x] Let untainted Blizzard Lua compare secret-wrapped numbers with public or secret-wrapped numbers using `<`, `<=`, `>`, and `>=`.
- [x] Reject inspection by tainted callers, including tainted ancestors calling untainted functions.
- [x] Preserve public-number ordering and existing secret arithmetic, equality, and nonnumeric-wrapper boundaries.
- [ ] Run unchanged Forever target-aura layout after targeting without a number/userdata comparison error.

## Evidence and limits

Forever `TargetFrameAuraFlowLayoutMixin:OnLayoutComplete` stores `secretwrap(lineCount)`; `TargetFrameMixin:ShouldAnchorSpellBarToAuraContainer` compares that value with zero or two. After XML child publication was corrected, this path exposed missing numeric ordering in rilua.

Numeric ordering is inferred simulator compatibility policy, not native-verified secret-value semantics. This change does not implement general secret propagation or arithmetic.

## Implementation and proof

- Pinned rilua `9ad8720b10ef7ed7b855fe741a6368c631af0dc2` implements the guarded VM ordering boundary. Its `docs/specs/table-security.md` owns the VM contract.
- Dependency implementation reports two new tests passing and 14 focused table-security tests passing; independent verification and simulator integration remain pending.
- Simulator behavior belongs in the existing grouped click-targeting regressions, followed by owned-process GUI testing. No vendor Lua changes or getter-unwrapping workaround.
