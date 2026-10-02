# Tooltip spell and mount identifiers

Retail 12.0.5 `C_TooltipInfo.GetMountBySpellID` and `GetSpellByID` must accept the chosen shared public spell-identifier model and enforce their documented NeverSecret optional arguments. Bounded batch56 implementation and saved parent GREEN are observed; independent acceptance remains pending. No native-verified capability is claimed. See [Lua API architecture](../lua-api.md) and [frame data flow](../frame-data-flow.md).

## What it must do

### Identifiers and meaningful payloads

- [ ] Use the existing strict public `c_spell::read_public_spell_identifier_at` contract: finite integral u32 NUMBER or UTF-8 STRING, with seeded lowercase aliases preceding numeric identity. This representation/validation policy is **inferred**, not native argument-type evidence.
- [ ] Numeric 19750 retains the real generated Flash of Light title, modeled cast/description lines, spell type, ID and width. Numeric mount 23338 retains the first actual default `world.mounts` match, Swift Palomino, its title and existing `Summons this mount.` description.
- [ ] DTO equivalence compares Color values through exactly four public numeric RGBA components (`GetRGBA()` or public numeric `r/g/b/a`), not per-instance method/object identity. All non-color fields retain strict recursive value/type equality with diagnostic paths.
- [ ] Explicit numeric-string, case-normalized name and full colored-link aliases produce DTO-equivalent results to their resolved numeric IDs for both queries. A seeded numeric alias wins over numeric identity; a full-link alias wins over its embedded number. Output ID is the resolved ID.
- [ ] Read alias changes/removal live. Queries do not mutate aliases or declared mounts; independent environments remain isolated. Mutating a returned DTO cannot change later results or another returned DTO.
- [ ] Preserve current unknown numeric policy: exactly one identified, line-empty, Spell-typed tooltip, including valid u32 endpoints.
- [ ] **INFERRED new unresolved-public-STRING miss policy:** exactly one fresh, unidentified, empty Spell-typed TooltipData with a fresh lines table. Never invent an ID. Unseeded numeric strings, catalog names, short links and full colored links remain misses; no general parsing/catalog-name lookup/acquisition/override graph.
- [ ] Results and nested DTO fields remain public under the chosen simulator contract. Native output secrecy remains unknown.

### Secret boundary and caller context

- [ ] Reject authentic VM-secret optional arguments in all documented positions, whether caller is secure or ordinarily tainted, before alias/catalog/payload acquisition. Mount arg2 `checkIndoors`; spell args2–6 `isPet`, `showSubtext`, `dontOverride`, `difficultyID`, `isLink`.
- [ ] **INFERRED conservative arg1 policy:** reject secret identifiers through the same public identifier helper without unwrapping or inspecting payloads. Native `AllowedWhenTainted` permissions are unknown; rejection does not establish them.
- [ ] Reject actual host-secret BOOL true and false, NUMBER, STRING, wrapped real Frame table and ordinary table across every optional position and both arg1 boundaries. Known numeric/name inputs and numeric/string misses cannot bypass NeverSecret checks.
- [ ] Failed queries preserve authentic wrapper secrecy, global/list/stack roots, live wrapper allocation identity, ordinary caller properties and owned state maps; valid public recovery still works after forced GC and failures.
- [ ] Preserve stack taint for rejection and ordinary public calls. Restore secure context after returning from an explicitly tainted closure.
- [ ] **INFERRED error policy:** provide nonempty public API context, without private string payload disclosure. Exact native errors and precedence are unknown; tests do not assert source text or a native error message.

### Public optional compatibility and frame route

- [ ] Preserve **current ignored-provider behavior**, not new flag semantics: documented nil/BOOL combinations for mount arg2 and spell args2/3/4/6, plus nil or public number17 at spell arg5, leave actual payload unchanged. No new public optional type/domain/finite validation is required for unused flags.
- [ ] Actual `GameTooltip:SetSpellByID` and `SetMountBySpellID` pass the original identifier to the same real namespace query. Numeric/name/full-link aliases preserve every direct-query TooltipData field/value, including public RGBA components, and equivalent actual rendered lines. Processing may add per-line `lineIndex` only: when present it must be public numeric and equal the actual line position. Other extra fields or lines fail equivalence. No query replacement, method-call spy, VM-shape assertion or duplicate builder counts as frame proof.
- [ ] Secret frame identifiers and mount arg2 fail without replacing prior exposed TooltipData or rendered lines; valid public frame recovery remains meaningful.

## How it works

- [Lua API architecture](../lua-api.md)
- [Frame data flow](../frame-data-flow.md)
- [Widget system](../widget-system.md)

## Implementation inventory

### Exact declarations and four uncredited source IDs

`data/patch-api/sources/12.0.5-api-changes.txt`, register `12.0.5-register.json`, and coverage `12.0.5-page-coverage.json` retain these exact IDs:

| Source ID | Literal delta | Current gap |
| --- | --- | --- |
| `global api-C_TooltipInfo-GetMountBySpellID-333` | arg1.Type number → SpellIdentifier | Shared strict public alias-first resolver implemented; Saved GREEN observed; independent acceptance pending. |
| `global api-C_TooltipInfo-GetMountBySpellID-334` | arg2 NeverSecret | Actual VM-secret rejection before lookup implemented; Saved GREEN observed; independent acceptance pending. |
| `global api-C_TooltipInfo-GetSpellByID-336` | arg1.Type number → SpellIdentifier | Shared strict public alias-first resolver implemented; Saved GREEN observed; independent acceptance pending. |
| `global api-C_TooltipInfo-GetSpellByID-337` | arg2–6 NeverSecret | Actual VM-secret rejection before lookup implemented; Saved GREEN observed; independent acceptance pending. |

