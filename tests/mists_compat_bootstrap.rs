#![cfg(feature = "client-mists")]

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;
use wow_ui_sim::xml::{FrameXml, XmlElement};

type MistsStartupApiShape = (
    i32,
    i32,
    String,
    i32,
    i32,
    bool,
    i32,
    i32,
    i32,
    i32,
    i32,
    String,
);

fn blizzard_ui_dir() -> std::path::PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn mists_lua_source(relative_path: &str) -> String {
    std::fs::read_to_string(blizzard_ui_dir().join(relative_path)).unwrap_or_else(|error| {
        panic!("Mists Lua source {relative_path} should be readable: {error}")
    })
}

fn mists_money_input_frame_xml_path() -> std::path::PathBuf {
    blizzard_ui_dir().join("Blizzard_MoneyFrame/Classic/MoneyInputFrame.xml")
}

fn mists_blizzard_toc(addon: &str, toc_name: &str) -> std::path::PathBuf {
    blizzard_ui_dir().join(addon).join(toc_name)
}

fn load_mists_money_frame_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    let fonts_toc = mists_blizzard_toc("Blizzard_Fonts_Shared", "Blizzard_Fonts_Shared.toc");
    load_addon(&env.loader_env(), &fonts_toc).expect("Blizzard_Fonts_Shared should load");

    let toc_path = mists_blizzard_toc("Blizzard_MoneyFrame", "Blizzard_MoneyFrame_Classic.toc");
    load_addon(&env.loader_env(), &toc_path).expect("Blizzard_MoneyFrame should load");
    env
}

fn find_top_level_frame<'a>(elements: &'a [XmlElement], name: &str) -> &'a FrameXml {
    elements
        .iter()
        .find_map(|element| match element {
            XmlElement::Frame(frame) if frame.name.as_deref() == Some(name) => Some(frame),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected top-level frame template {name}"))
}

#[test]
fn mists_bootstrap_reports_pandaria_as_the_current_classic_expansion() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (i32, bool, bool, bool, bool) = env
        .eval(
            r#"
            return GetExpansionLevel(),
                ClassicExpansionAtLeast(LE_EXPANSION_MISTS_OF_PANDARIA),
                ClassicExpansionAtMost(LE_EXPANSION_MISTS_OF_PANDARIA),
                ClassicExpansionAtLeast(5),
                ClassicExpansionAtMost(LE_EXPANSION_CATACLYSM)
            "#,
        )
        .expect("Mists expansion helpers should be callable");

    assert_eq!(
        result,
        (4, true, true, false, false),
        "Mists Classic should report MoP as the current classic expansion"
    );
}

#[test]
fn mists_bootstrap_exposes_classic_addon_compatibility_globals() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let build_info: (String, String, String, i32) = env
        .eval(
            r#"
            local version, build, date, interface = GetBuildInfo()
            return version, build, date, interface
            "#,
        )
        .expect("Mists build info should be callable");

    assert_eq!(
        build_info,
        (
            "5.5.4".to_string(),
            "68042".to_string(),
            "Jun 2 2026".to_string(),
            50504,
        ),
        "Mists build info should match the pinned 5.5.4 Blizzard UI source"
    );

    let result: (bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            return GetRaidDifficultyID() == 14,
                GetLegacyRaidDifficultyID() == 3,
                type(GetTimePreciseSec()) == "number",
                Enum.ItemQuality.Good == Enum.ItemQuality.Uncommon,
                type(time({ year = "2026", month = "5", day = "16", hour = "12", min = "30", sec = "45" })) == "number"
            "#,
        )
        .expect("Mists addon compatibility globals should be callable");

    assert_eq!(
        result,
        (true, true, true, true, true,),
        "Mists should expose classic-era globals used by installed addons"
    );
}

#[test]
fn mists_bootstrap_exposes_raid_marker_system_probe() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let enabled: bool = env
        .eval("return IsRaidMarkerSystemEnabled()")
        .expect("Mists raid marker system probe should be callable");

    assert!(
        !enabled,
        "headless simulator should leave raid marker placement disabled by default"
    );
}

