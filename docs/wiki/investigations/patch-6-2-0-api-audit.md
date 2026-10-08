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

[Proof ledger](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/p620-proof-ledger.md) gives exact commands, revisions, full logs and invalidations. Runtime src/tests remain pinned at `da30fc360`; later evidence/docs commits do not invalidate these scopes.

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

[Validator](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/validate.py) is read-only. Register/sweep sets use historical helpers at recorded revisions; prior-validator set comes from fixed `git ls-tree`, not live files. Counts derive from retained files. No absolute cwd/target equality. [Portability proof](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/p620-validator-portability.json): independent clone passes, added register/validator files leave counts unchanged, whitespace tampering fails, exact restoration passes. Unmerged p624 register inventory is read from Git objects, not assumed present in this checkout.

The 7.0.3 integrated gate initially rejected the new shared module through a live runtime hash check. Final repair is copied exactly from master `15b417367` (validator and self-hash artifact), per coordination instruction; local alternative is superseded. Original failure logs remain retained.

Initial captured Cargo execution orphaned after Pyrun returned no result; its owned process tree was cancelled and exit verified. No proof credited. Subsequent commands use intentional file-backed asynchronous workers, not poll-waiting. Imported 6.2.4 fixture heading failure is retained/invalidated; its exact upstream correction passes.

Changed Rust readability review found no new suppressions, deep nesting or duplicated bodies. No full suite, agents/model CLIs, push, merge, working-directory tool call, vendor/Wowless/WowlessData edit or protected host-state change. No WoW install: CASC/native visual tests are not claimed.

## Recorded limits

Two item-link contracts remain pending because static item IDs/context clones lack historical specialization, variable bonus-ID, upgrade and cross-level scaling relationships. Difficulty checks prove identifiers only; native five-player metadata remains unmodeled. Automated diff is explicitly not expanded. Cost policy has only the existing Flash of Light fixture. Integration must replace the first-position 6.2.2 then 6.2.4 placeholders; neither branch was merged here.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/6.2.0-api-changes.provenance.json).
- [Occurrence ledger](../../../data/patch-api/sources/6.2.0-page-coverage.json).
- [Evidence](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/).
- [Spec](../../specs/patch-6-2-0-publication-sweep.md).

## See Also

- [[patch-audit-validator-portability]] — fixed historical scopes, not moving live file sets.
- [[patch-7-0-3-api-audit]] — reference audit workflow.