All four currently have `audit-pending` status and empty capabilities. This implementation changes no source/register/coverage/accounting. Parent's 214 pending / 134 bounded / 14 partial checkpoint receives no credit from these fixtures.

Actual profile cache inspected October 2, 2026:
`~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua`, complete function blocks at lines629–644 and1045–1063:

| Query | Required arg1 | Nilable NeverSecret optional arguments |
| --- | --- | --- |
| `GetMountBySpellID` | `spellID: SpellIdentifier` | arg2 `checkIndoors: bool` |
| `GetSpellByID` | `spellID: SpellIdentifier` | arg2 `isPet: bool`; arg3 `showSubtext: bool`; arg4 `dontOverride: bool`; arg5 `difficultyID: number`; arg6 `isLink: bool` |

Both declare `SecretArguments = "AllowedWhenTainted"`, `MayReturnNothing = true`, and a `data: TooltipData` return with `Nilable = false`. These are declarations, not runtime/error/permission/miss evidence. The selected unresolved-string policy does **not** claim native MayReturnNothing semantics.

### Existing meaningful producers and state

- `src/c_api/c_tooltip_info_spell_mount.rs`: owns both handlers and their sole `retail-12-0-5` publication. Rejects actual VM-secret arg1 and all declared optional positions before shared identifier resolution; public errors include method, argument index and policy without payload inspection or taint changes. Ordinary public optionals remain ignored.
- `src/c_api/mod.rs`: feature-gates the focused module under `retail-12-0-5`.
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs`: invokes the C API registrar after the namespace exists in globals. A single crate-visible `tooltip_for_spell_identifier` bridge dispatches resolved numeric IDs to unchanged shared producers, or unresolved strings to the existing fresh empty Spell DTO builder. Only the tooltip module visibility is widened; child producers/builders remain private.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs`: both old numeric handlers and registrations compile only without `retail-12-0-5`; earlier profile behavior remains unchanged.
- `src/lua_api/globals/missing_surface/tooltip_info/spell.rs`: shared finite spell builder and mount builder, spell catalog first then `world.mounts`; retains numeric unknown identity. Mount description is currently at sparse lines index3.
- `src/lua_api/globals/missing_surface/tooltip_info/builders.rs`: existing fresh empty/identified TooltipData builders and line/color construction; reused later, never copied.
- `data/spells.rs`: real generated 19750 entry at lines52582–52588, name `Flash of Light`; no fake production catalog entry.
- `src/spell_lookup.rs`: existing spell catalog facade; 23338 has no spell row in inspected generated catalog and reaches host mount state.
- `src/lua_api/state_defaults.rs`: actual default mount18 Swift Palomino, spell23338, icon132261, collected/usable true, mount_type230. A later default mount69 shares23338; current first match wins. Tests assert the existing fixture precondition rather than fabricate catalog data.
- `src/lua_api/state_types/collections.rs`: declared `MountData` fields; no new state/producer fixtures or modified model.
- `src/c_api/c_spell.rs`: already crate-visible strict public identifier helper and shared alias-first resolver over existing `spell_id_aliases`; unchanged.
- `src/lua_api/frame/methods/widgets/tooltip/content.rs`: actual setters forward original argument values to the real namespace query, store exposed `processingInfo.tooltipData`, and apply rendered lines. Mount forwards arg2; spell setter currently forwards only arg1.
- `src/lua_api/frame/methods/widgets/tooltip/line_data.rs`: **frame precondition limit**: `GetSpell()` uses `TooltipData.spell_id`, which setters populate only from original numeric inputs. String aliases currently have no GetSpell name/ID; numeric mount23338 names fall back to `Spell 23338`, not the mount title. Thus these fixtures compare actual rendered lines and exposed query DTO, not an invented GetSpell equivalence. No frame patch is authorized here.
- `src/lua_api/methods.rs`: original frames are backed `Val::Table` objects, not presumed Userdata. Tests require actual `GetObjectType()` and existing table backing metadata before wrapping the real frame.

Namespace registration uses the constructor's `GcRef<Table>`, already globally rooted before callback installation. Shared payload constructors return `Val`; handlers push that value immediately before any further allocation. Existing producer allocation/rooting and frame processing behavior are unchanged. No whole-tooltip refactor, copied builders, catalog/state additions, color/frame payload patch or vendor behavior patch.

## Tests asserting this spec

`tests/tooltip_spell_mount_identifiers.rs`: **22 focused tests**, `#[cfg(feature = "retail-12-0-5")]`. Existing `build.rs` discovers top-level test modules into the grouped `integration` target (`tests/integration.rs`); no new Cargo target or harness edit.

