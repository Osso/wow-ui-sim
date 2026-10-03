# Combat restrictions handoff — prose-2026-03-25-074

Authored 2026-10-03. **Recommended status: partial. Not merge-ready.** Repository and coverage JSON unchanged. Eight effect-only producer-boundary RED fixtures and an isolated host state-transition draft are staged. No cargo, tests, Git mutations, agents, model CLIs or runtime execution occurred. Only scratch files were written.

## Source reconciliation

`data/patch-api/sources/12.0.5-api-changes.txt:74` names twelve calls and says they “will no longer be able to be called by addons during combat”. March 31 line 169 later says:

> The recent restrictions to countdown, ready check, ping and loot method APIs have been loosened to only apply when in chat messaging lockdown rather than in all combat.

Result: **seven combat-restricted calls and five chat-lockdown-restricted calls; none wholly unrestricted.** Applying “ready check” to both named ready-check calls is an explicit family-to-call interpretation, supported by the accepted ready-check capability, not a new speculative family expansion. Keeping the seven others on combat is the literal unmodified remainder of line 74.

Lines 141–144 concern UnitIsUnit comparisons, not these group calls. Line 168 concerns private-aura APIs, not these calls. Neither is a blanket reversal. Line 214 excludes the new restrictions in Classic; it does not reverse the Retail 12.0.5 list. A full-file search found no other later modifier of the named group-call policy.

Neither line 74 nor line 169 states native denied return values, arity, exceptions or error text. **No new Lua combat-denial shape has been invented.** Existing chat errors remain their separately accepted, explicitly inferred simulator behavior; they are not evidence that a combat denial must throw.

## Per-API matrix

Paths and line numbers below are current repository source, not staged replacements. “Existing proof” refers to inspected tests/recorded capability acceptance, not a new run.

| Sentence call | Final Retail policy / source | Current simulator producer / behavior | Existing test assertion or gap |
|---|---|---|---|
| PromoteToLeader | Combat, line 74 | `src/c_api/c_party_info.rs:239–244`: resolves party index, writes `party_leader_index`, no combat guard | `test_patch_12_0_7_safe_global_bridges`, `src/loader/tests/wow_api_globals/startup_globals.rs:283–286`: party1 makes player nonleader and party1 leader; player promotion reverses. No combat test. |
| PromoteToAssistant | Combat, line 74 | `src/c_api/c_party_info.rs:233–237`: ignores selected unit, sets `everyone_assistant=true`, no guard | Same startup test, lines 281–282: expects `IsEveryoneAssistant()==true`. This proves existing incorrect coarse representation, not individual promotion or restriction. |
| DemoteAssistant | Combat, line 74 | `src/c_api/c_party_info.rs:227–231`: ignores selected unit, sets everyone flag false, no guard | Same startup test, lines 279–280: expects everyone flag false after demoting player. No per-member/combat proof. |
| SetEveryoneIsAssistant | Combat, line 74 | `src/c_api/c_party_info.rs:246–250`: writes supplied boolean, no guard | Same startup test, lines 277–278: flag becomes true. No combat denial/recovery proof. |
| DoReadyCheck | Chat lockdown, line 169 supersedes 74 | `src/c_api/c_party_info.rs:252–255` → `src/lua_api/globals/group_verbs.rs:162–170`: shared guard before active=true/response=None and synchronous READY_CHECK | `public_start_blocks_fresh_state_on_both_combat_axes`: false pcall, unchanged state, zero events; `unlocked_public_lifecycle_ignores_combat_for_both_responses`: active/waiting and one event even in combat. |
| ConfirmReadyCheck | Chat lockdown, line 169 supersedes 74 | `src/c_api/c_party_info.rs:257–260` → `src/lua_api/globals/group_verbs.rs:173–193`: guard before response storage and confirm/finish events | `public_confirm_blocks_inactive_state_for_both_responses_and_combat_axes` and `public_confirm_preserves_active_response_then_recovers_after_unlock`: original response/active/events preserved, then new true/false response and exactly two ordered events after unlock. |
| ConvertToParty | Combat, line 74 | No named Rust/Lua producer found in `src`; existing `IsInRaid` infers kind from count at `src/lua_api/globals/group_queries.rs:254–259` | No behavioral conversion/restriction proof. Loadability/UI helper inventories are not conversion proof. |
| ConvertToRaid | Combat, line 74 | No named Rust/Lua producer found; group type is inferred from roster count at `src/c_api/c_party_info.rs:141–150` | Same gap. Generic/lazy namespace callability, if present, cannot prove conversion effects. |
| ConfirmConvertToRaid | Combat, line 74 | No named Rust/Lua producer found | Same gap. Cached modern declaration exists but is not a working simulator producer or original 12.0.5 native characterization. |
| C_PartyInfo.DoCountdown | Chat lockdown, line 169 supersedes 74 | `src/c_api/c_party_info/countdown.rs:27–34`: shared guard at 28, live request commit/publication at 30–31 | `lockdown_combat_matrix_preserves_active_request_and_guards_before_validation`: preserves active 17s request and event count on blocked replace/cancel, combat-only allows 29s replacement, unlock permits cancellation. |
| C_PartyInfo.SetRestrictPings | Chat lockdown, line 169 supersedes 74 | `src/c_api/c_party_info/ping_restrictions.rs:23–27`: shared guard at 24, live setting mutation after parsing | `lockdown_blocks_mutation_but_not_reads_independently_of_combat`: all four combinations, original 2 retained if locked, otherwise changes to 1; flags unchanged and getter remains readable. No ping-delivery claim. |
| C_PartyInfo.SetLootMethod | Chat lockdown, line 169 supersedes 74 | `src/c_api/c_party_info/loot_method.rs:60–73`: guard at 61 before mutation/event | `lockdown_blocks_before_effects_while_combat_alone_allows_changes_and_reads`: four combinations; locked master state/threshold retained with no change event; unlocked Freeforall stored and one change event. |

