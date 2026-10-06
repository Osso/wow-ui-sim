# Patch 11.2.5 publication sweep

## Contract

Probe all 163 consolidated inventory occurrences from page 641912, revision 6726772, in unmodified cached Game UI. Publication/absence only; no signatures, outputs, security, behavior or native parity claim. No 11.x Cargo feature exists. Default retail carries 12.1.0; apply 11.2.7, 12.0.0, 12.0.1, 12.0.5, 12.0.7 and 12.1.0 chronologically. Latest later add/remove wins; changed rows preserve publication. Source directions and supersession IDs remain recorded. 11.2.7 already excludes older registers.

Use `tests/common/publication_sweep.rs`; exact observed gap IDs equal `tests/data/patch_11_2_5_sweep_known_gaps.json`. `P1125_SWEEP_OUT` writes every observation before assertion; `P1125_SWEEP_REGISTER` accepts a full same-sized negative-control register. One altered unsuperseded row must add exactly one gap. Retail-only file gating; any retirement must preserve classic contracts and supported retail epochs. No vendor rewrite, fabricated models, or concurrent 11.2.7 producer edits.

## Acceptance

- [x] Review every non-OK row; fix cheap root causes and retain explicit model gaps.
- [x] Exact gap baseline and one-row negative control.
- [x] Unique inventory/extract ledger and complete extract scout.
- [x] Seven isolated sweeps, local debug retail.
- [x] Format, retail check, Mists test check without non-vendor warnings, startup `[]`.

## Location preference publication fix

Both location visibility globals already have a state-backed implementation, synchronous event notifications, strict boolean/secret-caller validation and per-environment isolation. Their module and registration were gated to Forever only despite the 11.2.5 introduction. Publish that unchanged model from `retail-12-0-0` as well; classic builds retain absence, Forever retains its existing contract. Reuse the five behavioral tests under the same widened gate. RED: all five fail on missing globals. No recent-allies namespace producer changes.

## Extract seeding

Treat the four nested Deprecated resource links and the 11.2.0→11.2.5 build comparison as editorial for this page only. Keep the socketing relocation summary and all enum/structure parent/member statements pending. Existing extracts and coverage ledgers are never rewritten. Initial sweep: 116 OK / 47 gaps; two globals close through existing modeled state, 45 explicit member/3D gaps remain.

## Final local proof — 2026-10-06

Runtime/test/extractor revision `57b50a24cab703d3f006485ff7e28f215b90c33a`; subsequent coverage/evidence/docs changes do not invalidate these commands. [Proof ledger](../../data/patch-api/evidence/11.2.5-session-2026-10-06/p1125-proof.json) retains commands, revisions, exit codes and local log paths. No independent agent/model/native acceptance.

| Isolated sweep | Rows | OK | Exact gaps | Result |
|---|---:|---:|---:|---|
| 11.2.5 | 163 | 118 | 45 | PASS |
| 11.2.7 | 508 | 387 | 121 | PASS |
| 12.0.0 | 1010 | 987 | 23 | PASS |
| 12.0.1 | 225 | 222 | 3 | PASS |
| 12.0.5 | 363 | 352 | 11 | PASS |
| 12.0.7 | 174 | 171 | 3 | PASS |
| 12.1.0 | 778 | 773 | 5 | PASS |

All seven run alone, local debug retail, one filter/process, `--nocapture --test-threads=1`. All six later observation maps are identical to retained 11.2.7-session proof; existing known-gap fixtures/registers/coverage ledgers unchanged. 11.2.7 already lists only newer registers; no supersession wiring change was needed. Four 11.2.5 expectations reverse: two CVars removed by 11.2.7, one shop member removed later and one catalog event removed later. All four observe absence. No page CVar default mismatches.

Every initial non-OK row reviewed: 47 → 45 after two modeled-global gate fixes. Retained gaps: 42 namespace members and three intentionally unsupported 3D methods. One-row negative control flips `C_AddOns.GetAddOnName` added → removed; expected exit 101, 45 → 46 gaps, exactly one changed observation/new ID, none resolved.

Removed-publication policy is not always strict absence: seven of the 22 source removals observe absence; fifteen globals retain exact cached deprecation aliases. Their identity/source attribution satisfies the existing shared sweep policy but proves neither successor behavior nor historical/native parity. The ledger records this distinction explicitly.

Coverage ledger: 163 inventory + 73 extract = 236 unique IDs; 92 partial-development-green / 22 bounded-coverage / 103 audit-pending / 19 metadata-only. Extract: 58 pending / 15 editorial, all assigned across five ranked scout batches. Existing six registers regenerate byte-identically; all eight header counts exact.

Location preference tests: five RED (nil globals), then five GREEN after producer/state gate alignment; intermediate compile failure is retained in the proof ledger. Extract fixtures seven PASS (new editorial RED captured first); register fixtures two PASS; new plaintext reproduction PASS. Formatting, local retail check and Mists test check PASS. Mists emits no errors or non-vendor warnings; six pre-existing iced manifest-key deprecations plus vendor summary remain unsuppressed. Changed Rust readability manually audited: new sweep is one spec construction; other Rust edits only gate existing fields/modules/tests.

Startup build separate, no timeout. Bounded `timeout 90` local `--no-build --run -- --no-addons --no-saved-vars lua-errors` exits 0, JSON `[]`, CLEAN / zero unique and zero occurrences. No release/extra target directory/vendor edit/Blizzard monkey-patch/concurrent namespace producer edit/push/merge.
