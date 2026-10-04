# Retail 12.0.7 B11 audit handoff

Status: authoring complete; not integrated or runtime-tested.

Active goal: author B11 and grouped B25/B26 evidence, exact master anchors, staged public-API behavioral tests and spec. Completion: evidence and all OLD anchors statically checked; runtime verification explicitly deferred. Exclusions: repository/git mutations, cargo/build/test/simulator execution, agents/model CLIs. Writes restricted to this audit cache.

Proof ledger: no runtime commands authorized or executed. Static evidence inspection complete; proof ledger below.

## Baseline and evidence

Read-only `git rev-parse HEAD`: `7702befe8a567c225d9e8680594186e9689734f4`. All anchors target those current bytes. Cache is later retail, NOT authenticated build 68182; annotations/signatures are contract evidence, not native historical proof. B11 lists six rows; B25/B26 are separately listed, included here because requested related event/marker scope. No coverage ledger updates are authored.

### global api-C_PartyInfo-ConfirmReadyCheck-038

Source `data/patch-api/sources/12.0.7-api-changes.txt:38`: `C_PartyInfo.ConfirmReadyCheck`.

Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:103–112`:
```lua
			Name = "ConfirmReadyCheck",
			Type = "Function",
			HasRestrictions = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "isReady", Type = "bool", Nilable = false },
			},
		},
```

Current `src/c_api/c_party_info.rs:257–260`:
```rust
fn c_party_info_confirm_ready_check(state: &mut LuaState) -> LuaResult<u32> {
    crate::lua_api::globals::group_verbs::confirm_ready_check(state)?;
    Ok(0)
}
```

Change: Inherited bounded ready-check-lockdown only. Do not promote entire row: AllowedWhenUntainted/extras, native restrictions and historical cache epoch remain unproven.

### global api-C_PartyInfo-DemoteAssistant-039

Source `data/patch-api/sources/12.0.7-api-changes.txt:39`: `C_PartyInfo.DemoteAssistant`.

Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:144–154`:
```lua
			Name = "DemoteAssistant",
			Type = "Function",
			HasRestrictions = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "name", Type = "cstring", Nilable = false },
				{ Name = "exactNameMatch", Type = "bool", Nilable = true },
			},
		},
```

Current `src/c_api/c_party_info.rs:227–231`:
```rust
fn c_party_info_demote_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = Option::<String>::from_stack(state, 1)?;
    borrow_state_mut(state)?.everyone_assistant = false;
    Ok(0)
}
```

Change: Replace global everyone flag mutation with per-member assistant set; strict name/exactNameMatch validation, all-argument authentication and explicit restriction input.

### global api-C_PartyInfo-DoReadyCheck-040

Source `data/patch-api/sources/12.0.7-api-changes.txt:40`: `C_PartyInfo.DoReadyCheck`.

Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:172–175`:
```lua
			Name = "DoReadyCheck",
			Type = "Function",
			HasRestrictions = true,
		},
```

Current `src/c_api/c_party_info.rs:252–255`:
```rust
fn c_party_info_do_ready_check(state: &mut LuaState) -> LuaResult<u32> {
    crate::lua_api::globals::group_verbs::start_ready_check(state)?;
    Ok(0)
}
```

Change: Inherited bounded ready-check-lockdown only. Do not promote entire row: AllowedWhenUntainted/extras, native restrictions and historical cache epoch remain unproven.

### global api-C_PartyInfo-IsGUIDInGroup-041

Source `data/patch-api/sources/12.0.7-api-changes.txt:41`: `C_PartyInfo.IsGUIDInGroup`.

Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:420–434`:
```lua
			Name = "IsGUIDInGroup",
			Type = "Function",
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "guid", Type = "WOWGUID", Nilable = false },
				{ Name = "category", Type = "luaIndex", Nilable = true, Documentation = { "If not provided, the active party is used" } },
			},

			Returns =
			{
				{ Name = "isInGroup", Type = "bool", Nilable = false },
			},
		},
```

Current `src/c_api/c_party_info.rs:166–180`:
```rust
fn c_party_info_is_guid_in_group(state: &mut LuaState) -> LuaResult<u32> {
    let guid = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let is_member = {
        let sim = borrow_state(state)?;
        sim.party_group_active
            && (guid == SEEDED_LOCAL_CHARACTER_GUID
                || sim
                    .party_members
                    .iter()
                    .enumerate()
                    .any(|(index, _)| party_member_guid(index) == guid))
    };
    state.push(Val::Bool(is_member));
    Ok(1)
}
```

Change: Authenticate guid/category/extras before validation; home category reads current roster, explicit other-category host GUID map reads live; unsupported categories false (INFERRED).

### global api-C_PartyInfo-PromoteToAssistant-042

Source `data/patch-api/sources/12.0.7-api-changes.txt:42`: `C_PartyInfo.PromoteToAssistant`.

Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:495–505`:
```lua
			Name = "PromoteToAssistant",
			Type = "Function",
			HasRestrictions = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "name", Type = "cstring", Nilable = false },
				{ Name = "exactNameMatch", Type = "bool", Nilable = true },
			},
		},
