# Bounded 4.3.0 integration verification

OVERALL: PASS for assigned source/accounting/successor-integration scope. Not native/full-suite/CI acceptance.

Worktree: `/home/osso/.worktrees/wow-ui-sim-p430-source`
HEAD: `4eaadd2333b655426237fa154bdf42a221b2bde4`; observed `git status --short` empty. Session cwd unchanged; no cwd-switch tools used.

## Coverage matrix

| Claim | Evidence | Result / proof boundary |
|---|---|---|
| Source identity | Pin pageid 167555, revision 1639407; response and raw-source SHA256 match; response content exactly equals committed wikitext | PASS; local pinned evidence, no new network/native proof |
| Inventory/accounting | Register header counts 76 added + 7 removed = 83; result IDs and section/direction/symbol identities match exactly; 58 `ok`, 25 gaps exactly equal reviewed fixture | PASS |
| Ledger | 84 rows: 58 bounded-coverage, 25 audit-pending, 1 metadata-only; every note nonempty; bounded credit exactly `current-retail-publication-or-absence`; register SHA256 matches ledger | PASS; zero behavior/signature/security credit |
| Actual 4.3.4 successor | `tests/patch_4_3_0_publication_sweep.rs:19` includes existing register first; 4.3.4 revision 3743181 has 10 additions/1 removal; symbol intersection with 4.3.0 is empty | PASS; actual include cannot change these 83 expectations |
| Classic isolation | 65 included successors exist, all default retail client line; no 5.5.x included; test is client-retail gated at line 2; common helper lines 386–398 explicitly skips mismatched client lines | PASS; no Classic behavior claim |
| Test reachability | build.rs:463–497 discovers top-level Rust modules; build.rs:114–123 generates prefork registry; tests/prefork_full_ui.rs:13,188 includes generated modules/registry; retained runtime output names the 4.3.0 and 4.3.4 cases | PASS |
| Runtime-equivalence to main f0baf34d4 | `git diff --name-only f0baf34d4 HEAD -- src Cargo.toml Cargo.lock build.rs Interface tests/common` empty; prefork harness diff empty | PASS; this change adds audit data/tests/docs, not runtime/vendor behavior |
| Integrated publication receipt applicability | Receipt start revision 288bb3af3981f0039a502ac053f4203eee077570; diff to HEAD empty for src/tests/tools/Cargo.toml/Cargo.lock/build.rs/data/patch-api/sources | PASS; rebase does not invalidate covered code/source scope |
| Existing publication run | `/tmp/p430-integrated-publication-result.json`: exit 0; exact stdout: `test result: ok. 67 passed; 0 failed; 67 total` | PASS; retained receipt, not rerun |
| Current negative control | Start receipt names exact current HEAD; result exit 1; exact stdout: `test result: FAILED. 0 passed; 1 failed; 1 total`; output gap IDs 25→26, sole extra `wt-global-api-CanReplaceGuildMaster-12` after fabricated `P430MissingPublicationProbe`; no gap removed | PASS for expected rejection; assertion failure at tests/common/publication_sweep.rs:544 |
| Current/historical 4.3.4 coexistence | Historical own/discovery outputs: 11 rows, 7 gaps. Current integrated output: 11 rows, 6 gaps; GetSessionTime row passes. Current negative: 7 gaps with GetSessionTime still passing. Current known-gap fixture has 6 IDs | PASS; immutable seven-gap pre-model evidence is not current six-gap accounting |
| Fresh formatting | Exact command `cargo fmt --check`, cwd named worktree, exit 0, empty stdout/stderr | PASS; formatting only, not compilation |

## Historical proof applicability

Read `data/patch-api/evidence/4.3.0-session-2026-10-09/validate.py`: explicitly a recorded-development validator, not current-head acceptance. It verifies source reproduction, source/response/log/result seals, historical tree identities, and reviewed accounting. It does not require historical sweep bytes to equal the current successor-wired test.

All five retained `.proof.json` log seals pass; retained result seals pass wherever present. Accounting log exact output: `Ran 8 tests in 0.005s` / `OK`; tree-pin log: `Ran 2 tests in 0.157s` / `OK`. Current tree-pin input hashes match. Historical discovery/reviewed sweep source hashes differ from current test, expected after the single-line actual successor replacement; those runs remain historical, not fresh current-code proof. Integrated 83-row observations exactly equal historical reviewed observations, independently consistent with the empty 4.3.4 symbol intersection. User reports prior agent187 validator PASS; this retry does not rerun or independently claim that invocation. Main portability run left untouched.

## Rust readability / artifacts

Read rust-readability skill and audited only actual successor include at line 19. No readability violations: literal compile-time include, meaningful source filename, existing oldest-first list position, no branches/state/warning suppressions. TODO/FIXME/HACK/XXX/allow markers: zero on added line. No speculative changes warranted.

[EXIST] PASS — source/register/ledger/known-gap fixture/sweep/validator all read in named worktree.
[SUBSTANTIVE] PASS — real 83-row register, 84-row ledger, 87-line sweep, 139-line validator.
[WIRED] PASS — generated harness and retained runtime case output establish reachability.
[ANTI-PATTERN] PASS — new one-line include contains none of required markers.

Historical spec/wiki prose still describes queued placeholder/coordination boundaries; literal current Rust uses actual include. Treat that prose as historical context, not current source availability evidence.

## Fresh command receipts / exclusions

`/tmp/p430-integration-verifier-fmt-start.json`
`/tmp/p430-integration-verifier-fmt-result.json`
`/tmp/p430-integration-verifier-fmt.log`

Fresh formatting ran once only. Captured result was empty stdout/stderr, exit 0. A subsequent attempt to retrieve result via `ctx.p430_fmt` failed (persistent context key unavailable); receipt/log reconstructed from exact already-observed CommandResult, no command rerun.

Retained test stderr contains six iced_wgpu manifest deprecated lint-key warnings (large-enum-variant, map-entry, match-wildcard-for-single-variants, redundant-closure-for-method-calls, trivially-copy-pass-by-ref, type-complexity). No zero-warning claim.

No edits/commits/delegation/Bash/cargo check/build/broad test reruns. Only /tmp report/receipt artifacts written. No native compatibility, full-suite, CI, portability completion, historical source-byte compilation, output/security/signature parity, or final project acceptance claimed.
