//! 12.1.0 managed AuraContainer groups, slots, filters, and sorting (B12a).
//!
//! Every container is created by addon-tainted Lua through the cached
//! `CustomAuraContainerTemplate`; auras come from the native player aura model
//! and reach the container through `UNIT_AURA`.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::{
    assert_no_lua_errors, aura, debuff, icons, load_env, set_player_auras, settle,
};
use wow_ui_sim::lua_api::state::AuraInfo;

#[test]
fn managed_container_creates_anchors_and_follows_live_unit_aura_in_batches_of_ten() {
    let env = load_env();
    env.exec(
        r#"
        AuditInitCalls = 0
        AuditLive = AuditContainer({{'buffs', 'HELPFUL', {
            sortMethod = AuraContainerSortMethod.AuraInstanceIDOnly,
            initializeFrame = function(frame)
                AuditInitCalls = AuditInitCalls + 1
                AuditInitIcon(frame)
            end,
        }}})
        assert(AuditLive:GetObjectType() == 'ManagedAuraContainer')
        function AuditCheckManagedFrames(key)
            for index = 1, AuditLive:GetAuraGroupFrameCount(key) do
                local frame = AuditLive:GetAuraGroupFrame(key, index)
                assert(frame:GetObjectType() == 'AuraButton', 'container creates AuraButtons')
                assert(frame:GetParent() == AuditLive, 'buttons are parented to the container')
                if AuditIsShown(frame) then
                    local _, relativeTo = frame:GetPoint(1)
                    assert(AuditPlain(relativeTo) == AuditLive, 'shown buttons are anchored to the container')
                end
            end
            assert(AuditLive:GetAuraGroupFrame(key, AuditLive:GetAuraGroupFrameCount(key) + 1) == nil)
        end
    "#,
    )
    .unwrap();
    // A batch is allocated up-front, before any aura exists.
    set_player_auras(&env, vec![]);
    settle(&env);
    assert_eq!(
        env.eval::<String>("return AuditLive:GetAuraGroupFrameCount('buffs')..'/'..AuditInitCalls")
            .unwrap(),
        "10/10"
    );
    assert_eq!(icons(&env, "AuditLive", "buffs"), "");

    set_player_auras(&env, (1001..=1003).map(aura).collect());
    settle(&env);
    assert_eq!(icons(&env, "AuditLive", "buffs"), "1001,1002,1003");
    env.exec("AuditCheckManagedFrames('buffs')").unwrap();

    // Eleven auras exceed the first batch: exactly one more batch of ten.
    set_player_auras(&env, (1001..=1011).map(aura).collect());
    settle(&env);
    assert_eq!(
        env.eval::<String>("return AuditLive:GetAuraGroupFrameCount('buffs')..'/'..AuditInitCalls")
            .unwrap(),
        "20/20"
    );
    assert_eq!(
        icons(&env, "AuditLive", "buffs"),
        "1001,1002,1003,1004,1005,1006,1007,1008,1009,1010,1011"
    );
    env.exec("AuditCheckManagedFrames('buffs')").unwrap();

    // Removal through UNIT_AURA hides released buttons but keeps them owned.
    set_player_auras(&env, vec![aura(1004), aura(1009)]);
    settle(&env);
    assert_eq!(icons(&env, "AuditLive", "buffs"), "1004,1009");
    assert_eq!(
        env.eval::<f64>("return AuditLive:GetAuraGroupFrameCount('buffs')")
            .unwrap(),
        20.0
    );
    assert_no_lua_errors(&env);
}

#[test]
fn managed_container_has_no_add_aura_frame_and_rejects_addon_reparenting() {
    let env = load_env();
    set_player_auras(&env, vec![aura(1001)]);
    env.exec(
        r#"
        AuditOwned = AuditContainer({{'buffs', 'HELPFUL'}})
    "#,
    )
    .unwrap();
    settle(&env);
    env.exec(
        r#"
        AuditAddon(function()
            assert(AuditOwned.AddAuraFrame == nil, 'AddAuraFrame was removed')
            assert(AuditOwned.AddAuraGroup ~= nil and AuditOwned.AddAuraSlot ~= nil)
            local button = AuditOwned:GetAuraGroupFrame('buffs', 1)
            assert(button.AddAuraFrame == nil)
            local other = CreateFrame('Frame', nil, UIParent)
            local ok, err = pcall(button.SetParent, button, other)
            assert(not ok, 'addon reparenting of a managed aura button must fail')
            assert(button:GetParent() == AuditOwned, 'parent is unchanged: ' .. tostring(err))
            ok = pcall(button.SetParent, button, nil)
            assert(not ok and button:GetParent() == AuditOwned)
        end)
    "#,
    )
    .unwrap();
    assert_no_lua_errors(&env);
}

