# Identity follow-ups handoff

**State:** authored/staged only; not applied, compiled, or runtime-tested.
**Producer:** local author, no delegated agents/models.
**Base:** `aa29d7d7f06b14f33990c36ae736b98579b00b78` (HEAD observed during authoring).
**Review read:** `data/patch-api/evidence/12.0.5-session-2026-10-03/b98-verify-identity.md`, including remaining collisions/merge risks.

Staging root:
`/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad/staging/identity-followups/`

## Artifacts and exact integration

- **[state/tests]** `tests/model_set_unit_identity_followups.rs`: complete new file, six tests.
- **[state/tests]** `tests/raid_roster_identity_followups.rs`: complete new file, five tests. No new host state/type is needed.
- **[producer]** `src/lua_api/globals/group_queries.rs`: complete proposed file mirrored from the base, four disjoint exact replacements.
- **[state/spec]** `docs/specs/model-unit-identity-guard.md` and `docs/specs/instanced-identity.md`: complete proposed files, ten exact replacements total.
- **Exact edits:** `exact-edits.json` has path/tag/oldText/newText, base and staged-file hashes; `exact-edits.md` contains all old/new anchors and complete new test code. All 14 old anchors occur exactly once in the corresponding base blob; applying them reproduces the mirrored files exactly.

Do not overwrite a concurrent worker's repo files with the full staged copies. Use verified old/new replacements, reconcile conflicting changes, and add the two test files only if still absent. Top-level tests are auto-discovered by `build.rs` into `tests/integration.rs`; no harness registration edit is needed.

## (a) Actual model SetUnit coverage — expected GREEN on base

The five model tests create actual frames through `CreateFrame`, not generic Frame mocks. Each seeds prior host binding `party2`, obtains the real existing `party1` GUID, explicitly classifies it, then checks:

1. Public token + classified identity: `pcall` status plus exactly one public nil, no error; binding remains `party2`.
2. Missing identity: exactly one public false; binding remains `party2`.
3. Classification cleared: exactly one public true; stored binding becomes `party1`.

| Test | Widget constructed | Dispatcher removed* | Secrecy denial removed** |
|---|---|---|---|
| `player_model_set_unit_obeys_identity_contract` | PlayerModel | FAIL | FAIL |
| `dress_up_model_set_unit_obeys_identity_contract` | DressUpModel | FAIL | FAIL |
| `cinematic_model_set_unit_obeys_identity_contract` | CinematicModel | FAIL | FAIL |
| `tabard_model_set_unit_obeys_identity_contract` | TabardModel | FAIL | FAIL |
| `model_scene_set_unit_obeys_identity_contract` | ModelScene | FAIL | FAIL |

`tooltip_set_unit_keeps_content_binding_and_callback` asserts player tooltip content/name/level, displayed unit, visibility, one OnTooltipSetUnit callback, and no model binding. Expected PASS on base; would remain PASS for either counterfactual above. It would FAIL if SetUnit were indiscriminately routed to the model handler. This control protects the tooltip branch; it is not independent proof that the dispatcher exists.

* Precisely: remove the final shared SetUnit dispatcher registration, leaving the prior tooltip registration winning. The model tests fail denial and, independently, public stored-binding behavior. Missing false alone may still pass on the tooltip path.
** Precisely: remove the secret-identity denial branch but keep missing/public assignment behavior. The model tests fail secret nil/binding preservation; missing/public phases alone need not fail.

CinematicModel, TabardModel and DressUpModel map to `WidgetType::PlayerModel`; no distinct enum variants or actor SetUnit coverage are claimed. Row 534 remains partial until these authored tests actually pass and acceptance evidence is updated. No 3D/native-success or older-profile proof is added.

## (b) Smallest roster producer fix

Current cached names are created as interned strings and marked secret unconditionally. This both poisons equal ordinary values and ignores explicit identity state.

Pass the existing roster index into `read_raid_roster_name`. For a cached name under `all(retail-12-0-5, any(profile-retail, client-ptr))`, map index 1 to `player`, index n > 1 to `party(n - 1)`, call the existing shared `unit_misc::unit_identity_is_secret`, and return existing `identity_output`. That helper already uses trusted `wrap_host_secret_string` for secret results under exactly this cfg split. Do not use `raidN`: the actual roster provider is player + party records, and raid GUID resolution is absent.

Outside that cfg, keep `secret = true`; `identity_output` retains the original interned registry marking there, including the cached player name. This is requested legacy compatibility, not a new classifier. Uncached records return the existing localized `UNKNOWN` value before classification/wrapping. Invalid/inactive rows still produce twelve nils; every valid row still returns twelve results and the eleven non-name fields remain unchanged. Propagate the shared predicate's error through `LuaResult` rather than swallowing it. Remove the now-unused direct `mark_secret_value` import.