```

Current `src/c_api/c_party_info.rs:233–237`:
```rust
fn c_party_info_promote_to_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = Option::<String>::from_stack(state, 1)?;
    borrow_state_mut(state)?.everyone_assistant = true;
    Ok(0)
}
```

Change: Promote only resolved member/player; no raid-wide promotion.

### global api-C_PartyInfo-PromoteToLeader-043

Source `data/patch-api/sources/12.0.7-api-changes.txt:43`: `C_PartyInfo.PromoteToLeader`.

Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:507–517`:
```lua
			Name = "PromoteToLeader",
			Type = "Function",
			HasRestrictions = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "name", Type = "cstring", Nilable = false },
				{ Name = "exactNameMatch", Type = "bool", Nilable = true },
			},
		},
```

Current `src/c_api/c_party_info.rs:239–244`:
```rust
fn c_party_info_promote_to_leader(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let leader_index = party_member_index_from_unit(state, &unit)?;
    borrow_state_mut(state)?.party_leader_index = leader_index;
    Ok(0)
}
```

Change: Do not convert unknown target to player leadership; resolve name/token and maintain valid leader index; publish PARTY_LEADER_CHANGED after mutation (INFERRED).

### global api-C_PartyInfo-SetEveryoneIsAssistant-044

Source `data/patch-api/sources/12.0.7-api-changes.txt:44`: `C_PartyInfo.SetEveryoneIsAssistant`.

Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:534–548`:
```lua
			Name = "SetEveryoneIsAssistant",
			Type = "Function",
			HasRestrictions = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "isAssistant", Type = "bool", Nilable = false },
			},

			Returns =
			{
				{ Name = "updated", Type = "bool", Nilable = false },
			},
		},
```

Current `src/c_api/c_party_info.rs:246–250`:
```rust
fn c_party_info_set_everyone_is_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = Option::<bool>::from_stack(state, 1)?.unwrap_or(false);
    borrow_state_mut(state)?.everyone_assistant = enabled;
    Ok(0)
}
```

Change: Return exactly one public bool updated from actual toggle difference (INFERRED), not zero results; read explicit restriction flag.

### global api-C_PartyInfo-UninviteUnit-045

Source `data/patch-api/sources/12.0.7-api-changes.txt:45`: `C_PartyInfo.UninviteUnit`.

Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:596–607`:
```lua
			Name = "UninviteUnit",
			Type = "Function",
			HasRestrictions = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "name", Type = "cstring", Nilable = false },
				{ Name = "reason", Type = "cstring", Nilable = true },
				{ Name = "exactNameMatch", Type = "bool", Nilable = true },
			},
		},
```

Current `src/c_api/c_party_info.rs:206–225`:
```rust
fn c_party_info_uninvite_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let removed = {
        let mut sim = borrow_state_mut(state)?;
        if let Some(index) = crate::lua_api::globals::unit_api::parse_party_index(&unit)
            && index < sim.party_members.len()
        {
            sim.party_members.remove(index);
            true
        } else {
            let before = sim.party_members.len();
            sim.party_members.retain(|member| member.name != unit);
            before != sim.party_members.len()
        }
    };
    if removed {
        crate::lua_api::globals::group_verbs::push_event(state, "GROUP_ROSTER_UPDATE")?;
    }
    Ok(0)
}
```

Change: Validate optional reason and exactNameMatch; resolve once, remove exactly one member, rebase leader index and clear removed assistant name; notify after mutation.

### prose-undated-015

Source `data/patch-api/sources/12.0.7-api-changes.txt:15`: `- Added secure action raidtarget option "set-unmarked" and /tm ~marker syntax.`.

Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_FrameXML/SecureTemplates.lua:593–596`:
```lua
		elseif ( action == "set-unmarked" and GetRaidTargetIndex(unit) == nil ) then
			SetRaidTarget(unit, marker);
		elseif ( action == "clear" ) then
			SetRaidTarget(unit, 0);
```

Current/change: Numeric-only parser src/lua_api/globals/spell_macro_verbs.rs:187–210. Add gated ~ prefix and live GetRaidTargetIndex no-op if already marked. SecureTemplates set-unmarked already implements conditional operation; do not patch vendor.

### prose-undated-017

Source `data/patch-api/sources/12.0.7-api-changes.txt:17`: `- GROUP_FORMED sent when player joins a follower dungeon or delve alone.`.

Cached `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:639–648`:
```lua
			Name = "GroupFormed",
			Type = "Event",
			LiteralName = "GROUP_FORMED",
			SynchronousEvent = true,
			Payload =
			{
				{ Name = "category", Type = "number", Nilable = false },
				{ Name = "partyGUID", Type = "WOWGUID", Nilable = false },
			},
		},
