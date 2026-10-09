# Historical retail Patch 2.2.0 source accounting

Account the frozen 2007 retail page, not Classic 2.5 or modern retail runtime behavior. [Audit](../wiki/investigations/patch-2-2-0-api-audit.md) records scope and proof boundaries; [literal ledger](../../data/patch-api/evidence/2.2.0-session-2026-10-09/historical-page-coverage.json) retains every source row.

## What it must do

- [ ] Validate page 184149/revision 6428980, timestamp 2025-08-03T13:04:54Z, exact response/body hashes and manifest-linked 101-page registry ending at 1.0.0.
- [ ] Account every API/widget/script/setting/event-family occurrence, header, prose/example and default extract row; never expand linked contracts or infer concrete event members.
- [ ] Preserve optional/split signature fragments, literal returns, modified-click defaults and source replacement claims without native/model credit.
- [ ] Reject omissions, fabricated signatures, proof credit and foreign/queued supersession.
- [ ] Replay compact sealed source/history without Git, target, network or mutable later state; reject serialized ledger/log tampering and restore original bytes.

## How it works

- [Historical source audit](../wiki/investigations/patch-2-2-0-api-audit.md).

## Implementation inventory

- `data/patch-api/evidence/2.2.0-session-2026-10-09/validate.py` — own literal accounting and frozen replay.
- Same directory: raw response/body, manifest/registry, copied unchanged tools, default outputs and ledger.

## Tests asserting this spec

- Owned `test_source_accounting.py` — omission, signature, client-history and zero-credit controls. Development RED recorded; GREEN pending at initial implementation commit.

## Known gaps (current cycle)

- [ ] Main owns runtime/native measurement, actual later-retail integration and final gates. No new runtime proposal or model credit in this SOURCE slice.

## Out of scope

Linked documentation expansion; Classic 2.5/Wrath 3.4/Era supersession; invented state/defaults/shims/fallbacks/aliases; runtime changes, network, push/merge/rebase/deploy/delegation; broad/check/lint/type/readability/coverage/startup/final gates. Shared generator/extractor are unchanged; no new opt-in parsing mode needed for a separate bounded source ledger.
