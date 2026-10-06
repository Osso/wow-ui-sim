# Retired 11.x API successors

Retail 12.0.0+ keeps all 21 symbols in `tests/data/patch_12_0_0_deprecated.json` absent natively and after cached Blizzard loading. Successor probes must pass without a known-gap allowance. Earlier legacy publication tests remain epoch-gated; model tests use current namespaces.

| Successor | Contract and modeled scope |
|---|---|
| C_SpellBook.GetSpellBookItemName | Name and subName from seeded spellbook/spell catalog; invalid slot has no name. |
| C_Spell.GetSpellTexture | iconID and originalIconID are file data IDs, never paths; unknown spell returns nothing. 12.1.0 also returns nil conditionalIconID until conditional icons are modeled. |
| C_MerchantFrame.GetItemInfo | Ordered `merchant_items` selects catalog identity. Missing slot/item returns nothing. ID-only host offers mean free, single-unit, unlimited stock, purchasable/usable, no extended cost or quest starter; merchant economics are not modeled. |
| C_ChallengeMode.GetChallengeCompletionInfo | Host completion snapshot with all documented fields and members, independent of weekly run history. Empty state yields documented zero/false/nil/empty-list record; time is milliseconds. |
| C_Log.LogMessage | Appends message to simulator console sink, returns zero values. |
| C_SpellActivationOverlay.IsSpellOverlayed | Boolean host proc membership; absent membership is false. No automatic proc generation. |

`C_Spell.IsSpellUsable` identifies usable known spells independently of cooldown readiness; cooldown model tests observe `C_Spell.GetSpellCooldown().isActive`. Removed weekly-reward API has no listed successor, so its legacy state-surface tests are gated rather than assigning a new contract to another namespace.

Proof: native behavior tests in `tests/retirement_successors.rs`, model migrations in existing integration tests, and cached retirement/successor proof in `tests/patch_12_0_0_deprecated.rs`. Cached generated API docs and unmodified consumer Lua are the compatibility target. No page-coverage ledger changes.

## Fallout test disposition

Gated tests retain their earlier-epoch assertions. The existing 21-row retirement case covers Retail absence; no duplicate absence tests were added.

| Original test | Disposition |
|---|---|
| c_item_api::globals_and_inventory::test_get_spell_cooldown | Migrated: C_Spell.GetSpellCooldown fields. |
| c_mythic_plus_probes::is_weekly_reward_available_false_by_default | Gated: not retail-12-0-0; removed API has no listed successor. |
| c_mythic_plus_probes::is_weekly_reward_available_toggle | Gated: not retail-12-0-0. |
| cooldown_probes::get_spell_cooldown_reads_spell_cooldowns_entry | Migrated: C_Spell.GetSpellCooldown, active/enabled state and configured span. |
| cooldown_probes::get_spell_cooldown_zero_when_no_cooldown | Migrated: C_Spell.GetSpellCooldown zero span/enabled/modRate. |
| global_function_diff_coverage::curated_extra_global_functions_exist_at_runtime | Fixed: retired IsArtifactRelicItem removed from both curated set and diff file. |
| item_socket_info::artifact_relic_detection_supports_item_ids_and_links | Migrated: C_ItemSocketInfo.IsArtifactRelicItem, including full hyperlink input. |
| legacy_macro_spell_globals::legacy_macro_and_spellbook_globals_exist_for_addons | Gated: not retail-12-0-0, legacy publication assertion. |
| legacy_macro_spell_globals::legacy_spell_globals_exist_for_addons | Gated: not retail-12-0-0, legacy publication assertion. |
| map_canvas_pins::test_default_map_has_world_quests | Migrated: C_TaskQuest.GetQuestsOnMap. |
| map_canvas_pins::test_isle_of_dorn_has_seeded_world_quests | Migrated: C_TaskQuest.GetQuestsOnMap. |
| spell_state_probes::is_usable_spell_false_on_cooldown | Migrated/renamed: spell_readiness_false_on_cooldown checks C_Spell.IsSpellUsable and GetSpellCooldown.isActive separately. |
| spell_state_probes::is_usable_spell_true_when_known_and_no_cooldown | Migrated: C_Spell.IsSpellUsable. |
| talent_spec_probes::get_num_spell_tabs_returns_one | Migrated/renamed: get_num_spell_book_skill_lines_returns_seeded_lines checks seven seeded C_SpellBook skill lines. |
| talent_spec_probes::get_spell_tab_info_nil_for_out_of_range_index | Migrated/renamed: get_spell_book_skill_line_info_nil_for_out_of_range_index uses C_SpellBook.GetSpellBookSkillLineInfo. |
| talent_spec_probes::get_spell_tab_info_returns_class_name_and_spec_id | Migrated/renamed: get_spell_book_skill_line_info_returns_class_and_spec_data checks Paladin and Retribution skill lines. |
| world_quest_api::test_get_quests_for_player_by_map_id_matches_get_quests_on_map | Gated: not retail-12-0-0; specifically compares legacy alias with current API. |