#[test]
fn mists_toc_game_token_resolves_to_mists_subdirectory() {
    let toc = TocFile::parse(
        std::path::Path::new("Blizzard_CharacterFrame"),
        r#"
        ## Interface: 50503
        [Game]\PaperDollFrameUtil.lua [AllowLoadGameType cata, mists]
        "#,
    );

    assert_eq!(
        toc.files,
        vec![std::path::PathBuf::from("Mists/PaperDollFrameUtil.lua")],
        "Mists TOC [Game] token should select the Mists source variant"
    );
}

#[test]
fn mists_loads_money_frame_before_uipanels_game() {
    let addons = wow_ui_sim::loader::discover_blizzard_addons_for_screen(
        &blizzard_ui_dir(),
        ScreenKind::Game,
    );
    let money_frame_index = addons
        .iter()
        .position(|(name, _)| name == "Blizzard_MoneyFrame")
        .expect("Mists game startup should include Blizzard_MoneyFrame");
    let ui_panels_index = addons
        .iter()
        .position(|(name, _)| name == "Blizzard_UIPanels_Game")
        .expect("Mists game startup should include Blizzard_UIPanels_Game");

    assert!(
        money_frame_index < ui_panels_index,
        "Blizzard_MoneyFrame must load before Blizzard_UIPanels_Game so MoneyInputFrameTemplate exists before TradeFrame.xml instantiates TradePlayerInputMoneyFrame; indexes were MoneyFrame={money_frame_index}, UIPanels={ui_panels_index}"
    );
}

#[test]
fn mists_money_input_template_xml_wires_gold_silver_copper_parent_keys() {
    let ui = wow_ui_sim::xml::parse_xml_file(&mists_money_input_frame_xml_path())
        .expect("Mists MoneyInputFrame.xml should parse");
    let template = find_top_level_frame(&ui.elements, "MoneyInputFrameTemplate");

    let child_keys: Vec<&str> = template
        .all_frame_elements()
        .into_iter()
        .filter_map(|(frame, _tag)| frame.parent_key.as_deref())
        .collect();

    assert!(
        child_keys.contains(&"gold"),
        "MoneyInputFrameTemplate should wire the gold edit box via parentKey"
    );
    assert!(
        child_keys.contains(&"silver"),
        "MoneyInputFrameTemplate should wire the silver edit box via parentKey"
    );
    assert!(
        child_keys.contains(&"copper"),
        "MoneyInputFrameTemplate should wire the copper edit box via parentKey"
    );
}

#[test]
fn mists_money_input_template_runtime_syncs_coin_parent_keys() {
    let env = load_mists_money_frame_env();

    let result: (String, String, String, String, String, String) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "MoneyInputFrameParentKeyProbe", UIParent, "MoneyInputFrameTemplate")
            return type(frame.gold),
                type(frame.silver),
                type(frame.copper),
                frame.gold:GetName(),
                frame.silver:GetName(),
                frame.copper:GetName()
            "#,
        )
        .expect("MoneyInputFrameTemplate should instantiate under CreateFrame");

    assert_eq!(
        result,
        (
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "MoneyInputFrameParentKeyProbeGold".to_string(),
            "MoneyInputFrameParentKeyProbeSilver".to_string(),
            "MoneyInputFrameParentKeyProbeCopper".to_string()
        ),
        "MoneyInputFrameTemplate inheritance and parentKey sync should publish gold/silver/copper children; missing copper is not a MoneyFrame API-state issue"
    );
}

#[test]
fn mists_trade_player_input_money_frame_widget_has_copper_child() {
    let env = load_mists_money_frame_env();

    let result: (String, String, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "TradePlayerInputMoneyFrame", UIParent, "MoneyInputFrameTemplate")
            return type(frame.copper),
                TradePlayerInputMoneyFrameCopper:GetName(),
                frame.copper == TradePlayerInputMoneyFrameCopper
            "#,
        )
        .expect("TradePlayerInputMoneyFrame should instantiate from MoneyInputFrameTemplate");

    assert_eq!(
        result,
        (
            "table".to_string(),
            "TradePlayerInputMoneyFrameCopper".to_string(),
            true
        ),
        "TradePlayerInputMoneyFrame must expose its copper edit box through both parentKey and named global wiring"
    );
}

