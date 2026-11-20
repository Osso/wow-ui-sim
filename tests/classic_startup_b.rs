//! Original cached Classic consumers. Staged source only; no native parity claim.
#![cfg(any(feature = "client-era", feature = "client-anniversary"))]
use sha2::{Digest, Sha256};
use wow_ui_sim::loader::{discover_blizzard_startup_addons_for_screen, load_startup_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn ui() -> std::path::PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("active Classic cache")
}
fn original(file: &str) -> String {
    let bytes = std::fs::read(ui().join(file)).expect(file);
    let expected = include_str!("classic_startup_b_pins.tsv")
        .lines()
        .find_map(|line| line.split_once('\t').filter(|(name, _)| *name == file))
        .expect("pinned consumer")
        .1;
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        expected,
        "changed {file}"
    );
    String::from_utf8(bytes).unwrap()
}
fn env(files: &[&str]) -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec("ClassicBErrors = {}; seterrorhandler(function(e) ClassicBErrors[#ClassicBErrors+1] = tostring(e) end)").unwrap();
    for file in files {
        env.loader_env().exec(&original(file)).unwrap();
    }
    env
}
fn run(env: &WowLuaEnv, source: &str) {
    let result = env.exec(source);
    env.exec("assert(#ClassicBErrors == 0, table.concat(ClassicBErrors, '\\n'))")
        .unwrap();
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "{:?}",
        env.state().borrow().lua_errors
    );
    result.unwrap();
}
// These are byte-exact statements from pinned consumers, not rewritten algorithms.
// Isolate a publication boundary without replacing the API or downstream renderer.
fn statements(file: &str, start: &str, end: &str) -> String {
    let source = original(file);
    let begin = source.find(start).unwrap();
    let finish = source[begin..].find(end).unwrap() + begin;
    source[begin..finish].to_owned()
}

#[test]
fn classic_b_r2_original_quest_count_statements() {
    assert_eq!(wow_ui_sim::global_strings::GLOBAL_STRINGS.len(), 19_624);
    assert_eq!(
        wow_ui_sim::global_strings::get_global_string("OPTION_TOOLTIP_HD_TEXTURES"),
        None,
        "absent shipped tooltip must not be invented or borrowed"
    );
    let env = env(&[]);
    run(
        &env,
        "QuestLogQuestCount = UIParent:CreateFontString(); QuestLogCountMiddle = UIParent:CreateTexture(); MAX_QUESTLOG_QUESTS = 20",
    );
    let body = statements(
        "Blizzard_UIPanels_Game/Vanilla/QuestLogFrame.lua",
        "QuestLogQuestCount:SetText(format(QUEST_LOG_COUNT_TEMPLATE",
        "-- ScrollFrame update",
    );
    run(&env, &format!("local numQuests = 3; {body}"));
    run(
        &env,
        "assert(QuestLogQuestCount:GetText() == 'Quests: |cffffffff3/20|r')",
    );
    // Restore-missing must use the same selected dataset, without overwriting vendor writes.
    env.exec("QUEST_LOG_COUNT_TEMPLATE = nil").unwrap();
    // Public post-load lifecycle invokes the real missing-string restoration.
    env.apply_post_load_workarounds();
    run(&env, &format!("local numQuests = 0; {body}"));
    run(
        &env,
        "assert(QuestLogQuestCount:GetText() == 'Quests: |cffffffff0/20|r'); QUEST_LOG_COUNT_TEMPLATE = 'owned %d/%d'",
    );
    env.apply_post_load_workarounds();
    run(&env, "assert(QUEST_LOG_COUNT_TEMPLATE == 'owned %d/%d')");
}

