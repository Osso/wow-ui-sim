# Patch 6.2.0 API audit

Pinned pageid **149103**, revision **1460518** (2023-09-27T22:20:46Z), refetched 2026-10-08. Narrative parent page: no inventory rows, not a stub/redirect. Its automated-diff transclusion is an explicit unexpanded source boundary.

## Coverage

| Statement | Coverage | Limit | Proof |
|---|---|---|---|
| Mythic/Timewalker IDs | Existing DifficultyUtil.ID values 23/24 | Native five-player difficulty metadata remains temporary defaults | Prefork identifier test |
| Item string layout and upgrades | Recorded, not modeled | ID-only item parsing lacks specialization/bonus/upgrade relationships and cross-level stat fixtures | Occurrence ledger and complete caller scan |
| Spell tooltip cost | New C API backing cost policy; widget uses hyperlink producer | One existing Flash of Light resource-cost fixture, not complete spell catalog | Discovery fails on linked MANA; targeted GREEN pending |
| Automated diff | Reference retained | External diff page not pinned/expanded; no inferred APIs | Source/extract equality |

All eleven retained extract occurrences are accounted individually. Six metadata rows, three bounded rows, two pending item-link contracts. Numeric ID checks do not claim actual dungeon gameplay or server metadata.

## Implementation and retirement boundary

`src/c_api/spell_tooltip_cost.rs` owns the existing one-spell resource-cost fixture and retail hyperlink policy. Existing C_TooltipInfo builder delegates to it; GameTooltip:SetHyperlink uses GetHyperlink rather than the direct spell producer. Pre-Warlords profiles retain costs. No shims or vendor edits.

No explicitly removed member exists on this parent page; no retirement proposed. Full `/usr/bin/grep -w` src/tests caller scans are saved untruncated, including arbitrary call syntax. Empty retirement set and fixed master/p624 register scope will be recorded.

Extractor `--retain-patch-diff-reference` originates unchanged from **6.2.4 commit 9d153dc80**; fixture heading correction originates from **c79de9881**. No duplicate tool implementation or generator flag needed. Prior register/extract reproduction uses recorded provenance flags or historical verified flags where old provenance predates flag fields.

## Verification status

Discovery: zero-entry sweep passes; cached spell-link assertion fails because hyperlink includes `10% of Base MANA`. Targeted GREEN, all publication sweeps, caller regressions, Mists, format, startup comparison and portable validators remain pending. Initial captured Cargo execution orphaned after Pyrun returned no result; its owned process tree was cancelled and exit verified. It supplies no proof. Subsequent commands use intentional file-backed asynchronous workers, not poll-waiting.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/6.2.0-api-changes.provenance.json).
- [Occurrence ledger](../../../data/patch-api/sources/6.2.0-page-coverage.json).
- [Evidence](../../../data/patch-api/evidence/6.2.0-session-2026-10-08/).
- [Spec](../../specs/patch-6-2-0-publication-sweep.md).

## See Also

- [[patch-audit-validator-portability]] — fixed historical scopes, not moving live file sets.
- [[patch-7-0-3-api-audit]] — reference audit workflow.