#[test]
fn mists_world_map_set_opacity_reproduces_nil_opacity_arithmetic() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let source = mists_lua_source("Blizzard_WorldMap/Cata/Blizzard_WorldMap.lua");
    env.exec(&source)
        .expect("Cata/Mists WorldMap Lua should define opacity helpers");

    let (ok, err): (bool, String) = env
        .eval(
            r#"
            WorldMapFrame = {
                SetAlpha = function() end,
                ScrollContainer = { SetAlpha = function() end },
            }
            QuestMapFrame = { SetAlpha = function() end }

            local ok, err = pcall(WorldMapFrame_SetOpacity, nil)
            return ok, tostring(err)
            "#,
        )
        .expect("WorldMapFrame_SetOpacity pcall should return a status");

    assert!(!ok, "nil world map opacity should fail during arithmetic");
    assert!(
        err.contains("opacity") || err.contains("arithmetic") || err.contains("nil"),
        "expected nil opacity arithmetic failure, got: {err}"
    );
}

#[test]
fn mists_bootstrap_supplies_legacy_startup_api_shapes() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let result = query_mists_startup_api_shape(&env);

    assert_eq!(
        result,
        expected_mists_startup_api_shape(),
        "Mists startup helpers should return non-nil legacy API shapes"
    );
}

fn query_mists_startup_api_shape(env: &WowLuaEnv) -> MistsStartupApiShape {
    env.eval(
        r#"
            local skillName, _header, _isExpanded, skillRank, tempPoints, _modifier, skillMaxRank = GetSkillLineInfo(GetSelectedSkill())
            SetGuildRosterSelection(7)
            local hk, contribution = GetPVPThisWeekStats()
            return GetNumClasses(),
                GetSelectedSkill(),
                skillName,
                skillRank,
                tempPoints,
                HonorSystemEnabled(),
                hk,
                contribution,
                GetCurrencyListSize(),
                GetGuildRosterSelection(),
                skillMaxRank,
                type(C_ProductChoice.GetChoices())
            "#,
    )
    .expect("Mists legacy startup APIs should be callable")
}

fn expected_mists_startup_api_shape() -> MistsStartupApiShape {
    (
        11,
        1,
        "Weapon Skills".to_string(),
        1,
        0,
        false,
        0,
        0,
        19,
        7,
        1,
        "table".to_string(),
    )
}

#[test]
fn mists_honor_frame_shared_reproduces_missing_honor_system_enabled() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.exec(
        r#"
        rawset(_G, "HonorSystemEnabled", nil)
        HonorFrame_GetCurrencyFrame = function()
            return {
                Hide = function() end,
                Show = function() end,
            }
        end
        "#,
    )
    .expect("install HonorFrame reproduction fixtures");
    let source = mists_lua_source("Blizzard_UIPanels_Game/Classic/HonorFrame_Shared.lua");
    env.exec(&source)
        .expect("HonorFrame_Shared.lua should define functions before OnLoad runs");

    // Exclude bootstrap/source-load observations from the fault-injection interval.
    let previous_accesses = std::mem::take(&mut env.state().borrow_mut().nil_symbol_accesses);
    let result = env.eval::<(bool, String)>(
        r#"
        local frame = { RegisterEvent = function() end }
        local handler = HonorFrame_OnLoad
        local previousDedupe = rawget(_G, "__wow_logged_nil_symbols")
        rawset(_G, "__wow_logged_nil_symbols", {})
        local ok, err = pcall(handler, frame)
        rawset(_G, "__wow_logged_nil_symbols", previousDedupe)
        return ok, tostring(err)
        "#,
    );
    let accesses = std::mem::replace(
        &mut env.state().borrow_mut().nil_symbol_accesses,
        previous_accesses,
    );
    let (ok, err) = result.expect("HonorFrame_OnLoad pcall should return a status");

    assert!(!ok, "HonorFrame_OnLoad should reproduce the nil global");
    assert!(
        err.contains("attempt to call a nil value"),
        "expected a nil-call failure, got: {err}"
    );
    assert_eq!(
        accesses.len(),
        1,
        "expected only the injected global lookup during HonorFrame_OnLoad: {accesses:?}"
    );
    assert_eq!(accesses[0].container, "_G");
    assert_eq!(accesses[0].key, "HonorSystemEnabled");
}