| Capability | Fixture cases | Proof level |
| --- | --- | --- |
| Real spell/mount DTO and aliases | Numeric producer guards; both alias DTO families; numeric precedence; link precedence; live changes | Historical numeric controls PASS; initial alias failures observed. Corrected compiled RED pending. |
| Miss/read-only/isolation/strictness | Numeric endpoints; unidentified public string misses; result mutation/freshness; two environments; invalid identifiers | Corrected parent RED: controls PASS; alias/miss/read-only/isolation/strictness failures genuine. Saved GREEN observed; independent acceptance pending. Chosen strictness/string miss inferred. |
| Public flags and taint | Documented ignored optional combinations; secure/ordinary-tainted public DTO equivalence | Corrected public RGBA baseline passes before alias boundary failure. Saved GREEN observed; independent acceptance pending. Ignored flags receive no semantic credit. |
| Authentic secret boundary | All six VM secret kinds for arg1/mount arg2; three paired kind matrices for each spell position; forced GC | Corrected parent compiled RED observed genuine failures; Saved GREEN observed; independent acceptance pending. Optional ordering requires parent source audit, not claimed read-observation instrumentation. |
| Actual frame consumer | Original identifier DTO/rendered-line equivalence; rejected secret inputs retain prior payload and recover | Corrected query-preservation/lineIndex oracle reaches genuine alias boundary failure; Saved GREEN observed; independent acceptance pending. Secret retention test unchanged. GetSpell alias identity and full frame optional semantics excluded. |

GC identity snapshots compare actual host `Val`, userdata GcRef and live allocation sequence before/after returning to the secure host boundary, then compare globally rooted list/stack-export entries. Metadata snapshots do not create VM roots or inspect private payloads. Lua verifies secrecy and original caller/frame/table properties. Pinned rilua6044544 denies tainted secret-BOOL `rawequal`; these fixtures never require it or relax queries. Secret publication roots wrappers during insertion and roots original real tables while wrapping them.

Existing meaningful controls remain in `tests/tooltip_mount.rs`, `tests/tooltip_item_sources.rs` and `tests/tooltip_item_spell.rs`: numeric mount title/ID and frame lines, generated spell identity, and actual tooltip content. They do not establish the new alias/NeverSecret contract. Batch53 `tests/break_up_large_numbers.rs` supplies the authentic host-wrapper/root identity pattern, not a replacement tooltip query.

### Input-stage proof ledger

- Base inspected: `b1d8486563b85e02ec4e2d9787833020e4a95d47`; unrelated dirty source preserved without body access.
- Owned formatter only: `rustfmt --edition 2024 --config skip_children=true tests/tooltip_spell_mount_identifiers.rs`, explicit repository cwd. Result recorded in handoff; no compiler invoked.
- Historical input revision `7ea54c824e3dfd4e1ccaebc0fad6399baa5de45c`: parent `cargo test --test integration --no-run --message-format=json` exited0 in104.26011972106062s; selected22 run exited101 with3PASS/19FAIL in3.852031323942356s. Artifacts: `/tmp/patch-12.0.5-batch56-red-build-result.json`, `/tmp/patch-12.0.5-batch56-red-run.json` and saved full outputs. This is mixed fixture/producer evidence, not genuine corrected RED: public-nil/repeat numeric DTO and numeric frame comparisons failed the old scalar oracle; remaining17 failures include alias input/security/strictness boundaries, not17 established root causes.
- Read-only diagnosis `/tmp/patch-12.0.5-tooltip-dto-equivalence-diagnosis.md` inferred fresh-color identity as a likely fixture cause, not confirmed channel behavior. `builders::color_table` invokes `CreateColor`; naked-environment `color_defaults.rs` creates per-instance methods, while cached `Blizzard_SharedXMLBase/Color.lua` uses `ColorMixin`. Both `GetRGBA()` contracts return `r/g/b/a`; comparator now extracts only these four public numeric components without comparing method identity. No executed corrected fixture proof yet.
- Parent's separate full-UI diagnostic `/tmp/patch-12.0.5-tooltip-dto-diagnostic.lua` and `.stdout/.stderr` reportedly exited0 in5.204861s. Saved output has no repeated numeric spell/mount differences and reports spell-frame additions `lines.1..4.lineIndex`; this does not prove naked-environment color equivalence. Frame oracle now preserves original query fields and validates only correctly positioned lineIndex enrichment; existing rendered-line and secret failure-retention assertions remain.
- Correction `215f0bb84cc86b611dcc2f4288f87754436967b6` changes only semantic public RGBA comparison and equivalent frame fields/lineIndex. Parent corrected compilation exited0 in58.19581049506087s; selected22 exited101 with3PASS/19 genuineFAIL in3.5979648019419983s. Numeric19750, default mount23338 and unknown-numeric controls PASS; both former DTO scalar baseline failures now pass their numeric baseline before actual alias number/string boundary failure. Artifacts: `/tmp/patch-12.0.5-batch56-red-fixed-build-result.json`, `-build.jsonl`, `-build.stderr`, `-run.json`, `-run.stdout`, `-run.stderr`. Executable SHA256 `74043f3c4ada1833904a24f00c16d89250eeeca1f4362f1a3b62a2e616e2cbc1`. Proof includes preserved unowned dirty source hash `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`, not clean-revision proof. Historical uncorrected run is not the producer RED gate.
- Implementation uses that corrected compiled RED; no new tests/build/check/readability/coverage/gates/operations/delegation/model CLI/push were run by the implementer. Owned Rust formatting only; parent must compile the changed producer and observe GREEN/source audit/acceptance. No requirement boxes or four source IDs are promoted.

## Known gaps (current cycle)

