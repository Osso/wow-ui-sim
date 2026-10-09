# Retail Patch 5.0.4 API audit

Audit verified 2026-10-09. Pinned main revision 3706382 and separate diff 3923766; parent revision 6423642 gives TOC 50001. This is retail Mists prepatch, not Classic 5.5.x. Parsed-source work through c40622385 is preserved; no reparsing implementation restart.

## Coverage boundary

626 unique occurrences: 77 main and 549 diff. The [ledger](../../../data/patch-api/sources/5.0.4-page-coverage.json) separately accounts for 73 main extract statements and five signatures: **704 IDs**, 469 bounded publication/absence, 223 pending, 12 metadata after integration. Historical accounting was 467 bounded/225 pending. Historical [gap review](../../../data/patch-api/evidence/5.0.4-session-2026-10-08/gap-review.json) retains 159 sweep mismatches; [integrated review](../../../data/patch-api/evidence/5.0.4-session-2026-10-08/integrated/gap-review.json) has 157. Both retain 61 pending prose statements and five pending signatures. Publication never credits argument/output/security parity or pre-existing placeholder behavior.

The cheap meaningful gap is `C_PetBattles.GetPetType`: existing per-side `PetBattlePet.pet_type` now backs reads. Concrete RED returns `(nil,nil)` instead of `(7,9)`; GREEN proves ally/enemy, mutation to 8, cleared roster, nonpositive slot and invalid owner in the cached environment. Bare-environment proof is an independent acceptance boundary. Code stays in `src/c_api/`; no new state, synthetic species-to-display mapping or Lua shim. Full pet battle, journal filtering, old challenge/talent/map workflows, POI geometry and historical observed bugs remain unproved.

## Probe limitations and preserved compatibility

`LE_PARTY_CATEGORY_HOME`/`INSTANCE` are numeric constants rejected by the function-only probe. `UIPanelButtonTemplate` is a template, not a global API. `SPELL_CAST_SUCCESS` is a combat-log sub-event, not a frame event. These remain explicit probe/semantic limits, not invented absent APIs.

Qualified and bare whole-word current-retail scans exclude Documentation; all src/tests matches are retained untruncated for 121 removal occurrences (102 spellings). No new retirement. Current group aliases, raid-roster consumers, legacy talent consumers and permanent unsupported 3D methods stay preserved. Later retail registers readd `GetExpertisePercent`, `isRaidFinderDungeonDisplayable`, `Cooldown:GetDrawEdge` and `Cooldown:SetDrawEdge`; none retired. Merged 5.1.0 retirement markers at pinned master `1c9984d2e` now resolve `C_PetJournal.GetSummonedPetID` and `C_PetJournal.SummonPetByID`: raw=nil/lookup=function becomes raw=nil/lookup=nil. Only these two fixture/ledger rows change; no new retirement or gap silencing.

## Historical successors and proof

Real 5.1.0 register bytes come from integrated master 85a259d90, retained with their blob digest inside this audit. Remaining real successors run through the latest committed retail 12.1.0 register; no Classic 5.5.x enters this chain. Rebased integration uses the canonical merged 5.1.0 source register; the old copied register was byte-identical and remains historical evidence.

