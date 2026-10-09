# Bounded historical retail 2.4.2 handoff

Branch `p242-source`; isolated `/home/osso/.worktrees/wow-ui-sim-p242-source`; base `f95eed96e`. No push/merge/deploy/delegation/model CLI or final gates. Main owns integration and native/current loaded-Game acceptance.

## Source and accounting

Frozen page 81145/revision 803933, **2021-12-28T02:03:05Z revision timestamp**, **2008 retail** history. Full pinned finite registry: 101 identities through 1.0.0. Manifest, response and raw hashes are in `source-pin.json`; response/raw identity asserted. Source copies/register/extract/provenance under `data/patch-api/sources/2.4.2-*`.

53 unique ledger IDs: 22 exact nonblank raw rows, 11 publication occurrences, 11 separate signatures, six constants, three context API references. Statuses: **2 bounded / 44 pending / 7 metadata**. Five editorial headings, no numerical count headers; no explicit event/CVar/console-command inventory. No native parity or linked-page expansion.

## Supported measurement

Own `patch_2_4_2_factory` target with `--no-default-features --features client-retail`: two published widget methods, nine missing globals; **2/2 tests pass with reviewed gaps**. This is bare factory, not modern cached Game (legacy wrappers not loaded). Current `C_CurrencyInfo.GetCoinText` concrete amounts/separators pass independently; no historical alias/native credit or new runtime model. Nine factory gaps and all semantic/signature limits remain precise in the ledger. Source lacks addToStart position/default, strreplace semantics, grant/quest/roster/resolver backing contracts; no guess/shim installed.

Negative control substitutes an actually published widget method: **9 → 10**, exact new `p242-negative-control`, exit 101. Initial weaker missing-global replacement retained separately. No retirements; no affected runtime callers to migrate.

## Portable originals and later receipts

`historical-manifest.json`: **23 original file seals / 2,229 hashed archived blobs**. Archive 4,287,031 bytes. Original ledger/gaps/source pins/registry, parsers, runtime code/build/test inputs and ignored-but-force-tracked owned logs retained. Validator anchors manifest SHA-256 `dae8a4298ab6d31fc72cd2a6c9e6613e1043dab0316707e0f0378e00ff50309b`; validator SHA-256 `44d7fd119d6d5c4836613e5ccd1d9491a3e979b5f5728be4b6c1fa0cc120c244`.

Fresh evidence-only copy with `PATH=/nonexistent` passes 1/1; no Git, target, current mutable source/UI cache/vendor inputs. Serialized ledger, receipt log and compressed archive tampering each reject at the exact seal; originals restored and identical summary verified. Archived Rust is provenance, not rebuilt by replay. `closures/` is reserved for separate future closure evidence; do not rewrite originals. `portable-proof/` retains subsequent receipt/summary/seals separately.

Own source tests 2/2 and accounting/omission tests 2/2 pass. Differential replay preserves 151 old/new default source controls and 78 existing recorded-option controls; inherited errors and saved-output mismatches remain unchanged (listed in wiki/summary), not repaired or falsely claimed reproducible.

## Successors

Only applied: actual retail 3.2.0/3.3.0/3.3.3/3.3.5/4.0.1. 3.2.0 changes GetQuestLogSpellLink; no add/remove supersession. Other four have no own-symbol overlap.

Queued 3.0.2/3.0.3/3.0.8/3.1.0 frozen sources have zero exact own-symbol overlap; main supplies actual registers at integration. `queued-overlaps.json` is retained original substring scout; authoritative exact review is `overlap-review.json` (GetCoinTextureString ≠ GetCoinText). No Classic 2.5.x, Era or Wrath 3.4.x credit.

## Commands/revisions

| Proof | Revision | Scope/result |
|---|---|---|
| Source fixtures | 040c37714 | Own frozen source RED → 2/2 GREEN |
| Factory/model | acf89c976 | Own retail target RED → 2/2 GREEN, nine remaining gaps |
| Negative | acf89c976 | Own factory publication filter only; 9 → 10, exit 101 |
| Source accounting | c8bdc1c1c | 2/2 GREEN, omissions reject |
| Portable replay | 576d4dcca / fbf655ef9 | 1/1; later fixture only exposes already-asserted summary |

Every native command used owned CARGO_TARGET_DIR and explicit worktree cwd. Command ledger has exact discovery commit, argv, exit and proof scope. Rust test formatted before commit; whole-file client-retail cfg protects aggregate includes. Six inherited iced manifest deprecations and six inherited headless non-vendor warnings retained without suppression/adjacent repairs. No broad/check/lint/readability/profile/startup/full-suite/final gates run. Subsequent docs/receipt changes do not invalidate code-scope proof.

See [audit](../../../../docs/wiki/investigations/patch-2-4-2-api-audit.md), [spec](../../../../docs/specs/patch-2-4-2-publication-sweep.md), `portable-proof/summary.json` and `command-ledger.json`.