```

Current/change: GROUP_FORMED registered src/event/valid_events_a.rs:718; no solo transition producer. Add explicit optional category/GUID plus follower flag and identity latch; reconcile on existing OnUpdate boundary against live delve/instance/group state. No event until complete host payload configured.

### Inherited ready-check credit verified

`Cargo.toml:120`: `retail-12-0-7 = ["retail-12-0-5"]`. Existing ledger capability (read, not rerun):
```json
{
  "id": "ready-check-lockdown",
  "symbols": [
    "C_PartyInfo.DoReadyCheck",
    "C_PartyInfo.ConfirmReadyCheck",
    "ReadyCheck"
  ],
  "scope": "Explicit chat-lockdown versus combat-only predicate; atomic rejection and unlock recovery in bounded simulator operations. Native producer/security/error/permissions, exhaustive API inventory and all-profile parity unknown; no C_Ping action or ping delivery claim.",
  "spec": "docs/specs/chat-lockdown-ready-checks.md",
  "tests": [
    "tests/chat_lockdown_ready_checks.rs"
  ],
  "implementation": "f62420536",
  "proof": "bounded-independent-pass",
  "ledger": "/tmp/patch-12.0.5-ready-check-lockdown-independent-proof.md",
  "gate_scope": "6 ready-check + 16 controls = 22 selected PASS; fresh source-identical fmt/check0 at6f264ce3e; inspected saved evidence only, no reruns. Linked spec owns inferred policies and provenance limits."
}
```

Rows 038/040 may inherit **only** bounded lockdown behavior. Source rows are still audit-pending with empty capability arrays. Existing evidence artifact path is historical ledger content, not a newly written artifact.

### deprecated-api-177 — migration context, not extra native credit

Source `data/patch-api/sources/12.0.7-api-changes.txt:177`: `Deprecated_PartyInfo.lua: ConfirmReadyCheck/DemoteAssistant/DoReadyCheck/PromoteToAssistant/PromoteToLeader/SetEveryoneIsAssistant/UninviteUnit/IsGUIDInGroup -> C_PartyInfo namespace.`. Cached `Blizzard_DeprecatedPartyInfo/Deprecated_PartyInfo.lua:4–6` gates on `loadDeprecationFallbacks`; forwarding at lines 13/17/21/25/29/33/37/41. Existing raw UninviteUnit registration already gated at `src/lua_api/globals/group_verbs.rs:234–235`; tests/group_verbs.rs already selects namespace under 12.0.7. No new fallback/wrapper; no vendor edits.

## Anchored edits

Machine-readable companion: `edits-p1207-b11.json`. Each OLD matched exactly once against baseline, and regions are disjoint. Apply all against original bytes, not sequential fuzzy matching. For RED: apply `state` and `test-support`, copy new producer modules so declarations resolve, but withhold every `producer` edit. Modules compiled but unwired are not producer proof. No new files are integrated here. Older-epoch cfg branches preserve profile behavior; they are not runtime fallback paths.

### E01 — `state` — `src/lua_api/state/sim_state.rs:508`

OLD (exact):
```rust
    pub everyone_assistant: bool,
```

NEW:
```rust
    pub everyone_assistant: bool,
    /// INFERRED explicit per-member roles; empty by default, names match roster identity.
    #[cfg(feature = "retail-12-0-7")]
    pub party_assistants: HashSet<String>,
    #[cfg(feature = "retail-12-0-7")]
    pub party_assistant_exclusions: HashSet<String>,
    /// INFERRED explicit restriction input; false, not inferred from combat or rank.
    #[cfg(feature = "retail-12-0-7")]
    pub party_operations_restricted: bool,
    /// Explicit non-home category GUID membership; empty default, no invented GUIDs.
    #[cfg(feature = "retail-12-0-7")]
    pub party_category_guids: HashMap<i32, HashSet<String>>,
    /// Explicit solo-entry inputs; incomplete payload never publishes GROUP_FORMED.
    #[cfg(feature = "retail-12-0-7")]
    pub solo_follower_dungeon: bool,
    #[cfg(feature = "retail-12-0-7")]
    pub solo_group_category: Option<i32>,
    #[cfg(feature = "retail-12-0-7")]
    pub solo_group_guid: Option<String>,
    /// INFERRED identity latch reset on observed exit, not a server formation epoch.
    #[cfg(feature = "retail-12-0-7")]
    pub(crate) solo_group_last: Option<(i32, String)>,
```

### E02 — `state` — `src/lua_api/state.rs:466`

OLD (exact):
```rust
            everyone_assistant: false,
```

NEW:
```rust
            everyone_assistant: false,
            #[cfg(feature = "retail-12-0-7")]
            party_assistants: HashSet::new(),
            #[cfg(feature = "retail-12-0-7")]
            party_assistant_exclusions: HashSet::new(),
            #[cfg(feature = "retail-12-0-7")]
            party_operations_restricted: false,
            #[cfg(feature = "retail-12-0-7")]
            party_category_guids: HashMap::new(),
            #[cfg(feature = "retail-12-0-7")]
            solo_follower_dungeon: false,
            #[cfg(feature = "retail-12-0-7")]
            solo_group_category: None,
            #[cfg(feature = "retail-12-0-7")]
            solo_group_guid: None,
            #[cfg(feature = "retail-12-0-7")]
            solo_group_last: None,
