# Patch 12.1.0 page audit

Row-by-row audit of the [Patch 12.1.0/API changes](https://warcraft.wiki.gg/wiki/Patch_12.1.0/API_changes) page. Ledger and SSOT for row status: [12.1.0-page-coverage.json](../../../data/patch-api/sources/12.1.0-page-coverage.json). Earlier occurrence-level work on 12.1 is in [patch-12-1-api-audit](patch-12-1-api-audit.md); its statuses are cross-references, not proof for this ledger. Method follows the [12.0.7 audit](patch-12-0-7-api-audit.md).

## Source

- Plaintext extract (333 non-blank lines): notes, blue posts, enumerations, structures, deprecated API.
- The extract drops six collapsed tables. They come from the raw wikitext (revision 6886719) via [12.1.0-wikitext-register.json](../../../data/patch-api/sources/12.1.0-wikitext-register.json): 778 symbols across Global API, FrameXML, ScriptObjects, Widgets, Events and CVars.
- 1,111 rows in total. Triage and batch plans: [evidence directory](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/).

## Round 1 — proof batches C01–C10 — 2026-10-04

**1,022 pending / 24 bounded / 1 partial / 64 metadata.**

Tests and specs only, no producer change: AuraContainer/AuraButton creation, FrameXML helper moves, TOC `[Bootstrap]`, OnUpdate modes, XML mixins, event-registration aspect, inheritance, aura groups and options, aura sound removal.

- Master: 8 new tests passed / 0 failed, startup `lua-errors` `[]`, `cargo fmt --check` exit 0 ([log](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/round-1-master-green.log.txt)). Seventeen reused existing tests ran only in the integration worktree ([result](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/p1210-r1-result.md)).
- [Review](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/p1210-r1-review.md): ACCEPT WITH QUALIFICATIONS — 24 bounded, 1 partial, 2 pending; no vacuous tests.
- Pending: disabled-addon bootstrap (not exercised); AuraContainer intrinsic event-registration restriction (observed mask 0 — a real gap, modelable).
- The OnUpdate rows rest on four passing tests; `on_update_modes_process_actual_managed_aura_dirty_phases` was failing before this audit and still is.

## Round 2 — publication sweep — 2026-10-04

**391 pending / 151 bounded / 505 partial / 64 metadata.**

One data-driven test over the 778 inventory symbols ([spec](../../specs/patch-12-1-0-publication-sweep.md)): 631 match the page, 147 do not.

- 504 added/changed symbols are published: partial, "publication only" — a looser credit rule than the rest of the audit, chosen for breadth.
- 127 removed symbols are absent by raw and ordinary lookup: bounded.
- 147 gaps stay pending and are baselined in `tests/data/patch_12_1_0_sweep_known_gaps.json`, so the test fails on any change to the gap set. Roughly: 85 FrameXML helpers not found (many `*_LoadUI` / `Show…Frame` functions, possibly a load-on-demand bootstrap loading gap), 35 missing Global API functions, 20 removed symbols still published, 15 widget methods, events and CVars.

Master: sweep test passed, startup `lua-errors` `[]`, `cargo fmt --check` exit 0 ([log](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/round-2-master-green.log.txt)). Per-symbol output: [result](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/p1210-sweep-result.json). No independent review.
