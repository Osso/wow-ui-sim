# Historical Retail Patch 1.12.0 source accounting

Bounded audit of the immutable [frozen redirect](../../data/patch-api/source-cache/legacy-2026-10-09/1.12.0-wikitext.txt), not reconstructed native API history. [Audit wiki](../wiki/investigations/patch-1-12-0-api-audit.md) describes evidence and epochs.

## What it must do

- [x] Validate exact manifest/response/wikitext identity, hashes, 45 bytes and 101-page registry ending at 1.0.0.
- [x] Preserve the complete raw redirect and unexpanded linked target; derive counts and reject omitted boundaries, invented inventory and changed counts.
- [x] Keep original historical Retail distinct from Classic/Forever; retain separate original-Retail successor references without applying contracts or supersession.
- [x] Replay unchanged historical generator CLI defaults and extractor `extract_text` defaults byte-for-byte.
- [x] Copy historical evidence without Git/target/current tools; reject serialized ledger/log tampering and restore original bytes without resealing.

## How it works

- [Evidence, model limits and epochs](../wiki/investigations/patch-1-12-0-api-audit.md)

## Implementation inventory

- `data/patch-api/evidence/1.12.0-session-2026-10-09/audit.py`: source validation, literal accounting and historical default replay.
- Same directory `ledger.json`: derived source accounting, precise missing contract and separate history references.
- Same directory `historical-tools/`: unchanged base tool bytes, no shared changes.

## Tests asserting this spec

- `data/patch-api/evidence/1.12.0-session-2026-10-09/test_source_accounting.py`: five targeted source/default/history controls.
- Same directory `test_portable.py`: three copied replay/serialized tamper/restoration controls.

## Known gaps (current cycle)

- [ ] Redirect target revision/content and patch-specific delta are absent from this frozen page. Callable identities, arguments, returns, defaults, event payloads, state transitions and security rules remain UNPROVEN.
- [ ] No native 1.12 measurements, loaded UI or meaningful modeled subset. Current default Retail 12.1 cannot supply that evidence.

## Out of scope

Network expansion, invented aliases/signatures/defaults, foreign-history supersession, vendor/cache changes, broad/final gates, push/merge/deploy/delegation. Main integrates queued 1.13.2 first for order only.
