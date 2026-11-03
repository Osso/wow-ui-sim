//! F04 assertions against original 12340 FrameXML consumers.
//! Native source: ActionButton.lua, ActionBarFrame.xml, MainMenuBar.xml,
//! BonusActionBarFrame.lua, MultiActionBars.lua and PetActionBarFrame.lua.
#[path = "wrath_callback_assertions.rs"]
pub mod callbacks;

use super::{build_wrath_env, wrath_env};
use rilua::Val;
use wow_ui_sim::lua_api::{WowLuaEnv, state::PetActionSlot};

pub fn bootstrap_twice() {
    super::super::with_perf_lock(|| {
        let first = build_wrath_env();
        first.exec("assert(MainMenuBar:IsVisible())").unwrap();
        drop(first);
        let second = build_wrath_env();
        second.exec("assert(MainMenuBar:IsVisible())").unwrap();
    });
}

pub fn button_size() {
    wrath_env()
        .exec("assert(ActionButton1:GetWidth() == 36 and ActionButton1:GetHeight() == 36)")
        .unwrap();
}

pub fn showgrid() {
    wrath_env()
        .exec(
            r#"
        for i = 1, 12 do
            assert(_G["ActionButton"..i]:GetAttribute("showgrid") > 0)
        end
    "#,
        )
        .unwrap();
}

pub fn buttons_visible() {
    wrath_env()
        .exec(
            r#"
        for i = 1, 12 do assert(_G["ActionButton"..i]:IsVisible()) end
    "#,
        )
        .unwrap();
}

pub fn main_visible() {
    wrath_env().exec("assert(MainMenuBar:IsVisible())").unwrap();
}

pub fn main_size() {
    wrath_env()
        .exec("assert(MainMenuBar:GetWidth() == 1024 and MainMenuBar:GetHeight() == 53)")
        .unwrap();
}

pub fn right_bars_hidden() {
    wrath_env()
        .exec("assert(not MultiBarLeft:IsVisible() and not MultiBarRight:IsVisible())")
        .unwrap();
}

