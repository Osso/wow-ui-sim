# Retail Patch 5.0.4 API audit

Pinned main revision 3706382 and separate diff 3923766; parent revision 6423642 gives TOC 50001. This is retail Mists prepatch, not Classic 5.5.x. Parsed-source work through c40622385 is preserved; no reparsing implementation restart.

## Coverage boundary

626 unique occurrences: 77 main and 549 diff. The [ledger](../../../data/patch-api/sources/5.0.4-page-coverage.json) separately accounts for 73 main extract statements and five signatures: **704 IDs**, 467 bounded publication/absence, 225 pending, 12 metadata. Exact [gap review](../../../data/patch-api/evidence/5.0.4-session-2026-10-08/gap-review.json) retains 159 sweep mismatches, 61 pending prose statements and five pending signatures. Publication never credits argument/output/security parity or pre-existing placeholder behavior.

The cheap meaningful gap is `C_PetBattles.GetPetType`: existing per-side `PetBattlePet.pet_type` now backs reads. Concrete RED returns `(nil,nil)` instead of `(7,9)`; GREEN proves ally/enemy, mutation to 8, cleared roster, nonpositive slot and invalid owner in the cached environment. Bare-environment proof is an independent acceptance boundary. Code stays in `src/c_api/`; no new state, synthetic species-to-display mapping or Lua shim. Full pet battle, journal filtering, old challenge/talent/map workflows, POI geometry and historical observed bugs remain unproved.

## Probe limitations and preserved compatibility

`LE_PARTY_CATEGORY_HOME`/`INSTANCE` are numeric constants rejected by the function-only probe. `UIPanelButtonTemplate` is a template, not a global API. `SPELL_CAST_SUCCESS` is a combat-log sub-event, not a frame event. These remain explicit probe/semantic limits, not invented absent APIs.

Qualified and bare whole-word current-retail scans exclude Documentation; all src/tests matches are retained untruncated for 121 removal occurrences (102 spellings). No new retirement. Current group aliases, raid-roster consumers, legacy talent consumers and permanent unsupported 3D methods stay preserved. Later retail registers readd `GetExpertisePercent`, `isRaidFinderDungeonDisplayable`, `Cooldown:GetDrawEdge` and `Cooldown:SetDrawEdge`; none retired. Two namespace spelling removals already fixed in integrated 5.1.0 remain pending on this pre-integration base; coordinator owns migration.

## Successors and proof

Real 5.1.0 register bytes come from integrated master 85a259d90, retained with their blob digest inside this audit. Remaining real successors run through the latest committed retail 12.1.0 register; no Classic 5.5.x enters this chain. Future coordinator integration replaces the local 5.1.0 pin reference with its canonical merged path.

At edf957802, own prefork 2/2 and all retail sweep/factory cases 63/63 pass: 10,779 observations on 62 pages. Other-page outputs are losslessly compressed, not truncated inventories. Affected pet-battle integration 36/36 passes. Python fixtures 101/101 pass. All 67 registers reproduce; 64 extracts reproduce, retaining the exact inherited 12.0.5/12.0.7/12.1.0 failures. Initial probes without older recorded flags remain failed evidence, not parser defects. Mists check passed; final warning analysis, Classic controls, remaining scoped acceptance and portability gate are recorded separately as they finish. No whole-suite/native-gameplay completion claim.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/5.0.4-api-changes.provenance.json).
- [Session evidence](../../../data/patch-api/evidence/5.0.4-session-2026-10-08/).
- [Sweep spec](../../specs/patch-5-0-4-publication-sweep.md).
- [Pet-type spec](../../specs/pet-battle-pet-type.md).

## See Also

- [[patch-5-2-0-api-audit]] — nearest integrated audit available in this base checkout.
- [[patch-audit-validator-portability]] — pinned proof across later audits.
