# Retail 12.0.7 remaining rows — authoring handoff

Base master: fbe3e20404ceb1285be23a6abe24b0f8aa91221b

Authoring only. No repo/git mutation, cargo, builds, tests, simulator or delegation. Static evidence only; no passing claims.

Active goal: classify all 18 requested rows; stage full new files and exact uniquely anchored state/producer/test-support edits, behavioral WowLuaEnv tests and spec. Exclusions: integration, 3D, speculative native policy. Required verification: static anchor uniqueness and API/type inspection only; execution deferred to integrator.

## Progress
- Source and contract inspection started.

## Early findings

- Duration option/raw-value types and QuestHub race getter have no exact declaration or consumer in current later retail cache. Historical signatures/creation routes/security are missing; do not invent factories.
- Club declarations name opaque IDs but provide no runtime representation; existing guild projection generates numeric row indices and no member-change producer.
- Timeline notification implementation is in the 12.1.5 subsystem, not a 12.0.7 lifecycle provider. Retail legacy color setter stores Lua override tables only. Registration alone is not event proof.
- Model unit-token assignment already has Rust `last_unit` state; investigating a non-rendering GUID getter, without 3D.

## Anchored edits against pinned master

Apply state/test-support before RED; withhold E01/E02 and new producer N01. No new state edits: existing `PlayerModelState.last_unit: Option<String>` defaults to None. Its Rust target/focus identity source is reused, not duplicated. E03 is an existing behavioral fixture change, not a standalone presence assertion.

### E01 [producer] `src/lua_api/frame/methods/widgets/model.rs`

OLD (one match at pinned master):
```text
mod model_scene_actors;
```

NEW:
```text
mod model_scene_actors;
#[cfg(feature = "retail-12-0-7")]
mod model_unit_guid;
```

### E02 [producer] `src/lua_api/frame/methods/widgets/model.rs`

OLD (one match at pinned master):
```text
    ("GetModelFileID", get_model_file_id),
```

NEW:
```text
    ("GetModelFileID", get_model_file_id),
    #[cfg(feature = "retail-12-0-7")]
    ("GetModelUnitGUID", model_unit_guid::get_model_unit_guid),
```

### E03 [test-support] `src/loader/tests/wow_api_globals/startup_globals.rs`

OLD (one match at pinned master):
```text
            if actor.GetModelUnitGUID ~= nil then
                return "ModelSceneActorBase:GetModelUnitGUID=" .. type(actor.GetModelUnitGUID)
            end
```

NEW:
```text
            if actor:GetModelUnitGUID() ~= "" then
                return "ModelSceneActorBase:GetModelUnitGUID unbound default"
            end
            if select('#', actor:GetModelUnitGUID()) ~= 1 then
                return "ModelSceneActorBase:GetModelUnitGUID arity"
            end
```

## Per-row decisions and exact blocked ledger notes

Summary: **a: 1 (137); b: 1 (145, later-epoch only); c: 16 (111–118, 048, 158–162, prose 013/014).** All remain unproved until integration. Later cache may postdate 12.0.7. Where an exact declaration is absent, the nearby quoted declaration is explicitly non-authority for the missing type/API. There is no nonexistent declaration to quote.

### `scriptobjects-DurationTextFormattingOptions-GetAddRemainingText-111` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:111`: "DurationTextFormattingOptions:GetAddRemainingText".

