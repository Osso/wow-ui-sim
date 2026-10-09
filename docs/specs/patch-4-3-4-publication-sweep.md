# Patch 4.3.4 publication audit

Audit only historical retail page 389302, revision 3743181 (2021-08-22T03:09:40Z). [Audit](../wiki/investigations/patch-4-3-4-api-audit.md).

## Requirements

- Preserve committed browser response, pin and exact 866-byte source. Keep this inventory literal; separately pinned primary API contracts may support a modeled row without importing linked inventory.
- Reproduce all 10 additions and one removal with existing default generator/extractor flags. Retain the single navigation context; no prose/signature statements occur on this page.
- Probe current cached retail publication/absence with the integrated literal 5.0.1 redirect and separate 5.0.4 destination, followed by all later retail registers; exclude Classic histories.
- Derive expected gaps and coverage accounting from actual observations. Identity/type discovery never proves outputs, signatures, security or modeled service behavior.
- Implement only cheap source-backed real models. Do not fabricate missing service contracts, add shims or retire APIs without current cached-retail and complete source/test caller scans. Preserve Classic behavior.
- Keep targeted development proof and compact historical inputs; final broad/check/smoke/acceptance gates belong to coordinator.
- Retail `GetSessionTime()` returns numeric seconds from the simulator client-state creation clock; login and character-screen transitions do not reset that origin. Creating a new client state starts a new clock. Classic publication remains unchanged.

## Implementation inventory

`data/patch-api/sources/4.3.4-*`, `tests/patch_4_3_4_publication_sweep.rs`, `tests/data/patch_4_3_4_sweep_known_gaps.json`, and `data/patch-api/evidence/4.3.4-session-2026-10-09/`. Retail clock binding: `src/lua_api/globals/register.rs`; behavioral tests: `tests/patch_4_3_4_session_clock.rs`.

## Tests

Own prefork filter `patch_4_3_4_publication_sweep`, followed by a scratch-register negative control. Discovery RED records seven missing globals, three superseded removals and one absence; reviewed-gap GREEN passes 1/1. Same-cardinality fabricated-global control fails at exactly seven → eight gaps. Own default register/extract replay is byte-identical. Those receipts preserve pre-model discovery. The two client-clock behavioral tests reproduce nil-call RED and pass GREEN 2/2 at pre-rebase `160f8209c`; their code is unchanged at `575a93e30`. [Audit and receipts](../wiki/investigations/patch-4-3-4-api-audit.md#coordinator-client-clock-model) bound this proof to the modeled clock, not function identity or native parity. Current fixture has six remaining publication gaps and one modeled clock. Coordinator owns pending broad publication/Mists/check/build acceptance; historical seven-gap receipts remain immutable.

## Exclusions

Whole handoff audit, linked inventory expansion, Classic compatibility claims, runtime shims, vendor/cache edits, native precision/event-delivery parity, delegation, push, merge and deployment.
