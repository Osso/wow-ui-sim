# Retail Patch 5.4.7 API audit

Audit Warcraft Wiki pageid 549368 at revision 5298169, from the 2014 retail Mists line. Its linked patch page revision 6854922 states TOC 50400 and February 18, 2014 release. The API page itself states build 5.4.7.17956, not an interface number. See [audit](../wiki/investigations/patch-5-4-7-api-audit.md).

## What it must do

- [x] Account for three added functions and six added events, with exact current publication/absence observations and reviewed gaps. Apply the retail later-register chain oldest first; reserve 5.4.8 then redirect-only 6.0.1 before 6.0.2. Never include Mists Classic 5.5.x.
- [x] Retain all three chat-prose statements and two diff captions without treating uncertainty or build metadata as implemented behavior.
- [x] Prove the unmodified cached `BNSendGameData` wrapper preserves a real outbound intent for an online game account, drops the namespace status return, and appends nothing after that account goes offline. These are bounded existing simulator policies, not network delivery or native 2014 limits.
- [x] Reuse existing specialization catalog behavioral tests rather than duplicating them; distinguish those contracts from function publication.
- [x] No runtime retirements without a page removal, whole-word cached/caller scans, and pinned master/queued re-addition checks.
- [x] Keep shared proof inputs pinned to recorded revisions; derive register and sweep scope from historical Git inventories. Validate in clean and synthetic-future checkouts without ignored inputs.

## How it works

- [Audit and proof boundaries](../wiki/investigations/patch-5-4-7-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tests/patch_5_4_7_publication_sweep.rs` — prefork inventory probe using the existing common classifier.
- `tests/patch_5_4_7_behavior.rs` — cached legacy sender with state changes and observable intent records.
- `data/patch-api/sources/5.4.7-*` — pinned source, extract, register and accounting.
- `data/patch-api/evidence/5.4.7-session-2026-10-08/` — commands, scans, receipts and portable validator.

## Tests asserting this spec

- `tests/patch_5_4_7_publication_sweep.rs` and `tests/patch_5_4_7_behavior.rs`.
- `tests/patch_11_1_0_publication_fixes.rs::patch_11_1_0_specialization_names_use_catalog_identity` — existing concrete catalog identity and unknown-ID behavior.
- `tests/p1200_rest.rs` — existing namespace Battle.net outbound records.

## Known gaps (current cycle)

- [ ] CHAT_MSG_ADDON sender arg4: existing legacy synthetic echo uses the recipient, not a name-realm inbound sender; namespace outbound intent does not constitute inbound delivery.
- [ ] CHAT_MSG_WHISPER author arg2: no modeled incoming whisper producer or network transport; registration is not author normalization.
- [ ] Other CHAT_MSG_* realm-name claim is explicitly unchecked source speculation; no concrete event list or parity claim.

## Out of scope

Historical native transport/rate limits, auth/account-upgrade services, remote whisper delivery, arbitrary payload injection as a substitute for real producers, Classic 5.5.x audits, vendor edits, shims, full integration suites, push and merge.
