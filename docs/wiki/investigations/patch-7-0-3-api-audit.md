# Patch 7.0.3 API audit

Pinned page 549091, revision 5295335 (2017-09-18T11:16:26Z), fetched 2026-10-08. Source has 123 wikitext lines and 134 explicit publication identities, including nested profession inventories and prose retirements. This is a bounded current-retail audit, not reconstruction of the Legion client.

## Concrete coverage matrix

| Boundary | Modeled/preserved | Missing/problematic | Proof |
|---|---|---|---|
| Recipe name search | Per-environment text; case-insensitive search over existing learned catalogue; nil clear; invalid-input rejection; list-update event; empty-filter ordering unchanged | Native locale collation, reagent searching, other recipe filters | Bare + cached tests, Mists recipe test; profession caller regressions |
| Mount renames | Consumer-free GetMountInfo/GetMountInfoExtra absent on raw/ordinary/repeated lookup; ByID successors preserved | Summon retained for live callers; historical index/ID parity not inferred | Bare + cached tests; collection/diff regressions; complete scans |
| Widget probes | Line through CreateLine; Alpha endpoints through CreateAnimation('Alpha') | Native 3D UiCamera/Model behavior; live SetChange retained | Factory test; all publication sweeps |
| Current publication | 82/134 inventory observations pass, with no behavioral credit inferred from publication | 52 exact gaps: 32 missing historical profession producers, ten later-removal namespace lookups, four retained live removals, four old map/nameplate globals, two nameplate size methods | [Gap review](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/p703-gap-review.json) gives each ID's expected/observed state and reason |
| Complete page | 134 inventory + 44 extract IDs = 178; 87 bounded, 86 pending, five metadata-only | 34 substantive extract contracts remain pending | [Occurrence ledger](../../../data/patch-api/sources/7.0.3-page-coverage.json), literal raw/extract mapping and validator |

Initial discovery reports 57 gaps and a concrete failing name-filter test. Five closures: one modeled producer, two safe retirements and two corrected Alpha endpoint probes. No shim, guessed server data, vendor edit or 3D implementation was added. Existing table/function publication is never equated with historical or native behavior.

## Retirement boundary

All twenty explicitly removed identities have qualified and bare whole-word `/usr/bin/grep` scans: **80 complete outputs**, excluding `*Documentation*` in cached retail AddOns. Full src/tests scans have no syntax filter, so `pcall(Name, ...)` and `and Name then` cannot be missed. [Scan index](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/p703-retirement-scans.json) records commands, exits, matches, hashes and pre-implementation scope. [Decisions](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/p703-retirement-decisions.json) account for every member.

Only C_MountJournal.GetMountInfo and GetMountInfoExtra are newly retired: neither has cached or simulator callers, and no later register re-adds them. Retail-only tombstones prevent namespace autostub fabrication. [Later scan](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/p703-later-register-scan.json) reads fixed master and p710-page Git objects, never the parallel worktree. Other consumers remain: cached InspectUI's SetGlyph method; Summon collection callers; Alpha:SetChange, Model:GetModel and GameTooltip:SetTradeSkillItem behavioral callers. ShowHelm/ShowCloak/ShowingHelm/ShowingCloak are explicitly re-added by 12.0.0. Already absent identities need no code change. Unnamed glyph `etc.`, Multistrike and Amplify members are not guessed.

## Targeted verification

| Command / filter | Result |
|---|---|
| `cargo test --test prefork_full_ui -- patch_7_0_3` | 3/3: sweep, cached name search, cached retirements |
| `cargo test --test integration patch_7_0_3 -- --nocapture` | 3/3: name search, retirements, Line/Alpha factories |
| `cargo test --test prefork_full_ui -- publication_sweep` | 47/47; all 45 earlier sweep ID sets/ok-statuses unchanged |
| Integration `professions_api::`, `c_collection_api::`, `c_function_diff_coverage::` | 35/35, 37/37, 4/4 |
| Prefork `professions` | 21/21 |
| Integration `test_showuipanel_professions_crafting::` | 1/1; isolated SpellSearchUtil/MerchantFrame diagnostic signatures and counts exactly match master |
| Prefork `test_showuipanel_professions_crafting` | Zero selected cases; no coverage credited |
| Mists recipe-name integration test | 1/1 |
| Python extractor / generator / validator fixtures | 35/35, 30/30, 8/8 |
| Register / extract reproduction | 46/46 registers; 43/46 extracts, exactly three inherited failures |
| Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Exit 0; zero non-vendor warnings |
| `cargo fmt --check` | Exit 0 |
| Separate branch/master builds; addons-enabled `--no-saved-vars lua-errors` | Both exit 0 and exact arrays `[]` |
| Negative namespace substitution | Expected exit 1; exactly 52 → 53 failed IDs, no resolved/stale IDs |

