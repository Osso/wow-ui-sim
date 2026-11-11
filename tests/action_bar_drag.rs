use crate::common;

#[cfg(any(feature = "client-ptr", feature = "client-retail"))]
use wow_ui_sim::lua_api::state::{CursorInfo, CursorItemOrigin};
use std::path::PathBuf;
use wow_ui_sim::lua_api::WowLuaEnv;

const ACTION_BAR_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_Colors",
    "Blizzard_SharedXML",
    #[cfg(feature = "client-ptr")]
    "Blizzard_GamepadSmartNavigation",
    "Blizzard_SharedXMLGame",
    "Blizzard_UIPanelTemplates",
    "Blizzard_FrameXMLBase",
    "Blizzard_LoadLocale",
    "Blizzard_Fonts_Shared",
    "Blizzard_HelpPlate",
    "Blizzard_AccessibilityTemplates",
    "Blizzard_ObjectAPI",
    "Blizzard_UIParent",
    "Blizzard_TextStatusBar",
    "Blizzard_MoneyFrame",
    "Blizzard_POIButton",
    "Blizzard_Flyout",
    "Blizzard_GameMenuEsc",
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_ManagedFrameSystem",
    "Blizzard_UIParentUtil",
    "Blizzard_EditMode",
    #[cfg(feature = "client-ptr")]
    "Blizzard_StatusTrackingBar",
    "Blizzard_GarrisonBase",
    "Blizzard_GameTooltip",
    #[cfg(feature = "client-ptr")]
    "Blizzard_StaticPopup",
    "Blizzard_UIParentPanelManager",
    #[cfg(feature = "client-ptr")]
    "Blizzard_GamepadSharedUtility",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    #[cfg(feature = "client-ptr")]
    "Blizzard_RaidWarning",
    "Blizzard_FrameXML",
    "Blizzard_UIPanels_Game",
    "Blizzard_MapCanvasSecureUtil",
    "Blizzard_MapCanvas",
    "Blizzard_SharedMapDataProviders",
    "Blizzard_WorldMap",
    "Blizzard_PingUI",
    #[cfg(feature = "client-ptr")]
    "Blizzard_SpellTypeIconOverlay",
    "Blizzard_ActionBar",
];

fn seed_action_slot(env: &WowLuaEnv, slot: u32, spell_id: u32) {
    env.state().borrow_mut().action_bars.insert(slot, spell_id);
}

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn load_action_bar_addons(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    if !common::load_classic_game_startup_inventory(env) {
        for addon_name in ACTION_BAR_ADDONS {
            common::load_required_blizzard_addon(env, &ui, addon_name);
        }
    }
}

fn fire_action_bar_startup(env: &WowLuaEnv) {
    env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("WoWUISim")])
        .unwrap();
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        env.fire_event(event).unwrap();
    }
    env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[rilua::Val::Bool(true), rilua::Val::Bool(false)],
    )
    .unwrap();
    env.fire_edit_mode_layouts_updated().unwrap();
    env.fire_event("ACTIONBAR_SHOWGRID").unwrap();
}

fn env_with_action_bar() -> common::LockedEnv {
    common::lock_env(|| {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_size(1024.0, 768.0);
        load_action_bar_addons(&env);
        env.apply_post_load_workarounds();
        fire_action_bar_startup(&env);
        env
    })
}

fn assert_action_button_template_has_receive_drag() {
    let chain = wow_ui_sim::xml::get_template_chain("ActionBarButtonTemplate");
    let code_template = chain
        .iter()
        .find(|entry| entry.name == "ActionBarButtonCodeTemplate")
        .unwrap();
    assert!(
        code_template
            .frame
            .scripts()
            .is_some_and(|scripts| !scripts.on_receive_drag.is_empty()),
        "template chain should include OnReceiveDrag on ActionBarButtonCodeTemplate"
    );
}