```

### E03 — `test-support` — `src/c_api/c_party_info.rs:26`

OLD (exact):
```rust
mod ping_restrictions;
```

NEW:
```rust
mod ping_restrictions;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod roles_1207;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod solo_1207;
```

### E04 — `producer` — `src/c_api/c_party_info.rs:166`

OLD (exact):
```rust
fn c_party_info_is_guid_in_group(state: &mut LuaState) -> LuaResult<u32> {
    let guid = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let is_member = {
        let sim = borrow_state(state)?;
        sim.party_group_active
            && (guid == SEEDED_LOCAL_CHARACTER_GUID
                || sim
                    .party_members
                    .iter()
                    .enumerate()
                    .any(|(index, _)| party_member_guid(index) == guid))
    };
    state.push(Val::Bool(is_member));
    Ok(1)
}
```

NEW:
```rust
fn c_party_info_is_guid_in_group(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::is_guid_in_group(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
        let guid = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
        let is_member = {
            let sim = borrow_state(state)?;
            sim.party_group_active
                && (guid == SEEDED_LOCAL_CHARACTER_GUID
                    || sim
                        .party_members
                        .iter()
                        .enumerate()
                        .any(|(index, _)| party_member_guid(index) == guid))
        };
        state.push(Val::Bool(is_member));
        Ok(1)
    }
}
```

### E05 — `producer` — `src/c_api/c_party_info.rs:206`

OLD (exact):
```rust
fn c_party_info_uninvite_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let removed = {
        let mut sim = borrow_state_mut(state)?;
        if let Some(index) = crate::lua_api::globals::unit_api::parse_party_index(&unit)
            && index < sim.party_members.len()
        {
            sim.party_members.remove(index);
            true
        } else {
            let before = sim.party_members.len();
            sim.party_members.retain(|member| member.name != unit);
            before != sim.party_members.len()
        }
    };
    if removed {
        crate::lua_api::globals::group_verbs::push_event(state, "GROUP_ROSTER_UPDATE")?;
    }
    Ok(0)
}
```

NEW:
```rust
fn c_party_info_uninvite_unit(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::uninvite_unit(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
        let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
        let removed = {
            let mut sim = borrow_state_mut(state)?;
            if let Some(index) = crate::lua_api::globals::unit_api::parse_party_index(&unit)
                && index < sim.party_members.len()
            {
                sim.party_members.remove(index);
                true
            } else {
                let before = sim.party_members.len();
                sim.party_members.retain(|member| member.name != unit);
                before != sim.party_members.len()
            }
        };
        if removed {
            crate::lua_api::globals::group_verbs::push_event(state, "GROUP_ROSTER_UPDATE")?;
        }
        Ok(0)
    }
}
```

### E06 — `producer` — `src/c_api/c_party_info.rs:227`

OLD (exact):
```rust
fn c_party_info_demote_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = Option::<String>::from_stack(state, 1)?;
    borrow_state_mut(state)?.everyone_assistant = false;
    Ok(0)
}
```

NEW:
```rust
fn c_party_info_demote_assistant(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::demote_assistant(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
        let _unit = Option::<String>::from_stack(state, 1)?;
        borrow_state_mut(state)?.everyone_assistant = false;
        Ok(0)
    }
}
```

### E07 — `producer` — `src/c_api/c_party_info.rs:233`

OLD (exact):
```rust
fn c_party_info_promote_to_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = Option::<String>::from_stack(state, 1)?;
    borrow_state_mut(state)?.everyone_assistant = true;
    Ok(0)
}
```

NEW:
```rust
fn c_party_info_promote_to_assistant(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::promote_to_assistant(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
        let _unit = Option::<String>::from_stack(state, 1)?;
        borrow_state_mut(state)?.everyone_assistant = true;
        Ok(0)
    }
}
```

### E08 — `producer` — `src/c_api/c_party_info.rs:239`

OLD (exact):
```rust
fn c_party_info_promote_to_leader(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let leader_index = party_member_index_from_unit(state, &unit)?;
    borrow_state_mut(state)?.party_leader_index = leader_index;
    Ok(0)
}
```

NEW:
```rust
fn c_party_info_promote_to_leader(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::promote_to_leader(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
        let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
        let leader_index = party_member_index_from_unit(state, &unit)?;
        borrow_state_mut(state)?.party_leader_index = leader_index;
        Ok(0)
    }
}
```

### E09 — `producer` — `src/c_api/c_party_info.rs:246`

OLD (exact):
```rust
fn c_party_info_set_everyone_is_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = Option::<bool>::from_stack(state, 1)?.unwrap_or(false);
    borrow_state_mut(state)?.everyone_assistant = enabled;
    Ok(0)
}
```

NEW:
```rust
fn c_party_info_set_everyone_is_assistant(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::set_everyone_is_assistant(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
        let enabled = Option::<bool>::from_stack(state, 1)?.unwrap_or(false);
        borrow_state_mut(state)?.everyone_assistant = enabled;
        Ok(0)
    }
}
```

### E10 — `producer` — `src/c_api/c_party_info.rs:13`

OLD (exact):
```rust
use crate::lua_api::state::SEEDED_LOCAL_CHARACTER_GUID;
```

NEW:
```rust
#[cfg(not(feature = "retail-12-0-7"))]
use crate::lua_api::state::SEEDED_LOCAL_CHARACTER_GUID;
```

### E11 — `producer` — `src/c_api/c_party_info.rs:186`

OLD (exact):
```rust
fn party_member_index_from_unit(state: &mut LuaState, unit: &str) -> LuaResult<Option<usize>> {
```

NEW:
```rust
#[cfg(not(feature = "retail-12-0-7"))]
fn party_member_index_from_unit(state: &mut LuaState, unit: &str) -> LuaResult<Option<usize>> {
```

### E12 — `producer` — `src/lua_api/globals/group_queries_relationships.rs:239`

OLD (exact):
```rust
pub(super) fn unit_is_group_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let assistant = {
        let st = borrow_state(state)?;
        if !st.everyone_assistant {
            false
        } else {
            matches!(unit.as_str(), "player" | "pet" | "vehicle")
                || visible_party_member(&st, &unit).is_some()
        }
    };
    state.push(Val::Bool(assistant));
    Ok(1)
}
```

NEW:
```rust
pub(super) fn unit_is_group_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    #[cfg(feature = "retail-12-0-7")]
    let assistant = {
        let sim = borrow_state(state)?;
        crate::c_api::c_party_info::roles_1207::is_assistant(&sim, &unit)
    };
    #[cfg(not(feature = "retail-12-0-7"))]
    let assistant = {
        let st = borrow_state(state)?;
        if !st.everyone_assistant {
            false
        } else {
            matches!(unit.as_str(), "player" | "pet" | "vehicle")
                || visible_party_member(&st, &unit).is_some()
        }
    };
    state.push(Val::Bool(assistant));
    Ok(1)
}
```

### E13 — `producer` — `src/lua_api/globals/group_queries_relationships.rs:254`

OLD (exact):
```rust
pub(super) fn unit_leads_any_group(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let leads = {
        let st = borrow_state(state)?;
        if !st.party_group_active || st.party_members.is_empty() {
            false
        } else if matches!(unit.as_str(), "player" | "pet" | "vehicle") {
            st.party_leader_index.is_none() || st.everyone_assistant
        } else if let Some(idx) = resolve_unit_party_index(&st, &unit) {
            st.party_leader_index == Some(idx) || st.everyone_assistant
        } else {
            false
        }
    };
    state.push(Val::Bool(leads));
    Ok(1)
}
```

NEW:
```rust
pub(super) fn unit_leads_any_group(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let leads = {
        let st = borrow_state(state)?;
        #[cfg(feature = "retail-12-0-7")]
        let assistant = crate::c_api::c_party_info::roles_1207::is_assistant(&st, &unit);
        #[cfg(not(feature = "retail-12-0-7"))]
        let assistant = st.everyone_assistant;
        if !st.party_group_active || st.party_members.is_empty() {
            false
        } else if matches!(unit.as_str(), "player" | "pet" | "vehicle") {
            st.party_leader_index.is_none() || assistant
        } else if let Some(idx) = resolve_unit_party_index(&st, &unit) {
            st.party_leader_index == Some(idx) || assistant
        } else {
            false
        }
    };
    state.push(Val::Bool(leads));
    Ok(1)
}
```

### E14 — `producer` — `src/lua_api/globals/group_verbs.rs:103`

OLD (exact):
```rust
    st.everyone_assistant = false;
    Ok(())