pub fn endcaps() {
    wrath_env().exec(r#"
        for _, cap in ipairs({MainMenuBarLeftEndCap, MainMenuBarRightEndCap}) do
            assert(cap:IsVisible())
            assert(cap:GetWidth() == 128 and cap:GetHeight() == 128)
            assert(string.lower((cap:GetTextureFilePath():gsub("\\", "/"))) == "interface/mainmenubar/ui-mainmenubar-endcap-dwarf")
        end
    "#).unwrap();
}

pub fn bag_layout() {
    wrath_env()
        .exec(
            r#"
        local backpack = MainMenuBarBackpackButton
        assert(backpack:IsVisible() and backpack:GetWidth() == 37 and backpack:GetHeight() == 37)
        local previous = backpack
        for i = 0, 3 do
            local bag = _G["CharacterBag"..i.."Slot"]
            assert(bag:IsVisible() and bag:GetParent() == MainMenuBarArtFrame)
            local gap = i == 0 and 5 or 4
            assert(math.abs(bag:GetRight() + gap - previous:GetLeft()) < 0.01)
            previous = bag
        end
    "#,
        )
        .unwrap();
}

fn seed_watched_faction(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.factions = vec![wow_ui_sim::lua_api::state::FactionEntry {
        faction_id: 72,
        name: "F04 watched faction".into(),
        standing: 5,
        bottom: 3000,
        top: 9000,
        earned: 5200,
        has_rep: true,
        is_watched: true,
        ..Default::default()
    }];
    state.watched_faction_index = 1;
}

pub fn status_layout() {
    let env = wrath_env();
    seed_watched_faction(&env);
    env.exec(
        r#"
        ReputationWatchBar_Update(79)
        assert(MainMenuExpBar:GetWidth() == 1024 and MainMenuExpBar:GetHeight() == 13)
        assert(ReputationWatchBar:IsShown() and MainMenuExpBar:IsShown())
        assert(ReputationWatchStatusBar:GetHeight() == 8)
        local point, relative, relativePoint, x, y = ReputationWatchBar:GetPoint(1)
        assert(point == "BOTTOM" and relative == MainMenuBar and relativePoint == "TOP")
        assert(x == 0 and y == -3)
        ReputationWatchBar_Update(MAX_PLAYER_LEVEL)
        assert(ReputationWatchStatusBar:GetHeight() == 13 and not MainMenuExpBar:IsShown())
    "#,
    )
    .unwrap();
}

pub fn button_children() {
    wrath_env().exec(r#"
        assert(ActionButton1:IsVisible() and ActionButton1:GetWidth() == 36 and ActionButton1:GetHeight() == 36)
        assert(ActionButton1Icon and ActionButton1HotKey and ActionButton1NormalTexture)
        assert(ActionButton1Icon:GetParent() == ActionButton1)
        assert(ActionButton1HotKey:GetParent() == ActionButton1)
        assert(ActionButton1NormalTexture == ActionButton1:GetNormalTexture())
    "#).unwrap();
}

pub fn button_normal_texture() {
    wrath_env().exec(r#"
        local texture = ActionButton1NormalTexture
        assert(texture:IsVisible() and texture:GetWidth() == 66 and texture:GetHeight() == 66)
        assert(string.lower((texture:GetTextureFilePath():gsub("\\", "/"))) == "interface/buttons/ui-quickslot2")
    "#).unwrap();
}

pub fn button_hotkey() {
    let env = wrath_env();
    env.exec(
        r#"
        SetBinding("1", "ACTIONBUTTON1")
    "#,
    )
    .unwrap();
    env.fire_event("UPDATE_BINDINGS").unwrap();
    env.exec("assert(ActionButton1HotKey:IsVisible() and ActionButton1HotKey:GetText() == '1')")
        .unwrap();
}

pub fn main_layout() {
    wrath_env()
        .exec(
            r#"
        assert(MainMenuBar:GetWidth() == 1024 and MainMenuBar:GetHeight() == 53)
        for i = 1, 12 do
            local button = _G["ActionButton"..i]
            assert(button:GetParent() == MainMenuBarArtFrame)
            assert(math.abs(button:GetBottom() - MainMenuBarArtFrame:GetBottom() - 4) < 0.01)
            local expectedLeft = MainMenuBarArtFrame:GetLeft() + 8 + (i - 1) * 42
            assert(math.abs(button:GetLeft() - expectedLeft) < 0.01)
        end
    "#,
        )
        .unwrap();
}

fn seed_slot(env: &WowLuaEnv, slot: u32, spell: u32) {
    env.state().borrow_mut().action_bars.insert(slot, spell);
    env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[Val::Num(slot as f64)])
        .unwrap();
}

pub fn pickup_icon() {
    let env = wrath_env();
    seed_slot(&env, 1, 853);
    env.exec(
        r#"
        assert(ActionButton1Icon:IsShown() and HasAction(1))
        PickupAction(1, false)
        assert(not ActionButton1Icon:IsShown() and not HasAction(1))
        PlaceAction(1)
        assert(ActionButton1Icon:IsShown() and HasAction(1))
    "#,
    )
    .unwrap();
}

pub fn drag_round_trip() {
    let env = wrath_env();
    seed_slot(&env, 1, 853);
    env.exec(
        r#"
        -- Wrath ActionButton.lua:169,246 passes the action texture filename through.
        assert(GetActionTexture(1) == "ICONS/Spell_Holy_SealOfMight")
        assert(ActionButton1Icon:IsShown() and ActionButton1Icon:GetTexture() == GetActionTexture(1))
        assert(ActionButton1:GetScript("OnDragStart") and ActionButton1:GetScript("OnReceiveDrag"))
        -- Native client option: unlock action dragging in this isolated fixture.
        LOCK_ACTIONBAR = "0"
    "#,
    )
    .unwrap();
    let id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("ActionButton1")
        .unwrap();
    env.fire_script_handler(id, "OnDragStart", vec![]).unwrap();
    env.exec("assert(not HasAction(1) and GetCursorInfo() == 'spell')")
        .unwrap();
    env.fire_script_handler(id, "OnReceiveDrag", vec![])
        .unwrap();
    env.exec("assert(HasAction(1) and GetCursorInfo() == nil and GetActionTexture(1) == 'ICONS/Spell_Holy_SealOfMight' and ActionButton1Icon:IsShown() and ActionButton1Icon:GetTexture() == GetActionTexture(1))").unwrap();
}

pub fn icon_identity() {
    let env = wrath_env();
    seed_slot(&env, 1, 853);
    env.exec(
        r#"
        assert(GetActionTexture(1) == "ICONS/Spell_Holy_SealOfMight")
        assert(ActionButton1Icon:IsShown() and ActionButton1Icon:GetTexture() == GetActionTexture(1))
    "#,
    )
    .unwrap();
}

pub fn key_release_cast() {
    let env = wrath_env();
    seed_slot(&env, 1, 19750);
    env.exec(
        r#"
        assert(UnitCastingInfo("player") == nil and ActionButton1:GetButtonState() == "NORMAL")
        ActionButtonDown(1)
        assert(ActionButton1:GetButtonState() == "PUSHED" and UnitCastingInfo("player") == nil)
        ActionButtonUp(1)
        assert(ActionButton1:GetButtonState() == "NORMAL")
        assert(select(9, UnitCastingInfo("player")) == 19750)
    "#,
    )
    .unwrap();
}

pub fn multibar_release_cast() {
    let env = wrath_env();
    seed_slot(&env, 61, 19750);
    env.exec(r#"
        assert(MultiBarBottomLeftButton1.action == 61)
        MultiActionButtonDown("MultiBarBottomLeft", 1)
        assert(MultiBarBottomLeftButton1:GetButtonState() == "PUSHED" and UnitCastingInfo("player") == nil)
        MultiActionButtonUp("MultiBarBottomLeft", 1)
        assert(MultiBarBottomLeftButton1:GetButtonState() == "NORMAL")
        assert(select(9, UnitCastingInfo("player")) == 19750)
    "#).unwrap();
}

pub fn slot_changed() {
    let env = wrath_env();
    env.state().borrow_mut().action_bars.clear();
    env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[Val::Num(0.0)])
        .unwrap();
    env.exec("assert(ActionButton1.action == 1 and not ActionButton1Icon:IsShown())")
        .unwrap();
    seed_slot(&env, 1, 853);
    env.exec("assert(GetActionTexture(1) == 'ICONS/Spell_Holy_SealOfMight' and ActionButton1Icon:IsShown() and ActionButton1Icon:GetTexture() == GetActionTexture(1))")
        .unwrap();
}

pub fn tooltip() {
    let env = wrath_env();
    seed_slot(&env, 3, 853);
    let id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("ActionButton3")
        .unwrap();
    env.fire_script_handler(id, "OnEnter", vec![]).unwrap();
    env.exec("assert(GameTooltip:GetOwner() == ActionButton3 and GameTooltip:IsShown() and ActionButton3.action == 3)").unwrap();
}

pub fn page_change() {
    let env = wrath_env();
    env.exec(
        r#"
        ChangeActionBarPage(1)
        F04_PAGE_EVENTS = 0
        MainMenuBarArtFrame:HookScript("OnEvent", function(_, event)
            if event == "ACTIONBAR_PAGE_CHANGED" then F04_PAGE_EVENTS = F04_PAGE_EVENTS + 1 end
        end)
        ActionBar_PageUp()
        assert(GetActionBarPage() == 2 and F04_PAGE_EVENTS == 1)
        assert(MainMenuBarPageNumber:GetText() == "2" and ActionButton1.action == 13)
        ChangeActionBarPage(6)
        ActionBar_PageUp()
        assert(GetActionBarPage() == 1 and ActionButton1.action == 1)
    "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().action_bar_page, 1);
}

pub fn pet_update() {
    let env = wrath_env();
    env.state().borrow_mut().pet_actions[0] = PetActionSlot {
        has_action: true,
        name: Some("F04 Bite".into()),
        texture: Some("Interface\\Icons\\Ability_Druid_FerociousBite".into()),
        ..Default::default()
    };
    env.fire_event("PET_BAR_UPDATE").unwrap();
    env.exec(r#"
        assert(PetActionButton1.tooltipName == "F04 Bite")
        assert(PetActionButton1Icon:IsShown())
        assert(string.lower((PetActionButton1Icon:GetTextureFilePath():gsub("\\", "/"))) == "interface/icons/ability_druid_ferociousbite")
        assert(PetActionButton2.tooltipName == nil and not PetActionButton2Icon:IsShown())
    "#).unwrap();
}

pub fn possess_visibility() {
    let env = wrath_env();
    env.exec("assert(not PossessBarFrame:IsShown()); assert(type(IsPossessBarVisible()) == 'boolean' and select('#', IsPossessBarVisible()) == 1)").unwrap();
    env.state()
        .borrow_mut()
        .action_bar_state
        .possess_bar_visible = true;
    env.fire_event("UPDATE_BONUS_ACTIONBAR").unwrap();
    env.exec("assert(IsPossessBarVisible() and PossessBarFrame:IsShown() and not ShapeshiftBarFrame:IsShown())").unwrap();
    env.state()
        .borrow_mut()
        .action_bar_state
        .possess_bar_visible = false;
    env.fire_event("UPDATE_BONUS_ACTIONBAR").unwrap();
    env.exec("assert(not IsPossessBarVisible() and not PossessBarFrame:IsShown() and ShapeshiftBarFrame:IsShown())").unwrap();
}

pub fn reputation() {
    let env = wrath_env();
    seed_watched_faction(&env);
    env.exec(r#"
        ReputationWatchBar_Update(79)
        local name, reaction, minimum, maximum, value = GetWatchedFactionInfo()
        assert(name and maximum > minimum and value > minimum)
        local low, high = ReputationWatchStatusBar:GetMinMaxValues()
        assert(low == 0 and high == maximum - minimum)
        assert(ReputationWatchStatusBar:GetValue() == value - minimum)
        assert(ReputationWatchStatusBarText:GetText() == name.." "..(value-minimum).." / "..(maximum-minimum))
        assert(ReputationWatchBar:IsShown() and MainMenuExpBar:IsShown())
        ReputationWatchBar_Update(MAX_PLAYER_LEVEL)
        assert(ReputationWatchBar:IsShown() and not MainMenuExpBar:IsShown())
    "#).unwrap();
}

pub fn rested_color() {
    let env = wrath_env();
    env.state().borrow_mut().player_xp.rest_state = 2;
    env.fire_event("UPDATE_EXHAUSTION").unwrap();
    env.exec("local r,g,b = MainMenuExpBar:GetStatusBarColor(); assert(math.abs(r - 0.58) < 0.001 and g == 0 and math.abs(b - 0.55) < 0.001)").unwrap();
    env.state().borrow_mut().player_xp.rest_state = 1;
    env.state().borrow_mut().player.is_resting = true;
    env.fire_event("UPDATE_EXHAUSTION").unwrap();
    env.exec("local r,g,b = MainMenuExpBar:GetStatusBarColor(); assert(r == 0 and math.abs(g - 0.39) < 0.001 and math.abs(b - 0.88) < 0.001)").unwrap();
}

pub fn multibar_grid() {
    wrath_env()
        .exec(
            r#"
        local bars = {"MultiBarBottomLeft", "MultiBarBottomRight", "MultiBarRight", "MultiBarLeft"}
        for _, bar in ipairs(bars) do
            for i = 1, 12 do _G[bar.."Button"..i]:SetAttribute("showgrid", 0) end
        end
        MultiActionBar_ShowAllGrids()
        for _, bar in ipairs(bars) do
            for i = 1, 12 do assert(_G[bar.."Button"..i]:GetAttribute("showgrid") == 1) end
        end
        MultiActionBar_HideAllGrids()
        for _, bar in ipairs(bars) do
            for i = 1, 12 do assert(_G[bar.."Button"..i]:GetAttribute("showgrid") == 0) end
        end
    "#,
        )
        .unwrap();
}

pub fn stance_icons() {
    wrath_env()
        .exec(
            r#"
        ShapeshiftBar_Update()
        assert(GetNumShapeshiftForms() == 3 and ShapeshiftBarFrame:IsShown())
        for i = 1, 3 do
            local texture, name, active, castable = GetShapeshiftFormInfo(i)
            assert(type(name) == "string" and name ~= "")
            assert(active == nil and castable == 1)
            assert(_G["ShapeshiftButton"..i]:IsShown())
            assert(not _G["ShapeshiftButton"..i]:GetChecked())
            assert(_G["ShapeshiftButton"..i.."Icon"]:GetTexture() ~= nil)
        end
        assert(not ShapeshiftButton4:IsShown())
    "#,
        )
        .unwrap();
}

pub fn stance_selection() {
    let env = wrath_env();
    {
        let mut state = env.state().borrow_mut();
        for form in &mut state.shapeshift_forms {
            form.is_active = false;
        }
        state.shapeshift_forms.truncate(2);
    }
    env.fire_event("UPDATE_SHAPESHIFT_FORMS").unwrap();
    env.exec(
        r#"
        assert(not ShapeshiftButton1:GetChecked() and not ShapeshiftButton2:GetChecked())
        ShapeshiftBar_ChangeForm(2)
        assert(GetShapeshiftForm() == 2 and ShapeshiftBarFrame.lastSelected == 2)
        assert(not ShapeshiftButton1:GetChecked() and ShapeshiftButton2:GetChecked())
    "#,
    )
    .unwrap();
    let state = env.state().borrow();
    assert!(!state.shapeshift_forms[0].is_active);
    assert!(state.shapeshift_forms[1].is_active);
}

pub fn vehicle_leave() {
    let env = wrath_env();
    env.state().borrow_mut().player.controlling_vehicle = true;
    env.exec("MainMenuBarVehicleLeaveButton:Click()").unwrap();
    assert!(!env.state().borrow().player.controlling_vehicle);
}

pub fn xp_update() {
    let env = wrath_env();
    {
        let mut state = env.state().borrow_mut();
        state.player.level = 79;
        state.player.xp = 12345;
        state.player.xp_max = 50000;
        state.player_xp.exhaustion = Some(5000);
    }
    env.exec("ReputationWatchBar_Update(79)").unwrap();
    env.fire_event_with_args("PLAYER_XP_UPDATE", &[env.lua_string("player")])
        .unwrap();
    env.exec(
        r#"
        assert(MainMenuExpBar:GetValue() == 12345)
        local minimum, maximum = MainMenuExpBar:GetMinMaxValues()
        assert(minimum == 0 and maximum == 50000)
        assert(ExhaustionTick:IsShown())
        local _, relative, _, offset = ExhaustionTick:GetPoint(1)
        assert(relative == MainMenuExpBar and math.abs(offset - 355.2256) < 0.01)
    "#,
    )
    .unwrap();
}

pub fn range_color() {
    let env = wrath_env();
    seed_slot(&env, 1, 19750);
    env.exec("TargetUnit('player'); ActionButton1HotKey:SetText('1')")
        .unwrap();
    let guid: String = env.eval("return UnitGUID('player')").unwrap();
    for in_range in [false, true] {
        env.state()
            .borrow_mut()
            .action_spell_ranges
            .entry(19750)
            .or_default()
            .in_range_by_guid
            .insert(guid.clone(), in_range);
        env.exec("ActionButton1.rangeTimer = -1; ActionButton_OnUpdate(ActionButton1, 0.1)")
            .unwrap();
        let color: (f64, f64, f64) = env
            .eval("return ActionButton1HotKey:GetVertexColor()")
            .unwrap();
        let expected = if in_range {
            (0.6, 0.6, 0.6)
        } else {
            (1.0, 0.1, 0.1)
        };
        for (actual, expected) in [color.0, color.1, color.2]
            .into_iter()
            .zip([expected.0, expected.1, expected.2])
        {
            assert!(
                (actual - expected).abs() < 0.001,
                "native numeric IsActionInRange must tint the hotkey: {color:?}"
            );
        }
    }
}

pub fn native_dependencies() {
    let env = wrath_env();
    env.exec(r#"
        assert(type(SecureActionButton_OnClick) == "function")
        assert(type(ActionButton_OnLoad) == "function" and type(ActionButton_OnEvent) == "function")
        assert(type(ShapeshiftBar_UpdateState) == "function" and type(PetActionBar_Update) == "function")
        assert(ActionButton1:GetScript("OnEvent") and ActionButton1:GetScript("OnReceiveDrag"))
        assert(ActionButton1.action == 1 and ActionButton1Icon:IsShown())
    "#).unwrap();
}
