# Patch 1.60.1 full literal SOURCE accounting

Frozen page 707613/revision 6902509 (`2026-10-07T05:30:51Z`) lives in `data/patch-api/source-cache/legacy-2026-10-09`. The literal page identifies Forever/Camelot and TOC 16001, not an epoch inferred from the version number. [Audit/proof matrix](../wiki/investigations/patch-1-60-1-api-audit.md).

## What it must do
- [ ] Validate exact response/body bytes and manifest membership against the 101-page registry ending at 1.0.0, entirely offline.
- [ ] Retain every nonblank row, inventory occurrence, signature/unspecified signature, prose description, reference boundary and heading/count. Preserve literal duplicates, misspellings and count disagreements without inferred aliases or repairs.
- [ ] Reproduce the shared generator's unchanged default bytes with no flags; keep the literal non-inventory mirror explicitly unexpanded, not rendered MediaWiki evidence.
- [ ] Preserve immutable source-only seals below 5 MB per file; reject missing/altered ledger rows, false/foreign proof, disk ledger/log tampering and replay copied without Git/target/current tools.
- [ ] Separately measure the existing configured player-name read before login under `client-wowforever`, if the offline bounded target can run. Do not infer corrected return tuples or token aliases.

## How it works
- [Source accounting and limits](../wiki/investigations/patch-1-60-1-api-audit.md)
- [Portable evidence](../../data/patch-api/evidence/1.60.1-session-2026-10-09/)

## Implementation inventory
- Evidence `accounting.py`: per-occurrence register enrichment, complete source/signature/prose/reference/header ledgers and literal mirror.
- Evidence `validate.py`: exact portable historical replay and original byte seals.
- Evidence `historical-gen_patch_wikitext_register.py`: immutable existing default parser; no shared-parser changes.
- Evidence `original/`: exact source/manifest/registry/register/ledger/gaps, initial RED and sparse code snapshots.
- `patch-tests/patch_1_60_1_source_model.rs` and explicit Cargo test target: existing-model player-state read only, no runtime implementation changes.

## Tests asserting this spec
- Evidence `test_source.py`: eight owned fixtures, per-row omission controls, duplicate/malformed input, literal metadata/signatures, unexpanded links, foreign-credit rejection and copied disk tamper/restoration.
- `patch-tests/patch_1_60_1_source_model.rs`: two changing names and an isolated second environment, no PLAYER_LOGIN dispatch or loaded UI.

## Known gaps (current cycle)
- [ ] All 1,876 inventory occurrences, 1,712 signature occurrences and 211 prose limits are behaviorally UNPROVEN in original source accounting.
- [ ] Native/historical/security and loaded-UI parity are not proved by source publication or matching interface values. Source build 70205 differs from configured compatibility build 69977.
- [ ] Main owns actual same-line successors, integration and native/final acceptance. Literal `next=1.60.2` is navigation only; no successor expansion or wholesale Retail/Era/TBC supersession.

## Out of scope
Network, production runtime changes, shims/coercion/defaults/fallbacks, vendor/cache/Wowless/canonical writes, rebase/push/merge/deploy/delegation/model CLIs, startup, check/lint/readability/coverage/broad tests and final gates. PLAN remains unstaged under the governing plan-md policy.
