# Retail 12.0.7 party operations, solo formation and conditional target markers

B11 replaces raid-wide assistant shortcuts with member-specific inputs; B25 publishes solo formation from explicit host entry state; B26 adds `/tm ~marker`. Source: [12.0.7 retained excerpt](../../data/patch-api/sources/12.0.7-api-changes.txt), rows 039/041/042/043/044/045 and prose015/017. Cached retail declarations may postdate 12.0.7: they establish candidate signatures, not historical/native execution proof. All requirements remain unchecked because this slice is authored, not integrated or run.

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

- [ ] New behavior is gated on `retail-12-0-7`; older epochs keep their existing paths. New role sets and category map default empty; new restriction and follower flags default false; solo payload and latch default absent. These defaults are INFERRED simulator policy.
- [ ] Individual promotion/demotion affects only the resolved roster member or player. Unknown or ambiguous short names do not change roles or leader. INFERRED: full names compare exactly; non-exact short names match the name before the realm separator; unit tokens resolve against current roster.
- [ ] `UnitIsGroupAssistant` and `UnitLeadsAnyGroup` read per-member role inputs live. INFERRED: individual demotion overrides the everyone toggle through an exclusion set; changing that toggle clears exclusions but retains explicit promotions. Inactive-group assistant queries return false.
- [ ] Leader promotion reads current roster, leaves unknown targets unchanged, and notifies `PARTY_LEADER_CHANGED` after mutation. INFERRED: repeated assignment emits no event.
- [ ] Everyone toggle returns exactly one public boolean. INFERRED: `updated` is true only for an effective flag change in an active group, false for duplicate or inactive requests.
- [ ] Uninvite validates optional reason and exact-name selector, removes one resolved member, clears that member's roles, rebases a surviving leader index, and synchronously notifies `GROUP_ROSTER_UPDATE` with the new roster visible. INFERRED: removing the leader selects local-player leadership; self-uninvite is a no-op.
- [ ] Home membership reads the existing active roster and its existing synthetic GUID convention; category 2 reads only explicit host GUID membership. Unconfigured category 2 returns false. INFERRED: nil category routes to home; only 1/2 are accepted. No fabricated category-2 membership or fallback to home.
- [ ] Explicit host restriction rejects mutations atomically and permits recovery when cleared. INFERRED: restrictions are not derived automatically from combat or leadership. Membership remains readable under the flag.
- [ ] Host input changes are read on every operation/query; inputs and mutations remain environment-local. Leaving the roster clears role sets.

### Secret arguments and taint

- [ ] Every B11 party API authenticates **all** arguments, including ignored extras, with `rilua::table_security::unwrap_secret` before validating any argument or accessing model state. Secure callers may pass genuine secrets; tainted callers may pass public inputs but every secret position/extra rejects without mutation.
- [ ] Original secret wrappers and caller taint are preserved. Public output policy is INFERRED; no host booleans are wrapped constants.
- [ ] `C_Macro.RunMacroText` authenticates text, button and all extras before validating text. INFERRED: nil button stays accepted because existing cached/host callers omit it; supplied button must be a UTF-8 string. Tainted secret extras take precedence over earlier invalid input. No output-restriction parity is claimed.

### Solo formation and markers

- [ ] On the existing shared `fire_on_update` boundary, a solo active Delve or explicit follower-dungeon entry emits `GROUP_FORMED(category, partyGUID)` only when the host supplies category 1/2 and a nonempty GUID. No identity is synthesized. Listener sees existing instance state after mutation, with exactly two public payload values.
- [ ] INFERRED: identical entry identity emits once; an observed exit resets the latch; a changed category/GUID or observed reentry emits again. Ordinary solo instances, grouped entries, incomplete payloads and defaults do not emit.
- [ ] `/tm ~n` uses the existing macro condition/selected-unit machinery and marker map. A valid unmarked target is assigned; a marked target is unchanged and emits no marker update. Existing numeric commands remain unchanged.
- [ ] INFERRED: invalid/out-of-range or repeated prefixes are atomic no-ops; `~0` does not clear an existing marker and performs the existing zero assignment on an unmarked target. Collision handling remains the existing `SetRaidTarget` policy.
- [ ] Execute the entire unmodified cached `SecureTemplates.lua`, then invoke its real `SECURE_ACTIONS.raidtarget` with `action='set-unmarked'`: assign once, preserve an existing marker on a second call. No copied approximation, vendor patch or source-substring assertion.

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

## Known gaps (current cycle)

- [ ] No compilation, RED/GREEN, startup or consumer runtime proof: prohibited by authoring request. Integrator must establish it before checking requirements or awarding page credit.
- [ ] Cached Lua is not authenticated build 68182. Native restriction derivation, exact errors, role-event timing and identity semantics remain inferred.
- [ ] Solo producer needs explicit host category/GUID/follower input. No game/server join source is integrated. No inferred empty-default payload is proposed.
- [ ] Existing home GUIDs are position-derived and can renumber after removal. Stable member identity, category-2 roster mutations and aliases outside supported tokens are not modeled.
- [ ] Loading full cached SecureTemplates may expose missing bootstrap dependencies. The authored test requires that cache; no shim or substitute is authorized.

## Out of scope

- Ready-check rows 038/040: existing 12.0.5 `ready-check-lockdown` capability is inherited through `retail-12-0-7`, only for its bounded scope. Its secret-input and native restrictions gaps remain.
- Full historical-page/native parity, automatic permission enforcement, network party service, arbitrary GUID domain, GUID syntax validation and durable server identities: absent evidence/model boundary.
- Retired raw `UninviteUnit`: already gated off on master; existing group-verb test already calls the namespace under this epoch. No duplicate change.
- `!` marker prefix, secure-click authorization, protected-frame enforcement, `SetRaidTarget` argument-policy overhaul and slash UI registration: beyond the requested conditional syntax. Its separate cached AllowedWhenUntainted declaration is not satisfied by public-only secure-action tests.
- No NeverSecret declaration applies to the six B11 operations or macro entry. Such a policy must reject secrets for all callers if later authenticated; none is invented here.