#[test]
fn disabling_the_container_clears_every_button_until_reenabled() {
    let env = load_env();
    set_player_auras(&env, vec![aura(1001), aura(1002), debuff(2001, None)]);
    env.exec(
        r#"
        AuditToggle = AuditContainer({
            {'buffs', 'HELPFUL', {sortMethod = AuraContainerSortMethod.AuraInstanceIDOnly}},
            {'debuffs', 'HARMFUL'},
        })
    "#,
    )
    .unwrap();
    settle(&env);
    assert_eq!(icons(&env, "AuditToggle", "buffs"), "1001,1002");
    assert_eq!(icons(&env, "AuditToggle", "debuffs"), "2001");
    env.exec("AuditAddon(function() AuditToggle:SetEnabled(false) end)")
        .unwrap();
    settle(&env);
    assert_eq!(icons(&env, "AuditToggle", "buffs"), "");
    assert_eq!(icons(&env, "AuditToggle", "debuffs"), "");
    // A disabled container ignores live aura changes.
    set_player_auras(&env, vec![aura(1003)]);
    settle(&env);
    assert_eq!(icons(&env, "AuditToggle", "buffs"), "");
    env.exec("AuditAddon(function() AuditToggle:SetEnabled(true) end)")
        .unwrap();
    settle(&env);
    assert_eq!(icons(&env, "AuditToggle", "buffs"), "1003");
    assert_no_lua_errors(&env);
}

/// Mark of the Wild: flagged never-secret in the 12.1.0 spell data.
const NEVER_SECRET_SPELL: i32 = 1126;
/// Contextually secret spell.
const CONTEXTUAL_SPELL: i32 = 9072;

fn filter_fixture() -> Vec<AuraInfo> {
    let mut stealable = aura(102);
    stealable.is_stealable = true;
    stealable.is_from_player_or_player_pet = false;
    stealable.source_unit = "party1".into();
    let mut permanent = aura(103);
    permanent.duration = 0.0;
    permanent.expiration_time = 0.0;
    let mut long = aura(104);
    long.duration = 600.0;
    long.expiration_time = 600.0;
    let mut never_secret = debuff(204, None);
    never_secret.spell_id = NEVER_SECRET_SPELL;
    let mut contextual = debuff(205, None);
    contextual.spell_id = CONTEXTUAL_SPELL;
    vec![
        aura(101),
        stealable,
        permanent,
        long,
        debuff(201, Some("Magic")),
        debuff(202, Some("Curse")),
        debuff(203, None),
        never_secret,
        contextual,
    ]
}

/// (case, filter string, candidateFilters literal, expected shown icons)
const CANDIDATE_FILTER_CASES: &[(&str, &str, &str, &str)] = &[
    ("no candidate filters", "HELPFUL", "nil", "101,102,103,104"),
    ("include spell IDs", "HELPFUL", "{includeSpellIDs = {[101] = true, [104] = true}}", "101,104"),
    ("exclude spell IDs", "HELPFUL", "{excludeSpellIDs = {[101] = true}}", "102,103,104"),
    ("max duration hides longer and permanent", "HELPFUL", "{maxDuration = 60}", "101,102"),
    ("max duration is inclusive", "HELPFUL", "{maxDuration = 600}", "101,102,104"),
    ("stealable true", "HELPFUL", "{isStealable = true}", "102"),
    ("stealable false negates", "HELPFUL", "{isStealable = false}", "101,103,104"),
    ("player-or-pet false negates", "HELPFUL", "{isFromPlayerOrPlayerPet = false}", "102"),
    ("player-or-pet true", "HELPFUL", "{isFromPlayerOrPlayerPet = true}", "101,103,104"),
    ("can apply false", "HELPFUL", "{canApplyAura = false}", ""),
    ("include dispel types", "HARMFUL", "{includeDispelTypes = {Magic = true}}", "201"),
    ("exclude dispel types", "HARMFUL", "{excludeDispelTypes = {Magic = true, Curse = true}}", "203,204,205"),
    (
        "spell ID filters skip secret debuffs on friendly units",
        "HARMFUL",
        "{includeSpellIDs = {[201] = true}}",
        "201,202,203,205",
    ),
    (
        "never-secret debuffs honour exclude on any unit",
        "HARMFUL",
        "{excludeSpellIDs = {[1126] = true, [9072] = true}}",
        "201,202,203,205",
    ),
    (
        "never-secret debuffs honour include on any unit",
        "HARMFUL",
        "{includeSpellIDs = {[9072] = true}}",
        "201,202,203,205",
    ),
    ("filter string and candidates combine", "HELPFUL|PLAYER", "{isStealable = true}", ""),
];

