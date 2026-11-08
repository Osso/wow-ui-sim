# War Mode quest currency rewards

Current cached Retail/PTR `QuestUtils.lua` exposes `QuestUtils_AddQuestCurrencyRewardsToTooltip`. Its ordinary currency path consumes separate desired-mode, percentage, quest eligibility, currency bonus and faction-grant inputs. The [supplied currency bonus query](currency-war-mode-bonus.md) is already verified; that does not establish this public caller.

Six checked behavioral requirements below retain historical `a5eaf4fc6` proof (44 executions/22 names). Current `91324b103` has source-equivalence plus independent bounded Retail/PTR affected runtime/normal and final fmt/default check PASS on 2026-10-12 UTC. [Public packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/README.md) owns current counts and independently verified new-retention status; native/full-parent completion is not established.

## What it must do

- [x] Project live supplied desired War Mode and reward percentage as one boolean and one number, independent of generic `player.pvp_enabled`. Initial disabled/10-percent state is chosen simulator policy preserving existing percentage compatibility, not native state/catalog evidence; separate default-percentage getter remains unchanged.
- [x] Project independently supplied quest eligibility as one boolean; absent/unrepresented input is false (preserves existing normal behavior), not an inference from world-quest classification or a native eligibility catalog.
- [x] Project supplied currency-to-immediate-reputation faction IDs as one nullable number; no supplied relation means nil, not native unknown-ID truth. No inferred join or normal faction/eligibility membership seed.
- [x] Required numeric relation identities reject non-number types without coercion; fractional, nonfinite or outside-i32 numeric identities are unrepresented and return false eligibility/nil faction, never truncated aliases. With `retail-12-1-0` secret support, authentication precedes number validation/lookup; wrapper secrecy and caller-taint preservation verified in both current profiles.
- [x] Unmodified loaded public QuestUtils ordinary reward handler emits exact currency lines and configured bonus percentage only when every real gate allows it. Cover disabled mode, ineligible/non-world quest, nullable/false currency flags, faction exclusion, repeated currency first-instance limits and independent currencies.
- [x] Live updates/removal and separate environments preserve state ownership; current Retail/PTR publication/startup and relevant existing query behavior stay coherent without earlier-profile backports or current-profile fallback alternates.

## How it works