```

NEW:
```rust
    st.everyone_assistant = false;
    #[cfg(feature = "retail-12-0-7")]
    {
        st.party_assistants.clear();
        st.party_assistant_exclusions.clear();
    }
    Ok(())
```

### E15 — `producer` — `src/lua_api/on_update.rs:34`

OLD (exact):
```rust
    let total_started = Instant::now();
```

NEW:
```rust
    #[cfg(feature = "retail-12-0-7")]
    crate::c_api::c_party_info::solo_1207::tick(env.rilua_mut().state_mut())?;

    let total_started = Instant::now();
```

### E16 — `producer` — `src/lua_api/globals/spell_macro_verbs.rs:198`

OLD (exact):
```rust
    // INFERRED: invalid/out-of-range numeric input is an atomic no-op.
    // Native numeric coercion and !/~ marker prefixes are not modeled here.
    let Ok(marker) = marker.parse::<u8>() else {
```

NEW:
```rust
    // INFERRED: invalid/out-of-range input is an atomic no-op; ! is unmodeled.
    #[cfg(feature = "retail-12-0-7")]
    let (marker, only_unmarked) = match marker.strip_prefix('~') {
        Some(number) => (number, true),
        None => (marker.as_str(), false),
    };
    #[cfg(not(feature = "retail-12-0-7"))]
    let marker = marker.as_str();
    let Ok(marker) = marker.parse::<u8>() else {
```

### E17 — `producer` — `src/lua_api/globals/spell_macro_verbs.rs:206`

OLD (exact):
```rust
    let function = LuaApiMut::get_global_val(state, "SetRaidTarget");
    let unit = create_string(state, &unit);
```

NEW:
```rust
    #[cfg(feature = "retail-12-0-7")]
    if only_unmarked {
        let sim = borrow_state(state)?;
        let Some(target) = super::targeting_verbs::resolve_unit_snapshot(&sim, &unit) else {
            return Ok(());
        };
        if sim.unit_raid_target_icons.contains_key(&target.guid) {
            return Ok(());
        }
    }
    let function = LuaApiMut::get_global_val(state, "SetRaidTarget");
    let unit = create_string(state, &unit);
```

### E18 — `test-support` — `src/loader/tests/wow_api_globals/startup_globals.rs:274`

OLD (exact):
```rust
            if IsEveryoneAssistant() ~= false then return "party-demote-assistant" end
```

NEW:
```rust
            if IsEveryoneAssistant() ~= true or UnitIsGroupAssistant("player") ~= false then return "party-demote-assistant" end
```

### E19 — `test-support` — `src/loader/tests/wow_api_globals/startup_globals.rs:276`

OLD (exact):
```rust
            if IsEveryoneAssistant() ~= true then return "party-promote-assistant" end
```

NEW:
```rust
            if UnitIsGroupAssistant("party1") ~= true or UnitIsGroupAssistant("player") ~= false then return "party-promote-assistant" end
```

### E20 — `test-support` — `tests/unit_relation_probes.rs:190`

OLD (exact):
```rust
    env.state().borrow_mut().everyone_assistant = true;
    let b: bool = env
```

NEW:
```rust
    env.state().borrow_mut().everyone_assistant = true;
    #[cfg(feature = "retail-12-0-7")]
    {
        env.state().borrow_mut().party_group_active = true;
    }
    let b: bool = env
```

### E21 — `test-support` — `tests/group_verbs.rs:78`

OLD (exact):
```rust
        st.everyone_assistant = true;
    }
    env.exec("LeaveParty()").unwrap();
```

NEW:
```rust
        st.everyone_assistant = true;
        #[cfg(feature = "retail-12-0-7")]
        {
            st.party_assistants.insert("Ada-Realm".into());
            let player_name = st.player.name.clone();
            st.party_assistant_exclusions.insert(player_name);
        }
    }
    env.exec("LeaveParty()").unwrap();
