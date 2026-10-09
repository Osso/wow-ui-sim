# Historical retail Patch 3.1.0 source/publication audit

Frozen page 233195, revision 2259594, timestamp 2013-05-22T17:34:42Z. [Implementation, receipts and exact proof boundaries](../wiki/investigations/patch-3-1-0-api-audit.md).

## What it must do

- [x] Pin supplied response/body against the committed legacy manifest; historical 2009 retail is separate from Wrath Classic 3.4.x.
- [x] Preserve default generator bytes; `--legacy-function-labels` opts into labeled NEW/UPDATED/REMOVED rows and explicit addenda without promoting questions to publication contracts.
- [x] Account for every nonblank source line, full extract row and literal parenthetical fragment, preserving return prefixes, malformed syntax and precise UNPROVEN limits.
- [x] Retain own cached-retail observations, exact known gaps and fabricated-global negative controls; publication never proves native/model parity.
- [x] Expose the existing real player-orientation getter on retail; configured 0, π÷2 and π radians read unchanged as one result. No native PTR correction, unavailable-state or movement-production claim.
- [x] Preserve sealed original source/ledger/gap/receipt snapshots; reproduce them in a relocated fresh process without Git, target or current mutable files.
- [x] Reject original source/log/ledger/gap/archive/manifest tampering and ignore synthetic future closures; reproduce original and current ledgers byte-identically in disposable roots.

## How it works

- [Source accounting, bounded model and archive](../wiki/investigations/patch-3-1-0-api-audit.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in plain function labels and supplemental occurrences; defaults unchanged.
- `tools/build_patch_3_1_0_accounting.py`: original source-only accounting by default; `--current-radians` requires separate current observations and bounded state proof.
- `src/lua_api/state_types/character_world.rs`: existing nullable player-orientation field shared in player state, radians documented; initial unknown is simulator policy.
- `src/lua_api/globals/real/player_facing.rs`, `real/mod.rs`, `globals/register.rs`: existing real getter now registered for retail; existing WowForever-only admin setter unchanged.
- `tests/patch_3_1_0_publication_sweep.rs`: own publication and concrete radians-state cases.
- `data/patch-api/sources/3.1.0-*`: source/register/full extract/current ledger/signature fixtures.
- `data/patch-api/evidence/3.1.0-session-2026-10-09/`: immutable original historical archive/validator; separate `current/` receipts.

## Tests asserting this spec

- `tools/test_patch_3_1_0_source.py`: 3/3 parser/default restoration and disposable original/current ledger byte-reproduction cases.
- `tests/patch_3_1_0_publication_sweep.rs`: publication GREEN 1/1 and configured-radians GREEN 1/1; current negative rejects 59 → 60 gaps.
- `tools/test_patch_3_1_0_validator.py`: 2/2 relocated no-Git/no-target historical replay/tamper/future-current isolation cases; original 61 → 62 negative retained.

## Integration status — 2026-10-09

Integrated at `614402d56`, pushed (main handoff). Actual retail 3.2.0, 3.3.0, 3.3.3, 3.3.5 and 4.0.1 successors are applied; `IsPlayerResolutionAvailable` is removed from the current gap list. [Current bounded proof and retained reports](../wiki/investigations/patch-3-1-0-api-audit.md#main-successor-integration--2026-10-09) are the status SSOT: retail 2/2 observes 52/58 at `614402d56`, default/Mists checks exit 0 with six inherited iced warnings, startup `[]` exit 0 and negative 58→59; source 3/3 at `6a84c2b09`. Full suite at `614402d56` FAIL: 23 integration, one Garrison prefork, six lib failures, same sets as `c17f1b4bb`, zero new delta. 74 exact sweeps plus two other passing cases are not acceptance. Original 49/61 and implementer-current 51/59 remain immutable historical receipts; no broader latest-HEAD proof. The getter reuses the nullable facing-radians read, with no input producer, native default or native parity credit.

## Known gaps (current cycle)

- [x] Bounded integrated publication/configured-radians verification at `614402d56`; historical 51/59 receipts preserved. No placeholder or Wrath Classic supersession credit.
- [ ] 51 raw prose/context rows remain pending; 31 literal fragments have no modeled credit. Two player-facing fragments have only configured-current-state credit and still retain native/historical limits. All full native contracts remain UNPROVEN.
- [ ] Historical talent/glyph group/preview state, controller-token aura ordering/legacy tuple, secure restoration/hover cancellation/noncombat lifecycle, item-location table encoding/mutation, slash-command/macro dispatch and native PTR corrections remain unsupported.
- [ ] Broader profile-runtime, latest integrated scope, affected callers and final acceptance remain main-owned. Bounded checks/startup pass; retained full suite fails, not acceptance.

## Out of scope

Linked secure guide/API pages, native 2009-client execution, movement/input producer, other-unit facing, invented native defaults/coercion/security, compatibility shims/fallbacks, retirements and other runtime changes. No vendor/cache edits, rebase/integration, delegation, push, merge or deployment in this slice.