#[cfg(feature = "client-ptr")]
fn install_ptr_action_bar_trace(env: &WowLuaEnv) {
    env.exec(
        r#"
        R29ActionBarEvents = {}
        local listener = CreateFrame('Frame')
        for _, event in ipairs({
            'ADDON_ACTION_BLOCKED',
            'ACTIONBAR_HIDEGRID',
            'ACTIONBAR_SHOWGRID',
            'ACTIONBAR_SLOT_CHANGED',
            'ACTIONBAR_UPDATE_COOLDOWN',
            'ACTIONBAR_UPDATE_STATE',
            'CURSOR_CHANGED',
        }) do
            listener:RegisterEvent(event)
        end
        listener:SetScript('OnEvent', function(_, event, ...)
            local values = {event}
            for index = 1, select('#', ...) do
                values[#values + 1] = tostring(select(index, ...))
            end
            R29ActionBarEvents[#R29ActionBarEvents + 1] = table.concat(values, ',')
        end)
        function ResetR29ActionBarEvents()
            R29ActionBarEvents = {}
        end
        function ReadR29ActionBarEvents()
            return table.concat(R29ActionBarEvents, ';')
        end
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
fn read_ptr_action_bar_trace(env: &WowLuaEnv) -> String {
    env.eval("return ReadR29ActionBarEvents()")
        .expect("read PTR action-bar event trace")
}

#[cfg(feature = "client-ptr")]
fn action_button_id(env: &WowLuaEnv, name: &str) -> u64 {
    env.state()
        .borrow()
        .widgets
        .get_id_by_name(name)
        .unwrap_or_else(|| panic!("missing {name}"))
}

#[test]
fn pickup_action_accepts_ignore_removal_arg_and_place_restores_slot() {
    let env = WowLuaEnv::new().unwrap();
    seed_action_slot(&env, 1, 853);

    let ok: bool = env
        .eval(
            r#"
            local ok, err = pcall(function()
                PickupAction(1, false)
            end)
            TEST_PICKUP_ACTION_ERR = err
            return ok
            "#,
        )
        .unwrap();
    assert!(
        ok,
        "PickupAction(slot, ignoreRemoval) errored: {}",
        env.eval::<String>("return tostring(TEST_PICKUP_ACTION_ERR)")
            .unwrap()
    );
    assert!(!env.eval::<bool>("return HasAction(1)").unwrap());

    // Retail and PTR project mode 3 as ("spell", book slot, book type,
    // spell ID, ...) (WM46, WM56, R29); other profiles keep ("spell", spell ID).
    let projection = if cfg!(feature = "client-retail") || cfg!(feature = "client-ptr") {
        "local kind, _, _, spellID = GetCursorInfo(); return kind, spellID"
    } else {
        "return GetCursorInfo()"
    };
    let (cursor_type, cursor_spell_id): (String, i32) = env.eval(projection).unwrap();
    assert_eq!(cursor_type, "spell");
    assert_eq!(cursor_spell_id, 853);

    env.exec("PlaceAction(1)").unwrap();

    assert!(env.eval::<bool>("return HasAction(1)").unwrap());
    assert!(env.eval::<bool>("return GetCursorInfo() == nil").unwrap());
    assert!(
        env.eval::<bool>("return type(C_ActionBar.GetActionTexture(1)) == 'string'")
            .unwrap()
    );

    env.exec("PickupAction(1, true)").unwrap();
    assert!(env.eval::<bool>("return HasAction(1)").unwrap());
    let (copy_type, copy_spell_id): (String, i32) = env.eval(projection).unwrap();
    assert_eq!((copy_type.as_str(), copy_spell_id), ("spell", 853));
    env.exec("ClearCursor()").unwrap();
    assert!(env.eval::<bool>("return HasAction(1)").unwrap());
    assert!(env.eval::<bool>("return GetCursorInfo() == nil").unwrap());
}

#[test]
fn pickup_action_updates_action_button_icon_immediately() {
    wrath_action_case!(pickup_icon);

    common::with_timeout(120, move || {
        let env = env_with_action_bar();
        seed_action_slot(&env, 1, 853);
        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(1.0)])
            .unwrap();

        let before_drag: bool = env
            .eval("return ActionButton1.icon:IsShown() and HasAction(1)")
            .unwrap();
        assert!(
            before_drag,
            "action button should show its icon before PickupAction"
        );

        env.exec("PickupAction(1, false)").unwrap();

        let after_pickup: bool = env
            .eval("return (not ActionButton1.icon:IsShown()) and not HasAction(1)")
            .unwrap();
        assert!(
            after_pickup,
            "PickupAction should fire ACTIONBAR_SLOT_CHANGED so the source icon hides immediately"
        );

        env.exec("PlaceAction(1)").unwrap();

        let after_place: bool = env
            .eval("return ActionButton1.icon:IsShown() and HasAction(1)")
            .unwrap();
        assert!(
            after_place,
            "PlaceAction should fire ACTIONBAR_SLOT_CHANGED so the icon restores immediately"
        );
    });
}

#[test]
fn action_button_drag_round_trip_keeps_spell_visible() {
    wrath_action_case!(drag_round_trip);

    common::with_timeout(120, move || {
        let env = env_with_action_bar();
        env.exec(r#"SetCVar("lockActionBars", "0")"#)
            .expect("unlock action bars for drag input");
        let bars_locked: bool = env
            .eval(r#"return Settings.GetValue("lockActionBars")"#)
            .expect("read action-bar lock setting");
        assert!(!bars_locked, "drag test requires unlocked action bars");
        assert_action_button_template_has_receive_drag();
        let button_id = env
            .state()
            .borrow()
            .widgets
            .get_id_by_name("ActionButton1")
            .unwrap();
        seed_action_slot(&env, 1, 853);
        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(0.0)])
            .unwrap();
        env.fire_event("ACTIONBAR_UPDATE_STATE").unwrap();
        env.exec("if ActionButton1 then ActionButton1.icon:SetTexture(GetActionTexture(1)) end")
            .unwrap();

        let before_drag: bool = env
            .eval(
                "return ActionButton1.icon:IsShown() and ActionButton1.icon:GetTexture() == 135963",
            )
            .unwrap();
        assert!(
            before_drag,
            "action button should show spell texture 135963 before drag"
        );
        let has_receive_drag: bool = env
            .eval("return ActionButton1:GetScript('OnReceiveDrag') ~= nil")
            .unwrap();
        assert!(
            has_receive_drag,
            "action button should have an OnReceiveDrag handler"
        );

        env.fire_script_handler(button_id, "OnDragStart", vec![])
            .unwrap();
        let action_picked_up: bool = env
            .eval(
                r#"local cursorType = GetCursorInfo(); return cursorType == "spell" and not HasAction(1) and not ActionButton1.icon:IsShown()"#,
            )
            .unwrap();
        assert!(
            action_picked_up,
            "unlocked drag start should place the spell on the cursor and hide its action icon"
        );

        env.fire_script_handler(button_id, "OnReceiveDrag", vec![])
            .unwrap();

        let after_drag: bool = env
            .eval(
                "return ActionButton1.icon:IsShown() and ActionButton1.icon:GetTexture() == 135963 and HasAction(1)",
            )
            .unwrap();
        assert!(
            after_drag,
            "dragging off and back onto the same button should keep spell texture 135963 and its action"
        );
    });
}

#[test]
fn action_button_1_texture_path_resolves_to_icon_fdid() {
    wrath_action_case!(icon_identity);

    common::with_timeout(120, move || {
        let env = env_with_action_bar();
        seed_action_slot(&env, 1, 853);
        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(0.0)])
            .unwrap();
        env.fire_event("ACTIONBAR_UPDATE_STATE").unwrap();
        env.exec("if ActionButton1 then ActionButton1.icon:SetTexture(GetActionTexture(1)) end")
            .unwrap();

        let result: String = env
            .eval(
                r#"
                if not ActionButton1 then
                    return "missing_action_button_1"
                end
                if not ActionButton1.icon then
                    return "missing_action_button_1_icon"
                end

                local actionTexture = GetActionTexture(1)
                if actionTexture ~= "ICONS/Spell_Holy_SealOfMight" then
                    return string.format(
                        "action_texture_mismatch_expected_%s_actual_%s",
                        "ICONS/Spell_Holy_SealOfMight",
                        tostring(actionTexture)
                    )
                end

                local iconTexture = ActionButton1.icon:GetTexture()
                if iconTexture ~= 135963 then
                    return string.format(
                        "icon_fdid_mismatch_expected_%s_actual_%s",
                        tostring(135963),
                        tostring(iconTexture)
                    )
                end

                return "ok"
            "#,
            )
            .unwrap();

        assert_eq!(
            result, "ok",
            "ActionButton1 texture path should resolve to the expected icon FDID: {result}"
        );
    });
}