```

### E22 — `test-support` — `tests/group_verbs.rs:86`

OLD (exact):
```rust
    assert!(!st.everyone_assistant);
    drop(st);
```

NEW:
```rust
    assert!(!st.everyone_assistant);
    #[cfg(feature = "retail-12-0-7")]
    {
        assert!(st.party_assistants.is_empty());
        assert!(st.party_assistant_exclusions.is_empty());
    }
    drop(st);
```

### E23 — `producer` — `src/lua_api/globals/spell_macro_verbs.rs:151`

OLD (exact):
```rust
fn run_macro_text(state: &mut LuaState) -> LuaResult<u32> {
    let Some(text) = stack_string(state, 1) else {
        return Ok(0);
    };

    for line in text.lines() {
        run_macro_text_line(state, line)?;
    }
    Ok(0)
}
```

NEW:
```rust
#[cfg(feature = "retail-12-0-7")]
fn read_authenticated_macro_text(state: &LuaState) -> LuaResult<String> {
    // Authenticate all declared inputs and extras before validating text.
    for value in state.stack.iter().take(state.top).skip(state.base) {
        rilua::table_security::unwrap_secret(state, *value)?;
    }
    let text = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let button = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
    // INFERRED: nil button remains accepted for existing cached/host callers.
    if !matches!(button, Val::Nil) {
        read_macro_string(state, button)?;
    }
    read_macro_string(state, text)
}

#[cfg(feature = "retail-12-0-7")]
fn read_macro_string(state: &LuaState, value: Val) -> LuaResult<String> {
    if !matches!(value, Val::Str(_)) {
        return Err(rilua::runtime_error("C_Macro.RunMacroText: expected string"));
    }
    crate::lua_api::methods::val_to_string(state, value)
        .ok_or_else(|| rilua::runtime_error("C_Macro.RunMacroText: expected UTF-8 string"))
}