The ready-check fixtures are in `tests/chat_lockdown_ready_checks.rs`; other names map to `tests/party_countdown.rs`, `tests/party_ping_restrictions.rs`, `tests/party_loot_method.rs` respectively.

### Literal names versus active producers

The six historical non-conversion globals are **not directly registered by these Rust producers**. Current vendor `Blizzard_DeprecatedPartyInfo/Deprecated_PartyInfo.lua:8–34` conditionally provides the historical ready-check and leadership wrappers through C_PartyInfo, gated by `loadDeprecationFallbacks`. Startup inventory currently expects the six historical globals to be nil in a bare environment (`startup_globals.rs:385`). Tests therefore exercise actual C_PartyInfo endpoints; this does not assert vendor fallback loading or historical-global availability. No fallback globals or vendor changes are staged. `ReadyCheck()` is a distinct existing legacy alias with accepted chat guard coverage; do not confuse it with bare `DoReadyCheck()`.

## Accepted capability reuse

`12.0.5-page-coverage.json` records `bounded-independent-pass` for:

- `chat-lockdown-predicate`, implementation `18b09cbf9`, spec `docs/specs/chat-messaging-lockdown.md`, tests `tests/chat_messaging_lockdown.rs`.
- `party-countdown-lockdown`, `27a840b34`, `docs/specs/party-countdown.md`.
- `ready-check-lockdown`, `f62420536`, `docs/specs/chat-lockdown-ready-checks.md`.
- `ping-restriction-setting-lockdown`, `77ab2785f`, `docs/specs/party-ping-restrictions.md`.
- `loot-method-lockdown`, `f2e85fcb6`, `docs/specs/party-loot-method.md`.

The predicate producer at `src/c_api/c_chat_info.rs:32–36` reads exactly one per-environment boolean. `all_combat_and_lockdown_combinations_remain_independent` and `explicit_true_false_true_transitions_return_single_boolean` require exact-one boolean, no reason and combat independence. The shared guard at lines 20–30 reads that same input. Explicit state is at `src/lua_api/state/sim_state.rs:49` (chat) and existing `player.in_combat`. No second restriction input is needed.

These accepted scopes expressly exclude native activation, permissions/security/denial parity and exhaustive API/full-profile coverage. Their existence accounts for five of this sentence's twelve calls; it cannot close the other seven.

## Staged additions and deliberate hold

Staging root: `$SCRATCH/staging/combat-restrictions/`.

**STATE:** one epoch-gated `SimState.group_restrictions` field, default empty assistants/nonraid; existing leader/everyone flags and combat/chat inputs reused. The new state is a C API-owned backing model, not Lua workaround glue.