Cached declaration: **none for this exact receiver/method** after scanning all cached Lua files. Nearby distinct structure (not authority):
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingSharedDocumentation.lua:41–46`:
```text
			Name = "DurationTextBindingFormatOptions",
			Type = "Structure",
			Fields =
			{
				{ Name = "formatString", Type = "stringView", Nilable = false },
				{ Name = "components", Type = "table", InnerType = "DurationTextBindingFormatComponent", Nilable = false },
```

Existing today: no exact method/provider. The generic duration proxy is another receiver:
`src/lua_api/globals/lua_duration_object.rs:3–6`:
```text
//! Implements `C_DurationUtil.CreateDuration()` which returns a plain Lua
//! table with a shared metatable.  The result is `type() == "table"` — not
//! userdata — matching the rilua proxy convention used by `AbbreviateConfig`,
//! `FunctionContainer`, etc.
```
Existing nil-factory expectation (absence, not method proof):
`src/loader/tests/wow_api_globals/startup_globals.rs:100–102`:
```text
            if C_DurationUtil.CreateDurationTextFormattingOptions() ~= nil then
                return "DurationTextFormattingOptions factory"
            end
```

What must change: first obtain the missing authority; only then author the corresponding field/getter/setter on an authenticated simulator object. `src/c_api/duration_text_binding/state.rs:8` is `struct BindingSettings`, not either historical receiver. No proposed edit for this blocked row.

Exact ledger note:
> Blocked: DurationTextFormattingOptions:GetAddRemainingText has no exact receiver/method declaration or authenticated creation route in the retained later retail cache; obtain build-68182 type/factory reachability, full method signatures, defaults/domain/units and secret-argument annotations before implementing a genuine handle and Rust-backed values. Do not invent CreateDurationTextFormattingOptions or substitute the later DurationTextBindingFormatOptions structure. No implementation or bounded-coverage credit.

### `scriptobjects-DurationTextFormattingOptions-GetDurationType-112` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:112`: "DurationTextFormattingOptions:GetDurationType".

Cached declaration: **none for this exact receiver/method** after scanning all cached Lua files. Nearby distinct structure (not authority):
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingSharedDocumentation.lua:41–46`:
```text
			Name = "DurationTextBindingFormatOptions",
			Type = "Structure",
			Fields =
			{
				{ Name = "formatString", Type = "stringView", Nilable = false },
				{ Name = "components", Type = "table", InnerType = "DurationTextBindingFormatComponent", Nilable = false },
```

Existing today: no exact method/provider. The generic duration proxy is another receiver:
`src/lua_api/globals/lua_duration_object.rs:3–6`:
```text
//! Implements `C_DurationUtil.CreateDuration()` which returns a plain Lua
//! table with a shared metatable.  The result is `type() == "table"` — not
//! userdata — matching the rilua proxy convention used by `AbbreviateConfig`,
//! `FunctionContainer`, etc.
```
Existing nil-factory expectation (absence, not method proof):
`src/loader/tests/wow_api_globals/startup_globals.rs:100–102`:
```text
            if C_DurationUtil.CreateDurationTextFormattingOptions() ~= nil then
                return "DurationTextFormattingOptions factory"
            end
```

What must change: first obtain the missing authority; only then author the corresponding field/getter/setter on an authenticated simulator object. `src/c_api/duration_text_binding/state.rs:8` is `struct BindingSettings`, not either historical receiver. No proposed edit for this blocked row.

Exact ledger note:
> Blocked: DurationTextFormattingOptions:GetDurationType has no exact receiver/method declaration or authenticated creation route in the retained later retail cache; obtain build-68182 type/factory reachability, full method signatures, defaults/domain/units and secret-argument annotations before implementing a genuine handle and Rust-backed values. Do not invent CreateDurationTextFormattingOptions or substitute the later DurationTextBindingFormatOptions structure. No implementation or bounded-coverage credit.

### `scriptobjects-DurationTextFormattingOptions-SetAddRemainingText-113` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:113`: "DurationTextFormattingOptions:SetAddRemainingText".

Cached declaration: **none for this exact receiver/method** after scanning all cached Lua files. Nearby distinct structure (not authority):
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingSharedDocumentation.lua:41–46`:
```text
			Name = "DurationTextBindingFormatOptions",
			Type = "Structure",
			Fields =
			{
				{ Name = "formatString", Type = "stringView", Nilable = false },
				{ Name = "components", Type = "table", InnerType = "DurationTextBindingFormatComponent", Nilable = false },
```

Existing today: no exact method/provider. The generic duration proxy is another receiver:
`src/lua_api/globals/lua_duration_object.rs:3–6`:
```text
//! Implements `C_DurationUtil.CreateDuration()` which returns a plain Lua
//! table with a shared metatable.  The result is `type() == "table"` — not
//! userdata — matching the rilua proxy convention used by `AbbreviateConfig`,
//! `FunctionContainer`, etc.
```
Existing nil-factory expectation (absence, not method proof):
`src/loader/tests/wow_api_globals/startup_globals.rs:100–102`:
```text
            if C_DurationUtil.CreateDurationTextFormattingOptions() ~= nil then
                return "DurationTextFormattingOptions factory"
            end
```

What must change: first obtain the missing authority; only then author the corresponding field/getter/setter on an authenticated simulator object. `src/c_api/duration_text_binding/state.rs:8` is `struct BindingSettings`, not either historical receiver. No proposed edit for this blocked row.

Exact ledger note:
> Blocked: DurationTextFormattingOptions:SetAddRemainingText has no exact receiver/method declaration or authenticated creation route in the retained later retail cache; obtain build-68182 type/factory reachability, full method signatures, defaults/domain/units and secret-argument annotations before implementing a genuine handle and Rust-backed values. Do not invent CreateDurationTextFormattingOptions or substitute the later DurationTextBindingFormatOptions structure. No implementation or bounded-coverage credit.

### `scriptobjects-DurationTextFormattingOptions-SetDurationType-114` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:114`: "DurationTextFormattingOptions:SetDurationType".

Cached declaration: **none for this exact receiver/method** after scanning all cached Lua files. Nearby distinct structure (not authority):
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingSharedDocumentation.lua:41–46`:
```text
			Name = "DurationTextBindingFormatOptions",
			Type = "Structure",
			Fields =
			{
				{ Name = "formatString", Type = "stringView", Nilable = false },
				{ Name = "components", Type = "table", InnerType = "DurationTextBindingFormatComponent", Nilable = false },
```

Existing today: no exact method/provider. The generic duration proxy is another receiver:
`src/lua_api/globals/lua_duration_object.rs:3–6`:
```text
//! Implements `C_DurationUtil.CreateDuration()` which returns a plain Lua
//! table with a shared metatable.  The result is `type() == "table"` — not
//! userdata — matching the rilua proxy convention used by `AbbreviateConfig`,
//! `FunctionContainer`, etc.
```
Existing nil-factory expectation (absence, not method proof):
`src/loader/tests/wow_api_globals/startup_globals.rs:100–102`:
```text
            if C_DurationUtil.CreateDurationTextFormattingOptions() ~= nil then
                return "DurationTextFormattingOptions factory"
            end
```

What must change: first obtain the missing authority; only then author the corresponding field/getter/setter on an authenticated simulator object. `src/c_api/duration_text_binding/state.rs:8` is `struct BindingSettings`, not either historical receiver. No proposed edit for this blocked row.

Exact ledger note:
> Blocked: DurationTextFormattingOptions:SetDurationType has no exact receiver/method declaration or authenticated creation route in the retained later retail cache; obtain build-68182 type/factory reachability, full method signatures, defaults/domain/units and secret-argument annotations before implementing a genuine handle and Rust-backed values. Do not invent CreateDurationTextFormattingOptions or substitute the later DurationTextBindingFormatOptions structure. No implementation or bounded-coverage credit.

### `scriptobjects-DurationTextRawValue-GetMilliseconds-115` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:115`: "DurationTextRawValue:GetMilliseconds".

Cached declaration: **none for this exact receiver/method** after scanning all cached Lua files. Nearby distinct structure (not authority):
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingSharedDocumentation.lua:41–46`:
```text
			Name = "DurationTextBindingFormatOptions",
			Type = "Structure",
			Fields =
			{
				{ Name = "formatString", Type = "stringView", Nilable = false },
				{ Name = "components", Type = "table", InnerType = "DurationTextBindingFormatComponent", Nilable = false },
```

Existing today: no exact method/provider. The generic duration proxy is another receiver:
`src/lua_api/globals/lua_duration_object.rs:3–6`:
```text
//! Implements `C_DurationUtil.CreateDuration()` which returns a plain Lua
//! table with a shared metatable.  The result is `type() == "table"` — not
//! userdata — matching the rilua proxy convention used by `AbbreviateConfig`,
//! `FunctionContainer`, etc.
```
Existing nil-factory expectation (absence, not method proof):
`src/loader/tests/wow_api_globals/startup_globals.rs:103–105`:
```text
            if C_DurationUtil.CreateDurationTextRawValue() ~= nil then
                return "DurationTextRawValue factory"
            end
```

What must change: first obtain the missing authority; only then author the corresponding field/getter/setter on an authenticated simulator object. `src/c_api/duration_text_binding/state.rs:8` is `struct BindingSettings`, not either historical receiver. No proposed edit for this blocked row.

Exact ledger note:
> Blocked: DurationTextRawValue:GetMilliseconds has no exact receiver/method declaration or authenticated creation route in the retained later retail cache; obtain build-68182 type/factory reachability, full method signatures, defaults/domain/units and secret-argument annotations before implementing a genuine handle and Rust-backed values. Do not invent CreateDurationTextRawValue or substitute the later DurationTextBindingFormatOptions structure. No implementation or bounded-coverage credit.

### `scriptobjects-DurationTextRawValue-GetSeconds-116` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:116`: "DurationTextRawValue:GetSeconds".

Cached declaration: **none for this exact receiver/method** after scanning all cached Lua files. Nearby distinct structure (not authority):
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingSharedDocumentation.lua:41–46`:
```text
			Name = "DurationTextBindingFormatOptions",
			Type = "Structure",
			Fields =
			{
				{ Name = "formatString", Type = "stringView", Nilable = false },
				{ Name = "components", Type = "table", InnerType = "DurationTextBindingFormatComponent", Nilable = false },
```

Existing today: no exact method/provider. The generic duration proxy is another receiver:
`src/lua_api/globals/lua_duration_object.rs:3–6`:
```text
//! Implements `C_DurationUtil.CreateDuration()` which returns a plain Lua
//! table with a shared metatable.  The result is `type() == "table"` — not
//! userdata — matching the rilua proxy convention used by `AbbreviateConfig`,
//! `FunctionContainer`, etc.
```
Existing nil-factory expectation (absence, not method proof):
`src/loader/tests/wow_api_globals/startup_globals.rs:103–105`:
```text
            if C_DurationUtil.CreateDurationTextRawValue() ~= nil then
                return "DurationTextRawValue factory"
            end
```

What must change: first obtain the missing authority; only then author the corresponding field/getter/setter on an authenticated simulator object. `src/c_api/duration_text_binding/state.rs:8` is `struct BindingSettings`, not either historical receiver. No proposed edit for this blocked row.

Exact ledger note:
> Blocked: DurationTextRawValue:GetSeconds has no exact receiver/method declaration or authenticated creation route in the retained later retail cache; obtain build-68182 type/factory reachability, full method signatures, defaults/domain/units and secret-argument annotations before implementing a genuine handle and Rust-backed values. Do not invent CreateDurationTextRawValue or substitute the later DurationTextBindingFormatOptions structure. No implementation or bounded-coverage credit.

### `scriptobjects-DurationTextRawValue-SetMilliseconds-117` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:117`: "DurationTextRawValue:SetMilliseconds".

Cached declaration: **none for this exact receiver/method** after scanning all cached Lua files. Nearby distinct structure (not authority):
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingSharedDocumentation.lua:41–46`:
```text
			Name = "DurationTextBindingFormatOptions",
			Type = "Structure",
			Fields =
			{
				{ Name = "formatString", Type = "stringView", Nilable = false },
				{ Name = "components", Type = "table", InnerType = "DurationTextBindingFormatComponent", Nilable = false },
```

Existing today: no exact method/provider. The generic duration proxy is another receiver:
`src/lua_api/globals/lua_duration_object.rs:3–6`:
```text
//! Implements `C_DurationUtil.CreateDuration()` which returns a plain Lua
//! table with a shared metatable.  The result is `type() == "table"` — not
//! userdata — matching the rilua proxy convention used by `AbbreviateConfig`,
//! `FunctionContainer`, etc.
```
Existing nil-factory expectation (absence, not method proof):
`src/loader/tests/wow_api_globals/startup_globals.rs:103–105`:
```text
            if C_DurationUtil.CreateDurationTextRawValue() ~= nil then
                return "DurationTextRawValue factory"
            end
```

What must change: first obtain the missing authority; only then author the corresponding field/getter/setter on an authenticated simulator object. `src/c_api/duration_text_binding/state.rs:8` is `struct BindingSettings`, not either historical receiver. No proposed edit for this blocked row.

Exact ledger note:
> Blocked: DurationTextRawValue:SetMilliseconds has no exact receiver/method declaration or authenticated creation route in the retained later retail cache; obtain build-68182 type/factory reachability, full method signatures, defaults/domain/units and secret-argument annotations before implementing a genuine handle and Rust-backed values. Do not invent CreateDurationTextRawValue or substitute the later DurationTextBindingFormatOptions structure. No implementation or bounded-coverage credit.

### `scriptobjects-DurationTextRawValue-SetSeconds-118` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:118`: "DurationTextRawValue:SetSeconds".

Cached declaration: **none for this exact receiver/method** after scanning all cached Lua files. Nearby distinct structure (not authority):
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingSharedDocumentation.lua:41–46`:
```text
			Name = "DurationTextBindingFormatOptions",
			Type = "Structure",
			Fields =
			{
				{ Name = "formatString", Type = "stringView", Nilable = false },
				{ Name = "components", Type = "table", InnerType = "DurationTextBindingFormatComponent", Nilable = false },
```

Existing today: no exact method/provider. The generic duration proxy is another receiver:
`src/lua_api/globals/lua_duration_object.rs:3–6`:
```text
//! Implements `C_DurationUtil.CreateDuration()` which returns a plain Lua
//! table with a shared metatable.  The result is `type() == "table"` — not
//! userdata — matching the rilua proxy convention used by `AbbreviateConfig`,
//! `FunctionContainer`, etc.
```
Existing nil-factory expectation (absence, not method proof):
`src/loader/tests/wow_api_globals/startup_globals.rs:103–105`:
```text
            if C_DurationUtil.CreateDurationTextRawValue() ~= nil then
                return "DurationTextRawValue factory"
            end
```

What must change: first obtain the missing authority; only then author the corresponding field/getter/setter on an authenticated simulator object. `src/c_api/duration_text_binding/state.rs:8` is `struct BindingSettings`, not either historical receiver. No proposed edit for this blocked row.

Exact ledger note:
> Blocked: DurationTextRawValue:SetSeconds has no exact receiver/method declaration or authenticated creation route in the retained later retail cache; obtain build-68182 type/factory reachability, full method signatures, defaults/domain/units and secret-argument annotations before implementing a genuine handle and Rust-backed values. Do not invent CreateDurationTextRawValue or substitute the later DurationTextBindingFormatOptions structure. No implementation or bounded-coverage credit.

### `global api-C_QuestHub-GetDragonridingRacesForAreaPOI-048` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:48`: "C_QuestHub.GetDragonridingRacesForAreaPOI".

Cached declaration: **exact getter absent**. Current namespace instead declares an unrelated membership predicate:
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/QuestHubInfoDocumentation.lua:10–18`:
```text
		{
			Name = "IsAreaPOICurrentlyRelatedToHub",
			Type = "Function",
			SecretArguments = "AllowedWhenUntainted",

			Arguments =
			{
				{ Name = "areaPoiID", Type = "number", Nilable = false },
				{ Name = "hubAreaPoiID", Type = "number", Nilable = false },
```

Existing today — unconditional empty table, not content state:
`src/c_api/c_quest_hub.rs:40–45`:
```text
#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn get_dragonriding_races_for_area_poi(state: &mut LuaState) -> LuaResult<u32> {
    let races = create_table(state);
    state.push(races);
    Ok(1)
}
```

What must change: obtain historical arguments, arity/nilability, populated ordered race-result schema and secret policy; replace the probe with a per-environment POI/content input and live Rust getter once schema is known. `src/loader/tests/wow_api_globals/startup_globals.rs:299` only asserts table shape and does not prove races. No edit proposed; the old empty default is not promoted to modeled proof. No cached consumer of this exact getter was located.

Exact ledger note:
> Blocked: C_QuestHub.GetDragonridingRacesForAreaPOI currently returns an unconditional empty table; the exact getter is absent from the later retail documentation/consumer cache. Missing build-68182 argument/return declaration, populated ordered race-result schema and secret policy prevent an honest host-state producer. Obtain historical declaration or native populated output before replacing the probe; empty-array tests alone give no credit.

### `widgets-ModelSceneActorBase-GetModelUnitGUID-137` — a (author)

Source `data/patch-api/sources/12.0.7-api-changes.txt:137`: "ModelSceneActorBase:GetModelUnitGUID - ret1.ConditionalSecret".

Exact cached declaration:
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/FrameAPIModelSceneFrameActorBaseDocumentation.lua:138–148`:
```text
			Name = "GetModelUnitGUID",
			Type = "Function",

			Arguments =
			{
			},

			Returns =
			{
				{ Name = "guid", Type = "WOWGUID", Nilable = false },
			},
```

Existing today — explicit optional binding and Rust assignment:
`src/widget/frame_types.rs:226–234`:
```text
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerModelState {
    pub do_blend: bool,
    pub keep_model_on_hide: bool,
    pub last_unit: Option<String>,
    pub last_item: Option<String>,
    pub last_item_appearance: Option<String>,
    pub active_anim_kit: Option<i32>,
}
```
`src/lua_api/frame/methods/widgets/model/model_unit.rs:33–40`:
```text
        } else {
            let frame = sim
                .widgets
                .get_mut_visual(id)
                .ok_or_else(|| runtime_error("model unit assignment frame does not exist"))?;
            frame.model_state_mut().player_model_state.last_unit = Some(unit);
            Val::Bool(true)
        }
```
GUID lookup reads current target/focus state, not Lua:
`src/lua_api/globals/unit_misc.rs:203–212`:
```text
pub(crate) fn existing_guid_for_unit(
    sim: &crate::lua_api::state::SimState,
    unit: &str,
) -> Option<String> {
    if !super::group_queries::unit_exists_in_state(sim, unit) {
        return None;
    }
    let guid = guid_for_unit(sim, unit);
    (guid != UNKNOWN_CREATURE_GUID && !guid.is_empty()).then_some(guid)
}
```
`src/lua_api/globals/unit_misc.rs:223–229`:
```text
fn target_guid(sim: &crate::lua_api::state::SimState) -> String {
    sim.current_target
        .as_ref()
        .map(|target| target.guid.clone())
        .unwrap_or_else(|| UNKNOWN_CREATURE_GUID.to_string())
}

```

What must change: N01 adds a genuine-native-backed actor getter reading existing binding/identity state live; E01/E02 register it only under `retail-12-0-7`; E03 replaces the old nil-method expectation. Six N02 tests cover default, live mutation, isolation, public output, handle validation and both caller contexts. No new state fields, no renderer, no Lua method calls, no alternate provider.

**INFERRED policies**: unbound/missing identity is one empty nonnil GUID string; bound token resolves current identity rather than historical snapshot; absent input annotation interpreted NotAllowed (all secrets/extras rejected before validation). These are bounded simulator choices, not authenticated native defaults. Source removal of ConditionalSecret justifies public output even when UnitGUID is restricted. No cached Lua consumer of GetModelUnitGUID was located outside generated declaration, so no observed empty-default conflict.

Risk: proposed getter exposes current restricted identity by design of the source delta; do not strengthen native-parity claims or change existing SetModelByUnit rejection rules. Existing ClearModel does not clear last_unit; lifecycle parity remains excluded rather than silently broadened.

### `events-CLUB_MEMBER_ADDED-158` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:158`: "CLUB_MEMBER_ADDED [2].Type number -> ClubMemberOpaqueId".

Exact cached declaration:
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1291–1296`:
```text
			LiteralName = "CLUB_MEMBER_ADDED",
			SynchronousEvent = true,
			Payload =
			{
				{ Name = "clubId", Type = "ClubId", Nilable = false },
				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
```

Existing today — registerable name:
`src/event/valid_events_a.rs:421`:
```text
    "CLUB_MEMBER_ADDED",
```
Shared numeric row-index projection (not opaque identity):
`src/lua_api/globals/missing_surface/club_info.rs:187–194`:
```text
    let member_count = borrow_state(state)?.world.guild_members.len();
    let array = create_table(state);
    for zero_based_index in 0..member_count {
        let member_id = member_id_from_index(zero_based_index);
        set_table_array(state, array, member_id, Val::Num(member_id as f64));
    }
    state.push(array);
    Ok(1)
```

What must change: obtain opaque-ID authority, then replace numeric roster-index identities with a coherent single C_Club backing model and real membership/presence/removal/role/update transitions before dispatch. No code/test edit proposed for this row. Cached Communities consumers reuse memberId in C_Club lookup/invitation paths; arbitrary string/userdata substitution would risk breaking those paths.

Exact ledger note:
> Blocked: CLUB_MEMBER_ADDED changes payload[2] from number to ClubMemberOpaqueId, but the retained declaration names the alias without defining its runtime representation, equality/lifetime or historical native payload. Existing C_Club derives numeric IDs from mutable guild-row indices and has no real member-transition producer. Obtain build-68182 opaque-ID and populated event evidence before modeling identities and emitting this tuple; registration or arbitrary injected events give no credit.

### `events-CLUB_MEMBER_PRESENCE_UPDATED-159` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:159`: "CLUB_MEMBER_PRESENCE_UPDATED [2].Type number -> ClubMemberOpaqueId".

Exact cached declaration:
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1302–1308`:
```text
			LiteralName = "CLUB_MEMBER_PRESENCE_UPDATED",
			SynchronousEvent = true,
			Payload =
			{
				{ Name = "clubId", Type = "ClubId", Nilable = false },
				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
				{ Name = "presence", Type = "ClubMemberPresence", Nilable = false },
```

Existing today — registerable name:
`src/event/valid_events_a.rs:422`:
```text
    "CLUB_MEMBER_PRESENCE_UPDATED",
```
Shared numeric row-index projection (not opaque identity):
`src/lua_api/globals/missing_surface/club_info.rs:187–194`:
```text
    let member_count = borrow_state(state)?.world.guild_members.len();
    let array = create_table(state);
    for zero_based_index in 0..member_count {
        let member_id = member_id_from_index(zero_based_index);
        set_table_array(state, array, member_id, Val::Num(member_id as f64));
    }
    state.push(array);
    Ok(1)
```

What must change: obtain opaque-ID authority, then replace numeric roster-index identities with a coherent single C_Club backing model and real membership/presence/removal/role/update transitions before dispatch. No code/test edit proposed for this row. Cached Communities consumers reuse memberId in C_Club lookup/invitation paths; arbitrary string/userdata substitution would risk breaking those paths.

Exact ledger note:
> Blocked: CLUB_MEMBER_PRESENCE_UPDATED changes payload[2] from number to ClubMemberOpaqueId, but the retained declaration names the alias without defining its runtime representation, equality/lifetime or historical native payload. Existing C_Club derives numeric IDs from mutable guild-row indices and has no real member-transition producer. Obtain build-68182 opaque-ID and populated event evidence before modeling identities and emitting this tuple; registration or arbitrary injected events give no credit.

### `events-CLUB_MEMBER_REMOVED-160` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:160`: "CLUB_MEMBER_REMOVED [2].Type number -> ClubMemberOpaqueId".

Exact cached declaration:
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1314–1319`:
```text
			LiteralName = "CLUB_MEMBER_REMOVED",
			SynchronousEvent = true,
			Payload =
			{
				{ Name = "clubId", Type = "ClubId", Nilable = false },
				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
```

Existing today — registerable name:
`src/event/valid_events_a.rs:423`:
```text
    "CLUB_MEMBER_REMOVED",
```
Shared numeric row-index projection (not opaque identity):
`src/lua_api/globals/missing_surface/club_info.rs:187–194`:
```text
    let member_count = borrow_state(state)?.world.guild_members.len();
    let array = create_table(state);
    for zero_based_index in 0..member_count {
        let member_id = member_id_from_index(zero_based_index);
        set_table_array(state, array, member_id, Val::Num(member_id as f64));
    }
    state.push(array);
    Ok(1)
```

What must change: obtain opaque-ID authority, then replace numeric roster-index identities with a coherent single C_Club backing model and real membership/presence/removal/role/update transitions before dispatch. No code/test edit proposed for this row. Cached Communities consumers reuse memberId in C_Club lookup/invitation paths; arbitrary string/userdata substitution would risk breaking those paths.

Exact ledger note:
> Blocked: CLUB_MEMBER_REMOVED changes payload[2] from number to ClubMemberOpaqueId, but the retained declaration names the alias without defining its runtime representation, equality/lifetime or historical native payload. Existing C_Club derives numeric IDs from mutable guild-row indices and has no real member-transition producer. Obtain build-68182 opaque-ID and populated event evidence before modeling identities and emitting this tuple; registration or arbitrary injected events give no credit.

### `events-CLUB_MEMBER_ROLE_UPDATED-161` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:161`: "CLUB_MEMBER_ROLE_UPDATED [2].Type number -> ClubMemberOpaqueId".

Exact cached declaration:
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1325–1331`:
```text
			LiteralName = "CLUB_MEMBER_ROLE_UPDATED",
			SynchronousEvent = true,
			Payload =
			{
				{ Name = "clubId", Type = "ClubId", Nilable = false },
				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
				{ Name = "roleId", Type = "number", Nilable = false },
```

Existing today — registerable name:
`src/event/valid_events_a.rs:424`:
```text
    "CLUB_MEMBER_ROLE_UPDATED",
```
Shared numeric row-index projection (not opaque identity):
`src/lua_api/globals/missing_surface/club_info.rs:187–194`:
```text
    let member_count = borrow_state(state)?.world.guild_members.len();
    let array = create_table(state);
    for zero_based_index in 0..member_count {
        let member_id = member_id_from_index(zero_based_index);
        set_table_array(state, array, member_id, Val::Num(member_id as f64));
    }
    state.push(array);
    Ok(1)
```

What must change: obtain opaque-ID authority, then replace numeric roster-index identities with a coherent single C_Club backing model and real membership/presence/removal/role/update transitions before dispatch. No code/test edit proposed for this row. Cached Communities consumers reuse memberId in C_Club lookup/invitation paths; arbitrary string/userdata substitution would risk breaking those paths.

Exact ledger note:
> Blocked: CLUB_MEMBER_ROLE_UPDATED changes payload[2] from number to ClubMemberOpaqueId, but the retained declaration names the alias without defining its runtime representation, equality/lifetime or historical native payload. Existing C_Club derives numeric IDs from mutable guild-row indices and has no real member-transition producer. Obtain build-68182 opaque-ID and populated event evidence before modeling identities and emitting this tuple; registration or arbitrary injected events give no credit.

### `events-CLUB_MEMBER_UPDATED-162` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:162`: "CLUB_MEMBER_UPDATED [2].Type number -> ClubMemberOpaqueId".

Exact cached declaration:
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1337–1342`:
```text
			LiteralName = "CLUB_MEMBER_UPDATED",
			SynchronousEvent = true,
			Payload =
			{
				{ Name = "clubId", Type = "ClubId", Nilable = false },
				{ Name = "memberId", Type = "ClubMemberOpaqueId", Nilable = false },
```

Existing today — registerable name:
`src/event/valid_events_a.rs:425`:
```text
    "CLUB_MEMBER_UPDATED",
```
Shared numeric row-index projection (not opaque identity):
`src/lua_api/globals/missing_surface/club_info.rs:187–194`:
```text
    let member_count = borrow_state(state)?.world.guild_members.len();
    let array = create_table(state);
    for zero_based_index in 0..member_count {
        let member_id = member_id_from_index(zero_based_index);
        set_table_array(state, array, member_id, Val::Num(member_id as f64));
    }
    state.push(array);
    Ok(1)
```

What must change: obtain opaque-ID authority, then replace numeric roster-index identities with a coherent single C_Club backing model and real membership/presence/removal/role/update transitions before dispatch. No code/test edit proposed for this row. Cached Communities consumers reuse memberId in C_Club lookup/invitation paths; arbitrary string/userdata substitution would risk breaking those paths.

Exact ledger note:
> Blocked: CLUB_MEMBER_UPDATED changes payload[2] from number to ClubMemberOpaqueId, but the retained declaration names the alias without defining its runtime representation, equality/lifetime or historical native payload. Existing C_Club derives numeric IDs from mutable guild-row indices and has no real member-transition producer. Obtain build-68182 opaque-ID and populated event evidence before modeling identities and emitting this tuple; registration or arbitrary injected events give no credit.

### `events-ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED-145` — b (existing implementation, later-epoch qualified)

Source `data/patch-api/sources/12.0.7-api-changes.txt:145`: "ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED".

Exact cached declaration:
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterTimelineDocumentation.lua:479–487`:
```text
			Name = "EncounterTimelineEventColorChanged",
			Type = "Event",
			LiteralName = "ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED",
			UniqueEvent = true,
			Documentation = { "Fired when an event has met a condition that should trigger a color change." },
			Payload =
			{
				{ Name = "eventID", Type = "EncounterTimelineEventID", Nilable = false },
			},
```

Existing producer and trigger:
`src/c_api/c_encounter_timeline/notifications.rs:32`:
```text
            event(state, "ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED", id)?;
```
`src/c_api/c_encounter_timeline/notifications.rs:46–48`:
```text
pub(super) fn event(state: &mut LuaState, name: &str, id: u32) -> LuaResult<()> {
    dispatch_event_now(state, name, &[Val::Num(f64::from(id))])
}
```
`src/c_api/c_encounter_timeline/layout.rs:4`:
```text
pub(super) const HIGHLIGHT_TIME: f64 = 5.0;
```
Gate limitation:
`src/c_api/mod.rs:75–76`:
```text
#[cfg(feature = "retail-12-1-5")]
pub(crate) mod c_encounter_timeline;
```
`src/event/valid_events.rs:98` separately admits the name under 12.0.7. Its admission is not a lifecycle producer. `src/lua_api/globals/missing_surface/encounter_events.rs:130` is a legacy color setter storing Lua overrides without producing this event.

What must change: **only N03 proving tests**, four behavioral cases for the already modeled 12.1.5 timer-driven notification, public exact eventID, visible committed state, unchanged-tick no duplicate, empty default, cancellation, environment isolation and tainted listener. No producer edits or backport. The later feature transitively includes `retail-12-0-7`; default Retail 12.1.0 lacks this subsystem. Native UniqueEvent coalescing, color setter/alpha notification, warning distinctions and strict-12.0.7 lifecycle remain unproved. Do not close the row as whole-line/native or strict epoch coverage.

Exact strict-epoch gap note:
> Existing color-change notification is produced by the retail-12-1-5 timeline model, not the strict retail-12-0-7 runtime. Authored tests target only that later explicit-state producer when enabled; they have not run. Strict-12.0.7 admission alone has no notification lifecycle; missing historical lifecycle/state/color-trigger contract remains blocked. No automatic backport or full-row promotion.

### `prose-undated-013` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:13`: "- SimulateMouse APIs no longer carry taint when used, restricted to gamepad action and disallowed with forbidden/locked/script-inaccessible/protected mouse foci in combat.".

Exact later-cache declarations and focus precondition:
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua:266–275`:
```text
			Name = "SimulateMouseClick",
			Type = "Function",
			RequiresLimitedInput = true,
			MouseFocusValidForLimitedInput = true,
			SecretArguments = "AllowedWhenUntainted",
			Documentation = { "Effectively the same as SimulateMouseDown plus SimulateMouseUp and consumes limited input for both." },

			Arguments =
			{
				{ Name = "button", Type = "mouseButton", Nilable = false },
```
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua:279–287`:
```text
			Name = "SimulateMouseDown",
			Type = "Function",
			RequiresLimitedInput = true,
			MouseFocusValidForLimitedInput = true,
			SecretArguments = "AllowedWhenUntainted",
			Documentation = { "Insecure code can only call this once in response to gamepad input hardware events." },

			Arguments =
			{
```
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua:292–300`:
```text
			Name = "SimulateMouseUp",
			Type = "Function",
			RequiresLimitedInput = true,
			MouseFocusValidForLimitedInput = true,
			SecretArguments = "AllowedWhenUntainted",
			Documentation = { "Insecure code can only call this once in response to gamepad input hardware events." },

			Arguments =
			{
```
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua:305–313`:
```text
			Name = "SimulateMouseWheel",
			Type = "Function",
			RequiresLimitedInput = true,
			MouseFocusValidForLimitedInput = true,
			SecretArguments = "AllowedWhenUntainted",
			Documentation = { "Insecure code can only call this once in response to gamepad input hardware events." },

			Arguments =
			{
```
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua:331–333`:
```text
			FailureMode = "ReturnNothing",
			Documentation = { "Requires that all mouse foci are not forbidden, hidden from the global environment, fully locked down, script inaccessible, or protected frames (while in combat)" },
		},
```

Existing today: no exact SimulateMouse provider in `src/`; the nearest limited-input material is enum registration in `src/loader/tests/wow_api_globals/patch_12_0_0_limited_input_type_enums.rs:8`, not host action authority or input consumption. Existing frame restriction tests do not create that capability.

What must change: establish genuine gamepad-action context, consumable input budget, committed focus eligibility and dispatch effects, with historical allocation/refill/Click double-consumption rules and unchanged caller taint. If implemented, unwrap_secret ALL arguments/extras before validation for AllowedWhenUntainted. No implementation authored without this authority.

Exact ledger note:
> Blocked: SimulateMouseClick/Down/Up/Wheel are declared with RequiresLimitedInput, MouseFocusValidForLimitedInput and AllowedWhenUntainted, but simulator has no authenticated gamepad-action context/consumable budget provider. Missing build-68182 budget lifetime, Click consumption and locked/focus eligibility authority prevents non-shim input simulation and taint proof. Obtain that authority, then model real input side effects and all denial boundaries; no always-allowed context or generic event injection earns coverage.

### `prose-undated-014` — c (blocked)

Source `data/patch-api/sources/12.0.7-api-changes.txt:14`: "- debugstack and debuglocals return secret values if current function or caller accessed secret value.".

Cached declaration: no exact generated declaration found. Cached consumer is not declaration authority:
`/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_ScriptErrors/Blizzard_ScriptErrors.lua:59–60`:
```text
	local stack = debugstack(debugStackLevel);
	local locals = debuglocals(debugStackLevel, skipFunctionsAndUserdata);
```

Existing today — ordinary Lua string stack/locals producers:
`src/lua_api/workarounds/temporary/debug_environment_defaults.rs:121–125`:
```text
  function debugstack(level, count1, count2)
    if not debug or not debug.getinfo then
      return ""
    end
    local start = (tonumber(level) or 1) + 1
```
`src/lua_api/workarounds/temporary/debug_environment_defaults.rs:147–150`:
```text
    if stack ~= "" then stack = stack .. "\n" end
    return stack
  end
end
```
`src/lua_api/workarounds/temporary/debug_environment_defaults.rs:153–156`:
```text
  function debuglocals(level)
    if not debug or not debug.getinfo or not debug.getlocal then
      return ""
    end
```
Pinned dependency `Cargo.toml:26` uses rilua rev `6044544b960cd68b4b0c58bb3373412757c2caee`. Its local `src/vm/callinfo.rs:24–63` has stack/base/PC/tail-call/taint fields but no accessed-secret history; lines 56–62 describe `pub taint: Option<String>`. Scan of pinned rilua src found no accessed_secret/secret_access history. This is bounded static evidence, not an exhaustive VM proof.

What must change: add VM-backed current/caller secret-access tracking with historical lifetime/reset rules and rooted secret debugger results. A secret local is not past access; stack taint is not access history. A global boolean or replaceable Lua check would be a symptom shim. No existing tests changed; existing debug-default/security tests do not assert this boundary.

Exact ledger note:
> Blocked: debugstack/debuglocals currently compose public Lua strings; the pinned rilua call-frame state exposes taint, not current/caller accessed-secret history. Missing VM capability and native scope/lifetime/reset authority prevent this delta. Obtain historical access-history semantics and implement that frame history in rilua before producing rooted secret debug outputs; never substitute global taint or presence of a secret local.

## Staged full-new-file inventory

All paths below are relative to `staging/p1207-remaining/`; copy only during separately authorized integration.

| ID | Tag | Path | Role |
|---|---|---|---|
| N01 | producer | `src/lua_api/frame/methods/widgets/model/model_unit_guid.rs` | Non-rendering native actor getter |
| N02 | test-support | `tests/p1207_remaining_model_unit_guid.rs` | Six new behavioral tests |
| N03 | test-support | `tests/p1207_remaining_timeline_color_event.rs` | Four tests of existing later-epoch producer |
| N04 | test-support | `docs/specs/retail-12-0-7-remaining-audit.md` | Prescriptive spec; all requirements unchecked |

No test-module registration edit: `tests/integration.rs:1` includes generated integration_tests.rs; project collects test files into one binary. All new helpers/tests have unique names scoped to their modules; no u32 env.eval, EventQueue API assumptions, secretwrap inside tainted code, fake-object protocols or source-substring assertions.

## Existing tests that change / stay unchanged

- E03 changes `src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_widget_compatibility_surface`: old getter-absence assertion becomes unbound return-value/arity assertion.
- Existing duration factory-absence fixture at the same file:100–105 remains unchanged; those rows are blocked and no guessed factories are introduced.
- Existing C_QuestHub table-shape assertion at :299 and event-name admission fixture remain unchanged; neither gets behavior credit from this handoff.
- Existing model-assignment identity guards (`tests/cast_events_identity.rs`) remain unchanged; no setter behavior is changed. Existing later timeline tests remain unchanged; N03 adds direct color-notification coverage.

## Integrator sequencing — not executed here

1. Revalidate pinned-master anchors before applying changes; if master drifted, reconcile rather than blindly replay.
2. Stage N02/N03/N04 and E03 for RED; withhold E01/E02/N01. Six model tests and updated startup fixture should fail on missing getter, not compile/type mistakes. N03 tests existing behavior and is not expected to fail solely because this producer is withheld; it runs only when the later timeline feature is enabled.
3. Stage N01 and E01/E02 for GREEN. No additional state is required: optional last_unit and host target/focus GUIDs are already explicit Rust state. Do not remove the 12.0.7 gate or enable 3D.
4. Execute separately authorized focused proof. Keep authored-but-unrun, later-epoch and inferred-policy limits visible; do not update ledger status to bounded-coverage based on this handoff alone.

## Static proof ledger — 2026-10-04

| Operation | Exact scope/revision | Result | Invalidated? |
|---|---|---|---|
| Read-only `git rev-parse master` | Start of authoring | `fbe3e20404ceb1285be23a6abe24b0f8aa91221b` | No subsequent repo/git writes by this task |
| Read-only `git show <base>:<path>` plus in-memory OLD-count assertions | E01/E02 model.rs and E03 startup_globals.rs, pinned base above | All three OLD blocks match exactly once at base and current files | No later anchored edit changes |
| Cache/read-only source inspection | Source lines 13/14/48/111–118/137/145/158–162; documented cached declaration ranges and existing producers | All 18 row headings present once; 16 individual `Blocked:` notes; one separate strict-epoch gap for row145 | No later row changes |
| In-memory artifact inventory | Four new files, E01–E03 | 10 `#[test]` cases in two test files; one producer; one spec | No later staged changes |
| Manual Rust/API/readability inspection | N01/N02/N03; relevant IntoStack, native_frame_id_from_val, actor construction and direct RustFn registration | Native-backed actor validation; String pushes one result; secret receiver/extras checked before receiver validation; no suppression attributes, deeply nested logic or alternate providers added | Compilation/execution not established |

**Not run:** cargo, tests, builds, simulator, rustfmt, compiler, lint, independent agents/models. Static checks do not establish that authored Rust compiles or tests pass. Repo, coverage ledger and git state were not modified by this authoring task.

### Artifact SHA-256

- `docs/specs/retail-12-0-7-remaining-audit.md`: `c2a5ba08024489a56f817d2ae4ed45414b6bea970220d4cdbf99ba925efd1073`
- `src/lua_api/frame/methods/widgets/model/model_unit_guid.rs`: `12abe43a108a0f0aa8d20f05535f4b38aca16ff6d578d9ad165f71b66474b9e1`
- `tests/p1207_remaining_model_unit_guid.rs`: `10ee6446555edda577c463c9798dc06c88b57eca3ea242fa342de2b70719a158`
- `tests/p1207_remaining_timeline_color_event.rs`: `bb1c6ae2cd2bda68654e9884ce652685bfce8bb7a7bc27f5128f0a61d13cf288`

## Final handoff totals

- 18 individual decisions: 1 a, 1 b (later-epoch qualification), 16 c.
- 3 anchored edits: 0 state, 2 producer, 1 test-support.
- 4 staged full new files: 1 producer, 2 test files (10 cases), 1 spec.
- Blocked rows: 111/112/113/114/115/116/117/118, 048, 158/159/160/161/162, prose013/prose014. Row145 also retains a strict-12.0.7 lifecycle gap, without misclassifying existing later implementation as absent.
