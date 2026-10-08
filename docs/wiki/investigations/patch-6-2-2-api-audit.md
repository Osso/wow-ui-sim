# Patch 6.2.2 API page audit

Refetched pageid 122147 on 2026-10-08 and pinned revision **6209268**, timestamp **2025-01-03T15:17:31Z**. Entire source is `{{apichanges|6.2.2|prev=6.2.0|next=6.2.4}}`: a navigation-only stub, **not a redirect**. No linked page or template expansion is treated as an API-change statement.

## Coverage boundary

| Boundary | Accounted | Missing or excluded | Proof |
|---|---|---|---|
| Inventory | Zero entries and zero header counts | No meaningful behavior to model from this page | Pinned fetch/raw bytes; empty reproduced register |
| Extract | One `source-context-001` row, metadata-only with no capabilities | No behavioral/native parity credit | Existing extractor defaults; exact row ledger |
| Discovery | Empty observed and known-gap sets | Zero modeled gaps; zero problematic contracts | Prefork own sweep 1/1 |
| Retirements | Zero source removal identities and zero changes | No member scan targets, including qualified/bare/caller/later-register scans | Explicit empty retirement decision artifact |

An empty page is not evidence that the game patch changed no APIs. Nothing was reconstructed or guessed. No simulator/runtime, C API, shared tool, Blizzard/vendor, Wowless, or WowlessData behavior changed. Addons-enabled `lua-errors` comparison is not required by the shared-path condition: no shared/runtime path changed. No CASC/native-client visual proof is claimed on this host.

## Targeted verification

| Command | Result |
|---|---|
| `cargo test --test prefork_full_ui -- patch_6_2_2` | 1/1; result `{}` |
| `cargo test --test prefork_full_ui -- publication_sweep` | 49/49 sweep/factory cases; all historical page ID sets and exact known-gap sets checked |
| Own scratch register with one fabricated entry | Expected exit 1; row-count gate rejects 1 vs 0 before publication probes |
| `python3 -B tools/test_gen_patch_wikitext_register.py` | 31/31 |
| `python3 -B tools/test_extract_patch_non_inventory.py` | 35/35 |
| `python3 -B tools/test_patch_audit_validation.py` | 8/8 |
| `python3 -B …/reproduce_sources.py` | 48/48 registers, 45/48 extracts; inherited 12.0.5/12.0.7/12.1.0 failures unchanged |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Exit 0; zero non-vendor warnings |
| `cargo fmt --check` | Exit 0 |
| Prior validators at `dfede62de` | 25/25, including integrated gates |
| Own `validate.py` | PASS, read-only |
| `prove_portability.py` | Relocated snapshot PASS; unrelated future register/sweep/validator PASS; source-whitespace tamper rejected; exact restoration PASS |

[Proof ledger](../../../data/patch-api/evidence/6.2.2-session-2026-10-08/p622-proof-ledger.md) records exact commands, revisions, hashes, complete logs, scope and invalidation. Long Cargo commands ran asynchronously into logs under the owned `p622-page` target; no full integration suite or poll-wait. Initial acceptance wrapper expected libtest exit 101, but the prefork harness returns 1 for failed cases. Only that wrapper expectation was corrected; actual negative proof remains valid, and no Cargo command was repeated for the correction.

## Historical portability and integration

Register scope uses `historical_registers` at `c30c6c1cb`; sweep scope uses the recorded execution revision. Prior-validator scope comes from `git ls-tree` at exact base `dfede62de1f48ef2af1a4d37a18d274e26df3d81`, never live file enumeration. Counts derive from those files and retained results. Validator has no current-checkout cwd/target equality gate. Original source bytes remain protected; unrelated accounting ledgers and remaining-page scheduling may legitimately evolve in later audits. Missing historical Git objects fail explicitly.

The sweep starts with one-line **6.2.4**, then **7.0.1** integration placeholders, followed by 7.0.3, 7.1.0, and all later registers. Integrator replaces each placeholder with its merged register. Own page has no symbols, so these integrations cannot supersede an own gap or retirement. No other worktree was touched; no push, merge, agent/model CLI, working-directory switch, or `__pycache__` creation.

Wiki index/log line counts were checked before every commit against 2675/398; both append-only updates preserve inherited content. Changed Rust readability audit found no violations: the only Rust addition is the declarative zero-row sweep using the established harness.

## Sources

- [Pinned fetch](../../../data/patch-api/evidence/6.2.2-session-2026-10-08/p622-fetch.json).
- [Provenance](../../../data/patch-api/sources/6.2.2-api-changes.provenance.json).
- [Raw source](../../../data/patch-api/sources/6.2.2-api-changes.wikitext).
- [Register](../../../data/patch-api/sources/6.2.2-wikitext-register.json) and [ledger](../../../data/patch-api/sources/6.2.2-page-coverage.json).
- [Spec](../../specs/patch-6-2-2-publication-sweep.md).
- [Validator and evidence](../../../data/patch-api/evidence/6.2.2-session-2026-10-08/validate.py).

## See Also

- [[patch-7-0-3-api-audit]] — sweep, source reproduction and proof conventions.
- [[patch-audit-validator-portability]] — fixed historical source and validator scope.