**STATE / PRODUCER-DOMAIN:** public `c_party_info::combat_restrictions` host command model with seven named commands. `apply_addon_command` refuses combat before changing anything, then performs concrete leader/individual-assistant/everyone/small-group-kind transitions. It never reads chat lockdown, dispatches events or changes roster membership. Rust outcomes are INTERNAL decisions, not Lua return values. It is **not registered** as a Lua API.

**INFERRED:** per-member names as keys; no active/empty group is invalid; fresh group kind is party; a bounded conversion only handles at most four remote members; ordinary ConvertToRaid and explicit confirmation share an immediate non-destructive effect. These are bounded simulator choices, not native permissions/denial parity. Name changes/duplicates, roster removal/replacement, everyone-flag interaction, large raids and destructive/pending confirmation require separate characterization/reconciliation. No unsupported case is redirected to a count-based fallback.

**PRODUCER TESTS:** eight complete Rust fixtures, first-line Retail epoch cfg, real `WowLuaEnv`, actual API calls, ordinary addon stack taint, real roster/event observation. Denied calls are wrapped in pcall but their denied values/arity/error are never asserted. Positive calls must succeed and actually mutate. This allows failure-effect coverage without inventing a native denial shape. Tests remain RED until adapters are implemented.

**HELD:** seven Lua command adapters and query wiring. Source does not specify combat-denial shape; selecting error/nil/false/zero-results would violate the task. Current secret argument/authentication and trusted-caller behavior are not a license to choose one. Existing accepted chat runtime errors are not copied into combat callbacks. No claim of an effective API fix is made.

Required integration includes authenticated caller classification from VM context (never a Lua argument), original secret/argument authentication before target interpretation, real C callbacks to the model, per-member assistant queries and role lifecycle, explicit raid kind in `IsInRaid`, `GetNumRaidMembers`, `C_PartyInfo.GetActiveGroupType` and `IsPartyFull`, coherent existing leader readers, and source-backed blocked return handling. Replace epoch-specific roster-count inference; do not retain it as an alternate path. Query code is not secretly patched to make tests pass.

## Expected RED against current producers — not executed

| Authored fixture | Expected behavioral failure |
|---|---|
| `promote_to_leader_preserves_leader_in_combat_then_recovers` | Combat currently changes None leader index to Some(0). |
| `promote_to_assistant_preserves_roles_in_combat_then_changes_one_member` | Combat currently makes everyone an assistant; unlocked current producer also changes everyone instead of only Alice. |
| `demote_assistant_preserves_roles_in_combat_then_changes_one_member` | Current combat demotion clears everyone's assistant status; unlocked individual demotion also removes Bob's status. |
| `everyone_assistant_setting_is_atomic_in_combat_and_recovers` | Combat currently flips existing boolean. |
| `convert_to_raid_blocks_combat_then_converts_small_party` | No concrete callback: expected missing-producer assertion, or generic call leaves small party nonraid. |
| `confirm_convert_to_raid_blocks_combat_then_converts_small_party` | Same missing/live-effect boundary. |
| `convert_to_party_blocks_combat_then_restores_small_raid_to_party` | Cannot seed small raid through missing ConfirmConvertToRaid; later reverse transition is unimplemented. |
| `leader_restriction_and_live_combat_state_are_environment_isolated` | First environment mutates leader during combat. |

No compile/run RED or GREEN is claimed. The new producer fixtures reference only existing public Rust APIs/fields: missing behavior is tested dynamically, not as a missing Rust type/import. Applying only the state draft does **not** make them pass.

## Existing expectations requiring change after live integration

`test_patch_12_0_7_safe_global_bridges` currently encodes individual promotion/demotion as changing everyone flag. `deferred-startup-expectations.json` contains two verified unique, nonoverlapping replacements against the current working tree: demoting player no longer clears the everyone setting; then clear everyone explicitly before checking party1 individual promotion without enabling everyone. This is an **INFERRED everyone-versus-individual policy**, not native-verified behavior. Do not apply these expectation edits without their producer/query change.