fn run_macro_text(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-7")]
    let text = read_authenticated_macro_text(state)?;
    #[cfg(not(feature = "retail-12-0-7"))]
    let Some(text) = stack_string(state, 1) else {
        return Ok(0);
    };

    for line in text.lines() {
        run_macro_text_line(state, line)?;
    }
    Ok(0)
}
```

### E24 — `test-support` — `tests/group_verbs.rs:128`

OLD (exact):
```rust
    let first_name = env.state().borrow().party_members[0].name.clone();
```

NEW:
```rust
    let first_name = env.state().borrow().party_members[0].name.clone();
    #[cfg(feature = "retail-12-0-7")]
    {
        env.state().borrow_mut().party_group_active = true;
    }
```

### E25 — `test-support` — `tests/group_verbs.rs:146`

OLD (exact):
```rust
    let second_name = env.state().borrow().party_members[1].name.clone();
```

NEW:
```rust
    let second_name = env.state().borrow().party_members[1].name.clone();
    #[cfg(feature = "retail-12-0-7")]
    {
        env.state().borrow_mut().party_group_active = true;
    }
```

## Consumer compatibility and additional declaration evidence

All cache paths below are relative to `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`. Full occurrence list: `consumer-evidence-p1207-b11.txt`. Static inspection only; no startup proof. Home membership keeps the existing roster producer instead of replacing it with an empty GUID map; empty map governs only the separate category-2 domain. Empty role sets preserve false query defaults; false restriction input does not disable cached actions. Absent solo payload emits no event instead of inventing nil/non-GUID data. No known empty-default exception was found in the inspected call sites, but this is not a consumer runtime acceptance claim.

`Blizzard_UnitPopupShared/UnitPopupSharedButtonMixins.lua:844–847`:
```lua

function UnitPopupPromoteButtonMixin:OnClick(contextData)
	C_PartyInfo.PromoteToLeader(contextData.unit, true);
end
```

`Blizzard_UnitPopupShared/UnitPopupSharedButtonMixins.lua:1957–1961`:
```lua
function UnitPopupSetRaidAssistButtonMixin:OnClick(contextData)
	local fullName = UnitPopupSharedUtil.GetFullPlayerName(contextData);
	C_PartyInfo.PromoteToAssistant(fullName, true);
end

```

`Blizzard_CompactRaidFrames/Mainline/Blizzard_CompactRaidFrameManager.lua:1231–1234`:
```lua
function RaidFrameEveryoneIsAssistMixin:OnClick()
	PlaySound(SOUNDKIT.IG_MAINMENU_OPTION_CHECKBOX_ON);
	C_PartyInfo.SetEveryoneIsAssistant(self:GetChecked());
end
```

`Blizzard_StaticPopup_Game/GameDialogDefs.lua:2713–2716`:
```lua
	EditBoxOnEnterPressed = function(editBox, data)
		local dialog = editBox:GetParent();
		C_PartyInfo.UninviteUnit(data, editBox:GetText());
		dialog:Hide();
```

`Blizzard_Channels/ChannelFrame.lua:567–568`:
```lua
function ChannelFrameMixin:OnGroupFormed(partyCategory, partyGUID)
end
```

`Blizzard_ChatFrameBase/Shared/SlashCommands.lua:1398–1427`:
```lua
end);

SlashCommandUtil.CheckAddSlashCommand(SLASH_COMMAND.TARGET_MARKER, SLASH_COMMAND_CATEGORY.TARGET_MARKER, function(msg)
	local marker, target = SecureCmdOptionParse(msg);
	if ( not target ) then
		target = "target";
	end

	-- Prefixing with an "!" will prevent toggling the marker if it's already assigned.
	if ( marker and string.find(marker, "^!") ) then
		marker = tonumber(string.match(marker, "%d+"));

		if ( GetRaidTargetIndex(target) == marker ) then
			return;
		end
	-- Prefixing with a "~" will prevent setting the marker if the unit already has any marker.
	elseif ( marker and string.find(marker, "^~") ) then
		marker = tonumber(string.match(marker, "%d+"));

		if ( GetRaidTargetIndex(target) ~= nil ) then
			return;
		end
	else
		marker = tonumber(marker);
	end

	if ( marker ) then
		SetRaidTarget(target, tonumber(marker));	--Using /tm 0 will clear the target marker.
	end
end);
```

`Blizzard_APIDocumentationGenerated/UIMacrosDocumentation.lua:41–51`:
```lua
			Name = "RunMacroText",
			Type = "Function",
			HasRestrictions = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "text", Type = "cstring", Nilable = false },
				{ Name = "button", Type = "cstring", Nilable = false },
			},
		},
```

`Blizzard_APIDocumentationGenerated/RaidMarkersDocumentation.lua:95–104`:
```lua
			Name = "SetRaidTarget",
			Type = "Function",
			HasRestrictions = true,
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "target", Type = "UnitToken", Nilable = false },
				{ Name = "userIndex", Type = "luaIndex", Nilable = false },
			},
