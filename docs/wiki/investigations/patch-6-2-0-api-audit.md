# Patch 6.2.0 API audit

Pinned pageid **149103**, revision **1460518** (2023-09-27T22:20:46Z), refetched 2026-10-08. Narrative parent page: no inventory rows, not a stub/redirect. Its automated-diff transclusion is an explicit unexpanded source boundary.

## Coverage

| Statement | Coverage | Limit | Proof |
|---|---|---|---|
| Mythic/Timewalker IDs | Existing DifficultyUtil.ID values 23/24 | Native five-player difficulty metadata remains temporary defaults | Prefork identifier test |
| Item string layout and upgrades | Recorded, not modeled | ID-only item parsing lacks specialization/bonus/upgrade relationships and cross-level stat fixtures | Occurrence ledger and complete caller scan |
| Spell tooltip cost | New C API backing cost policy; widget uses hyperlink producer | One existing Flash of Light resource-cost fixture, not complete spell catalog | RED linked MANA reproduced; cached and bare GREEN pass |
| Automated diff | Reference retained | External diff page not pinned/expanded; no inferred APIs | Source/extract equality |

All eleven retained extract occurrences are accounted individually. Six metadata rows, three bounded rows, two pending item-link contracts. Numeric ID checks do not claim actual dungeon gameplay or server metadata.

## Implementation and retirement boundary

`src/c_api/spell_tooltip_cost.rs` owns the existing one-spell resource-cost fixture and retail hyperlink policy. Existing C_TooltipInfo builder delegates to it; GameTooltip:SetHyperlink uses GetHyperlink rather than the direct spell producer. Pre-Warlords profiles retain costs. No shims or vendor edits.

No explicitly removed member exists on this parent page; no retirement proposed. Full `/usr/bin/grep -w` src/tests caller scans are saved untruncated, including arbitrary call syntax. [Scan index](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/p620-caller-scans.json) pins output hashes. [Retirement record](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/p620-retirement-scans.json) records the empty member/cached-scan set and fixed master/p624 Git inventories; no later re-addition can apply to an empty set.

Extractor `--retain-patch-diff-reference` originates unchanged from **6.2.4 commit 9d153dc80**; fixture heading correction originates from **c79de9881**. No duplicate tool implementation or generator flag needed. Prior register/extract reproduction uses recorded provenance flags or historical verified flags where old provenance predates flag fields.

## Verification

[Historical proof ledger](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/p620-proof-ledger.md) retains original commands, revisions, full logs and invalidations. Original runtime `da30fc360` maps to `7c2173ccb` after rebasing onto master `8dd11c1b9`; [rebase mapping](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/rebase-mapping.json) records patch IDs, changed blobs, trees and the upstream-dropped repair. Historical counts below remain frozen, not current integration counts.

| Command / boundary | Result |
|---|---|
| `cargo test --test prefork_full_ui -- patch_6_2_0` | RED: 1/2, linked MANA failure; GREEN: 3/3 (sweep, IDs, cost) |
| `cargo test --test prefork_full_ui -- publication_sweep` | 50/50; 49 pages, 9,016 observations; all historical per-page known-gap sets match |
| `cargo test --test integration patch_6_2_0 -- --nocapture` | 1/1 bare behavior |
| Integration `tooltip_item_spell::`, `tooltip_basic::`, `tooltip_spell_mount_identifiers::` | 88/88, 62/62, 22/22 |
| Python extractor / generator / validator fixtures | 36/36, 31/31, 8/8 |
| Register / extract reproduction | 49/49 registers, 46/49 extracts; exact inherited failures 12.0.5/12.0.7/12.1.0; 249 prior source files preserved |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Exit 0, zero non-vendor warnings; six inherited iced manifest deprecations |
| `cargo fmt --check` | Exit 0 |
| Separate branch / immutable master `846a30663` builds, `timeout 90 … --no-saved-vars lua-errors` | Both exit 0, addons enabled, identical `[]` |
| Historical validators | All 30 `validate*.py` files present at fixed base pass; own gate adds the 31st |

[Historical validator](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/validate.py) remains read-only. Shared files and tool implementations now come from pinned Git objects; historical register/sweep inventories are retained explicitly in the rebase mapping. Prior-validator sets come from fixed `git ls-tree`, not live discovery. The formerly unmerged p624 inventory is proved against identical blobs at reachable master `8dd11c1b9`, avoiding a dangling commit dependency. Original validator bytes and all historical records are preserved under [integrated evidence](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/). [Historical portability proof](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/p620-validator-portability.json) remains available.