If explicit raid kind replaces count inference, also revise or seed raid state in `tests/party_raid_probes.rs::is_in_raid_true_when_six_or_more_members` (current assertion relies on count), `tests/admin_party_api.rs::test_get_num_raid_members_counts_raid_including_player`, and `tests/party_loot_method.rs::master_selection_resolves_modeled_party_and_player_roster_identity`. `secure_group_headers.rs` uses SetPartySize(7), and some UI fixtures use large rosters: audit their explicit kind requirements rather than treating roster size as automatic conversion. Older-profile behavior is not authorized to change. These later tests are identified, not falsely credited as rewritten or run.

## Status recommendation

**partial**: five calls have accepted bounded final-predicate coverage. Four mutate state without combat guards, with two additionally using an incorrect shared everyone flag for individual commands. Three have no concrete conversion producer. The historical combat restriction is superseded only for the five named-family calls, so `metadata-only-superseded` for the whole sentence is dishonest. `bounded` would require implemented and proved effects for the remaining seven, including the concrete producer boundary. The coverage record remains audit-pending in the repository; no accounting/status mutation was authorized.

## Verification and application safety

- Exact three replacement anchors each counted once against current file contents; hashes stored in `exact-edits.json`. New paths verified absent in repository. Rechecked base hashes after authoring; no source changes invalidated these anchors.
- Scratch-only `rustfmt --edition 2024` on the new model and producer fixtures exited 0, empty stderr. No Cargo or runtime test command.
- Structural fixture check: eight tests, exact first-line cfg, no env.eval/u32, no internal raw-string closing delimiter. Manual readability review: short functions, centralized combat gate, no suppression, no nested state-held Lua dispatch.
- `source-snapshot.json` records current source/coverage hashes. Full mirrored current files are provided for review, but authoritative application uses exact replacements, never wholesale copies over concurrent edits.
- No independent verification, compiled proof, deployment/native proof or merge-readiness claim. The state draft is usable authoring material, not an effective runtime patch. Blocker is absent native combat-denial/caller contract; it is not a build failure.

## Exact current-tree replacements

The JSON below contains full old/new code for each edit; all tags distinguish state/domain from actual Lua producer changes. No existing Lua callback is replaced.

```json
[
  {
    "path": "src/c_api/c_party_info.rs",
    "tag": "state/producer-domain",
    "sha256_current": "aed0c4a89009f1f7a074893347ea065e964fa62496a5346594e26f5db1d4ee97",
    "anchor_occurrences": 1,
    "oldText": "#[cfg(feature = \"retail-12-0-5\")]\npub(crate) mod countdown;",
    "newText": "#[cfg(feature = \"retail-12-0-5\")]\npub mod combat_restrictions;\n#[cfg(feature = \"retail-12-0-5\")]\npub(crate) mod countdown;"
  },
  {
    "path": "src/lua_api/state/sim_state.rs",
    "tag": "state",
    "sha256_current": "9cd1de9134a60783c74aa351894de06338ca59886c31bbf14682af2bf1bcb0ee",
    "anchor_occurrences": 1,
    "oldText": "    pub party_group_active: bool,",
    "newText": "    pub party_group_active: bool,\n    /// Effect-only group command draft; not a native Lua return contract.\n    #[cfg(feature = \"retail-12-0-5\")]\n    pub group_restrictions: crate::c_api::c_party_info::combat_restrictions::GroupRestrictionState,"
  },
  {
    "path": "src/lua_api/state.rs",
    "tag": "state",
    "sha256_current": "215ab574003e2f44ebc0918d04274cd7c35f0ed94505b7dd8f08dd01301f3de5",
    "anchor_occurrences": 1,
    "oldText": "            party_group_active: $runtime.party_group_active,",
    "newText": "            party_group_active: $runtime.party_group_active,\n            #[cfg(feature = \"retail-12-0-5\")]\n            group_restrictions:\n                crate::c_api::c_party_info::combat_restrictions::GroupRestrictionState::default(),"
  }
]
```

## Deferred existing-test edits (not applied)

