# Patch 1.15.6 literal SOURCE accounting

Account frozen page 619994 / revision 6281951 / timestamp `2025-04-04T10:14:39Z` offline. Preserve literal rows and links without expanding wiki templates or asserting native/API semantics. [Audit](../wiki/investigations/patch-1-15-6-api-audit.md).

## What it must do

- [ ] Validate response/raw hashes and identity against the frozen manifest and 101-page registry ending 1.0.0.
- [ ] Preserve all four nonblank rows, Resources header, literal TOC11506 and navigation; never infer a client name.
- [ ] Account both unexpanded linked-diff occurrences and navigation template; invent no local APIs, prose, signatures, defaults or aliases.
- [ ] Keep configured Era/Anniversary11507 separate from historical/native proof; preserve pending same-Era1.15.7 inflight/1.15.8/1.15.9 without applying successors or foreign histories.
- [ ] Reject omissions, fabricated coverage, foreign supersession and identity tampering; retain original logs/seals independently of copied replay and serialized ledger/log tamper receipts.

## How it works

- [Literal coverage matrix](../wiki/investigations/patch-1-15-6-api-audit.md#literal-coverage-matrix).

## Implementation inventory

- `data/patch-api/evidence/1.15.6-session-2026-10-09/audit.py` — own literal adapter and serialized validator; shared tooling unchanged.
- Same directory: exact frozen response/source/manifest/registry, configuration, historical tools, pending-successor inputs and ledger.

## Tests asserting this spec

`python3 -B data/patch-api/evidence/1.15.6-session-2026-10-09/test_source_accounting.py`: eight SOURCE fixtures; retained RED eight assertion failures. GREEN and portable receipts pending.

## Known gaps (current cycle)

- [ ] Client name absent from literal page; template/linked-content API, state, security, signatures and native equivalence UNPROVEN.
- [ ] Main owns actual same-history successor integration and native/final gates.

## Out of scope

Runtime/model/default/shim/alias edits, transclusion/link expansion, vendor/cache/Wowless writes, foreign-history supersession, network, broad/check/lint/type/readability/final gates, delegation/model CLIs, push/merge/deploy. No concrete modeled behavior is specified by this metadata/link-only page.