### Expected RED with state/tests only; expected GREEN after producer

All five roster tests are expected to fail on the unchanged base for behavioral reasons, not compilation reasons:

| Test | First expected base failure |
|---|---|
| `roster_names_follow_public_state_and_instance_group_exemption` | Unclassified cached party name is secret, not public. Covers both map states and local-player exemption. |
| `classified_roster_name_does_not_poison_equal_ordinary_strings` | Classified roster read makes pre-existing equal ordinary literal secret. Also checks newly obtained equal literal, peer isolation and shared UnitName classification. |
| `clearing_roster_classification_changes_new_results_not_retained_secret` | New roster result stays secret after host classification clears. Retained output must remain secret. |
| `explicit_player_guid_classification_overrides_roster_player_exemption` | Unclassified local-player roster name is already secret. Then checks explicit player classification and recovery. |
| `uncached_classified_roster_name_keeps_public_unknown_and_live_tuple` | After cached classified read and host clearing, new cached output still carries old interned secrecy. Earlier uncached UNKNOWN and classified-result assertions should pass. |

Tuple controls assert exact arity and concrete rank/subgroup/level/class/classFile/zone/online/dead/role/masterLooter/assignedRole values. No serialized-source assertions, fabricated GUIDs, `env.eval::<u32>`, EventQueue calls, or unsafe raw-string delimiters.

## Existing-test expectation changes

**None required against the specified HEAD.** This differs from the pre-B98 expectations mentioned in the prompt:

- `security_api::test_party_roster_name_is_secret_value` calls **UnitName**, not GetRaidRosterInfo. Its fixture already explicitly classifies the party GUID; it remains secret/inaccessible.
- `security_api::test_party_full_name_marks_name_and_realm_secret` and `security_api::test_table_containing_party_identity_is_not_accessible` already use that same fixture; assertions remain unchanged.
- `tests/admin_party_api.rs` roster controls assert payloads, ranks, coverage/subgroups, not automatic cached-name secrecy; no edits needed.
- All four `tests/raid_roster_unknown_name.rs` controls retain payload/tuple/cache/missing-row expectations. No edits to its spec are needed: cached-name secrecy was explicitly outside that slice; the follow-up extends `instanced-identity.md` instead.

Behavior changing: scoped cached GetRaidRosterInfo names become public absent explicit classification, including local player and instanced group members. Classified names remain secret, but only the returned wrapper is restricted. Older-profile cached-name marking is preserved. Historical pre-B98 automatic-party secrecy expectations were already changed in B98, not by this proposal.

## Spec edits

`model-unit-identity-guard.md`: expand SetUnit contract to all five newly tested frame types plus existing Model; add tooltip dispatch contract; list authored tests; replace absent-PlayerModel-test gap with an unrun acceptance gate and preserve legacy-routing gap.

`instanced-identity.md`: include cached roster names in the shared policy, correct roster index mapping, wrapper/literal isolation, twelve-result contract, public UNKNOWN/cache lifecycle, tests and scoped behavior change. Remove obsolete exclusion of GetRaidRosterInfo and stale original-B98 staging language. Keep native/GUID-category gaps and historical B98 evidence distinct. No checkboxes or page-accounting rows are promoted by authoring alone.

## Proof ledger and integration order

1. Read base blobs/review and inspect relevant dispatch, model binding, GUID resolver, output wrapper, tooltip controls, roster cache controls and security fixtures. Proof: source-only at exact base.
2. Verify all 14 unique old anchors and exact staged projections. Proof: authoring assertions passed; no runtime evidence.
3. Standalone rustfmt on staged Rust files exited 0 using `skip_children=true` to avoid resolving/editing unstaged sibling modules. Initial invocation failed because the mirrored source lacks its sibling module. Restore the user-required one-line first cfg attribute in both tests afterward; no cargo fmt/check claim. Manual readability review completed; no suppressions added.
4. Owner applies **state/tests** first and records targeted model GREEN plus five roster RED results; then applies **producer** and **state/spec** edits and records targeted GREEN plus existing roster/security/tooltip controls. These steps were not executed here. Legacy behavior is source-preserved, not runtime-proven.

**Merge risk:** uncompiled/unexecuted proposal; scoped tests cannot prove older-profile runtime behavior. No repo writes, cargo/test execution, git mutations, service operations, agents or model CLIs were performed.