```json
{
  "path": "src/loader/tests/wow_api_globals/startup_globals.rs",
  "sha256_current": "718ff378b5041ec3f84b3c8b647a90d3bbec7af79cbc691c7a6c28f26da2ca87",
  "tag": "producer-test; DEFERRED until live adapter and query wiring",
  "edits": [
    {
      "oldText": "            if IsEveryoneAssistant() ~= false then return \"party-demote-assistant\" end",
      "newText": "            if IsEveryoneAssistant() ~= true then return \"party-demote-everyone-flag\" end\n            if UnitIsGroupAssistant(\"player\") ~= true then return \"party-everyone-still-applies\" end",
      "anchor_occurrences": 1
    },
    {
      "oldText": "            C_PartyInfo.PromoteToAssistant(\"party1\")\n            if IsEveryoneAssistant() ~= true then return \"party-promote-assistant\" end",
      "newText": "            C_PartyInfo.SetEveryoneIsAssistant(false)\n            C_PartyInfo.PromoteToAssistant(\"party1\")\n            if IsEveryoneAssistant() ~= false then return \"party-promote-not-everyone\" end\n            if UnitIsGroupAssistant(\"party1\") ~= true then return \"party-promote-member-assistant\" end",
      "anchor_occurrences": 1
    }
  ]
}
```

## New file: `src/c_api/c_party_info/combat_restrictions.rs` — STATE / PRODUCER-DOMAIN; host-only, no Lua binding

```rust
//! Effect-only draft for prose-2026-03-25-074, not a registered Lua adapter.
//! Outcomes describe Rust transitions; they are NOT native Lua denial shapes.
//! Explicit group kind and individual assistants are inferred simulator policy.
//! Only non-destructive small-group conversions are drafted. No size fallback.

use crate::lua_api::state::SimState;
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct GroupRestrictionState {
    pub assistant_names: HashSet<String>,
    pub is_raid: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum GroupMember {
    Player,
    Party(usize),
}

#[derive(Clone, Copy, Debug)]
pub enum GroupCommand {
    PromoteToLeader(GroupMember),
    PromoteToAssistant(GroupMember),
    DemoteAssistant(GroupMember),
    SetEveryoneIsAssistant(bool),
    ConvertToParty,
    ConvertToRaid,
    ConfirmConvertToRaid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupCommandOutcome {
    Applied,
    BlockedCombat,
    InvalidGroupOrMember,
}

/// The caller classification must come from VM context, never a Lua argument.
/// This draft only defines addon effects; trusted-call policy remains deferred.
pub fn apply_addon_command(sim: &mut SimState, command: GroupCommand) -> GroupCommandOutcome {
    if sim.player.in_combat {
        return GroupCommandOutcome::BlockedCombat;
    }
    if !sim.party_group_active || sim.party_members.is_empty() {
        return GroupCommandOutcome::InvalidGroupOrMember;
    }
    match command {
        GroupCommand::PromoteToLeader(member) => promote_leader(sim, member),
        GroupCommand::PromoteToAssistant(member) => change_assistant(sim, member, true),
        GroupCommand::DemoteAssistant(member) => change_assistant(sim, member, false),
        GroupCommand::SetEveryoneIsAssistant(enabled) => {
            sim.everyone_assistant = enabled;
            GroupCommandOutcome::Applied
        }
        GroupCommand::ConvertToParty => change_group_kind(sim, false),
        GroupCommand::ConvertToRaid | GroupCommand::ConfirmConvertToRaid => {
            change_group_kind(sim, true)
        }
    }
}

fn member_name(sim: &SimState, member: GroupMember) -> Option<String> {
    match member {
        GroupMember::Player => Some(sim.player.name.clone()),
        GroupMember::Party(index) => sim
            .party_members
            .get(index)
            .map(|member| member.name.clone()),
    }
}

fn promote_leader(sim: &mut SimState, member: GroupMember) -> GroupCommandOutcome {
    if member_name(sim, member).is_none() {
        return GroupCommandOutcome::InvalidGroupOrMember;
    }
    sim.party_leader_index = match member {
        GroupMember::Player => None,
        GroupMember::Party(index) => Some(index),
    };
    GroupCommandOutcome::Applied
}

fn change_assistant(sim: &mut SimState, member: GroupMember, enabled: bool) -> GroupCommandOutcome {
    let Some(name) = member_name(sim, member) else {
        return GroupCommandOutcome::InvalidGroupOrMember;
    };
    if enabled {
        sim.group_restrictions.assistant_names.insert(name);
    } else {
        sim.group_restrictions.assistant_names.remove(&name);
    }
    GroupCommandOutcome::Applied
}

fn change_group_kind(sim: &mut SimState, is_raid: bool) -> GroupCommandOutcome {
    // No destructive conversion: the bounded fixtures contain at most five players.
    if sim.party_members.len() > 4 {
        return GroupCommandOutcome::InvalidGroupOrMember;
    }
    sim.group_restrictions.is_raid = is_raid;
    GroupCommandOutcome::Applied
}
```