[Proof ledger](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/p703-proof-ledger.md) records commands, revisions, complete logs, scope hashes and invalidated development results. Retail proof is pinned at `69625973b`. The only later source change moves a misplaced cfg attribute back onto the original retail quality module; both modules' default-retail declarations are unchanged. The validator proves that exact transformation and identical remaining runtime scope. Mists is freshly checked/tested at `5168fb2d4`. No broad successful scope was rerun merely for an evidence/docs milestone. Six inherited vendor iced manifest deprecations remain unsuppressed. Changed Rust readability review found no new suppressions or deep nesting.

Master baseline is an immutable Git-archive snapshot inside the owned target, not a worktree. [Startup comparison](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/p703-startup-comparison.json) proves addons enabled and zero errors in both executions. The host has no WoW install; no CASC/native visual test is claimed. No full integration suite, push, merge, agents/model CLIs, working-directory switching, other-worktree mutation or vendor/Wowless/WowlessData edit.

## Historical validator and integration

[Validator](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/validate.py) passes read-only, with register/sweep scope from `historical_registers`/`historical_sweep_tests` at the fixed proof revision. Counts derive from retained files, not moving HEAD or receipt-derived subsets. Source/accounting, complete receipt/log hashes, input preservation, scans, negative control and prior-validator matrix remain checked. No current-checkout absolute cwd/target equality. All 25 prior historical/integrated validators pass; own validator supplies the additional gate. [Relocation proof](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/p703-validator-portability.json) exercises another checkout, added audit/register files, whitespace tamper rejection and exact restoration. Full Git history is required; no missing-history bypass.

Historical proof remains pinned to the original `aa57dd8f8` base and recorded revisions. The branch was subsequently rebased onto `25fbde058`; `cf44ebf10` replaced the first-position 7.1.0 placeholder with the merged register. Its symbols have **no intersection** with 7.0.3, including either retirement. No other worktree was read or modified.

## Integrated refresh — 2026-10-08

[Fresh command ledger](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/integrated/p703-final-command-ledger.md) retains all 30 commands, complete logs, source revisions and scope hashes. Runtime proof at `771a7d7cc` covers the unchanged final src/tests/tools/source bytes. Exact master `25fbde058` was built and tested from an immutable Git archive under the owned target, with every command's cwd still the p703-page worktree.

| Boundary | Integrated result |
|---|---|
| All publication sweeps | Branch 48/48 sweep/factory cases; master 47/47. All 46 other pages have identical ID sets and ok/gap statuses; 47 pages / 9,016 observations total. [Complete per-page comparison](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/integrated/gap-comparison.json). |
| Own sweep / negative control | 52 gaps unchanged; exact one-row nonexistent-namespace control gives 53, exit 1, no resolved/stale IDs. No supersession or `LATER_AUDIT_REPLACEMENTS` change needed. |
| Sources / refreshed receipts | 47/47 registers reproduce with recorded flags; 44/47 extracts reproduce. The exact three inherited failures match integrated 7.1.0 evidence. `extend_patch_audit_receipts.py` adds the merged 7.1.0 register/sweep rows and refreshes own rows. |
| Retail integration | Own 3/3; professions 35/35; crafting 23/23; trade info 1/1; collections 37/37; function-diff coverage 4/4; crafting panel 1/1. Harness setup results are recorded separately, not counted as feature cases. |
| Retail prefork | Own 3/3; professions 21/21; mount retirement 1/1. `crafting` and `trade_skill` select zero cases; no coverage credited. |
| Mists | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` exits 0, zero non-vendor warnings. Recipe filter 1/1; legacy trade skill 1/1; professions 35/35; mount journal 12/12. |
| Fixtures / formatting / startup | Generator/extractor/validator fixtures 31/35/8; format exits 0. Separately built branch and master, addons enabled, `timeout 90 … --no-saved-vars lua-errors`: both exit 0, exact arrays `[]`. |
| Validators / preservation | All 28 existing historical/integrated validators pass; new [integrated gate](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/integrated/validate.py) adds the 29th. All 243 historical artifacts remain byte-identical. |

[Rebase mapping](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/integrated/rebase-mapping.json) records seven original/rebased commit pairs and stable patch IDs. Five IDs match; two differ in the additive 7.1.0 rebase context. Every pair retains identical `src` bytes. Original receipt revisions and Git history remain mandatory: the integrated validator runs the untouched historical validator rather than substituting rebased input or dropping history checks. The runner resumes missing stages without repeating valid proofs. Six inherited vendor iced manifest deprecations remain unsuppressed; no non-vendor warning, source shim, runtime change or vendor edit was needed for integration.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/7.0.3-api-changes.provenance.json).
- [Publication register](../../../data/patch-api/sources/7.0.3-wikitext-register.json).
- [Spec](../../specs/patch-7-0-3-publication-sweep.md).
- [Evidence](../../../data/patch-api/evidence/7.0.3-session-2026-10-08/).

## See Also

- [[patch-7-2-0-api-audit]] — region factory and proof conventions.
- [[patch-8-0-1-api-audit]] — pre-patch accounting without native parity claims.
- [[patch-audit-validator-portability]] — fixed historical scope and exact input protection.
