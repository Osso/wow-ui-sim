# Retail 12.0.7 party operations, solo formation and conditional target markers

B11 replaces raid-wide assistant shortcuts with member-specific inputs; B25 publishes solo formation from explicit host entry state; B26 adds `/tm ~marker`. Source: [12.0.7 retained excerpt](../../data/patch-api/sources/12.0.7-api-changes.txt), rows 039/041/042/043/044/045 and prose015/017. Cached retail declarations may postdate 12.0.7: they establish candidate signatures, not historical/native execution proof. Default-Retail integration proves the bounded behaviors below; historical/native parity and alternate epochs remain unverified.

| Source ID suffix | API / behavior | Live input | Returns |
|---|---|---|---|
| 039 / 042 | `C_PartyInfo.DemoteAssistant` / `PromoteToAssistant` | active roster, `party_assistants`, `party_assistant_exclusions`, `everyone_assistant`, `party_operations_restricted` | 0 |
| 043 | `C_PartyInfo.PromoteToLeader` | active roster, `party_leader_index`, restriction flag | 0 |
| 044 | `C_PartyInfo.SetEveryoneIsAssistant` | active group, everyone flag, restriction flag | 1 public bool |
| 041 | `C_PartyInfo.IsGUIDInGroup` | home roster or explicit `party_category_guids[2]` | 1 public bool |
| 045 | `C_PartyInfo.UninviteUnit` | active roster, role sets, leader index, restriction flag | 0 |
| prose017 | solo `GROUP_FORMED` | existing `has_active_delve` / `world.in_instance`, explicit follower flag and optional category/GUID | event: category, partyGUID |
| prose015 | `/tm ~marker`, cached `set-unmarked` | existing `unit_raid_target_icons` and selected live unit | 0 |

## What it must do

### Party roles and membership

- [x] New behavior is gated on `retail-12-0-7`; older epochs keep their existing paths. New role sets and category map default empty; new restriction and follower flags default false; solo payload and latch default absent. These defaults are INFERRED simulator policy.
- [x] Individual promotion/demotion affects only the resolved roster member or player. Unknown or ambiguous short names do not change roles or leader. INFERRED: full names compare exactly; non-exact short names match the name before the realm separator; unit tokens resolve against current roster.
- [x] `UnitIsGroupAssistant` and `UnitLeadsAnyGroup` read per-member role inputs live. INFERRED: individual demotion overrides the everyone toggle through an exclusion set; changing that toggle clears exclusions but retains explicit promotions. Inactive-group assistant queries return false.
- [x] Leader promotion reads current roster, leaves unknown targets unchanged, and notifies `PARTY_LEADER_CHANGED` after mutation. INFERRED: repeated assignment emits no event.
- [x] Everyone toggle returns exactly one public boolean. INFERRED: `updated` is true only for an effective flag change in an active group, false for duplicate or inactive requests.
- [x] Uninvite validates optional reason and exact-name selector, removes one resolved member, clears that member's roles, rebases a surviving leader index, and synchronously notifies `GROUP_ROSTER_UPDATE` with the new roster visible. INFERRED: removing the leader selects local-player leadership; self-uninvite is a no-op.
- [x] Home membership reads the existing active roster and its existing synthetic GUID convention; category 2 reads only explicit host GUID membership. Unconfigured category 2 returns false. INFERRED: nil category routes to home; only 1/2 are accepted. No fabricated category-2 membership or fallback to home.
- [x] Explicit host restriction rejects mutations atomically and permits recovery when cleared. Host permission is independent of combat or leadership. Membership remains readable under the flag. Four leadership entry points additionally enforce the [12.0.5 addon-combat restriction](patch-12-0-5-pending-leadership.md); `UninviteUnit` is not in that historical list.
- [x] Host input changes are read on every operation/query; inputs and mutations remain environment-local. Leaving the roster clears role sets.

### Secret arguments and taint

- [x] Every B11 party API authenticates **all** arguments, including ignored extras, with `rilua::table_security::unwrap_secret` before validating any argument or accessing model state. Secure callers may pass genuine secrets; tainted callers may pass public inputs but every secret position/extra rejects without mutation.
- [x] Original secret wrappers and caller taint are preserved. Public output policy is INFERRED; no host booleans are wrapped constants.
- [x] `C_Macro.RunMacroText` authenticates text, button and all extras before validating text. INFERRED: nil button stays accepted because existing cached/host callers omit it; supplied button must be a UTF-8 string. Tainted secret extras take precedence over earlier invalid input. No output-restriction parity is claimed.

### Solo formation and markers

- [x] On the existing shared `fire_on_update` boundary, a solo active Delve or explicit follower-dungeon entry emits `GROUP_FORMED(category, partyGUID)` only when the host supplies category 1/2 and a nonempty GUID. No identity is synthesized. Listener sees existing instance state after mutation, with exactly two public payload values.
- [x] INFERRED: identical entry identity emits once; an observed exit resets the latch; a changed category/GUID or observed reentry emits again. Ordinary solo instances, grouped entries, incomplete payloads and defaults do not emit.
- [x] `/tm ~n` uses the existing macro condition/selected-unit machinery and marker map. A valid unmarked target is assigned; a marked target is unchanged and emits no marker update. Existing numeric commands remain unchanged.
- [x] INFERRED: invalid/out-of-range or repeated prefixes are atomic no-ops; `~0` does not clear an existing marker and performs the existing zero assignment on an unmarked target. Collision handling remains the existing `SetRaidTarget` policy.
- [x] Execute the entire unmodified cached `SecureTemplates.lua`, then invoke `SecureActionButton_OnClick` with type `raidtarget`, unit `player` and action `set-unmarked`: its lexical `SECURE_ACTIONS.raidtarget` assigns once and preserves an existing marker on a second click. No copied approximation, vendor patch or source-substring assertion.