#[test]
fn mists_honor_system_enabled_matches_disabled_legacy_honor_surface() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let (kind, enabled): (String, bool) = env
        .eval(
            r#"
            return type(HonorSystemEnabled()), HonorSystemEnabled()
            "#,
        )
        .expect("HonorSystemEnabled should be callable in Mists");

    assert_eq!(
        (kind, enabled),
        ("boolean".to_string(), false),
        "MoP Classic keeps HonorSystemEnabled as a global boolean gate; false hides the legacy HonorFrame honor currency surface"
    );
}

#[test]
fn mists_honor_pvp_api_contract_matches_classic_shapes() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    {
        let mut state = env.state().borrow_mut();
        state.player.honor_level = 5;
        state.pvp_honor.classic_honor_system_enabled = true;
        state.pvp_honor.yesterday_honorable_kills = 11;
        state.pvp_honor.yesterday_dishonorable_kills = 1;
        state.pvp_honor.this_week_honorable_kills = 22;
        state.pvp_honor.this_week_contribution = 330;
        state.pvp_honor.last_week_honorable_kills = 44;
        state.pvp_honor.last_week_dishonorable_kills = 2;
        state.pvp_honor.last_week_contribution = 550;
        state.pvp_honor.last_week_rank = 6;
        state.pvp_honor.lifetime_honorable_kills = 88;
        state.pvp_honor.lifetime_highest_rank = 9;
        state.pvp_honor.rank_progress = 0.25;
    }

    let shape: (String, bool, i32, i32, i32, i32) = env
        .eval(
            r##"
            return type(HonorSystemEnabled()),
                HonorSystemEnabled(),
                select("#", GetPVPYesterdayStats()),
                select("#", GetPVPThisWeekStats()),
                select("#", GetPVPLastWeekStats()),
                select("#", GetPVPLifetimeStats())
            "##,
        )
        .expect("Mists honor/PvP APIs should expose Classic call shapes");
    let values: (i32, i32, i32, i32, i32, i32, i32, i32, String, i32, f64) = env
        .eval(
            r##"
            local yesterdayHK, yesterdayDK = GetPVPYesterdayStats()
            local weekHK, weekContribution = GetPVPThisWeekStats()
            local lastWeekHK, lastWeekDK = GetPVPLastWeekStats()
            local lifetimeHK, _, highestRank = GetPVPLifetimeStats()
            local rankName, rankNumber = GetPVPRankInfo(UnitPVPRank("player"))
            return
                yesterdayHK,
                yesterdayDK,
                weekHK,
                weekContribution,
                lastWeekHK,
                lastWeekDK,
                lifetimeHK,
                highestRank,
                rankName,
                rankNumber,
                GetPVPRankProgress()
            "##,
        )
        .expect("Mists honor/PvP APIs should read simulator state");

    assert_eq!(
        shape,
        ("boolean".to_string(), true, 2, 2, 4, 3),
        "Mists honor/PvP startup APIs should keep the Classic return shapes HonorFrame_Shared.lua consumes"
    );
    assert_eq!(
        values,
        (11, 1, 22, 330, 44, 2, 88, 9, "Rank".to_string(), 5, 0.25),
        "Mists honor/PvP startup APIs should read Classic return shapes from simulator state"
    );
}

