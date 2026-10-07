# Patch 11.1.5 publication sweep

## Contract

Probe all 125 inventory occurrences from page 621744, revision 6726775 in unmodified cached Game UI. Default retail carries 12.1.0, not a reconstructed 11.1.5 client. Apply all later registers chronologically: 11.1.7, 11.2.0, 11.2.5, 11.2.7, 12.0.0, 12.0.1, 12.0.5, 12.0.7 and 12.1.0. Latest add/remove wins; changed rows preserve publication. Retain original direction and supersession ID.

Exact non-OK IDs must equal the reviewed known-gap fixture. `P1115_SWEEP_OUT` writes every observation before assertion; `P1115_SWEEP_REGISTER` allows a full same-sized negative control. Publication/absence does not establish signatures, outputs, security, behavior or native parity. Current consumers take precedence over historical removals: retain UpdateUIParentPosition, defined in cached `Blizzard_UIParentUtil/UIParentUtil.lua:13` and called in `Blizzard_Game/Shared/EventImplementation.lua:48,304`. Never delete cached deprecation wrappers.

## Bounded behavior

Seven retired namespace members must remain nil under repeated ordinary/raw lookup. Current cached retail Lua has no references to the three old GameEnvironmentManager members, three old GameModeManager members or SpellBook.GetTrackedNameplateCooldownSpells. Existing retail epoch gates preserve classic publication.

IsInGlobalEnvironment must report actual caller environment: true in global chunks, false in secure/custom environments, true after SwapToGlobalEnvironment. Custom-environment tail calls must retain the same result. No missing-caller or generic namespace placeholder may substitute for that behavior.

## Acceptance

- [x] Retain source/provenance, register, exact fixture and per-ID gap review.
- [x] Extract non-inventory rows without expanding the symbolic TOC interface.
- [x] Account for all 224 unique IDs and assign every extract row once.
- [x] Run ten isolated sweeps, negative control and bounded behavior tests.
- [x] Run prefork consumer regression, formatting, Mists check and startup [].

## Local proof

[Proof ledger](../../data/patch-api/evidence/11.1.5-session-2026-10-06/p1115-proof.json) retains exact commands/revisions. Sweeps run at `1fb6571e1`; checks/build/startup at `bc72538b5`; subsequent changes are evidence/docs only. Each sweep runs alone with `--nocapture --test-threads=1`.

| Sweep | Rows | OK | Exact gaps | Result |
|---|---:|---:|---:|---|
| 11.1.5 | 125 | 89 | 36 | PASS |
| 11.1.7 | 48 | 40 | 8 | PASS |
| 11.2.0 | 162 | 135 | 27 | PASS |
| 11.2.5 | 163 | 118 | 45 | PASS |
| 11.2.7 | 508 | 414 | 94 | PASS |
| 12.0.0 | 1010 | 989 | 21 | PASS |
| 12.0.1 | 225 | 222 | 3 | PASS |
| 12.0.5 | 363 | 352 | 11 | PASS |
| 12.0.7 | 174 | 171 | 3 | PASS |
| 12.1.0 | 778 | 773 | 5 | PASS |

Initial sweep: 81 OK / 44 gaps; eight closures, 36 retained. Two new behavioral tests pass after RED. First Lua environment implementation fails a custom-environment tail call; native caller observation fixes it without weakening the test. Existing environment regression and one isolated `prefork_full_ui` EditMode consumer case pass. No existing prefork case names the seven retired members; the new repeated-lookup test covers their exact boundary.

Negative control changes only C_ChatInfo.DropCautionaryChatMessage added → removed: expected exit 101, exactly one new gap, no resolutions (36 → 37). Thirteen extractor/register fixtures pass. `cargo fmt` and `cargo fmt --check` pass. Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` passes without non-vendor warnings; six pre-existing iced manifest warnings plus summary remain unsuppressed. Separate default retail build and bounded exit-0 startup return `[]`. Changed Rust lines reviewed locally for readability.

All ten registers regenerate byte-identically. Thirty-six later source/register/coverage/fixture files remain unchanged against `f3c07b9bd`. Seven later observations equal earlier 11.1.7 evidence; two differences are pre-existing Browser:NavigateTo retention and two rilua secret-helper closures, not this audit's changes.

Coverage: 125 inventory + 99 extract = 224 IDs; 73 partial-development-green, 15 bounded-coverage, 121 audit-pending, 15 metadata-only. One later reversal is metadata-only; 15 removals establish strict absence/registration rejection, zero deprecated-wrapper acceptances. Eighty-five extract contracts remain pending; 14 editorial rows receive no runtime credit. Ledger remains in-progress; page-accounting/publication audit complete, not historical/native API completeness.

[Artifact validator](../../data/patch-api/evidence/11.1.5-session-2026-10-06/p1115-validate.py) checks hashes, expectations, exact gaps, credit, allocation, proof summaries and preserved inputs. Raw Cargo logs are ignored local artifacts; retained result/summary files carry the portable evidence.

## Process exception

[Pyrun cwd incident](../../data/patch-api/evidence/11.1.5-session-2026-10-06/p1115-cwd-incident.json): first source capture/commit accidentally used canonical cwd. Exact commit moved to required worktree; canonical restored to clean original master before any tests. Subsequent commands explicitly select worktree cwd. This violates the never-touch-canonical process constraint despite restored final state. No agents/models, push, merge, vendor edits or Blizzard monkey-patching.