#[test]
fn candidate_filters_select_actual_buttons() {
    let env = load_env();
    set_player_auras(&env, filter_fixture());
    for (index, (_, filter, candidates, _)) in CANDIDATE_FILTER_CASES.iter().enumerate() {
        env.exec(&format!(
            "AuditFilter{index} = AuditContainer({{{{'g', '{filter}', {{
                sortMethod = AuraContainerSortMethod.AuraInstanceIDOnly,
                candidateFilters = {candidates},
            }}}}}})"
        ))
        .unwrap();
    }
    settle(&env);
    let mismatches: Vec<String> = CANDIDATE_FILTER_CASES
        .iter()
        .enumerate()
        .filter_map(|(index, (case, _, _, expected))| {
            let actual = icons(&env, &format!("AuditFilter{index}"), "g");
            (actual != *expected).then(|| format!("{case}: expected {expected:?}, got {actual:?}"))
        })
        .collect();
    assert!(mismatches.is_empty(), "{mismatches:#?}");
    assert_no_lua_errors(&env);
}

#[test]
fn candidate_filters_can_be_replaced_after_creation() {
    let env = load_env();
    set_player_auras(&env, filter_fixture());
    env.exec(
        r#"
        AuditReplace = AuditContainer({{'g', 'HELPFUL', {sortMethod = AuraContainerSortMethod.AuraInstanceIDOnly}}})
    "#,
    )
    .unwrap();
    settle(&env);
    assert_eq!(icons(&env, "AuditReplace", "g"), "101,102,103,104");
    env.exec(
        "AuditAddon(function() AuditReplace:SetAuraGroupCandidateFilters('g', {isStealable = true}) end)",
    )
    .unwrap();
    settle(&env);
    assert_eq!(icons(&env, "AuditReplace", "g"), "102");
    env.exec("AuditAddon(function() AuditReplace:SetAuraGroupFilterString('g', 'HARMFUL') end)")
        .unwrap();
    settle(&env);
    assert_eq!(icons(&env, "AuditReplace", "g"), "");
    env.exec("AuditAddon(function() AuditReplace:SetAuraGroupCandidateFilters('g', nil) end)")
        .unwrap();
    settle(&env);
    assert_eq!(icons(&env, "AuditReplace", "g"), "201,202,203,204,205");
    assert_no_lua_errors(&env);
}

fn sort_fixture() -> Vec<AuraInfo> {
    [(101, "Charlie", 30.0), (102, "Alpha", 10.0), (103, "Bravo", 0.0)]
        .into_iter()
        .map(|(id, name, expiration)| AuraInfo {
            name: name.into(),
            duration: expiration,
            expiration_time: expiration,
            ..aura(id)
        })
        .collect()
}

/// (sort method, direction, expected order)
const SORT_CASES: &[(&str, &str, &str)] = &[
    ("NameOnly", "Normal", "102,103,101"),
    ("NameOnly", "Reverse", "101,103,102"),
    ("ExpirationOnly", "Normal", "102,101,103"),
    ("ExpirationOnly", "Reverse", "103,101,102"),
    ("AuraInstanceIDOnly", "Normal", "101,102,103"),
    ("AuraInstanceIDOnly", "Reverse", "103,102,101"),
];

#[test]
fn sort_rule_and_direction_order_actual_buttons() {
    let env = load_env();
    set_player_auras(&env, sort_fixture());
    for (index, (method, direction, _)) in SORT_CASES.iter().enumerate() {
        env.exec(&format!(
            "AuditSort{index} = AuditContainer({{{{'g', 'HELPFUL', {{
                sortMethod = AuraContainerSortMethod.{method},
                sortDirection = AuraContainerSortDirection.{direction},
            }}}}}})"
        ))
        .unwrap();
    }
    settle(&env);
    for (index, (method, direction, expected)) in SORT_CASES.iter().enumerate() {
        assert_eq!(
            icons(&env, &format!("AuditSort{index}"), "g"),
            *expected,
            "{method} {direction}"
        );
    }
    // Re-sorting an existing group reorders its buttons.
    env.exec(
        "AuditAddon(function() AuditSort0:SetAuraGroupSortMethod('g', AuraContainerSortMethod.ExpirationOnly, AuraContainerSortDirection.Reverse) end)",
    )
    .unwrap();
    settle(&env);
    assert_eq!(icons(&env, "AuditSort0", "g"), "103,101,102");
    assert_no_lua_errors(&env);
}