#[test]
fn mists_trade_money_frame_onload_reproduces_missing_copper_child() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let (ok, err): (bool, String) = env
        .eval(
            r#"
            MoneyInputFrame_SetCompact = function() end
            MoneyInputFrame_SetOnValueChangedFunc = function() end
            TradeFrame_UpdateMoney = function() end

            local frame = {
                silver = { SetPoint = function() end },
                RegisterEvent = function() end,
            }

            local ok, err = pcall(function()
                MoneyInputFrame_SetCompact(frame, 56, 7)
                MoneyInputFrame_SetOnValueChangedFunc(frame, TradeFrame_UpdateMoney)
                frame:RegisterEvent("PLAYER_TRADE_MONEY")
                frame.copper:SetPoint("LEFT", "TradePlayerInputMoneyFrameSilver", "RIGHT", 11, 0)
                frame.silver:SetPoint("LEFT", "TradePlayerInputMoneyFrameGold", "RIGHT", 22, 0)
            end)

            return ok, tostring(err)
            "#,
        )
        .expect("TradePlayerInputMoneyFrame OnLoad reproduction should run under pcall");

    assert!(!ok, "missing TradePlayerInputMoneyFrame.copper should fail");
    assert!(
        err.contains("copper"),
        "expected missing copper child failure, got: {err}"
    );
}

#[test]
fn mists_bootstrap_registers_startup_cvar_defaults() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, String, String, bool, bool) = env
        .eval(
            r#"
            return GetCVar("worldMapOpacity"),
                GetCVar("NamePlateHorizontalScale"),
                GetCVar("NamePlateVerticalScale"),
                GetCVarBool("ShowClassColorInFriendlyNameplate"),
                GetCVarBool("ColorNameplateNameBySelection")
            "#,
        )
        .expect("Mists startup CVars should be readable");

    assert_eq!(
        result,
        (
            "1".to_string(),
            "1".to_string(),
            "1".to_string(),
            true,
            false
        ),
        "Mists startup CVars should have concrete defaults"
    );
}

#[test]
fn mists_skill_line_zero_selection_still_returns_numeric_rank() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (i32, String, i32, i32, i32) = env
        .eval(
            r#"
            SetSelectedSkill(0)
            local skillName, _header, _isExpanded, skillRank, tempPoints, _modifier, skillMaxRank = GetSkillLineInfo(GetSelectedSkill())
            return GetSelectedSkill(), skillName, skillRank, tempPoints, skillMaxRank
            "#,
        )
        .expect("Mists skill selection zero should still have a startup row");

    assert_eq!(
        result,
        (0, "Weapon Skills".to_string(), 1, 0, 1),
        "Mists SkillFrame startup should not receive nil rank fields"
    );
}

#[test]
fn mists_selected_skill_api_matches_skill_frame_tuple_shape() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (i32, String, bool, bool, i32, i32, i32, i32, bool, bool) = env
        .eval(
            r#"
            SetSelectedSkill(1)
            local selected = GetSelectedSkill()
            local skillName, header, isExpanded, skillRank, tempPoints,
                skillModifier, skillMaxRank, isAbandonable = GetSkillLineInfo(selected)
            local missingSkill = GetSkillLineInfo(2) == nil
            return selected,
                skillName,
                header,
                isExpanded,
                skillRank,
                tempPoints,
                skillModifier,
                skillMaxRank,
                isAbandonable,
                missingSkill
            "#,
        )
        .expect("Mists selected skill API should return the tuple SkillFrame destructures");

    assert_eq!(
        result,
        (
            1,
            "Weapon Skills".to_string(),
            false,
            false,
            1,
            0,
            0,
            1,
            false,
            true
        ),
        "SkillFrame expects concrete rank numbers for selected rows and nil for out-of-range rows"
    );
}