At edf957802, own prefork 2/2 and all retail sweep/factory cases 63/63 pass: 10,779 observations on 62 pages. Other-page outputs are losslessly compressed, not truncated inventories. Affected pet-battle integration 36/36 passes. Python fixtures 101/101 pass. All 67 registers reproduce; 64 main extracts and the separate 5.4.0 diff extract reproduce, retaining the exact inherited 12.0.5/12.0.7/12.1.0 failures. Initial probes without older recorded flags remain failed evidence, not parser defects. Mists controls 6/6, Mists pet-type read 1/1, own bare integration 1/1, affected prefork UI 7/7, retail line controls 3/3 and namespace library 24/24 pass. Default `cargo check --tests`, `cargo fmt --check` and Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` pass; zero non-vendor warnings. Installed Arch cargo/rustc is 1.99.0. `wow-sim --no-addons --no-saved-vars lua-errors` returns `[]`, exit 0; the initially attempted unsupported wow-cli invocation remains failed evidence. Exact negative control rejects 159 → 160 mismatches. No whole-suite/native-gameplay completion claim.

Portable gate at `2ffcc12e089d7b3da2dc4d681be6a834d16647de`: **57/57 clean, 58/58 synthetic later audit**, zero failures. All 56 prior validators remain byte-identical to their pinned blobs. Source and owned-log tampering are rejected at exact diagnostics; original bytes restored. [Command ledger](../../../data/patch-api/evidence/5.0.4-session-2026-10-08/command-ledger.json) records exact argv, code/input revisions, environments, exits and hashes; [gate report](../../../data/patch-api/evidence/5.0.4-session-2026-10-08/validator-gate.json) records both phases. Evidence uses compact tree/blob pins and lossless compressed other-page outputs; total owned evidence remains below 5 MB. Final receipt/wiki/spec-only commit does not invalidate runtime/tool proof. Coordinator owns rebase, integration, migration and CI; no push, merge, delegation, vendor edit or canonical/sibling worktree edit.

## Integration refresh — 2026-10-09

Pinned master `1c9984d2e8a5e672c5109d2234197303e6190317`, rebased runtime `6f9c997b7d45a3f3f119a678b319e2066c090082`. Initial integrated discovery proves exactly two attributable closures, **159 → 157**, no new gaps, no other changed own observations. Stale-fixture failures are retained as RED evidence; only resolved IDs are removed. Current 61 prose/five signature limits remain. Historical validator bytes and all original receipts are preserved in [integrated evidence](../../../data/patch-api/evidence/5.0.4-session-2026-10-08/integrated/); exact prior validator blobs and the original own ledger/fixture are replayed, never replaced by live unrelated hashes. Compact mapping pins fourteen original/rebased commits and their trees. All 54 historical artifact byte streams are preserved (original validator archived verbatim). Seven otherwise-unreachable required Git objects are retained in a verified 77,355-byte pack; replay imports exact objects and denies all fourteen original commit identities. No per-file rebase inventory; mapping is 13,475 bytes.

Current retail sweeps **64/64**, own prefork **2/2**, own bare integration **1/1**, pet-battle integration/prefork **36/7**, pet-battle library **2/2**, namespace library **24/24**, and retail line controls **3/3** pass. Mists sweeps/control **6/6** and own state read **1/1** pass. Exact same-worktree pinned-master comparison passes: retail **63/63**, Mists **6/6**, and identical selected caller/library cases. All **67 other pages / 10,216 observations** are byte-identical to master; only the two own closures differ from history. All **115 Python fixtures** pass. **68 registers / 65 main extracts** and the separate 5.4.0 diff reproduce; inherited failures remain exactly 12.0.5, 12.0.7 and 12.1.0. All 359 pinned-master source inputs and existing extraction modes are preserved, including source-only Classic pages; no Cata runtime claim.

Default check, Mists check and format pass, zero non-vendor warnings. No-addons/no-saved-vars startup returns `[]`, exit 0. Negative rejects exactly **157 → 158**. Initial stale-fixture failures and the source-only preservation diagnostic remain explicit; completed reproduction work was not rerun to recover logs. [Integrated command ledger](../../../data/patch-api/evidence/5.0.4-session-2026-10-08/integrated/command-ledger.md) pins argv, scope, revisions, exits and hashes. Portable gate and seal tampering controls are the remaining final acceptance steps. No full suite, push, merge, delegation, canonical/sibling/vendor edits or ignored PLAN dependency.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/5.0.4-api-changes.provenance.json).
- [Session evidence](../../../data/patch-api/evidence/5.0.4-session-2026-10-08/).
- [Sweep spec](../../specs/patch-5-0-4-publication-sweep.md).
- [Pet-type spec](../../specs/pet-battle-pet-type.md).

## See Also

- [[patch-5-1-0-api-audit]] — merged source register and pet-ID retirement attribution.
- [[patch-audit-validator-portability]] — pinned proof across later audits.