#[test]
fn aura_slots_show_only_the_preferred_aura() {
    let env = load_env();
    set_player_auras(&env, sort_fixture());
    env.exec(
        r#"
        AuditSlotHost = AuditContainer({})
        AuditSlot = AuditAddon(function()
            return AuditSlotHost:AddAuraSlot('soonest', 'HELPFUL', {
                sortMethod = AuraContainerSortMethod.ExpirationOnly,
                initializeFrame = AuditInitIcon,
            })
        end)
        assert(AuditSlot:GetObjectType() == 'AuraButton' and AuditSlot:GetParent() == AuditSlotHost)
        assert(AuditSlotHost:GetAuraGroupFrameCount('soonest') == 0, 'slots are not groups')
        function AuditSlotIcon()
            if not AuditIsShown(AuditSlot) then return 'hidden' end
            return tostring(AuditFrameIcon(AuditSlot))
        end
    "#,
    )
    .unwrap();
    settle(&env);
    let slot = || env.eval::<String>("return AuditSlotIcon()").unwrap();
    assert_eq!(slot(), "102");
    // Removing the preferred aura promotes the next candidate.
    set_player_auras(&env, sort_fixture().into_iter().filter(|a| a.aura_instance_id != 102).collect());
    settle(&env);
    assert_eq!(slot(), "101");
    env.exec(
        "AuditAddon(function() AuditSlotHost:SetAuraSlotSortMethod('soonest', AuraContainerSortMethod.NameOnly, AuraContainerSortDirection.Normal) end)",
    )
    .unwrap();
    settle(&env);
    assert_eq!(slot(), "103");
    env.exec(
        "AuditAddon(function() AuditSlotHost:SetAuraSlotCandidateFilters('soonest', {includeSpellIDs = {[101] = true}}) end)",
    )
    .unwrap();
    settle(&env);
    assert_eq!(slot(), "101");
    env.exec("AuditAddon(function() AuditSlotHost:SetAuraSlotFilterString('soonest', 'HARMFUL') end)")
        .unwrap();
    settle(&env);
    assert_eq!(slot(), "hidden");
    assert_no_lua_errors(&env);
}

#[test]
fn private_auras_are_shown_filtered_and_sorted_like_public_auras() {
    let env = load_env();
    let timed = |id: i32, expiration: f64| AuraInfo {
        duration: expiration,
        expiration_time: expiration,
        ..aura(id)
    };
    set_player_auras(&env, vec![timed(102, 10.0), timed(104, 40.0)]);
    env.exec(
        r#"
        local function private(id, helpful, expiration)
            return {
                auraInstanceID = id, spellId = id, icon = id, name = 'Private ' .. id,
                isHelpful = helpful, isHarmful = not helpful, sourceUnit = 'player',
                isFromPlayerOrPlayerPet = true, canApplyAura = helpful, applications = 1,
                duration = expiration, expirationTime = expiration,
            }
        end
        local state = C_UnitAurasPrivate._state
        state.privateAurasByUnit.player = {{auraInstanceID = 103}, {auraInstanceID = 105}}
        state.auraDataByUnit.player = {[103] = private(103, true, 20), [105] = private(105, false, 30)}
        AuditPrivate = AuditContainer({
            {'buffs', 'HELPFUL', {sortMethod = AuraContainerSortMethod.ExpirationOnly}},
            {'debuffs', 'HARMFUL', {sortMethod = AuraContainerSortMethod.ExpirationOnly}},
        })
    "#,
    )
    .unwrap();
    settle(&env);
    assert_eq!(icons(&env, "AuditPrivate", "buffs"), "102,103,104");
    assert_eq!(icons(&env, "AuditPrivate", "debuffs"), "105");
    assert_no_lua_errors(&env);
}

#[test]
fn target_frame_displays_target_auras_through_its_managed_container() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_AuraContainer", "Blizzard_UnitFrame"],
        &[],
    );
    let shown_target_buttons = || {
        env.eval::<String>(
            r#"
            local container = TargetFrame:GetAuraContainer()
            local shown = 0
            for _, child in ipairs({container:GetChildren()}) do
                local isShown = child:IsShown()
                if issecretvalue(isShown) then isShown = secretunwrap(isShown) end
                if child:GetObjectType() == 'AuraButton' and isShown then shown = shown + 1 end
            end
            return tostring(container:GetUnit()) .. ':' .. shown
        "#,
        )
        .unwrap()
    };
    env.exec("TargetUnit('enemy1')").unwrap();
    for _ in 0..3 {
        settle(&env);
    }
    let fixture_count = env.state().borrow().target_auras.len();
    assert_eq!(shown_target_buttons(), format!("target:{fixture_count}"));
    env.state()
        .borrow_mut()
        .target_auras
        .push(debuff(3001, Some("Poison")));
    env.fire_unit_aura_full_update("target").unwrap();
    settle(&env);
    assert_eq!(
        shown_target_buttons(),
        format!("target:{}", fixture_count + 1)
    );
}