Corrected parent compiled RED is recorded above; the two focused C API boundaries are implemented. Fixture preconditions remain explicit and unchanged; no monkey-patching or weakened query assertions.
- [ ] Fresh independent source-audit/acceptance; saved parent GREEN reconciled below, four source rows remain uncredited until accepted proof.
- [ ] Ordinary optional flags are currently ignored: indoor eligibility, pet/subtext/override selection, difficulty and link meanings remain **gaps**, not semantic support.
- [ ] Native alias vocabulary, arg types, permissions for secret arg1, error precedence, miss behavior and result secrecy remain unknown. Native probes are unavailable and not a completion gate for this chosen simulator policy.

## Out of scope

- Item330/331 `GetItemByID` contexts remain separate and pending; no item context model.
- Production state changes and producer work before parent compiled RED.
- Generic string parsing, catalog acquisition, fake catalog entries, mount acquisition, override graph and whole-tooltip refactor.
- New semantics or unrequested public optional type/domain/finite validation; old-profile contract changes.
- GetSpell alias-ID bookkeeping changes, missing frame optional forwarding, sparse mount-line layout redesign and output/native parity claims.
- Vendor patches, accounting promotion, broad gates, builds, test execution, push and native probes in this slice.

## Reconciled batch56 saved parent GREEN — 2026-10-02

### Observed scope, not acceptance

Producer `fb9d47a5ecbd450d9b869becec90b12fb559de3d` owns only exact source333/334/336/337 boundaries: sole epoch125 C API publication, older registrations/handlers disabled there, shared meaningful payload bridge, no new catalog/model. No producer, Frame or Color semantics were changed by fixture correction `215f0bb84cc86b611dcc2f4288f87754436967b6` or this reconciliation.

Historical initial RED remains3PASS/19FAIL with two DTO identity-oracle defects mixed into producer failures. Corrected RED remains3PASS/19genuineFAIL: numeric baselines now succeed, then string alias/rejection/public-context boundaries fail. The one-off full-UI `/tmp/patch-12.0.5-tooltip-dto-diagnostic.lua`, `.stdout`, `.stderr` confirms only four spell-line `lineIndex` enrichments, each matching its position. No `dataInstanceID` is inferred. Correction compares four public numeric RGBA components and preserves every other DTO field plus actual rendered-line checks; no query replacement or frame patch.

| Capability | Saved observation | Limit |
| --- | --- | --- |
| Numeric spell19750/mount23338, seeded numeric/name/full-link aliases | Focused22 PASS; meaningful title/description/ID and alias precedence, live updates, isolation, result mutation | Finite existing catalog/mount state; no acquisition/override model or native vocabulary proof |
| Unresolved strings and unknown numeric IDs | Fresh unidentified empty Spell DTO for strings; identified empty numeric DTO including u32 endpoints | String miss policy inferred; native MayReturnNothing unknown |
| Secret boundaries and caller roots | Actual optional secret kinds at mount2/spell2–6 reject before lookup; arg1 conservatively rejects; GC/root/wrapper/caller preservation and recovery PASS | Ordering also supported by producer source; no native permission/error/type/secrecy parity or read instrumentation claim |
| Public flags | Documented public combinations preserve payload and ordinary taint | Ignored compatibility inputs, not indoor/pet/subtext/override/difficulty/link semantics |
| Real frame route | Original identifiers reach actual query; DTO/RGBA/rendered lines, lineIndex and rejected-input preservation PASS | GetSpell string-alias identity and numeric mount title unsupported; missing full frame optional forwarding explicit gaps |

Fresh independent verifier and requirements/acceptance/data/pagecoverage/PLAN remain pending; no guessed verifier ID. All requirement boxes remain unchecked. Four source rows remain uncredited;214 pending/134 bounded/14 partial unchanged. Item330/331 remain pending. Native errors, permissions, types, outputs, catalogs and other-profile runtime behavior remain unknown. Bounded development evidence only, not final whole-page/goal or native acceptance.

### Commands, costs and provenance

All three saved compiler invocations: `cargo test --test integration --no-run --message-format=json`, repository cwd, exit0. Initial compile104.26011972106062s; corrected compile58.19581049506087s; GREEN compile**139.876873968984s**, separate from execution. Compiler JSONL was consumed completely; completion records and stderr retained.

Each GREEN filter below ran `timeout 90 target/debug/deps/integration-a11e89d240f9bd0c <filter> --test-threads=1` (saved commands use absolute executable paths). All five exit0; no duplicate successful names. Startup: `timeout 90 target/debug/wow-sim --no-addons --no-saved-vars lua-errors`, exit0, stdout`[]`.

| Filter | Actual distinct PASS | Runtime seconds |
| --- | ---: | ---: |
| `tooltip_spell_mount_identifiers::` | 22 | 3.1291435519233346 |
| `tooltip::` | 123 | 31.119388338993303 |
| `tooltip_mount::` | 2 | 0.34843387990258634 |
| `tooltip_item_sources::` | 6 | 0.8382455360842869 |
| `tooltip_talent::` | 2 | 0.3523080999730155 |
| Startup | 0 Lua errors | 4.109140621963888 |

Exact execution+startup sum: **39.89666002884041434s**, below60s target; no padding. Earlier112-tooltip prediction was wrong: actual substring filter selects123, including11 extra named action_macro/Blizzard tests below. Total22+123+2+6+2 =**155 distinct PASS**.