## New file: `tests/group_combat_restrictions.rs` — PRODUCER TESTS; expected RED, unexecuted

```rust
#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn grouped_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create group environment");
    env.exec("LeaveParty(); InviteToGroup('Alice'); InviteToGroup('Bob')")
        .expect("seed real group through producers");
    env.state().borrow_mut().events.drain();
    env.exec(
        r#"
        groupEvents = {}
        groupObserver = CreateFrame('Frame')
        groupObserver:RegisterEvent('GROUP_ROSTER_UPDATE')
        groupObserver:SetScript('OnEvent', function(_, event)
            table.insert(groupEvents, event)
        end)
        function callAddonGroupVerb(name, ...)
            assert(type(C_PartyInfo[name]) == 'function', 'missing group producer: ' .. name)
            local previous = debug.getstacktaint()
            debug.setstacktaint('CombatRestrictionAddon')
            local ok = pcall(C_PartyInfo[name], ...)
            debug.setstacktaint(previous)
            return ok
        end
        "#,
    )
    .expect("observe events and call actual producer with ordinary addon taint");
    env
}

fn set_context(env: &WowLuaEnv, combat: bool, lockdown: bool) {
    let mut state = env.state().borrow_mut();
    state.player.in_combat = combat;
    state.chat_messaging_lockdown = lockdown;
}

fn assert_no_group_events(env: &WowLuaEnv) {
    env.exec("assert(#groupEvents == 0, 'denial emitted roster event')")
        .expect("no synchronously dispatched denial event");
    assert!(env.state().borrow_mut().events.drain().is_empty());
}

#[test]
fn promote_to_leader_preserves_leader_in_combat_then_recovers() {
    for lockdown in [false, true] {
        let env = grouped_env();
        set_context(&env, true, lockdown);
        env.exec("callAddonGroupVerb('PromoteToLeader', 'party1')")
            .unwrap();
        assert_eq!(env.state().borrow().party_leader_index, None);
        assert_no_group_events(&env);
        set_context(&env, false, lockdown);
        env.exec("assert(callAddonGroupVerb('PromoteToLeader', 'party1')); assert(UnitIsGroupLeader('party1'))")
            .expect("unlocked leader transition ignores chat lockdown");
        assert_eq!(env.state().borrow().party_leader_index, Some(0));
    }
}

#[test]
fn promote_to_assistant_preserves_roles_in_combat_then_changes_one_member() {
    for lockdown in [false, true] {
        let env = grouped_env();
        set_context(&env, true, lockdown);
        env.exec(
            r#"
            callAddonGroupVerb('PromoteToAssistant', 'party1')
            assert(not UnitIsGroupAssistant('party1'))
            assert(not UnitIsGroupAssistant('party2'))
            assert(not IsEveryoneAssistant())
            "#,
        )
        .expect("combat preserves individual and everyone roles");
        assert_no_group_events(&env);
        set_context(&env, false, lockdown);
        env.exec(
            r#"
            assert(callAddonGroupVerb('PromoteToAssistant', 'party1'))
            assert(UnitIsGroupAssistant('party1'))
            assert(not UnitIsGroupAssistant('party2'), 'individual promotion leaked')
            assert(not IsEveryoneAssistant(), 'promotion changed everyone flag')
            "#,
        )
        .expect("one member promoted after combat ends");
    }
}

#[test]
fn demote_assistant_preserves_roles_in_combat_then_changes_one_member() {
    for lockdown in [false, true] {
        let env = grouped_env();
        env.exec("assert(callAddonGroupVerb('PromoteToAssistant', 'party1')); assert(callAddonGroupVerb('PromoteToAssistant', 'party2'))")
            .expect("seed individual assistants through producers");
        env.state().borrow_mut().events.drain();
        env.exec("groupEvents = {}").unwrap();
        set_context(&env, true, lockdown);
        env.exec(
            r#"
            callAddonGroupVerb('DemoteAssistant', 'party1')
            assert(UnitIsGroupAssistant('party1'))
            assert(UnitIsGroupAssistant('party2'))
            "#,
        )
        .expect("combat preserves existing assistants");
        assert_no_group_events(&env);
        set_context(&env, false, lockdown);
        env.exec(
            r#"
            assert(callAddonGroupVerb('DemoteAssistant', 'party1'))
            assert(not UnitIsGroupAssistant('party1'))
            assert(UnitIsGroupAssistant('party2'))
            "#,
        )
        .expect("one assistant demoted after combat ends");
    }
}

#[test]
fn everyone_assistant_setting_is_atomic_in_combat_and_recovers() {
    for lockdown in [false, true] {
        for initial in [false, true] {
            let env = grouped_env();
            env.state().borrow_mut().everyone_assistant = initial;
            set_context(&env, true, lockdown);
            env.exec(&format!(
                "callAddonGroupVerb('SetEveryoneIsAssistant', {})",
                !initial
            ))
            .unwrap();
            assert_eq!(env.state().borrow().everyone_assistant, initial);
            assert_no_group_events(&env);
            set_context(&env, false, lockdown);
            env.exec(&format!(
                "assert(callAddonGroupVerb('SetEveryoneIsAssistant', {}))",
                !initial
            ))
            .expect("unlocked everyone-assistant transition");
            assert_eq!(env.state().borrow().everyone_assistant, !initial);
        }
    }
}

fn assert_roster_preserved(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(GetNumGroupMembers() == 3)
        assert(UnitName('party1') == 'Alice')
        assert(UnitName('party2') == 'Bob')
        "#,
    )
    .expect("conversion preserves concrete roster");
}

#[test]
fn convert_to_raid_blocks_combat_then_converts_small_party() {
    for lockdown in [false, true] {
        let env = grouped_env();
        set_context(&env, true, lockdown);
        env.exec("callAddonGroupVerb('ConvertToRaid'); assert(not IsInRaid())")
            .expect("combat does not convert party");
        assert_roster_preserved(&env);
        assert_no_group_events(&env);
        set_context(&env, false, lockdown);
        env.exec("assert(callAddonGroupVerb('ConvertToRaid')); assert(IsInRaid()); assert(GetNumRaidMembers() == 3)")
            .expect("allowed conversion changes kind, not roster count");
        assert_roster_preserved(&env);
    }
}

#[test]
fn confirm_convert_to_raid_blocks_combat_then_converts_small_party() {
    for lockdown in [false, true] {
        let env = grouped_env();
        set_context(&env, true, lockdown);
        env.exec("callAddonGroupVerb('ConfirmConvertToRaid'); assert(not IsInRaid())")
            .expect("combat does not confirm conversion");
        assert_roster_preserved(&env);
        assert_no_group_events(&env);
        set_context(&env, false, lockdown);
        env.exec("assert(callAddonGroupVerb('ConfirmConvertToRaid')); assert(IsInRaid()); assert(GetNumRaidMembers() == 3)")
            .expect("allowed explicit confirmation changes kind");
        assert_roster_preserved(&env);
    }
}

#[test]
fn convert_to_party_blocks_combat_then_restores_small_raid_to_party() {
    for lockdown in [false, true] {
        let env = grouped_env();
        env.exec("assert(callAddonGroupVerb('ConfirmConvertToRaid')); assert(IsInRaid())")
            .expect("seed small raid through real conversion producer");
        env.state().borrow_mut().events.drain();
        env.exec("groupEvents = {}").unwrap();
        set_context(&env, true, lockdown);
        env.exec("callAddonGroupVerb('ConvertToParty'); assert(IsInRaid())")
            .expect("combat preserves small raid");
        assert_roster_preserved(&env);
        assert_no_group_events(&env);
        set_context(&env, false, lockdown);
        env.exec("assert(callAddonGroupVerb('ConvertToParty')); assert(not IsInRaid()); assert(GetNumRaidMembers() == 0)")
            .expect("allowed conversion restores party kind");
        assert_roster_preserved(&env);
    }
}

#[test]
fn leader_restriction_and_live_combat_state_are_environment_isolated() {
    let first = grouped_env();
    let second = grouped_env();
    set_context(&first, true, false);
    first
        .exec("callAddonGroupVerb('PromoteToLeader', 'party1')")
        .unwrap();
    second
        .exec("assert(callAddonGroupVerb('PromoteToLeader', 'party2'))")
        .unwrap();
    assert_eq!(first.state().borrow().party_leader_index, None);
    assert_eq!(second.state().borrow().party_leader_index, Some(1));
    set_context(&first, false, false);
    first
        .exec("assert(callAddonGroupVerb('PromoteToLeader', 'party1'))")
        .unwrap();
    set_context(&first, true, false);
    first
        .exec("callAddonGroupVerb('PromoteToLeader', 'player')")
        .unwrap();
    assert_eq!(first.state().borrow().party_leader_index, Some(0));
    assert_eq!(second.state().borrow().party_leader_index, Some(1));
}
```