The 7.0.3 integrated gate initially rejected the new shared module through a live runtime hash check. Final repair is copied exactly from master `15b417367` (validator and self-hash artifact), per coordination instruction; local alternative is superseded. Original failure logs remain retained.

Initial captured Cargo execution orphaned after Pyrun returned no result; its owned process tree was cancelled and exit verified. No proof credited. Subsequent commands use intentional file-backed asynchronous workers, not poll-waiting. Imported 6.2.4 fixture heading failure is retained/invalidated; its exact upstream correction passes.

Changed Rust readability review found no new suppressions, deep nesting or duplicated bodies. No full suite, agents/model CLIs, push, merge, working-directory tool call, vendor/Wowless/WowlessData edit or protected host-state change. No WoW install: CASC/native visual tests are not claimed.

## Integrated proof on master 8dd11c1b9

Runtime receipt scope is `ddd76addc`; subsequent changes are evidence/docs only. [Integrated receipts](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/) preserve the historical session and add:

| Boundary | Result |
|---|---|
| Own prefork sweep; integration/prefork `patch_6_2_0` | 1/1; 1/1 and 3/3 |
| All publication sweeps | Branch 52/52; pinned master 51/51; 51 pages, 9,051 observations |
| Other-page comparison | Every observation on all 50 other pages equals pinned master; [gap comparison](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/gap-comparison.json) |
| Register/extract reproduction | 51/51 registers, 48/51 extracts; exact three inherited failures unchanged |
| Integration spell-line modules | Item/spell 88/88, basic 62/62, spell/mount identifiers 22/22, GC roots 10/10, item sources 6/6, talents 2/2 (190/190) |
| Prefork tooltip cases | 34/34 |
| Addons-enabled `timeout 90 … --no-saved-vars lua-errors` | Exit 0, `[]` |
| All `tools/test_*.py` | 80/80: validator gate 4, extractor 36, generator 32, preservation helper 8 |
| Format/Mists check | Exit 0; zero non-vendor warnings, six inherited iced manifest deprecations |
| Prior historical/integrated validator matrix | 33/33, set derived from `git ls-tree` at pinned master |

Broader integration `tooltip_` diagnostic: 473 passed, one failed, one ignored. `tooltip_text_layout::test_tooltip_layout_is_clamped_to_viewport_edges` also fails on pinned master at the identical horizontal-width assertion. It uses custom `AddLine` text, not spell producers; retained as an existing layout failure, not an accepted spell-line regression or a silently skipped test.

[Integrated validator](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/validate.py) passes and checks committed, self-hashed receipts and pinned shared trees. [Command ledger](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/command-ledger.md) records acceptance commands and separate diagnostic failures. `python3 tools/check_patch_validators.py` at `a8b719035` reports **PASS** in both phases: clean **31/31**, synthetic unrelated later audit **32/32**. [Full portability report](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/portability.json) retains every validator result. Subsequent proof-output/wiki-only changes do not alter the sealed validator inputs.

Zero parent inventory means no later-register intersection or gap replacement. Both pending item-link contracts remain unmodeled; merged 6.2.2/6.2.4 audits introduced no backing-model changes. Negative controls preserve the zero-row count invariant (injection rejected before probing) and independently reject a one-field retained-source title tamper. Neither invents a publication observation or weakens an invariant.

## Recorded limits

Two item-link contracts remain pending because static item IDs/context clones lack historical specialization, variable bonus-ID, upgrade and cross-level scaling relationships. Difficulty checks prove identifiers only; native five-player metadata remains unmodeled. Automated diff is explicitly not expanded. Cost policy has only the existing Flash of Light fixture. Integrated sweep now uses the real merged 6.2.2 and 6.2.4 registers; neither intersects this zero-entry parent inventory, so no supersession or retirement invariant changed.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/6.2.0-api-changes.provenance.json).
- [Occurrence ledger](../../../data/patch-api/sources/6.2.0-page-coverage.json).
- [Evidence](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/).
- [Spec](../../specs/patch-6-2-0-publication-sweep.md).

## See Also

- [[patch-audit-validator-portability]] — fixed historical scopes, not moving live file sets.
- [[patch-7-0-3-api-audit]] — reference audit workflow.