#[test]
fn mists_bootstrap_supplies_pvp_currency_and_debugbar_shapes() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            local rankName, rankNumber = GetPVPRankInfo(0)
            local honor = C_CurrencyInfo.GetCurrencyInfo(Constants.CurrencyConsts.CLASSIC_HONOR_CURRENCY_ID)
            local lifetimeKills, lifetimeDishonorable, highestRank = GetPVPLifetimeStats()
            return rankName, rankNumber, honor.quantity, DebugBarManager:GetScaledInternalBarsHeight(),
                UnitPVPRank("player"), highestRank
            "#,
        )
        .expect("Mists PVP, currency, and debug bar startup APIs should be callable");

    assert_eq!(
        result,
        ("None".to_string(), 0, 0, 0, 0, 0),
        "Mists startup helpers should return concrete numeric fields"
    );
}

#[test]
fn mists_bootstrap_supplies_legacy_bank_and_aura_shapes() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (i32, i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            local _name, _icon, _count, _debuffType, duration, expirationTime,
                _source, _stealable, _personal, _spellID, _unused1, _unused2, _unused3,
                _unused4, timeMod = UnitBuff("player", 1)
            return NUM_BANKGENERIC_SLOTS,
                NUM_BANKBAGSLOTS,
                Constants.InventoryConstants.NumGenericBankSlots,
                Constants.InventoryConstants.NumBankBagSlots,
                duration,
                timeMod
            "#,
        )
        .expect("Mists bank constants and legacy aura tuple should be callable");

    assert_eq!(
        result,
        (28, 7, 28, 7, 3600, 1),
        "Mists startup should see bank constants and the legacy UnitBuff tuple shape"
    );
}

#[test]
fn mists_bootstrap_supplies_settings_label_globals() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, String) = env
        .eval(
            r#"
            return SHOW_AGGRO_PERCENTAGES,
                SHOW_COMBAT_HEALING_TEXT
            "#,
        )
        .expect("Mists settings labels should be strings");

    assert_eq!(
        result,
        ("Show aggro percentages".to_string(), "Healing".to_string()),
        "Mists settings variables should not register nil display names"
    );
}

#[test]
fn mists_bootstrap_supplies_hidden_non_priest_priest_bar_mixins() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, String, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "MistsPriestBarMixinProbe", UIParent)
            PriestBarMixin.OnLoad(frame)
            return type(PriestBarMixin.OnLoad),
                type(PriestBarOrbMixin),
                frame:IsShown()
            "#,
        )
        .expect("Mists priest bar mixin probe should run");

    assert_eq!(
        result,
        ("function".to_string(), "table".to_string(), false),
        "Mists should define priest bar mixins and hide the frame for non-priest startup"
    );
}

#[test]
fn mists_intrinsic_templates_supply_legacy_item_button_template_children() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, String, String, String, String) = env
        .eval(
            r#"
            local button = CreateFrame("CheckButton", "MistsLegacyBagSlotTemplateProbe", UIParent, "ItemButtonTemplate")
            return type(button),
                type(MistsLegacyBagSlotTemplateProbeCount),
                type(MistsLegacyBagSlotTemplateProbeNormalTexture),
                type(button.Count),
                type(button.NormalTexture)
            "#,
        )
        .expect("Mists legacy ItemButtonTemplate probe should run");

    assert_eq!(
        result,
        (
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
            "table".to_string(),
        ),
        "Mists bag slot XML expects ItemButtonTemplate to create Count and NormalTexture children"
    );
}

#[test]
fn mists_create_forbidden_frame_forwards_to_create_frame() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");

    let result: (String, String, bool) = env
        .eval(
            r#"
            local frame = CreateForbiddenFrame("Button", "MistsForbiddenProbe", UIParent, "UIPanelButtonTemplate")
            return frame:GetObjectType(), frame:GetName(), frame:IsForbidden()
            "#,
        )
        .expect("CreateForbiddenFrame should create a real forbidden frame");

    assert_eq!(
        result,
        (
            "Button".to_string(),
            "MistsForbiddenProbe".to_string(),
            true
        ),
        "CreateForbiddenFrame should preserve CreateFrame semantics and mark the frame forbidden"
    );
}
