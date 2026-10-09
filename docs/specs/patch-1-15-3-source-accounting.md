# Patch 1.15.3 literal SOURCE accounting

Account for the frozen Warcraft Wiki page without importing linked APIs or native compatibility. Source: `data/patch-api/evidence/1.15.3-session-2026-10-09/source.wikitext`. [Audit](../wiki/investigations/patch-1-15-3-api-audit.md) describes evidence and replay.

## What it must do

- [x] Validate exact page/revision/returned bytes against frozen manifest and registry.
- [x] Preserve every literal row, navigation template, header, TOC, summary and link occurrence; explicitly account for zero local API/event/CVar/widget/command/signature declarations.
- [x] Keep unspecified Dragonflight10.2.7/Cataclysm4.4.0 subset references unexpanded; preserve differing Gethe4.4.0 and Ketho1.15.2 comparison bases.
- [x] Keep configured11507 separate from source11503 and preserve six same-Era successor boundaries without semantic supersession.
- [x] Reject omission, fabricated credit and collapsed comparison bases.
- [x] Replay copied historical inputs without Git/target/current tools; reject serialized ledger/log tampering and restore exact originals. Keep original seals immutable, later receipts separate.

## How it works

- [Audit](../wiki/investigations/patch-1-15-3-api-audit.md).

## Implementation inventory

- `data/patch-api/evidence/1.15.3-session-2026-10-09/audit.py` — own literal accounting and seal validation.
- Same directory frozen inputs/configuration/successors/historical tools — portable evidence, no runtime dependencies.

## Tests asserting this spec

- Same directory `test_source_accounting.py` — literal/identity/omission/history controls.
- Same directory `test_portable.py` — copied fresh-process replay and serialized tamper restoration.

## Known gaps (current cycle)

- [ ] All five substantive summary/link contracts remain UNPROVEN; exact subset identities and behavior unspecified.
- [ ] Main owns ordered integration and native/security/loaded-UI/final gates.

## Out of scope

Linked-content/template expansion, foreign-history backfill, invented defaults/APIs/aliases, runtime/vendor/cache/Wowless/shared-tool changes and broad/final checks.