## How it works

- [Lua API](../lua-api.md)
- [Event system](../event-system.md)
- [Target-marker macro command](target-marker-macro-command.md) — existing numeric-command scope.
- [Delve instance state](delve-instance-state.md) — existing host instance composition.

## Implementation inventory

- `src/c_api/c_party_info/roles_1207.rs` — authenticated host-backed B11 operations and role lookup.
- `src/c_api/c_party_info/solo_1207.rs` — explicit solo-entry event producer.
- `src/c_api/c_party_info.rs` — epoch dispatch and module wiring.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — empty/false/absent inputs and initialization.
- `src/lua_api/globals/group_queries_relationships.rs`, `group_verbs.rs` — role query read-through and roster cleanup.
- `src/lua_api/on_update.rs` — solo event reconciliation on the shared tick.
- `src/lua_api/globals/spell_macro_verbs.rs` — authenticated macro entry and gated conditional marker parsing.

## Tests asserting this spec

- `tests/party_1207_audit.rs` — 11 behavioral cases: defaults/arity, roles/live inputs, leader/event state, removal/rebase/event state, category membership/live inputs, isolation, secure secrets, tainted positional/extras authentication priority, invalid/restricted atomicity/recovery, solo-entry lifecycle, conditional slash plus actual cached secure action.
- Existing test-support anchors update `src/loader/tests/wow_api_globals/startup_globals.rs`, `tests/unit_relation_probes.rs`, and `tests/group_verbs.rs` to stop asserting global-role shortcuts and cover cleanup.

## Development proof — 2026-10-04

Base `63b32995d`; producer `68572cce2`; cached-dispatch/import correction `74e6e9067`. Default-Retail local helper commands used exclusive target `~/.cache/wow-ui-sim-target-b100`, four jobs and one filter per invocation. Logs: `~/.cache/wow-ui-sim-audit/r5-{RED,GREEN}-*.log`; detailed ledger: `p1207-r5-result.md` beside them.

| Filter | RED pass/fail | GREEN pass/fail |
|---|---|---|
| `party_1207_audit::` | 0/11 | 11/0 |
| `group_verbs::` | — | 11/0 |
| `admin_party_api::` | — | 28/0 |
| `patch_12_0_7_removed_native_surface::` | — | 30/0 |
| `unit_relation_probes::` | — | 19/0 |
| `mouse_tm_commands::` | — | 5/0 |
| `c_party_info_probes::` | — | 11/0 |
| `delve_instance_state::` | — | 5/0 |
| `chat_lockdown_ready_checks::` | — | 6/0 |
| lib `startup_globals::test_patch_12_0_7_safe_global_bridges` | — | 1/0 |

First GREEN was 10/1: test incorrectly accessed global `SECURE_ACTIONS`, which is lexical in the cached consumer. Correction exercises its real public click dispatcher; no vendor mutation. Checked requirements describe bounded simulator behavior only, not native acceptance. No page coverage JSON was edited.

## Known gaps (current cycle)

- [x] Default-Retail tests compiled and ran: original producers failed all 11 new cases; integrated producers passed all 11. Full cached secure-click consumer passed. Whole startup CLI, historical epoch and alternate-profile proof remain excluded.
- [ ] Cached Lua is not authenticated build 68182. Native restriction derivation, exact errors, role-event timing and identity semantics remain inferred.
- [ ] Solo producer needs explicit host category/GUID/follower input. No game/server join source is integrated. No inferred empty-default payload is proposed.
- [ ] Existing home GUIDs are position-derived and can renumber after removal. Stable member identity, category-2 roster mutations and aliases outside supported tokens are not modeled.
- [x] Full cached SecureTemplates loads and its secure-click dispatcher executes in the integration fixture. The cache remains required; no shim or substitute is authorized.

## Out of scope

- Ready-check rows 038/040: existing 12.0.5 `ready-check-lockdown` capability is inherited through `retail-12-0-7`, only for its bounded scope. Its secret-input and native restrictions gaps remain.
- Full historical-page/native parity, automatic permission enforcement, network party service, arbitrary GUID domain, GUID syntax validation and durable server identities: absent evidence/model boundary.
- Retired raw `UninviteUnit`: already gated off on master; existing group-verb test already calls the namespace under this epoch. No duplicate change.
- `!` marker prefix, secure-click authorization, protected-frame enforcement, `SetRaidTarget` argument-policy overhaul and slash UI registration: beyond the requested conditional syntax. Its separate cached AllowedWhenUntainted declaration is not satisfied by public-only secure-action tests.
- No NeverSecret declaration applies to the six B11 operations or macro entry. Such a policy must reject secrets for all callers if later authenticated; none is invented here.
