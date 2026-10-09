# TBC Classic 2.5.3 source accounting

Account for frozen Warcraft Wiki page 53568/revision 523812 as separate TBC Classic history, literal TOC 20503. [Ledger](../../data/patch-api/sources/2.5.3-page-coverage.json); [audit](../wiki/investigations/patch-2-5-3-api-audit.md).

## What it must do

- [ ] Validate committed manifest/registry identity, response/content hashes and timestamp before copying frozen inputs; preserve full registry through 1.0.0.
- [ ] Account for every nonblank raw row, API occurrence, header, summary, signature absence, inline CVar default/description/scope and unexpanded linked resource. Reject omissions and invented credit.
- [ ] Preserve literal TBC TOC independently from configured interfaces. Anniversary 11507 is not TBC 205xx proof; profile absence is not an unsupported-API conclusion.
- [ ] Permit only pending 2.5.4/2.5.5/2.5.6 main-integration references; frozen successor inputs must not depend on sibling availability or establish positive supersession credit.
- [ ] Replay original sealed source/tools/ledger/logs from a fresh archive without Git, target, current tools/runtime/configuration files. Reject serialized ledger/log tampering and restore exact hashes.

## How it works

- [Source coverage matrix and proof boundaries](../wiki/investigations/patch-2-5-3-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/2.5.3-*` — raw source, stock non-inventory plaintext, TBC-labeled local register and full-row/contract ledger.
- `data/patch-api/evidence/2.5.3-session-2026-10-09/` — frozen response/manifest/registry/successors/configuration, original tooling, targeted tests, validator, replay controls, logs/seals/archive.

## Tests asserting this spec

`test_source_accounting.py` in the evidence directory tests exact counts, all-row omissions/literal credit, all inventory/header omissions and contract mutations, history isolation, identity/plaintext and configured-profile proof boundaries. `replay_controls.py` tests serialized tamper/restore and fresh archive replay. [Source proof ledger](../../data/patch-api/evidence/2.5.3-session-2026-10-09/source-proof.json) records exact revisions and commands; only targeted SOURCE development is authorized.

## Known gaps (current cycle)

- [ ] 25 unspecified callable signatures, three event payloads, all state/validation/security/native contracts and six CVar rows without explicit defaults remain UNPROVEN.
- [ ] SharedTooltipTemplate/GameTooltipTemplate removal of BackdropTemplate inheritance: runtime ancestry, inherited methods/backdrop effects and native equivalence UNPROVEN.
- [ ] Linked diff/deprecation/community changes not expanded; successor integration and any correct-client runtime/native gates belong to main.

## Out of scope

Runtime/profile/shared-classifier changes, shims/fallbacks, native probes, forced Anniversary correspondence, vendor/cache/Wowless edits, shared generator options, cross-client supersession, broad/check/lint/profile/startup/full/final gates, delegation, push/merge/deploy/ops. Empty inventories never grant positive API parity; absence of a matching configured profile never proves every API unsupported.