## New file: `docs/specs/group-combat-restrictions.md` — SPEC; all new requirements unverified

```markdown
# Group combat restrictions

Effect-only Retail 12.0.5 authoring draft for `prose-2026-03-25-074`, source line 74 of [retained changes](../../data/patch-api/sources/12.0.5-api-changes.txt). Line 169 supersedes the combat axis for countdown, both ready-check calls, ping restriction setting and loot-method setting only. See [Lua API state architecture](../lua-api.md). No native denial-return convention is stated by those lines.

## What it must do

- [ ] Preserve leader, individual assistants, everyone-assistant setting, group kind, roster and events when an addon invokes any of the seven still-combat-restricted calls in combat; allow the same concrete transitions after combat ends, independently of explicit chat lockdown.
- [ ] Reuse `player.in_combat` and `chat_messaging_lockdown`; do not derive one from the other or add a restriction-input bool.
- [ ] Individual promotion/demotion changes only the resolved member, not `everyone_assistant`. **INFERRED simulator representation:** keep resolved current member names in a set. Name changes, duplicates and roster lifecycle reconciliation require an adapter-owned contract before integration.
- [ ] **INFERRED simulator representation:** explicit `is_raid` false default, separate from roster size. Small non-destructive conversions preserve the two named members and player. `ConvertToRaid` and `ConfirmConvertToRaid` share the immediate effect in this bounded host draft; prompt/destructive conversion semantics are excluded, not replaced by a fallback.
- [ ] Block addon effects at the state boundary before any changes. Internal `GroupCommandOutcome` is a Rust decision only: never expose it as a fabricated Lua denial result.
- [ ] Keep the five chat-restricted calls on their accepted shared chat guard. Their existing error convention is already labelled simulator inference by their specs; it is not authority to invent a combat denial convention.

## How it works

- [Lua API state architecture](../lua-api.md)
- [Group event infrastructure](../event-system.md)
- [Existing ready-check restriction contract](chat-lockdown-ready-checks.md)

## Implementation inventory

- `src/c_api/c_party_info/combat_restrictions.rs`: staged host-only command/effect model; not registered as a Lua API.
- `src/c_api/c_party_info.rs`: staged public epoch-gated domain module declaration only; existing Lua callbacks remain untouched.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs`: staged per-environment domain state and default initialization.
- `tests/group_combat_restrictions.rs`: eight authored producer-boundary RED fixtures using existing runtime APIs. No native denied arity, value or exception assertion.

