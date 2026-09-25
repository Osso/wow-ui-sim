# Secret-number ordering

## What it must do

- [x] Let untainted Blizzard Lua compare secret-wrapped numbers with public or secret-wrapped numbers using `<`, `<=`, `>`, and `>=`.
- [x] Reject inspection by tainted callers, including tainted ancestors calling untainted functions.
- [x] Preserve public-number ordering and existing secret arithmetic, equality, and nonnumeric-wrapper boundaries.
- [x] Run bounded full-startup Forever target-aura layout after targeting without a number/userdata comparison error.

## Evidence and limits

Forever `TargetFrameAuraFlowLayoutMixin:OnLayoutComplete` stores `secretwrap(lineCount)`; `TargetFrameMixin:ShouldAnchorSpellBarToAuraContainer` compares that value with zero or two. After XML child publication was corrected, this path exposed missing numeric ordering in rilua.

Numeric ordering is inferred simulator compatibility policy, not native-verified secret-value semantics. This change does not implement general secret propagation or arithmetic.

## Implementation and proof

- Pinned rilua `9ad8720b10ef7ed7b855fe741a6368c631af0dc2` implements the guarded VM ordering boundary. Its `docs/specs/table-security.md` owns the VM contract.
- Independent rilua verification records 21/21 checks plus `cargo fmt` and `cargo check` in `/tmp/wow-unit-frame-bug/rilua-verify-ledger-20260925.json`; the inherited `strlen` warning remains outside this slice.
- Simulator integration is bounded 2/2 with zero Lua errors in `/tmp/wow-unit-frame-bug/aura-combo-green-20260925.log`: full startup → `TargetUnit("player")` → tick, plus the original PlayerFrame `PLAYER_ENTERING_WORLD` → ComboFrame `3 → 0` CVar pre-`OnLoad` regression. GUI/final simulator verification remains pending. No vendor Lua changes or getter-unwrapping workaround.
