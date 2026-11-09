use super::{blizzard_ui_dir, new_game_env};
use wow_ui_sim::loader::{discover_blizzard_addon_closure_for_screen, load_addon};
use wow_ui_sim::lua_api::{WowLuaEnv, state::AuraInfo};
use wow_ui_sim::screen::ScreenKind;

fn load_aura_container_ui() -> WowLuaEnv {
    let env = new_game_env();
    let roots = ["Blizzard_FrameXMLUtil", "Blizzard_AuraContainer"];
    for (name, toc_path) in
        discover_blizzard_addon_closure_for_screen(&blizzard_ui_dir(), ScreenKind::Game, &roots)
    {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("[load {name}] FAILED: {error}"));
    }
    env
}

fn run_secure_probe(env: &WowLuaEnv, code: &str) -> String {
    let wrapped = format!("AuraContainerTestResult = (function() {code} end)()");
    env.exec_rilua_secure(&wrapped)
        .expect("secure AuraContainer probe should evaluate");
    env.eval("return __secureenv.AuraContainerTestResult")
        .expect("secure AuraContainer probe result should be readable")
}

#[test]
fn aura_container_selects_public_private_and_edit_mode_partitions() {
    let env = load_aura_container_ui();
    let result = run_secure_probe(
        &env,
        r#"
            local container = CreateFromMixins(ManagedAuraContainerPrivateMixin)
            container.editModePreviewEnabled = true
            container.useEditModeSource = false
            container.ShouldIncludePrivateAuraSource = function() return false end

            local publicOnly = container:GetAuraSources()
            if publicOnly ~= AuraContainerAuraSourceLists.PublicOnly then return "public-list" end
            if #publicOnly ~= 1 or publicOnly[1] ~= AuraContainerPublicAuraSource then
                return "public-members"
            end

            container.ShouldIncludePrivateAuraSource = function() return true end
            local publicAndPrivate = container:GetAuraSources()
            if publicAndPrivate ~= AuraContainerAuraSourceLists.PublicAndPrivate then
                return "private-list"
            end
            if #publicAndPrivate ~= 2
                or publicAndPrivate[1] ~= AuraContainerPublicAuraSource
                or publicAndPrivate[2] ~= AuraContainerPrivateAuraSource then
                return "private-members"
            end

            container.useEditModeSource = true
            local editMode = container:GetAuraSources()
            if editMode ~= AuraContainerAuraSourceLists.EditMode then return "edit-list" end
            if #editMode ~= 1 or editMode[1] ~= AuraContainerEditModeAuraSource then
                return "edit-members"
            end
            return "ok"
            "#,
    );

    assert_eq!(result, "ok");
}

#[test]
fn aura_container_group_assigns_and_releases_frames_by_aura_instance() {
    let env = load_aura_container_ui();
    let result = run_secure_probe(
        &env,
        r#"
            local nextFrameToken = 0
            local releasedTokens = {}
            local provider = {
                AcquireFrame = function()
                    nextFrameToken = nextFrameToken + 1
                    return { token = nextFrameToken }
                end,
                ReleaseFrame = function(_, frame)
                    table.insert(releasedTokens, frame.token)
                end,
            }

            local owner = CreateFromMixins(AuraContainerAuraGroupOwnerMixin)
            owner.ShouldIncludeAuraInGroup = function() return true end
            owner.InitializeAuraGroupFrame = function(_, _, frame, _, auraData)
                frame.assignedAuraInstanceID = auraData.auraInstanceID
            end
            local manager = AuraContainerUtil.CreateAuraGroupManager(owner)
            local group = manager:RegisterAuraGroup("test", {
                filterString = "HELPFUL",
                frameProvider = provider,
                maxFrameCount = 2,
                auraComparator = function(a, b)
                    return a.auraInstanceID < b.auraInstanceID
                end,
            })

            manager:AddAura("player", { auraInstanceID = 22 })
            manager:AddAura("player", { auraInstanceID = 11 })
            manager:RefreshFrameAssignments("player")

            local firstFrame = group:GetFramesByAura()[11]
            local secondFrame = group:GetFramesByAura()[22]
            if firstFrame == nil or firstFrame.token ~= 1 or firstFrame.assignedAuraInstanceID ~= 11 then
                return "initial-first-ownership"
            end
            if secondFrame == nil or secondFrame.token ~= 2 or secondFrame.assignedAuraInstanceID ~= 22 then
                return "initial-second-ownership"
            end

            manager:RemoveAura("player", 11)
            manager:RefreshFrameAssignments("player")
            if releasedTokens[1] ~= 1 then return "released-removed-frame" end
            if group:GetFramesByAura()[11] ~= nil then return "removed-map-entry" end
            if group:GetFramesByAura()[22] ~= secondFrame then return "retained-frame" end

            manager:AddAura("player", { auraInstanceID = 11 })
            manager:RefreshFrameAssignments("player")
            local reboundFrame = group:GetFramesByAura()[11]
            if reboundFrame == nil or reboundFrame.token ~= 3 then return "rebound-frame" end
            if group:GetFramesByAura()[22] ~= secondFrame then return "rebound-retained-frame" end
            return "ok"
            "#,
    );

    assert_eq!(result, "ok");
}