## Tests asserting this spec

`tests/group_combat_restrictions.rs` names all seven remaining APIs, both chat axes, concrete allowed state changes, no denied mutation/events, live combat recovery and environment isolation. Tests are unexecuted. They deliberately do not pass with only the host model: the producer adapters and query wiring are held.

## Known gaps (current cycle)

- [ ] Characterize the seven native combat denial outcomes before binding the host decisions to Lua; do not assume `nil`, `false`, zero results or a runtime error.
- [ ] Supply authenticated caller classification and original argument/field authentication using actual rilua taint/secrets. Trusted-caller policy and full `AllowedWhenUntainted` parity remain unverified.
- [ ] Connect real C producers to the state model and public leader/assistant/raid queries; replace roster-count raid inference under this epoch rather than retaining an alternate path.
- [ ] Define role lifecycle on leaving, removal, renaming and roster replacement, and conversion confirmation/permissions. Do not merge the model as an effective API fix while these adapters are absent.
- [ ] Run compiled RED/GREEN and integration verification in the parent task; no execution authorized in this authoring task.

## Out of scope

Native error strings/denial conventions absent from retained evidence; destructive/oversized conversion and confirmation prompts; native role permissions; new legacy fallback globals; vendor Lua changes; Classic behavior. Existing chat-family contracts and accepted proof remain unchanged.
```