// Source-backed labels are supplied by the selected shipped Classic string dataset.
#[test]
fn classic_b_r2_original_microbutton_label_from_shipped_data() {
    let env = env(&[]);
    let definition = statements(
        "Blizzard_MicroMenu/Classic/MainMenuBarMicroButtons.lua",
        "function MicroButtonTooltipText(",
        "function MainMenuMicroButton_Init(",
    );
    run(&env, &definition);
    run(
        &env,
        "SetBinding('I', 'TOGGLELFGPARENT'); local text = MicroButtonTooltipText(LFG_BUTTON, 'TOGGLELFGPARENT'); assert(type(text) == 'string' and text:find(LFG_BUTTON, 1, true))",
    );
}
fn validate_original_setting_label(expression: &str, variable: &str, kind: &str, default: &str) {
    let env = env(&[]);
    // Appending the call keeps the original local validator in its own lexical
    // scope. No test replacement of Settings validation or the missing label.
    let source = original("Blizzard_Settings_Shared/Blizzard_Setting.lua");
    run(
        &env,
        &format!(
            "{source}\nErrorIfInvalidSettingArguments({expression}, '{variable}', '{kind}', {default}); assert(type({expression}) == 'string' and #{expression} > 0)"
        ),
    );
}
#[test]
fn classic_b_r2_original_hd_setting_label_from_shipped_data() {
    validate_original_setting_label(
        "OPTION_HD_TEXTURES",
        "PROXY_HIGH_RES_TEXTURES",
        "boolean",
        "true",
    );
}
#[test]
fn classic_b_r2_original_nameplate_setting_label_from_shipped_data() {
    validate_original_setting_label(
        "UNIT_NAMEPLATES_CLASS_COLOR",
        "UNIT_NAMEPLATES_CLASS_COLOR",
        "number",
        "0",
    );
}

#[test]
fn classic_b_r5_required_lod_dependency_fixture_remains_loadable() {
    let root = tempfile::tempdir().unwrap();
    for (name, metadata, script) in [
        (
            "Blizzard_Game",
            "## Dependencies: Blizzard_ClassicBRequired\n",
            "assert(classicBRequiredLoaded); classicBRootLoaded = true",
        ),
        (
            "Blizzard_ClassicBRequired",
            "## LoadOnDemand: 1\n",
            "classicBRequiredLoaded = true",
        ),
    ] {
        let dir = root.path().join(name);
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{name}.toc")),
            format!("## Interface: 11509\n{metadata}Test.lua\n"),
        )
        .unwrap();
        std::fs::write(dir.join("Test.lua"), script).unwrap();
    }
    let addons = discover_blizzard_startup_addons_for_screen(root.path(), ScreenKind::Game);
    let names: Vec<_> = addons.iter().map(|addon| addon.name.as_str()).collect();
    assert_eq!(names, ["Blizzard_ClassicBRequired", "Blizzard_Game"]);
    let env = env(&[]);
    env.state().borrow_mut().addon_base_paths = vec![root.path().to_owned()];
    for addon in addons {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None).unwrap();
    }
    run(
        &env,
        "assert(classicBRequiredLoaded and classicBRootLoaded)",
    );
}

#[test]
fn classic_b_r5_original_startup_keeps_transmog_lod_and_hard_dependencies() {
    let addons = discover_blizzard_startup_addons_for_screen(&ui(), ScreenKind::Game);
    assert!(!addons.iter().any(|addon| addon.name == "Blizzard_Transmog"));
    let index = |name| {
        addons
            .iter()
            .position(|addon| addon.name == name)
            .expect(name)
    };
    assert!(index("Blizzard_MoneyFrame") < index("Blizzard_UIPanels_Game"));
    let env = env(&[]);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for addon in &addons {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None).unwrap();
    }
    let errors = env.state().borrow().lua_errors.clone();
    assert!(
        !errors
            .iter()
            .any(|error| error.contains("Blizzard_Transmog")),
        "{errors:?}"
    );
    // Other startup roots remain eligible; their independent errors are not erased.
    run_targeted(
        &env,
        "assert(not C_AddOns.IsAddOnLoaded('Blizzard_Transmog')); assert(C_AddOns.IsAddOnLoaded('Blizzard_UIPanels_Game'))",
    );
}
fn run_targeted(env: &WowLuaEnv, source: &str) {
    env.exec(source).unwrap();
}