#[test]
fn aura_container_filters_helpful_harmful_and_player_auras() {
    let env = load_aura_container_ui();
    seed_filter_auras(&env);

    let result = run_secure_probe(
        &env,
        r#"
            local function includeAura(unit, filterString, auraData)
                local owner = CreateFromMixins(AuraContainerAuraGroupOwnerMixin)
                local manager = AuraContainerUtil.CreateAuraGroupManager(owner)
                local group = manager:RegisterAuraGroup("test", {
                    filterString = filterString,
                    frameProvider = {},
                })
                manager:AddAura(unit, auraData)
                return group:GetAuras()[auraData.auraInstanceID] ~= nil
            end

            local playerAuras = C_UnitAuras.GetUnitAuras("player", "HELPFUL")
            local playerAura = playerAuras[1]
            local partyAura = playerAuras[2]
            local targetAura = C_UnitAuras.GetUnitAuras("target", "HARMFUL")[1]
            if playerAura == nil or partyAura == nil or targetAura == nil then
                return "fixture"
            end

            if not includeAura("player", "HELPFUL", playerAura) then return "helpful-excluded" end
            if includeAura("player", "HARMFUL", playerAura) then return "helpful-in-harmful" end
            if not includeAura("target", "HARMFUL", targetAura) then return "harmful-excluded" end
            if includeAura("target", "HELPFUL", targetAura) then return "harmful-in-helpful" end
            if not includeAura("player", "HELPFUL|PLAYER", playerAura) then return "player-excluded" end
            if includeAura("player", "HELPFUL|PLAYER", partyAura) then return "party-in-player" end
            return "ok"
            "#,
    );

    assert_eq!(result, "ok");
}

#[test]
fn aura_container_honors_configured_sort_order() {
    let env = load_aura_container_ui();
    let result = run_secure_probe(
        &env,
        r#"
            local owner = CreateFromMixins(AuraContainerAuraGroupOwnerMixin)
            owner.ShouldIncludeAuraInGroup = function() return true end
            local manager = AuraContainerUtil.CreateAuraGroupManager(owner)
            local group = manager:RegisterAuraGroup("test", {
                filterString = "HELPFUL",
                frameProvider = {},
                auraComparator = function(a, b)
                    return a.auraInstanceID < b.auraInstanceID
                end,
            })

            local auras = {
                { auraInstanceID = 40, sourceUnit = "target", isPriorityAura = false, canApplyAura = false },
                { auraInstanceID = 42, sourceUnit = "target", isPriorityAura = true, canApplyAura = false },
                { auraInstanceID = 41, sourceUnit = "player", isPriorityAura = false, canApplyAura = false },
                { auraInstanceID = 43, sourceUnit = "target", isPriorityAura = false, canApplyAura = true },
            }
            for _, auraData in ipairs(auras) do
                manager:AddAura("player", auraData)
            end

            local ordered = {}
            group:GetAuras():Iterate(function(auraInstanceID)
                table.insert(ordered, tostring(auraInstanceID))
            end)
            return table.concat(ordered, ",")
            "#,
    );

    assert_eq!(result, "40,41,42,43");
}

fn seed_filter_auras(env: &WowLuaEnv) {
    env.exec("TargetUnit('party1')")
        .expect("select the unit with supplied target auras");

    let mut state = env.state().borrow_mut();
    state.player.buffs = vec![
        aura_info("Player Helpful Aura", "player", true, true, 101),
        aura_info("Party Helpful Aura", "party1", true, false, 102),
    ];
    state.target_auras = vec![aura_info(
        "Target Harmful Aura",
        "target",
        false,
        false,
        201,
    )];
}

fn aura_info(
    name: &str,
    source_unit: &str,
    is_helpful: bool,
    is_from_player: bool,
    id: i32,
) -> AuraInfo {
    AuraInfo {
        name: name.into(),
        spell_id: id,
        icon: id,
        duration: 30.0,
        expiration_time: 30.0,
        applications: 1,
        source_unit: source_unit.into(),
        is_helpful,
        is_raid: true,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: is_from_player,
        is_from_player_or_player_pet: is_from_player,
        dispel_type: None,
        aura_instance_id: id,
    }
}