Two additional assertions were exposed by corrected successor contracts: spell_api texture test now asserts file IDs; hero_talents icon test now resolves the actual ID through the texture manifest and loads nontransparent asset pixels. The lib startup test expects nil for empty merchant inventory, not a fabricated item record.

## Verification — 2026-10-05

[Revision-scoped proof ledger](../../data/patch-api/evidence/12.0.0-session-2026-10-05/retirement-fallout-proof.json) records commands, exact revisions, failures and scope invalidations. Raw logs remain locally at `target/retire-proof/`. All build-helper invocations selected `--build-host local`; debug/default Retail and the existing target directory only.

Runtime revision `d978182f0`; final test-only correction `130cfdd5d`:

| Gate | Observed result |
|---|---|
| Original fallout RED | 0 pass / 17 fail, all listed names reproduced. |
| Six successor RED | 0 pass / 6 fail before implementations. |
| Final bounded integration controls | 28 pass / 0 fail; includes all 12 nongated fallout cases, seven successor behaviors, diff controls, corrected hero asset and spell texture tests. |
| Full prefork_full_ui | 2,016 pass / 3 fail / 2,019 total. Both deprecated housing catalog cases and catalog_shop match supplied known failures. patch_12_0_0_deprecated passes within the full run. |
| Full integration with four requested skips | 10,648 pass / 27 fail / 18 ignored / 8 filtered. 25 failures match supplied baseline; additional hero path assertion was corrected and passes scoped control; extra aura flake reproduced unchanged at original base. No original fallout failure remains. |
| Four publication sweeps, individually --test-threads=1 | Each 1 pass / 0 fail. |
| Full lib, once | 1,970 pass / 7 fail. Six residual failures reproduced at original base; new empty-merchant assertion was corrected and exact scoped lib control passes (1 pass / 0 fail). |
| cargo fmt --check; local debug check/build | Exit 0; no new compiler warnings. Six pre-existing vendor manifest deprecated Clippy-key warnings remain unsuppressed and untouched. |
| Startup --no-addons --no-saved-vars lua-errors, timeout 90 | Exit 0, stdout []. |

Full suites were not repeated after the two test-only assertion corrections. Their implementation proof remains valid; focused controls cover changed assertions. The initial short-name lib filter with --exact ran zero tests and is explicitly invalidated in the ledger; the corrected fully qualified filter ran and passed one test.

### Baseline comparison and limitations

Original base `f1e59afe5` was temporarily restored **only inside this worktree** for bounded controls, then restored to committed task source/tests with no tracked diff. The unchanged random-buff test failed on attempt 25 at the same `duration > 0.0` boundary: default buff selection can put permanent Retribution/Devotion Aura (duration zero) first. That unrelated fixture defect was not changed.

The six original-base lib failures are transmog_situation enum values, debug_environment_defaults, housing_catalog_state, and three workarounds_editmode apply_system_anchors tests (cast/player sizing, nil singletons, compact unit startup refresh). They remain outside this task.

Five supplied integration baseline failures did not occur in this run: addon save, transmog temporary-shim classification, empty console command catalog, edit-mode profile option enums, and text shaping budget. This is a comparison, not a claim that unrelated behavior was fixed.

Raw per-symbol rg finds spelling examples inside the commented SpellBook transition guide and real calls in non-Mainline Commentator/TalentUI Classic/Mists/Cata files retained in the cache. Those are not active Retail consumers; Mainline Commentator TOC selects Mainline files. Current native/cached retirement proof and startup remain clean. Do not describe the entire cache tree as containing no matching text.

The cached successor cooldown probe clears inherited GCD/spell cooldown state and compares an eight-second floating span with 1e-9 tolerance, avoiding exact equality on `(start + duration) - start`. It still asserts active/enabled state and modRate; no runtime cooldown producer or vendor consumer was changed.

## Commits

- 17a4be11a — migrate/gate original fallout tests and remove retired extra-global entry.
- 706ad1957 — six successor contracts, state, native tests, zero-gap cached assertion and documentation.
- 821786bb9 — supported stack index/array write corrections.
- 47b1da4f4 — supported Lua integer result type in texture assertion.
- d978182f0 — isolate cached cooldown proof and simplify record serialization.
- 130cfdd5d — correct hero asset and empty merchant assertions.