Every run is dirty-combined proof, not a clean revision: preserved unowned dirty source diff SHA256 `6967f0b47312d926c2359bd29bc1c26d4d1d523522abf8d87e067104da170a1a`. That file body was not accessed. GREEN integration SHA256 `77f8f0f672d41491957b5cdc69f48f61cf69ff2334353b812a7c36dc38dde180`; wow-sim SHA256 `d9023a4a0579e72f0b9a8d5b82a4632784a84ab09e3e8cc4751d67e33c48ba03`. Saved build/run revision and executable hashes bind observation; later docs commits do not create new execution proof. Owned current source SHA256 snapshots (no clean-revision claim):

- `src/c_api/c_tooltip_info_spell_mount.rs`: `9fc34fc321217f382bc8025b54cd18701dc27da1c9f654db4c0f04868f810e83`
- `src/c_api/mod.rs`: `79ab8ac0ffb8be73f0e43cdf8f12488a0717ca23e580c49147c64337e422b7a0`
- `src/lua_api/globals/missing_surface.rs`: `dcad809c06164b55a0b82af718530f3e890bc0099d53449c13119ca78df9512e`
- `src/lua_api/globals/missing_surface/tooltip_info/mod.rs`: `4f2443a84c9bd6fe2b14d6499ad276bb2a20099ef68f5f021c9a730ec8f40969`
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs`: `5595585e22eebcf6ee2654e6facb8db6ca4bfc25d82797158b27727f3093252b`
- `tests/tooltip_spell_mount_identifiers.rs`: `24f2913ae56663dd6daa1cbe616cec98bfcce813ad51da15fcb127c6e7d815c9`

### Actual successful named coverage

#### `tooltip_spell_mount_identifiers::` — 22 PASS

```text
tooltip_spell_mount_identifiers::documented_public_optional_combinations_keep_current_ignored_payloads
tooltip_spell_mount_identifiers::forced_gc_preserves_stack_global_list_roots_and_caller_properties
tooltip_spell_mount_identifiers::frame_secret_inputs_fail_without_mutating_prior_actual_tooltip_payload
tooltip_spell_mount_identifiers::full_link_alias_wins_over_its_embedded_spell_number
tooltip_spell_mount_identifiers::known_mount_numeric_uses_existing_declared_host_mount
tooltip_spell_mount_identifiers::known_spell_numeric_keeps_generated_title_and_nonempty_lines
tooltip_spell_mount_identifiers::live_alias_changes_are_visible_without_cached_query_results
tooltip_spell_mount_identifiers::mount_never_secret_optional_rejects_all_vm_kinds_before_known_or_missing_payload
tooltip_spell_mount_identifiers::mount_number_name_and_colored_link_aliases_have_equivalent_dtos
tooltip_spell_mount_identifiers::numeric_alias_precedes_identity_and_returns_resolved_payload_id
tooltip_spell_mount_identifiers::numeric_unknowns_preserve_identified_line_empty_spell_tooltips
tooltip_spell_mount_identifiers::ordinary_tainted_public_calls_preserve_stack_taint_and_payload
tooltip_spell_mount_identifiers::original_frame_identifier_routes_match_direct_dto_and_actual_rendered_lines
tooltip_spell_mount_identifiers::result_mutation_and_repeated_queries_leave_payload_sources_read_only
tooltip_spell_mount_identifiers::secret_identifiers_conservatively_reject_all_actual_vm_representations
tooltip_spell_mount_identifiers::separate_environments_do_not_share_alias_updates
tooltip_spell_mount_identifiers::spell_never_secret_all_five_positions_reject_host_number_and_string
tooltip_spell_mount_identifiers::spell_never_secret_all_five_positions_reject_host_true_and_false
tooltip_spell_mount_identifiers::spell_never_secret_all_five_positions_reject_wrapped_real_frame_and_table
tooltip_spell_mount_identifiers::spell_number_name_and_colored_link_aliases_have_equivalent_dtos
tooltip_spell_mount_identifiers::strict_public_identifiers_reject_invalid_representations_in_both_contexts
tooltip_spell_mount_identifiers::unseeded_public_strings_return_one_unidentified_empty_spell_tooltip
```

#### `tooltip::` — 123 PASS

```text
action_macro_tooltip::action_macro_tooltip_query_absent_on_retail
action_macro_tooltip::explicit_macro_icon_is_returned_by_both_action_texture_queries_after_edit
action_macro_tooltip::macro_icon_queries_follow_move_clear_delete_and_empty_icon
action_macro_tooltip::spell_action_texture_queries_remain_nonempty_and_equal
blizzard_azerite_respec_ui_button_disabled_tooltip::blizzard_azerite_respec_ui_button_disabled_tooltip_depends_on_money
blizzard_ui_blizzard_achievementui::behavior_guild_member_tooltip::check_guild_members_tooltip_no_ops_outside_guild_view_and_when_num_members_is_zero
blizzard_ui_blizzard_achievementui::behavior_guild_member_tooltip::check_guild_members_tooltip_pairs_odd_index_left_with_even_index_right_via_add_double_line
blizzard_ui_blizzard_actionbar::behavior_button_tooltip::action_button_on_enter_sets_tooltip_for_populated_slot
blizzard_ui_blizzard_actionbar::behavior_paragon_tooltip::paragon_watch_bar_tooltip_uses_seeded_reward_quest_and_progress
blizzard_ui_blizzard_actionstatus::behavior_update_parent_resets_strata_to_tooltip::update_parent_resets_frame_strata_to_tooltip
blizzard_ui_blizzard_ardenweald_gardening::behavior_onleave_hides_tooltip::onleave_hides_tooltip_after_each_onenter_branch
tooltip::tooltip_allow_empty::test_allow_show_with_no_lines_keeps_zero_line_tooltip_renderable
tooltip::tooltip_basic::test_adddoubleline_and_numlines
tooltip::tooltip_basic::test_addline_and_numlines
tooltip::tooltip_basic::test_appendtext
tooltip::tooltip_basic::test_clearlines_resets_count
tooltip::tooltip_basic::test_copy_tooltip_copies_lines_and_spell_data_without_reowning
tooltip::tooltip_basic::test_createframe_gametooltip_type
tooltip::tooltip_basic::test_fadeout_hides_and_clears_owner
tooltip::tooltip_basic::test_gametooltip_exists_and_has_correct_type
tooltip::tooltip_basic::test_gametooltip_strata_is_tooltip
tooltip::tooltip_basic::test_getanchortype_after_setowner
tooltip::tooltip_basic::test_isobjecttype_for_other_types
tooltip::tooltip_basic::test_isobjecttype_frame_returns_true_for_gametooltip
tooltip::tooltip_basic::test_on_tooltip_cleared_fires_on_clearlines
tooltip::tooltip_basic::test_on_tooltip_cleared_fires_on_setowner
tooltip::tooltip_basic::test_other_tooltip_frames_exist
tooltip::tooltip_basic::test_repeated_identical_tooltip_refresh_keeps_cached_strata_buckets
tooltip::tooltip_basic::test_set_frame_stack_populates_lines_returns_frame_and_fires_script
tooltip::tooltip_basic::test_set_shapeshift_populates_spell_tooltip
tooltip::tooltip_basic::test_setminimumwidth_and_getminimumwidth
tooltip::tooltip_basic::test_setowner_and_isowned_and_getowner
tooltip::tooltip_basic::test_setpadding_and_getpadding
tooltip::tooltip_basic::test_settext_clears_and_sets_first_line
tooltip::tooltip_basic::tooltip_content_lifecycle_appends_remain_hidden_until_show
tooltip::tooltip_basic::tooltip_content_lifecycle_explicit_hide_releases_owner_when_already_hidden
tooltip::tooltip_basic::tooltip_content_lifecycle_setowner_hides_clears_and_retains_new_owner
tooltip::tooltip_basic::tooltip_content_lifecycle_settext_shows_owned_populated_tooltip
tooltip::tooltip_basic::tooltip_content_lifecycle_spell_payload_still_shows_after_setowner
tooltip::tooltip_basic::tooltip_owner_clear_lines_retains_owner
tooltip::tooltip_basic::tooltip_owner_hide_releases_owner_without_clearing_lines
tooltip::tooltip_basic::tooltip_owner_normal_frame_hide_does_not_change_tooltip_owner
tooltip::tooltip_basic::tooltip_owner_set_shown_false_releases_owner
tooltip::tooltip_cursor_dirty::adding_visible_tooltip_line_dirties_tooltip_rect
tooltip::tooltip_cursor_dirty::hidden_cursor_anchored_tooltip_does_not_dirty_on_mouse_move
tooltip::tooltip_cursor_dirty::inventory_tooltip_population_dirties_tooltip_rect
tooltip::tooltip_cursor_dirty::unchanged_cursor_tooltip_anchor_does_not_dirty_on_mouse_move
tooltip::tooltip_item_spell::inventory::test_set_bag_item_populates_tooltip
tooltip::tooltip_item_spell::inventory::test_set_inventory_item_empty_slot
tooltip::tooltip_item_spell::inventory::test_set_inventory_item_shows_tooltip
tooltip::tooltip_item_spell::inventory::test_set_inventory_item_tooltip_content
tooltip::tooltip_item_spell::spell_lines::test_add_line_does_not_invent_processing_info_value_color_segments
tooltip::tooltip_item_spell::spell_lines::test_add_line_preserves_explicit_processing_info_inline_color_segments
tooltip::tooltip_item_spell::spell_lines::test_clear_lines_clears_spell_id
tooltip::tooltip_item_spell::spell_lines::test_get_spell_returns_nil_when_no_spell
tooltip::tooltip_item_spell::spell_lines::test_get_spell_returns_spell_data_after_set
tooltip::tooltip_item_spell::spell_lines::test_set_spell_by_id_applies_full_line_inline_color_markup
tooltip::tooltip_item_spell::spell_lines::test_set_spell_by_id_applies_named_inline_color_markup
tooltip::tooltip_item_spell::spell_lines::test_set_spell_by_id_fires_on_tooltip_set_spell
tooltip::tooltip_item_spell::spell_lines::test_set_spell_by_id_get_left_line_does_not_invent_value_color_segments
tooltip::tooltip_item_spell::spell_lines::test_set_spell_by_id_get_left_line_uses_tooltip_line_color
tooltip::tooltip_item_spell::spell_lines::test_set_spell_by_id_makes_tooltip_visible
tooltip::tooltip_item_spell::spell_lines::test_set_spell_by_id_preserves_partial_inline_color_segments
tooltip::tooltip_item_spell::spell_lines::test_set_spell_by_id_unknown_spell_is_noop
tooltip::tooltip_item_spell::test_get_action_adds_colored_binding_line
tooltip::tooltip_item_spell::test_get_num_lines_returns_actual_count
tooltip::tooltip_item_spell::test_set_hyperlink_populates_lines
tooltip::tooltip_item_spell::test_set_hyperlink_short_format
tooltip::tooltip_item_spell::test_set_hyperlink_spell_link_populates_spell_tooltip
tooltip::tooltip_item_spell::test_set_item_by_id_6948_contains_hearthstone_line
tooltip::tooltip_item_spell::test_set_item_by_id_makes_tooltip_visible
tooltip::tooltip_item_spell::test_set_item_by_id_populates_lines
tooltip::tooltip_item_spell::test_set_spell_by_id_colors_cooldown_line
tooltip::tooltip_item_spell::test_set_spell_by_id_colors_title_and_metadata_lines
tooltip::tooltip_item_spell::test_set_spell_by_id_instant_cast
tooltip::tooltip_item_spell::test_set_spell_by_id_populates_lines
tooltip::tooltip_item_spell::test_set_spell_by_id_replaces_armor_placeholder_for_shield_of_the_righteous
tooltip::tooltip_item_spell::test_set_spell_by_id_replaces_damage_placeholders_in_description
tooltip::tooltip_item_spell::test_set_spell_by_id_replaces_shield_placeholders_from_player_health
tooltip::tooltip_item_spell::test_set_spell_by_id_shows_cast_time
tooltip::tooltip_item_spell::test_set_spell_by_id_uses_wrapped_description_for_tooltip_width
tooltip::tooltip_item_spell::test_set_unit_aura_by_aura_instance_id_populates_lines
tooltip::tooltip_item_spell::test_set_unit_aura_colors_like_spell_tooltip
tooltip::tooltip_item_spell::test_set_unit_aura_invalid_index_no_crash
tooltip::tooltip_item_spell::test_set_unit_aura_populates_lines
tooltip::tooltip_item_spell::test_set_unit_buff_by_aura_instance_id_respects_unit
tooltip::tooltip_item_spell::test_set_unit_buff_populates_lines
tooltip::tooltip_item_spell::test_set_unit_debuff_by_aura_instance_id_does_not_show_helpful_buffs
tooltip::tooltip_item_spell::test_set_unit_invalid_returns_false
tooltip::tooltip_item_spell::test_set_unit_party_member_populates_tooltip_and_fires_event
tooltip::tooltip_item_spell::test_set_unit_player_populates_tooltip
tooltip::tooltip_shrink_to_fit_wrapped::test_set_shrink_to_fit_wrapped_false_keeps_wrapped_line_width
tooltip::tooltip_text::test_add_atlas_increments_numlines
tooltip::tooltip_text::test_add_atlas_stores_atlas_name
tooltip::tooltip_text::test_add_double_line_without_color_uses_normal_font_color_for_both_sides
tooltip::tooltip_text::test_add_line_without_color_uses_normal_font_color
tooltip::tooltip_text::test_add_texture_increments_numlines
tooltip::tooltip_text::test_add_texture_stores_file_data_id
tooltip::tooltip_text::test_add_texture_with_string_id
tooltip::tooltip_text::test_addline_wrap_flag_stored
tooltip::tooltip_text::test_blank_unwrapped_line_does_not_collapse_wrapped_tooltip_width
tooltip::tooltip_text::test_clearlines_clears_texture_lines
tooltip::tooltip_text::test_double_line_width_includes_gap
tooltip::tooltip_text::test_get_custom_line_spacing_default_is_zero
tooltip::tooltip_text::test_get_left_line_after_set_item_by_id
tooltip::tooltip_text::test_get_left_line_has_correct_text
tooltip::tooltip_text::test_get_left_line_out_of_range_returns_nil
tooltip::tooltip_text::test_get_left_line_returns_fontstring
tooltip::tooltip_text::test_get_right_line_has_correct_text
tooltip::tooltip_text::test_get_right_line_no_right_text_returns_nil_text
tooltip::tooltip_text::test_set_custom_line_spacing_and_get
tooltip::tooltip_text::test_set_custom_line_spacing_on_custom_tooltip
tooltip::tooltip_text::test_set_custom_line_spacing_stores_in_tooltip_data
tooltip::tooltip_text::test_tooltip_fontstring_globals_exist
tooltip::tooltip_text::test_tooltip_height_grows_with_lines
tooltip::tooltip_text::test_tooltip_min_width_respected
tooltip::tooltip_text::test_tooltip_nineslice_child_accessible
tooltip::tooltip_text::test_tooltip_sizing_includes_padding
tooltip::tooltip_text::test_tooltip_sizing_skipped_when_hidden
tooltip::tooltip_text::test_wrapped_line_does_not_expand_width
tooltip::tooltip_text::test_wrapped_line_increases_height
tooltip::tooltip_text::test_wrapped_only_line_still_sets_tooltip_width
tooltip::tooltip_word_wrap_min_width::test_custom_word_wrap_min_width_expands_wrapped_only_tooltip_width
```

#### `tooltip_mount::` — 2 PASS

```text
tooltip_mount::c_tooltip_info_mount_by_spell_id_uses_seeded_mount_state
tooltip_mount::game_tooltip_set_mount_by_spell_id_populates_mount_lines
```

#### `tooltip_item_sources::` — 6 PASS

```text
tooltip_item_sources::c_tooltip_info_item_source_aliases_delegate_to_existing_paths
tooltip_item_sources::hyperlink_unit_guid_uses_modeled_player_and_target_tooltips
tooltip_item_sources::hyperlink_unsupported_and_malformed_links_return_nil_without_changing_item_spell
tooltip_item_sources::missing_unit_hyperlink_returns_nil_for_att_shaped_retry
tooltip_item_sources::tooltip_item_and_toy_payloads_retain_modeled_identity
tooltip_item_sources::tooltip_spell_identity_survives_missing_local_metadata
```

#### `tooltip_talent::` — 2 PASS

```text
tooltip_talent::c_tooltip_info_talent_reuses_spell_tooltips
tooltip_talent::game_tooltip_set_talent_populates_spell_lines
```

### Retained artifact ledger

Saved commands/results/full compiler and test outputs are authoritative, not a reconstructed rerun. SHA256 binds each artifact read for this reconciliation:

```text
8bc3b21debdf9532328ef300b914cb646a4f09c296b72a02465a397fffccb340  /tmp/patch-12.0.5-batch56-green-build-result.json
00bf53a5e3c782122b9992306e6439711064add87bce40c76113bc1cb532756b  /tmp/patch-12.0.5-batch56-green-build.jsonl
e6581ece4c4b1fd9bcdb70dc3c225fcc3b64229b760334f3bb2b907751ad1fe4  /tmp/patch-12.0.5-batch56-green-build.stderr
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/patch-12.0.5-batch56-green-run-0.stderr
02c21a127f0b5880ec8ab977bd24559882245dcd2aa86d39f08440a13654923a  /tmp/patch-12.0.5-batch56-green-run-0.stdout
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/patch-12.0.5-batch56-green-run-1.stderr
e71f3d09a230d31758d27f8b49f474fc222e475d5dfc86bae3053426219b0bd5  /tmp/patch-12.0.5-batch56-green-run-1.stdout
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/patch-12.0.5-batch56-green-run-2.stderr
e6b6f638f6bdcfd936aa588450e6f70391aef8e3a8ab2018ed7d6af0de103470  /tmp/patch-12.0.5-batch56-green-run-2.stdout
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/patch-12.0.5-batch56-green-run-3.stderr
fd95bfdd09a765050a191884baf8396a022117e059b11343e92391eea076f57d  /tmp/patch-12.0.5-batch56-green-run-3.stdout
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/patch-12.0.5-batch56-green-run-4.stderr
07a999fad66bf0dc1d1291687a217259a910091e0ba633bb026fb38c16efb84e  /tmp/patch-12.0.5-batch56-green-run-4.stdout
94f8dedfd329881a1726bd6456fca2ddd6995e54bd91c5ea39b80819ff18f08a  /tmp/patch-12.0.5-batch56-green-runs.json
70c6f95904cf4447382fa6a48e5c7a8f510db9ded7c57eb5fce0ed7b4068ab42  /tmp/patch-12.0.5-batch56-green-startup-run.json
7df25392021b144e979feec2f8a4b1d7a71d2f29ec9c6bfa47db8b803ca96072  /tmp/patch-12.0.5-batch56-green-startup.stderr
37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570  /tmp/patch-12.0.5-batch56-green-startup.stdout
034e6f39bd4a021f8424cbc0368bc7ac3c8f90b570ce3c229a861615e2458bba  /tmp/patch-12.0.5-batch56-red-build-result.json
f75c362f1db146b20c4b890ac1cfe5fe723045d8bf349feda3c1e3fe7b831d72  /tmp/patch-12.0.5-batch56-red-build.jsonl
44ccf9dc9f8c83a94a341df81cef47c512c63213f648fdfa8fcaac7036e402a0  /tmp/patch-12.0.5-batch56-red-build.stderr
d583bcebaceb7edc65d6f021a21d5ddf435f26d91b30f5530e135c6218a673de  /tmp/patch-12.0.5-batch56-red-fixed-build-result.json
4349f21c6d982ec701213d43619c538ce8429bbee39667d63108e40f9af6562f  /tmp/patch-12.0.5-batch56-red-fixed-build.jsonl
8f249c625bc195cb911f59e4e3b4c92ffe352620e075a05a6545c4130ea5e066  /tmp/patch-12.0.5-batch56-red-fixed-build.stderr
946606adc08f5e1d058ae3c070516def08479a737c15d38bd9c50e28a211e565  /tmp/patch-12.0.5-batch56-red-fixed-run.json
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/patch-12.0.5-batch56-red-fixed-run.stderr
2b380565da66d8c9e771974a05dfa4d58ea0f89484c0177f668d0190670b6b05  /tmp/patch-12.0.5-batch56-red-fixed-run.stdout
5035ac0c2694350aab9be08c22b05e71128d6a6a89c9d4e427878916ce860205  /tmp/patch-12.0.5-batch56-red-run.json
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  /tmp/patch-12.0.5-batch56-red-run.stderr
4cd70568cb05a70351b7c2a59a9c3c996e186f66cf851042154181dbefb714cf  /tmp/patch-12.0.5-batch56-red-run.stdout
4e2193e8283d13d686f3e4b845dbaa1a10d4eb833104e4fa495f60fd772e51a2  /tmp/patch-12.0.5-batch56-tooltip-test-list.stdout
ab7a6277fae02a293ddc98185656cd1bb691ec3b831b32f70a8080d022532a6d  /tmp/patch-12.0.5-tooltip-dto-diagnostic.lua
92b15d71b4e5166b992db2e39e70d150cec09190cbe7cfebc11d1f644289b710  /tmp/patch-12.0.5-tooltip-dto-diagnostic.stderr
476a3bd634ea212fc8a4d980f245f73cbef4095899c48067499661e0e921ca98  /tmp/patch-12.0.5-tooltip-dto-diagnostic.stdout
```