#[cfg(any(feature = "client-ptr", feature = "client-retail"))]
#[test]
fn cursor_info_projects_existing_payloads_with_profile_deltas() {
    let env = WowLuaEnv::new().unwrap();
    assert_eq!(
        env.eval::<i64>("return select('#', GetCursorInfo())").unwrap(),
        0,
        "an empty cursor returns no values"
    );

    env.state().borrow_mut().cursor_item = Some(CursorInfo::Action {
        slot: 1,
        spell_id: 853,
    });
    let (count, kind, book_type, spell_id, related_is_nil): (i64, String, String, i64, bool) = env
        .eval(
            r#"
            local kind, bookSlot, bookType, spellID, relatedID = GetCursorInfo()
            return select('#', GetCursorInfo()), kind, bookType, spellID, relatedID == nil
            "#,
        )
        .unwrap();
    assert_eq!(count, 5);
    assert_eq!(kind, "spell");
    assert_eq!(book_type, "spell");
    assert_eq!(spell_id, 853);
    assert!(related_is_nil);

    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        owned_identity: None,
        item_id: 6948,
        stack_count: 4,
        origin: CursorItemOrigin::Bag { bag: 0, slot: 1 },
        applied_transmog: None,
    });
    let (count, kind, item_id, link_kind): (i64, String, i64, String) = env
        .eval(
            r#"
            local kind, itemID, itemLink = GetCursorInfo()
            return select('#', GetCursorInfo()), kind, itemID, type(itemLink)
            "#,
        )
        .unwrap();
    assert_eq!(count, 3);
    assert_eq!(kind, "item");
    assert_eq!(item_id, 6948);
    assert!(matches!(link_kind.as_str(), "string" | "nil"));

    env.state().borrow_mut().cursor_item = Some(CursorInfo::Macro { macro_index: 4 });
    let (count, kind, macro_index): (i64, String, i64) = env
        .eval(
            r#"
            local kind, macroIndex = GetCursorInfo()
            return select('#', GetCursorInfo()), kind, macroIndex
            "#,
        )
        .unwrap();
    assert_eq!((count, kind.as_str(), macro_index), (2, "macro", 4));

    env.state().borrow_mut().cursor_item = Some(CursorInfo::Money { copper: 10 });
    let (count, kind, copper): (i64, String, i64) = env
        .eval(
            r#"
            local kind, copper = GetCursorInfo()
            return select('#', GetCursorInfo()), kind, copper
            "#,
        )
        .unwrap();
    assert_eq!((count, kind.as_str(), copper), (2, "money", 10));

    for category in [6_u32, 7] {
        env.state().borrow_mut().cursor_item = Some(CursorInfo::PetAction {
            slot: 1,
            spell_id: 0x00AB_CDEF,
            packed_action: (category << 24) | 0x00AB_CDEF,
        });
        let (count, kind, action_id, book_is_nil, book_index, related_is_nil):
            (i64, String, i64, bool, i64, bool) = env
                .eval(
                    r#"
                    local kind, actionID, bookIndex, relatedID = GetCursorInfo()
                    return select('#', GetCursorInfo()), kind, actionID,
                        bookIndex == nil, bookIndex or 0, relatedID == nil
                    "#,
                )
                .unwrap();
        assert_eq!(count, 4);
        assert_eq!(kind, "petaction");
        assert_eq!(action_id, 0x00AB_CDEF);
        assert!(related_is_nil);
        if cfg!(feature = "client-ptr") {
            assert_eq!((book_is_nil, book_index), (false, 0));
        } else {
            assert!(book_is_nil);
        }
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_action_button_drag_matches_r29_cursor_and_event_order() {
    common::with_timeout(120, move || {
        let env = env_with_action_bar();
        env.exec("SetCVar('lockActionBars', '0')")
            .expect("unlock action bars for drag input");
        env.fire_event("ACTIONBAR_HIDEGRID").unwrap();
        seed_action_slot(&env, 1, 853);
        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(0.0)])
            .unwrap();
        env.fire_event("ACTIONBAR_UPDATE_STATE").unwrap();
        env.exec("if ActionButton1 then ActionButton1.icon:SetTexture(GetActionTexture(1)) end")
            .unwrap();
        let button_id = action_button_id(&env, "ActionButton1");
        install_ptr_action_bar_trace(&env);

        env.fire_script_handler(button_id, "OnDragStart", vec![])
            .unwrap();
        let (count, kind, book_type, spell_id, related_is_nil): (i64, String, String, i64, bool) =
            env.eval(
                r#"
                local kind, bookSlot, bookType, spellID, relatedID = GetCursorInfo()
                return select('#', GetCursorInfo()), kind, bookType, spellID, relatedID == nil
                "#,
            )
            .unwrap();
        assert_eq!(count, 5);
        assert_eq!(kind, "spell");
        assert_eq!(book_type, "spell");
        assert_eq!(spell_id, 853, "spellID is the fourth cursor value");
        assert!(related_is_nil);
        assert!(env.eval::<bool>("return ActionButton1:GetShowGrid()").unwrap());
        assert!(!env.eval::<bool>("return HasAction(1)").unwrap());
        assert_eq!(
            read_ptr_action_bar_trace(&env),
            "ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,3,0,0;ACTIONBAR_SLOT_CHANGED,1"
        );

        env.fire_on_update(0.016).unwrap();
        assert_eq!(
            read_ptr_action_bar_trace(&env),
            "ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,3,0,0;ACTIONBAR_SLOT_CHANGED,1;ACTIONBAR_UPDATE_COOLDOWN"
        );

        env.exec("ResetR29ActionBarEvents()").unwrap();
        env.fire_script_handler(button_id, "OnReceiveDrag", vec![])
            .unwrap();
        assert_eq!(
            env.eval::<i64>("return select('#', GetCursorInfo())").unwrap(),
            0
        );
        assert!(env.eval::<bool>("return HasAction(1)").unwrap());
        assert!(!env.eval::<bool>("return ActionButton1:GetShowGrid()").unwrap());
        assert_eq!(
            read_ptr_action_bar_trace(&env),
            "ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,3,0;ACTIONBAR_SLOT_CHANGED,1"
        );
        env.fire_on_update(0.016).unwrap();
        assert_eq!(
            read_ptr_action_bar_trace(&env),
            "ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,3,0;ACTIONBAR_SLOT_CHANGED,1;ACTIONBAR_UPDATE_COOLDOWN"
        );

        seed_action_slot(&env, 2, 853);
        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(0.0)])
            .unwrap();
        env.fire_event("ACTIONBAR_UPDATE_STATE").unwrap();
        env.exec("if ActionButton2 then ActionButton2.icon:SetTexture(GetActionTexture(2)) end")
            .unwrap();
        let target_id = action_button_id(&env, "ActionButton2");
        env.exec("ResetR29ActionBarEvents()").unwrap();
        env.fire_script_handler(button_id, "OnDragStart", vec![])
            .unwrap();
        env.fire_on_update(0.016).unwrap();
        env.exec("ResetR29ActionBarEvents()").unwrap();
        env.fire_script_handler(target_id, "OnReceiveDrag", vec![])
            .unwrap();
        assert!(!env.eval::<bool>("return HasAction(1)").unwrap());
        assert_eq!(
            env.eval::<i64>("local _, id = GetActionInfo(2); return id")
                .unwrap(),
            853
        );
        assert_eq!(
            env.eval::<i64>("return select('#', GetCursorInfo())").unwrap(),
            0
        );
        assert_eq!(
            read_ptr_action_bar_trace(&env),
            "ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,3,0"
        );
        env.fire_on_update(0.016).unwrap();
        assert_eq!(
            read_ptr_action_bar_trace(&env),
            "ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,3,0"
        );
    });
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_action_button_drag_swaps_occupied_spell_without_cursor_change() {
    common::with_timeout(120, move || {
        let env = env_with_action_bar();
        env.exec("SetCVar('lockActionBars', '0')")
            .expect("unlock action bars for drag input");
        env.fire_event("ACTIONBAR_HIDEGRID").unwrap();
        seed_action_slot(&env, 1, 853);
        seed_action_slot(&env, 2, 642);
        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(0.0)])
            .unwrap();
        env.fire_event("ACTIONBAR_UPDATE_STATE").unwrap();
        env.exec(
            "if ActionButton1 then ActionButton1.icon:SetTexture(GetActionTexture(1)) end; if ActionButton2 then ActionButton2.icon:SetTexture(GetActionTexture(2)) end",
        )
        .unwrap();
        let source = action_button_id(&env, "ActionButton1");
        let target = action_button_id(&env, "ActionButton2");
        install_ptr_action_bar_trace(&env);

        env.fire_script_handler(source, "OnDragStart", vec![])
            .unwrap();
        env.fire_script_handler(target, "OnReceiveDrag", vec![])
            .unwrap();

        assert!(!env.eval::<bool>("return HasAction(1)").unwrap());
        assert_eq!(
            env.eval::<i64>("local _, id = GetActionInfo(2); return id")
                .unwrap(),
            853
        );
        let (count, kind, spell_id): (i64, String, i64) = env
            .eval(
                r#"
                local kind, bookSlot, bookType, spellID = GetCursorInfo()
                return select('#', GetCursorInfo()), kind, spellID
                "#,
            )
            .unwrap();
        assert_eq!((count, kind.as_str(), spell_id), (5, "spell", 642));
        assert_eq!(
            read_ptr_action_bar_trace(&env),
            "ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,3,0,0;ACTIONBAR_SLOT_CHANGED,1;ACTIONBAR_SHOWGRID;ACTIONBAR_SLOT_CHANGED,2"
        );
        env.fire_on_update(0.016).unwrap();
        assert_eq!(
            read_ptr_action_bar_trace(&env),
            "ACTIONBAR_SHOWGRID;CURSOR_CHANGED,false,3,0,0;ACTIONBAR_SLOT_CHANGED,1;ACTIONBAR_SHOWGRID;ACTIONBAR_SLOT_CHANGED,2;ACTIONBAR_UPDATE_COOLDOWN"
        );
    });
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_action_bar_lock_is_checked_by_the_lua_drag_consumer_only() {
    common::with_timeout(120, move || {
        let env = env_with_action_bar();
        env.exec("SetCVar('lockActionBars', '1')")
            .expect("lock action bars");
        assert!(
            env.eval::<bool>("return Settings.GetValue('lockActionBars')")
                .unwrap()
        );
        seed_action_slot(&env, 1, 853);
        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[rilua::Val::Num(0.0)])
            .unwrap();
        env.exec("if ActionButton1 then ActionButton1.icon:SetTexture(GetActionTexture(1)) end")
            .unwrap();
        let button_id = action_button_id(&env, "ActionButton1");

        env.fire_script_handler(button_id, "OnDragStart", vec![])
            .unwrap();
        assert!(env.eval::<bool>("return HasAction(1)").unwrap());
        assert_eq!(
            env.eval::<i64>("return select('#', GetCursorInfo())").unwrap(),
            0
        );

        env.exec("PickupAction(1)")
            .expect("the native pickup wrapper does not inspect the lock setting");
        assert!(!env.eval::<bool>("return HasAction(1)").unwrap());
        env.fire_script_handler(button_id, "OnReceiveDrag", vec![])
            .unwrap();
        assert!(env.eval::<bool>("return HasAction(1)").unwrap());
        assert_eq!(
            env.eval::<i64>("return select('#', GetCursorInfo())").unwrap(),
            0
        );
    });
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tainted_combat_pickup_is_blocked_and_place_clears_the_cursor() {
    let env = WowLuaEnv::new().unwrap();
    seed_action_slot(&env, 1, 853);
    env.state().borrow_mut().player.in_combat = true;
    install_ptr_action_bar_trace(&env);
    env.exec(
        r#"
        TaintedPickupAction = function() PickupAction(1) end
        TaintedPlaceAction = function() PlaceAction(2) end
        debug.setobjecttaint(TaintedPickupAction, 'R29TestAddon')
        debug.setobjecttaint(TaintedPlaceAction, 'R29TestAddon')
        TaintedPickupAction()
        "#,
    )
    .unwrap();
    assert!(env.eval::<bool>("return HasAction(1)").unwrap());
    assert_eq!(
        env.eval::<i64>("return select('#', GetCursorInfo())").unwrap(),
        0
    );
    assert_eq!(
        read_ptr_action_bar_trace(&env),
        "ADDON_ACTION_BLOCKED,R29TestAddon,PickupAction()"
    );

    env.exec("PickupAction(1)")
        .expect("secure pickup is allowed during combat");
    env.fire_on_update(0.016).unwrap();
    env.exec("ResetR29ActionBarEvents(); TaintedPlaceAction()")
        .unwrap();
    assert!(
        !env.eval::<bool>("return HasAction(2)").unwrap(),
        "blocked PlaceAction trace: {}",
        read_ptr_action_bar_trace(&env)
    );
    assert_eq!(
        env.eval::<i64>("return select('#', GetCursorInfo())").unwrap(),
        0
    );
    assert_eq!(
        read_ptr_action_bar_trace(&env),
        "ADDON_ACTION_BLOCKED,R29TestAddon,PlaceAction();ACTIONBAR_HIDEGRID;CURSOR_CHANGED,true,0,3,0"
    );
}