- [C API boundary](../../AGENTS.md#c-api-boundary)
- [Current query investigation](../wiki/investigations/patch-8-1-0-api-audit.md#currency-warmode-query--testspec-stage-2026-10-12)

## Implementation inventory

- `tests/quest_currency_war_mode.rs`: initial public query contracts and actual ordinary vendor-tooltip fixture; initial RED1 ordinary-baseline PASS/4 contractFAIL; expanded controls use real input scaffold, actual gate/percentage outputs and duplicate-currency first instances.
- `src/c_api/c_pvp.rs`: existing namespace seam; `war_mode.rs::WarModeState` owns desired/reward inputs and live current-profile getters.
- `src/c_api/item_spell/c_currency.rs`: current-profile faction getter projects the independently supplied relation.
- `src/c_api/c_quest_log.rs`: current-profile eligibility getter; earlier-profile false placeholder remains isolated in `quest_surface/register.rs`.
- `SimState.quest_war_mode_eligibility` and `currency_faction_grants`: empty supplied relations, independent of classification/display fields.

Committed producers `bea55a2ae` (faction/shared numeric helper), `a295a7d72` (PvP) and `067fdbb90` (quest/central registration) publish only for current Retail/PTR. Utility bootstrap registers the quest getter; later `quest_surface` registration cannot overwrite it because its canned false entry is excluded for these profiles. The current canned reward getter is likewise excluded; `GetWarModeRewardBonusDefault` remains 10, and earlier-profile providers remain unchanged. Desired=false/reward=10 and empty relations are independent supplied-state/default policy, not native catalogs or inference from generic PvP/world-quest state.

## Tests asserting this spec

`tests/quest_currency_war_mode.rs`; initial five-case RED1PASS/4FAIL remains separate. Expanded pre-producer RED executed at `/home/osso/.local/state/wow-ui-sim/verification/quest-currency-war-mode-current/positive-red-compile-20261012T091819552684Z`: compile exit0; runtime13 cases,1PASS/12FAIL, exit101,2.15s. Tests pinned to `9e3fd32551e67e06653e6224e94be8f56a6da33b`; docs HEAD `8a10302064ed91b8f0944eef475871902b311bf0`. First independent authentication saved matching467982416 bytes, digest `4e6d1f18bec3b6c186d9380e128c8a1881fe45165034d9bdb5d22f4e70dd0dbf`; reused without artifact reread or rehash. Main terminal failed exit101/MainPID0. Ordinary vendor caller baseline is the sole PASS; supplied-input contracts and positive bonus controls fail. Later gates/duplicate downstream assertions were not reached, not independently falsified.

## Known gaps (current cycle)

- [x] Expanded pre-producer RED executed and independently authenticated; initial five-case RED retained separately.
- [x] Source `a5eaf4fc6` Retail/PTR execution and fmt/default check independently verified: bounded PASS, not native parity or full handoff closure.
- [x] Subsequent profile-gate cleanup `91324b103`: source-equivalence plus independent bounded Retail/PTR affected runtime/normal and final fmt/default check PASS; historical changed-flag proof reused, not rerun.
- [x] Independent new-retention gate verified; [packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/README.md) owns volatile counts/status. Native/full-parent acceptance remains open.

## Current stage — 2026-10-12

Verified source `a5eaf4fc6` consolidates the accepted duplicate validator after PTR compilation finished; no source edit during that build. Independent2643 source-equivalence/readability PASS (manual metrics unavailable); prior2635 getter/wiring source proof reused.

| Pinned scope | Evidence | Limit |
|---|---|---|
| Precursor `4e19a8bad`, Retail epoch `retail-green-compile-20261012T092906296861Z` | 13 PASS/0 FAIL, 4.74s; normal four raw getters, arity, unseeded/default/type assertions, `[]`/0; independent2642 authenticates new integration and normal artifacts once | Precursor only, not current final proof |
| Same precursor, PTR epoch `ptr-green-compile-20261012T093314261880Z` | Compile0; integration artifact authenticated once by2644; six rejected admissions, zero runtime executions, resource-blocked; no normal binary | Independent2645 confirms worker JSON error despite controller systemd success |
| Verified a5e Retail | `retail-final-compile-20261012T094557134417Z`: 13 Quest + 9 changed-flag PASS; normal raw five-query/arity/unseeded/type assertions, startup `[]`/0, 291 Blizzard addons | Verified at a5e, not precursor credit |
| Verified a5e PTR | `ptr-final-compile-20261012T095752975869Z`: 13 Quest + 9 changed-flag PASS; same normal assertions, startup `[]`/0, 283 Blizzard addons | Verified at a5e, separate from blocked precursor |
| Verified a5e checks | `check-20261012T100335665643Z`: fmt/default check0; relevant sources unchanged | Six inherited manifest deprecations remain |
| Current913 Retail | [Independent runtime report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/retail-profile-gate-compile-20261012T110200680260Z/profile-gate-retail-runtime-report.md): affected Quest integration and loaded normal assertions PASS | Bounded current-profile proof only |
| Current913 PTR | [Actual-runtime report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/ptr-profile-gate-compile-20261012T111318504844Z/profile-gate-ptr-runtime-report.md): affected Quest integration and loaded normal assertions PASS | Original compile-only/PENDING reports preserved, not rewritten |
| Current913 final check and affected scope | [Final independent report](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/profile-gate-final-independent-report.md) and [proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/profile-gate-final-independent-proof.json): actual `check-20261012T112136450921Z` fmt/default check0; recorded relevant hashes unchanged | Six inherited manifest deprecations; runtime PASS does not close retention/native/full-parent gates |

Independent2656 `final-independent-report.md` and `proof.json` in `/home/osso/.local/state/wow-ui-sim/verification/quest-currency-war-mode-current/` record **BOUNDED PASS** for source `a5eaf4fc63e44027b1e9757b146b398c62e3a4b3`: 26 Quest + 18 changed-flag = 44 executions/22 unique names, excluding RED and precursor results. Four new seals first authenticated once; receipts retained in their epochs. Actual unmodified public vendor controls reach exact +25%→+40%, every bonus gate, duplicates, independent environments and secret-first validation. Default10 remains unchanged; its raw publication/arity is not separately asserted.

[Public packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/README.md) owns retention accounting; through-precursor retention PASS does not cover final streams. Final a5e streams and distinct source-only records are retained; packet owns counts and excludes unproduced proof. [Independent later-retention supplement](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/later-retention-independent-a9855840b-report.md) verifies the final/source-only copies; main-only followup-report copies remain distinct. Current `91324b103` removes five repeated profile guards and keeps one current-only namespace publication decision; [source-equivalence/readability PASS](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/profile-gate-source-report.md) and the linked final independent report establish separate bounded affected Retail/PTR proof on 2026-10-12 UTC. Rust source remains `91324b10380f9be3e8d196491e9a2d20bc597fa1`; build docs HEAD `298caf47a` and final-check docs HEAD `a60bf28c0` are not new Rust revisions. Four new seals first authenticated once; old a5e seals never rehashed. Original partial PENDING reports remain preserved separately from additive actual-runtime/final PASS. Actual retry2 worker completed, terminal MainPID0/empty group; both earlier admission blocks remain history. [Packet status](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/README.md) owns counts and independent retention proof for the [Retail manifest](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/profile-gate-retail-retention-manifest.json) and [final PTR/check/controller manifest](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/quest-currency-war-mode-rewards-2026-10-12/profile-gate-final-retention-manifest.json). No new flag/other-profile execution credit; unchanged flag-query9/profile historical proof is reused. [Wiki stage](../wiki/investigations/patch-8-1-0-api-audit.md#questutils-current-final-stage--2026-10-12) links this current bounded matrix; native/external/full-handoff acceptance remains separate.

## Out of scope

Native eligible-quest/currency/faction catalogs, active War Mode/phase/toggle/setter lifecycle, currency-container reward path, XP/money/other quest reward paths and earlier-profile semantics. They are distinct contracts; no mocked getter, vendor rewrite or native-parity claim substitutes for this ordinary currency caller boundary. Parent full handoff remains OPEN.
