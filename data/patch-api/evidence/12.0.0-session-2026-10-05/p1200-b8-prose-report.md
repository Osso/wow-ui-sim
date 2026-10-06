# B08 and unassigned prose — bounded outcome report

Worktree `p1200-extract-b8`, base `e9baac195`, final runtime `900d756ec`. No push/merge, agents/models, vendor edits or page-coverage-ledger edits.

14 B08 rows and combat-log registration have new bounded behavioral proof. The 22 deprecated rows were **already bounded on the supplied base** and are revalidated, not newly credited. General secret-operation prose remains pending.

This table projects [machine outcomes and revision-scoped command ledger](p1200-b8-prose-proof.json). Structural proof is current cached Retail Type/Nilable/InnerType plus concrete inputs, snapshots and state transitions, not native server policy parity.

## Per-source outcomes

| source_id | outcome | test |
|---|---|---|
| `structures-CraftingItemSlotModification-082` | proven-by-test | `patch_12_0_0_crafting_shapes::profession_crafting_slot_shapes_and_variable_quantities` |
| `structures-CraftingItemSlotModification-083` | proven-by-test | `patch_12_0_0_crafting_shapes::profession_crafting_slot_shapes_and_variable_quantities` |
| `structures-CraftingOrderReagentInfo-084` | proven-by-test | `patch_12_0_0_crafting_shapes::crafting_order_nested_inputs_outputs_and_snapshots` |
| `structures-CraftingOrderReagentInfo-085` | proven-by-test | `patch_12_0_0_crafting_shapes::crafting_order_nested_inputs_outputs_and_snapshots` |
| `structures-CraftingReagentInfo-086` | proven-by-test | `patch_12_0_0_crafting_shapes::crafting_order_nested_inputs_outputs_and_snapshots`, `patch_12_0_0_crafting_shapes::crafting_nested_allocations_consume_items_currency_and_publish_returns` |
| `structures-CraftingReagentInfo-087` | proven-by-test | `patch_12_0_0_crafting_shapes::crafting_order_nested_inputs_outputs_and_snapshots`, `patch_12_0_0_crafting_shapes::crafting_nested_allocations_consume_items_currency_and_publish_returns` |
| `structures-CraftingReagentSlotSchematic-088` | proven-by-test | `patch_12_0_0_crafting_shapes::profession_crafting_slot_shapes_and_variable_quantities` |
| `structures-CraftingReagentSlotSchematic-089` | proven-by-test | `patch_12_0_0_crafting_shapes::profession_crafting_slot_shapes_and_variable_quantities` |
| `structures-CraftingResourceReturnInfo-090` | proven-by-test | `patch_12_0_0_crafting_shapes::crafting_nested_allocations_consume_items_currency_and_publish_returns` |
| `structures-CraftingResourceReturnInfo-091` | proven-by-test | `patch_12_0_0_crafting_shapes::crafting_nested_allocations_consume_items_currency_and_publish_returns` |
| `structures-NewCraftingOrderInfo-110` | proven-by-test | `patch_12_0_0_crafting_shapes::crafting_order_nested_inputs_outputs_and_snapshots`, `patch_12_0_0_crafting_shapes::crafting_order_secret_arguments_obey_caller_taint` |
| `structures-NewCraftingOrderInfo-111` | proven-by-test | `patch_12_0_0_crafting_shapes::crafting_order_nested_inputs_outputs_and_snapshots`, `patch_12_0_0_crafting_shapes::crafting_order_secret_arguments_obey_caller_taint` |
| `structures-RegularReagentInfo-114` | proven-by-test | `patch_12_0_0_crafting_shapes::crafting_order_nested_inputs_outputs_and_snapshots` |
| `structures-RegularReagentInfo-115` | proven-by-test | `patch_12_0_0_crafting_shapes::crafting_order_nested_inputs_outputs_and_snapshots` |
| `prose-undated-005` | proven-by-test | `patch_12_0_0_prose::prose_combat_log_registration_errors_without_delivery` |
| `prose-undated-030` | still-pending | Captured prose supplies no operation-specific secure/tainted arithmetic, comparison, concatenation, indexing, branching or rendering oracle. Existing VM secrecy predicates/argument guards do not prove that full contract; rilua dependency edits outside this worktree are unauthorized. |
| `deprecated api-removal-summary-128` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-GetSpellInfo-130` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-GetNumSpellTabs-131` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-GetSpellTabInfo-132` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-GetSpellCooldown-133` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-GetSpellBookItemName-134` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-GetSpellTexture-135` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-GetSpellCharges-136` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-GetSpellDescription-137` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-GetSpellCount-138` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-IsUsableSpell-139` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-C_TaskQuest-GetQuestsForPlayerByMapID-142` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-GetMerchantItemInfo-143` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-C_ChallengeMode-GetCompletionInfo-144` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-C_MythicPlus-IsWeeklyRewardAvailable-145` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-IsActiveQuestLegendary-148` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-C_QuestLog-IsLegendaryQuest-149` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-C_QuestLog-IsQuestRepeatableType-150` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-ConsolePrint-153` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-message-154` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-IsSpellOverlayed-157` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |
| `deprecated api-IsArtifactRelicItem-160` | proven-by-test | `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` |