#[cfg(any(feature = "client-ptr", feature = "client-retail"))]
#[test]
fn action_slot_limits_match_the_client_profile() {
    let max_slot = if cfg!(feature = "client-ptr") {
        360
    } else {
        180
    };
    let env = WowLuaEnv::new().unwrap();
    seed_action_slot(&env, max_slot, 853);
    seed_action_slot(&env, max_slot + 1, 642);

    env.exec(&format!("PickupAction({})", max_slot + 1))
        .unwrap();
    assert!(env.eval::<bool>(&format!("return HasAction({})", max_slot + 1)).unwrap());
    assert_eq!(
        env.eval::<i64>("return select('#', GetCursorInfo())").unwrap(),
        0
    );

    env.exec(&format!("PickupAction({max_slot})")).unwrap();
    assert!(!env.eval::<bool>(&format!("return HasAction({max_slot})")).unwrap());
    env.exec(&format!("PlaceAction({})", max_slot + 1)).unwrap();
    let (kind, spell_id): (String, i64) = env
        .eval(&format!(
            "local kind, id = GetActionInfo({}); return kind, id",
            max_slot + 1
        ))
        .unwrap();
    assert_eq!((kind.as_str(), spell_id), ("spell", 642));
    assert_eq!(
        env.eval::<i64>("local _, _, _, spellID = GetCursorInfo(); return spellID")
            .unwrap(),
        853
    );

    env.exec(&format!("PlaceAction({max_slot})")).unwrap();
    assert!(env.eval::<bool>(&format!("return HasAction({max_slot})")).unwrap());
    assert_eq!(
        env.eval::<i64>("return select('#', GetCursorInfo())").unwrap(),
        0
    );
}