#[test]
fn classic_b_r7_original_product_choice_empty_alerts() {
    let env = env(&[
        "Blizzard_SharedXML/Classic/SharedUIPanelTemplates.lua",
        "Blizzard_UIPanels_Game/Classic/ProductChoice.lua",
    ]);
    run(
        &env,
        r#"
        local f = CreateFrame('Frame')
        f.TitleText = f:CreateFontString()
        local first, second = C_ProductChoice.GetChoices(), C_ProductChoice.GetChoices()
        assert(type(first) == 'table' and type(second) == 'table' and first ~= second)
        first[1] = 'caller-owned'
        assert(#second == 0 and #C_ProductChoice.GetChoices() == 0)
        ProductChoiceFrame_OnLoad(f)
        assert(f.selectedPageNum == 1 and f.TitleText:GetText() == RECRUIT_A_FRIEND_REWARDS)
        for _, event in ipairs({'PLAYER_LOGIN', 'PRODUCT_CHOICE_UPDATE'}) do
            ProductChoiceFrame_OnEvent(f, event)
            assert(f.mainAlertFrame == nil and f.secondAlertFrame == nil)
        end
    "#,
    );
}

#[test]
fn classic_b_r8_original_loot_history_empty_recycles_real_frames() {
    let env = env(&["Blizzard_FrameXML/Classic/LootHistory.lua"]);
    run(
        &env,
        r#"
        local f = CreateFrame('Frame')
        LootHistoryFrame_OnLoad(f)
        assert(#f.itemFrames == 0 and #f.usedPlayerFrames == 0)
        local row, player = CreateFrame('Button'), CreateFrame('Button')
        row:Show(); player:Show(); player.itemIdx = 1; player.playerIdx = 1
        f.itemFrames[1] = row; f.usedPlayerFrames[1] = player
        LootHistoryFrame_OnEvent(f, 'LOOT_HISTORY_FULL_UPDATE')
        assert(not row:IsShown() and not player:IsShown())
        assert(#f.usedPlayerFrames == 0 and f.unusedPlayerFrames[1] == player)
        assert(player.itemIdx == nil and player.playerIdx == nil)
    "#,
    );
}

#[test]
fn classic_b_r9_original_tracking_load_and_update_hide_empty_icon() {
    let env = env(&["Blizzard_Minimap/Classic/MinimapTracking_Simple.lua"]);
    run(
        &env,
        r#"
        MiniMapTracking = CreateFrame('Frame')
        MiniMapTrackingIcon = MiniMapTracking:CreateTexture()
        Mixin(MiniMapTracking, MinimapTrackingSimpleMixin)
        MiniMapTracking:Show(); MiniMapTracking:OnLoad()
        assert(not MiniMapTracking:IsShown())
        MiniMapTracking:Show(); MiniMapTracking:OnEvent('MINIMAP_UPDATE_TRACKING')
        assert(not MiniMapTracking:IsShown())
    "#,
    );
}

#[test]
fn classic_b_r11_original_guild_initialization_and_row_selection_statements() {
    let env = env(&[]);
    let init = statements(
        "Blizzard_UIPanels_Game/Classic/FriendsFrame.lua",
        "GuildFrame.notesToggle = 1;",
        "local currentGuildMOTD",
    );
    let click = statements(
        "Blizzard_UIPanels_Game/Classic/FriendsFrame.lua",
        "GuildFrame.previousSelectedGuildMember = GuildFrame.selectedGuildMember;",
        "-- Toggle guild details frame",
    );
    run(
        &env,
        "GuildFrame = CreateFrame('Frame'); GuildRowB = CreateFrame('Button', 'GuildRowB'); GuildRowB:CreateFontString('GuildRowBName'):SetText('Second')",
    );
    run(&env, &init);
    run(&env, "assert(GetGuildRosterSelection() == 0)");
    use wow_ui_sim::lua_api::state::GuildMember;
    env.state().borrow_mut().world.guild_members = vec![
        GuildMember {
            name: "First".into(),
            guid: None,
            rank_index: 1,
            online: true,
        },
        GuildMember {
            name: "Second".into(),
            guid: None,
            rank_index: 2,
            online: true,
        },
    ];
    run(
        &env,
        &format!("local self = GuildRowB; self.guildIndex = 2; {click}"),
    );
    run(
        &env,
        "assert(GetGuildRosterSelection() == 2 and GetGuildRosterInfo(GetGuildRosterSelection()) == 'Second'); assert(GuildFrame.selectedName == 'Second')",
    );
    let selected = statements(
        "Blizzard_UIPanels_Game/Classic/FriendsFrame.lua",
        "fullName, rank, rankIndex, level, class, zone, note, officernote, online = GetGuildRosterInfo(GetGuildRosterSelection());",
        "-- If there's a selected guildmember",
    );
    let details = statements(
        "Blizzard_UIPanels_Game/Classic/FriendsFrame.lua",
        "local displayedName = Ambiguate(fullName, \"guild\");",
        "if ( online ) then",
    );
    run(
        &env,
        "for _, name in ipairs({'Name','Level','ZoneText','RankText'}) do GuildFrame:CreateFontString('GuildMemberDetail'..name) end",
    );
    run(
        &env,
        &format!(
            "local fullName, rank, rankIndex, level, class, zone, note, officernote, online; {selected}\n{details}"
        ),
    );
    run(
        &env,
        "assert(GuildMemberDetailName:GetText() == 'Second'); local _, rank = GetGuildRosterInfo(2); assert(GuildMemberDetailRankText:GetText() == rank)",
    );
    env.state().borrow_mut().world.guild_members.truncate(1);
    run(&env, "assert(GetGuildRosterSelection() == 0)");
    run(&env, &init);
    run(
        &env,
        "assert(GetGuildRosterSelection() == 0); SetGuildRosterSelection(-1); assert(GetGuildRosterSelection() == 0); SetGuildRosterSelection(3); assert(GetGuildRosterSelection() == 0)",
    );
}

#[test]
fn classic_b_r15_original_combat_log_toc_and_secure_isolation() {
    let env = env(&[]);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for file in [
        "Blizzard_FrameXMLBase/Classic/Constants.lua",
        "Blizzard_CombatLogBase/Shared/CombatLogFilters.lua",
        "Blizzard_CombatLogBase/Vanilla/CombatLogColors.lua",
        "Blizzard_CombatLogBase/Blizzard_CombatLogBase.toc",
    ] {
        original(file);
    }
    let addons = discover_blizzard_startup_addons_for_screen(&ui(), ScreenKind::Game);
    let last = addons
        .iter()
        .position(|addon| addon.name == "Blizzard_CombatLogBase")
        .unwrap();
    for addon in &addons[..=last] {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None).unwrap();
    }
    let errors = env.state().borrow().lua_errors.clone();
    assert!(
        !errors
            .iter()
            .any(|error| error.contains("Blizzard_CombatLogBase")),
        "{errors:?}"
    );
    run_targeted(
        &env,
        r#"
        local c = COMBATLOG_DEFAULT_COLORS.schoolColoring
        assert(c[0].a == 1 and c[0].r == 1 and c[0].g == 1 and c[0].b == 1)
        assert(c[SCHOOL_MASK_FIRE].r == 1 and c[SCHOOL_MASK_FIRE].g == .5 and c[SCHOOL_MASK_FIRE].b == 0)
        assert(COMBATLOG_DEFAULT_COLORS.unitColoring[COMBATLOG_FILTER_MINE].r == .7)
        classicBPublicOnly = 71
        assert(rawget(__secureenv, 'classicBPublicOnly') == nil)
    "#,
    );
    env.exec_rilua_secure("classicBSecureOnly = 19; assert(classicBPublicOnly == nil)")
        .unwrap();
    run_targeted(
        &env,
        "assert(rawget(_G, 'classicBSecureOnly') == nil and __secureenv.classicBSecureOnly == 19)",
    );
}

#[test]
fn classic_b_r17_original_quest_material_uses_supplied_map() {
    let env = env(&["Blizzard_UIPanels_Game/Classic/QuestFrame.lua"]);
    run(&env, "assert(QuestFrame_GetMaterial() == 'Parchment')");
    {
        let mut state = env.state().borrow_mut();
        state.selected_quest_log_id = Some(80002);
        state
            .quest_log_entries
            .background_materials
            .insert(80002, "Bronze".into());
    }
    run(&env, "assert(QuestFrame_GetMaterial() == 'Bronze')");
}

#[test]
fn classic_b_r18_original_reputation_bar_has_no_retail_watch_fixture() {
    let env = env(&[]);
    original("Blizzard_ActionBar/Shared/ReputationBar.lua");
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for addon in discover_blizzard_startup_addons_for_screen(&ui(), ScreenKind::Game) {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None).unwrap();
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    run_targeted(
        &env,
        r#"
        local bar = CreateFrame('Frame', nil, UIParent, 'ReputationStatusBarTemplate')
        assert(bar.Update == ReputationStatusBarMixin.Update)
        bar:Update()
        assert(C_Reputation.GetWatchedFactionData() == nil)
        assert(C_Reputation.GetNumFactions() == 0)
        assert(REPUTATION_STATUS_BAR_LABEL_ACCOUNT_WIDE == nil)
    "#,
    );
    let errors: Vec<_> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|error| error.contains("ReputationBar.lua"))
        .cloned()
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    // A pre-existing supplied snapshot is not replaced by profile seeding.
    run_targeted(
        &env,
        r#"
        -- Existing query closures own this table; supply its fields in place,
        -- rather than replacing the debug global with an unrelated table.
        ClassicBSuppliedReputation = __wow_reputation_state
        local state = ClassicBSuppliedReputation
        state.selectedFaction=0; state.watchedFactionID=72; state.sortType=0; state.showLegacy=true
        state.factions={{factionID=72, name='Stormwind', reaction=5,
            standing=1000, bottom=0, top=3000, isHeader=false,
            currentStanding=1000, currentReactionThreshold=0, nextReactionThreshold=3000,
            isAccountWide=false, isLegacy=true}}
    "#,
    );
    env.apply_post_load_workarounds();
    run_targeted(
        &env,
        r#"
        assert(__wow_reputation_state == ClassicBSuppliedReputation)
        local data = C_Reputation.GetWatchedFactionData()
        assert(data.factionID == 72 and data.name == 'Stormwind')
        assert(data.currentStanding == 1000 and not data.isAccountWide)
        assert(C_Reputation.GetNumFactions() == 1)
    "#,
    );
}

#[test]
fn classic_b_r19_original_vanilla_rewards_skip_tbc_only_children() {
    let env = env(&[]);
    original("Blizzard_UIPanels_Game/Classic/QuestFrame.lua");
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for addon in discover_blizzard_startup_addons_for_screen(&ui(), ScreenKind::Game) {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None).unwrap();
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    run_targeted(
        &env,
        r#"
        QuestLog_Update()
        assert(GetClassicExpansionLevel() == LE_EXPANSION_CLASSIC)
        assert(not ClassicExpansionAtLeast(LE_EXPANSION_BURNING_CRUSADE))
        assert(QuestLogHonorFrame == nil and QuestLogPlayerTitleFrame == nil)
    "#,
    );
    let errors: Vec<_> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|error| error.contains("QuestFrame.lua:549"))
        .cloned()
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn classic_b_r20_original_unit_load_and_feedback_use_empty_health_knowledge() {
    let env = env(&[]);
    {
        let mut state = env.state().borrow_mut();
        state.player.health = 250;
        state.player.health_max = 500;
    }
    for file in [
        "Blizzard_UnitFrame/Classic/UnitFrame.lua",
        "Blizzard_UnitFrame/Classic/PlayerFrame.lua",
        "Blizzard_FrameXML/Classic/CombatFeedback.lua",
    ] {
        original(file);
    }
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui()];
    wow_ui_sim::xml::register_intrinsic_templates();
    for addon in discover_blizzard_startup_addons_for_screen(&ui(), ScreenKind::Game) {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None).unwrap();
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    // Invoke the real XML OnLoad and pooled party UpdateMember independently:
    // a captured earlier startup error must not hide either caller boundary.
    env.exec(r#"
        local bar = OverrideActionBar.healthBar
        R20OverrideOK, R20OverrideError = pcall(bar:GetScript('OnLoad'), bar)
        R20PartyOK, R20PartyError = pcall(PartyFrame.MemberFrame1.UpdateMember, PartyFrame.MemberFrame1)
    "#).unwrap();
    env.fire_on_update(0.05).unwrap();
    let consumer_result = env.exec(
        r#"
        assert(R20OverrideOK and R20PartyOK,
            'OverrideActionBar OnLoad: '..tostring(R20OverrideError)..
            '\nparty UpdateMember: '..tostring(R20PartyError))
    "#,
    );
    eprintln!("original consumer result: {consumer_result:?}");
    let errors: Vec<_> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|error| {
            error.contains("UnitFrame.lua:737") || error.contains("CombatFeedback.lua:104")
        })
        .cloned()
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    consumer_result.unwrap();
    run_targeted(
        &env,
        r#"
        for _, unit in ipairs({'player', 'pet', 'target', 'party1'}) do
            assert(ShouldKnowUnitHealth(unit) == false)
        end
        assert(PlayerFrame.feedbackText == PlayerHitIndicator)
        assert(PetFrame.feedbackText == PetHitIndicator)
        local health = PlayerFrame_GetHealthBar()
        health.forceShow = true
        SetCVar('statusTextDisplay', 'NUMERIC')
        UnitFrameHealthBar_Update(health, 'player')
        assert(health.showPercentage and not health.forceHideText)
        local text = assert(health.TextString:GetText())
        assert(text:find('50%', 1, true), text)
        assert(not text:find('250', 1, true) and not text:find('500', 1, true), text)
        CombatFeedback_OnUpdate(PlayerFrame, .05)
        CombatFeedback_OnUpdate(PetFrame, .05)
    "#,
    );
    env.state()
        .borrow_mut()
        .unit_health_knowledge
        .insert("player".into(), true);
    run_targeted(
        &env,
        r#"
        UnitFrameHealthBar_Update(PlayerFrame_GetHealthBar(), 'player')
        local health = PlayerFrame_GetHealthBar()
        assert(not health.showPercentage)
        local text = assert(health.TextString:GetText())
        assert(text:find('250', 1, true) and text:find('500', 1, true), text)
        assert(not text:find('%', 1, true), text)
    "#,
    );
}