```

The cached slash consumer extracts `%d+` after `~`, whereas authored parser requires the remaining string to be a valid u8. Malformed prefixes/coercions are **INFERRED bounded policies**, not native parity. SetRaidTarget has its own AllowedWhenUntainted declaration: untouched old argument handling is excluded; a public-value cached action test does not award that policy credit. No NeverSecret annotation applies to this slice's six B11 APIs or macro entry; no false NeverSecret claim or test is added.

## Staged new files and integration order

All four are full new files under `staging/p1207-b11/`, mirroring destination paths:

1. `src/c_api/c_party_info/roles_1207.rs` — producer module.
2. `src/c_api/c_party_info/solo_1207.rs` — producer module.
3. `tests/party_1207_audit.rs` — 11 public-API behavioral cases.
4. `docs/specs/party-12-0-7-audit.md` — unchecked contract in explicit-stat-inputs format.

Authoring is complete, NOT integrated or runtime-verified. Integrator may copy all four and apply only state/test-support anchors for RED: producers are unwired until producer edits land. Existing build.rs includes test modules into the ONE integration binary; do not create a second Cargo test target. Scope the eventual test filter to `party_1207_audit::` plus changed existing cases; no commands are executed here. Both caches and producer module files must exist for their declarations to resolve. Do not award whole-row credit merely because an older control case passes.

## Existing tests changed / preserved

- `test_patch_12_0_7_safe_global_bridges`: demoting player must not unset everyone mode; individual promotion must not toggle everyone mode.
- `unit_is_group_assistant_requires_everyone_assistant_flag`: active-group fixture required under this epoch; test remains a flag-read control, not exhaustive role proof.
- `tests/group_verbs.rs`: LeaveParty now asserts cleanup of explicit role sets; the two namespace uninvite cases explicitly activate their seeded roster. Raw UninviteUnit absence/migration assertions stay unchanged.
- `tests/mouse_tm_commands.rs`, `tests/c_party_info_probes.rs`, `tests/delve_instance_state.rs`, existing ready-check tests: no duplicate modifications; numeric and inherited behavior remain controls for later integration.

## Exclusions and blocked parts

**Evidence/native:** historical build 68182/cache reconciliation, native permission rules, exact native errors, all-profile compilation and actual client execution remain blocked by absent authenticated evidence / author-only scope. Every guessed policy is labeled INFERRED. No page ledger promotion, native parity or complete-row acceptance is claimed.

**Party model:** home GUIDs are existing position-derived identifiers and renumber after removal. Stable identities, roster mutation for category 2, realm/name casefold rules and other unit aliases remain unproven. No fake GUIDs, wrapped constants or implicit service integration are proposed.

**Solo producer:** shared-tick reconciliation is a bounded host-input producer, not a server join pipeline. Host must supply payload and follower entry. Unobserved exit/reentry between ticks is not detected; identity changes/observed exits drive the latch (INFERRED). Solo event publication does not redefine GetNumGroupMembers/IsInGroup or synthesize AI roster members.

**Marker boundary:** `!` syntax, malformed-prefix parity, native macro security, protected secure-click authorization, slash UI registration and SetRaidTarget secret-argument overhaul are excluded. Whole cached SecureTemplates execution is authored but not run; missing bootstrap dependencies/cache remain a possible integration blocker. Do not substitute snippets or shims if it fails.

**Ready checks:** rows038/040 inherit only existing bounded ready-check-lockdown credit. AllowedWhenUntainted extras and native/strict-epoch limitations remain unchanged. No new ready producer or duplicate test is authored.

## Static proof ledger — 2026-10-04

| Check | Exact scope | Result / limitations |
|---|---|---|
| Read-only `git rev-parse HEAD` | baseline | `7702befe8a567c225d9e8680594186e9689734f4` |
| Read-only `git show 7702befe8:<path>` equality | ten edited existing files | every OLD file equals pinned master; no repo/git writes |
| Python exact-match / disjoint-region validation | E01–E25 | each OLD once, pairwise nonoverlapping; JSON records original line |
| `rustfmt --edition 2024 --emit stdout --config skip_children=true` on proposed text via stdin | ten in-memory proposed existing files; query file refreshed after its only later change | parser exit0, no compiler/type/behavior proof; artifact `anchor-syntax-proof-p1207-b11.json` |
| Direct `rustfmt --edition 2024` | three staged Rust files | formatter exit0; tests unchanged afterward; later producer-only comments checked with direct rustfmt --check, exit0 |

Manual readability pass: named authentication/validation/resolution/mutation helpers; short borrow scopes; no warning suppressions or deep nesting. No Rust compiler, cargo, build, test, simulator, agents, models or operational commands were run. Runtime/type acceptance is deliberately unknown. The source hashes/formatter parse are static authoring checks, NOT behavioral tests.

Final counts: **8 primary rows**, **2 inherited ready-check rows**, **1 contextual migration row**; **25 anchored edits**, **11 authored tests**, **4 full staged new files**. More than the lost estimate (~22 edits) because both seeded uninvite fixtures and the macro argument authentication boundary are included explicitly. Staging content hashes: `staging-manifest-p1207-b11.json`.