All deprecated outcomes are **current native/cached** proof. Exact-12.0.0 historical execution remains pending: `on_update.rs:61` references a module gated to 12.0.5; task and original master produce the same E0433. That unrelated feature-gate defect is untouched.

## Commands and results

All helper calls used the absolute worktree script and `--build-host local`, debug, existing target directory. Exact argv, revisions, exit codes and invalidations live in the machine ledger. Ignored local raw logs are at the worktree root (`*-baseline.log`, `*-complete.log`, `prose-master.log`, `historical-{final,master}.log`).

| Filter / gate | Master e9baac195 | Final relevant proof |
|---|---|---|
| integration trade_skill | 14 pass | 14 pass |
| integration crafting | 28 pass | 36 pass, including eight B08 tests |
| integration profession | 97 pass / 1 ignored | 98 pass / 1 ignored |
| integration crafting_order | 1 pass | 5 pass |
| prose tainted combat-log test | Fails: RegisterEvent succeeds | 2 prose tests pass after script-registration exclusion and callback-control fixture correction |
| integration autoroll_compat / register_event | Relevant registration RED above | 18 / 7 pass |
| prefork_full_ui patch_12_0_0_deprecated_retirement_and_successors | Existing bounded base contract | 1 pass; all 21 retirements and 20 listed successor probes |
| exact-12.0.0 native retirement | Compile fail E0433 | Same compile fail; historical execution pending |
| four publication sweeps alone, --test-threads=1 | Existing baseline | Each 1 pass / 0 fail at final runtime |
| cargo fmt --check, explicit rustfmt check for dynamic integration modules | — | Exit 0 |
| local cargo check / build | — | Exit 0; no new warnings |
| startup --no-addons --no-saved-vars lua-errors, timeout 90 after separate build | — | Exit 0, application JSON [] |

Filter counts are controller summaries; child prefork controls also pass and are not double-counted. Six pre-existing vendor manifest deprecated Clippy-key warnings remain untouched. Exact-epoch compilation also reproduces three pre-existing unused imports at master. No broad full-project suites or profile-check matrix were run.

## INFERRED and resistant boundaries

[Contract](../../../../docs/specs/crafting-reagent-contracts.md#out-of-scope) owns inferred policy: exclusive item/currency identity, strict integral i32 argument bounds, complete required allocations with over-allocation permitted, local monotonic order IDs, and immediate crafting-result timing. Comments mark these decisions. Order container metadata is required host state, never fabricated sentinels; rejected requests preserve host responses. Item GUID/link reuse existing inventory/catalog producers.

`prose-undated-030`: captured text says only that some operations are restricted. It does not supply arithmetic/comparison/concatenation/indexing/branching/rendering outcomes for secure versus tainted paths. [Existing numeric-ordering contract](../../../../docs/specs/secret-number-ordering.md) explicitly remains inferred, not native operation parity. Argument authentication tests cannot substitute for that row; dependency changes outside the worktree are unauthorized.

## Commits

`590f67542`, `aff4a2e58`, `5c5f0dbe0`, `fc3bbcb64`, `76335ea10`, `d4fa9be12`, `e069e4ef3`, `6d6d56013`, `343a269b1`, `900d756ec`. Final documentation/proof commit follows; it changes no tested runtime code.
