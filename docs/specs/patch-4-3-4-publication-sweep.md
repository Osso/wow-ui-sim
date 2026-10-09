# Patch 4.3.4 publication audit

Audit only historical retail page 389302, revision 3743181 (2021-08-22T03:09:40Z). [Audit](../wiki/investigations/patch-4-3-4-api-audit.md).

## Requirements

- Preserve committed browser response, pin and exact 866-byte source; do not expand linked API pages.
- Reproduce all 10 additions and one removal with existing default generator/extractor flags. Retain the single navigation context; no prose/signature statements occur on this page.
- Probe current cached retail publication/absence with the integrated literal 5.0.1 redirect and separate 5.0.4 destination, followed by all later retail registers; exclude Classic histories.
- Derive expected gaps and coverage accounting from actual observations. Identity/type discovery never proves outputs, signatures, security or modeled service behavior.
- Implement only cheap source-backed real models. Do not fabricate missing service contracts, add shims or retire APIs without current cached-retail and complete source/test caller scans. Preserve Classic behavior.
- Keep targeted development proof and compact historical inputs; final broad/check/smoke/acceptance gates belong to coordinator.

## Implementation inventory

`data/patch-api/sources/4.3.4-*`, `tests/patch_4_3_4_publication_sweep.rs`, `tests/data/patch_4_3_4_sweep_known_gaps.json`, and `data/patch-api/evidence/4.3.4-session-2026-10-09/`.

## Tests

Own prefork filter `patch_4_3_4_publication_sweep`, followed by a scratch-register negative control. Discovery and reviewed-gap proofs pending; no runtime implementation changes.

## Exclusions

Whole handoff audit, external API-page semantics, Classic compatibility claims, runtime shims, vendor/cache edits, native parity, delegation, push, merge and deployment.
