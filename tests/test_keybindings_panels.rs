#![cfg(feature = "gui")]

//! Integration tests for keybinding dispatch — panel interaction tests.
//!
//! Covers spellbook, talents, collections, world map, escape menu, and social panels.

use crate::common;
#[path = "common/keybindings_panels_detail.rs"]
mod keybindings_panels_detail;
#[path = "common/token_ui_fixtures.rs"]
mod token_ui_fixtures;

use keybindings_panels_detail::{frame_is_shown, setup_env};
use wow_ui_sim::client_profile::{self, ClientProfile};
#[cfg(not(feature = "client-wrath"))]
use wow_ui_sim::iced_app::{
    RegistryQuadBatchParams, build_quad_batch_for_registry, compute_frame_rect,
};
use wow_ui_sim::lua_api::WowLuaEnv;
#[cfg(not(feature = "client-wrath"))]
use wow_ui_sim::render::{QuadBatch, QuadVertex, TextureRequest};
#[cfg(not(feature = "client-wrath"))]
use wow_ui_sim::widget::WidgetRegistry;

fn frame_is_visible(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!("return {frame_name} ~= nil and {frame_name}:IsVisible() == true");
    env.eval::<bool>(&code).unwrap_or(false)
}

#[cfg(not(feature = "client-wrath"))]
fn build_batch_for_root(env: &WowLuaEnv, root_name: &str) -> wow_ui_sim::render::QuadBatch {
    {
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
    }
    let buckets = {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let state = env.state().borrow();
    build_quad_batch_for_registry(
        RegistryQuadBatchParams::new(&state.widgets, (1024.0, 768.0), &buckets)
            .root_name(Some(root_name)),
    )
}

#[cfg(not(feature = "client-wrath"))]
fn quad_bounds_from_vertices(verts: &[QuadVertex]) -> (f32, f32, f32, f32) {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for vert in verts {
        min_x = min_x.min(vert.position[0]);
        min_y = min_y.min(vert.position[1]);
        max_x = max_x.max(vert.position[0]);
        max_y = max_y.max(vert.position[1]);
    }
    (min_x, min_y, max_x, max_y)
}

#[cfg(not(feature = "client-wrath"))]
fn quad_bounds(batch: &QuadBatch, request: &TextureRequest) -> (f32, f32, f32, f32) {
    let start = request.vertex_start as usize;
    let end = start + request.vertex_count as usize;
    quad_bounds_from_vertices(&batch.vertices[start..end])
}

#[cfg(not(feature = "client-wrath"))]
fn bounds_match_rect(bounds: (f32, f32, f32, f32), rect: wow_ui_sim::LayoutRect) -> bool {
    let tolerance = 0.1;
    (bounds.0 - rect.x).abs() <= tolerance
        && (bounds.1 - rect.y).abs() <= tolerance
        && (bounds.2 - (rect.x + rect.width)).abs() <= tolerance
        && (bounds.3 - (rect.y + rect.height)).abs() <= tolerance
}

#[cfg(not(feature = "client-wrath"))]
fn spellbook_paged_spells_frame_id(registry: &WidgetRegistry) -> Option<u64> {
    let player_spells_id = registry.get_id_by_name("PlayerSpellsFrame")?;
    let player_spells = registry.get(player_spells_id)?;
    let spellbook_id = *player_spells.children_keys.get("SpellBookFrame")?;
    let spellbook = registry.get(spellbook_id)?;
    spellbook.children_keys.get("PagedSpellsFrame").copied()
}

fn is_wrath() -> bool {
    client_profile::ACTIVE == ClientProfile::Wrath
}

fn record_native_binding_caller(env: &WowLuaEnv, function: &str) {
    env.exec(&format!(
        r#"
        local original = assert(_G[{function:?}])
        _G[{function:?}] = function(...)
            local caller = debug.getinfo(2, 'Sl')
            NativeBindingCaller = caller.source
            NativeBindingLine = caller.currentline
            return original(...)
        end
        "#
    ))
    .expect("instrument native binding target");
}

fn assert_native_binding_caller(env: &WowLuaEnv, line: u32) {
    env.exec(&format!("assert(NativeBindingCaller and NativeBindingCaller:match('FrameXML/Bindings%.xml$') and NativeBindingLine == {line}, 'binding must execute the native XML body on its original line')"))
        .expect("native binding source");
}

// Native Bindings.xml:635-637; SpellBookFrame.lua:14-29.
fn press_spellbook_key(env: &WowLuaEnv) {
    if is_wrath() {
        env.exec("assert(SetBinding('S', 'TOGGLESPELLBOOK')); assert(GetBindingAction('S') == 'TOGGLESPELLBOOK')")
            .expect("bind native spellbook action");
        record_native_binding_caller(env, "ToggleSpellBook");
        assert!(
            !frame_is_shown(env, "SpellBookFrame"),
            "spellbook starts closed"
        );
    }
    env.send_key_press("S", None).expect("S keybind failed");
    if is_wrath() {
        assert_native_binding_caller(env, 636);
        env.exec(
            r#"
            assert(SpellBookFrame:IsShown(), "S must open the native book")
            assert(SpellBookFrame.bookType == BOOKTYPE_SPELL, "S selects spells, not pets")
        "#,
        )
        .expect("native spellbook key transition");
    }
}

fn close_wrath_panel(env: &WowLuaEnv, key: &str, root: &str) {
    if is_wrath() {
        env.send_key_press(key, None)
            .expect("second keypress failed");
        assert!(!frame_is_shown(env, root), "second {key} must close {root}");
    }
}

#[cfg(not(feature = "client-wrath"))]
fn spellbook_root_name() -> &'static str {
    if is_wrath() {
        "SpellBookFrame"
    } else {
        "PlayerSpellsFrame"
    }
}

#[cfg(not(feature = "client-wrath"))]
fn find_first_visible_spellbook_icon_id(registry: &WidgetRegistry) -> Option<u64> {
    if is_wrath() {
        // INFERRED: native SpellButtonTemplate publishes globally named icons.
        return (1..=12).find_map(|index| {
            let id = registry.get_id_by_name(&format!("SpellButton{index}IconTexture"))?;
            let icon = registry.get(id)?;
            let has_texture = icon.texture.is_some() || icon.texture_file_data_id.is_some();
            let rect = compute_frame_rect(registry, id, 1024.0, 768.0);
            let has_area = rect.width > 0.0 && rect.height > 0.0;
            (icon.visible && has_area && has_texture).then_some(id)
        });
    }
    let paged_id = spellbook_paged_spells_frame_id(registry)?;
    let paged = registry.get(paged_id)?;

    paged
        .children
        .iter()
        .copied()
        .find_map(|view_frame_id| find_visible_spellbook_icon_in_view(registry, view_frame_id))
}

#[cfg(not(feature = "client-wrath"))]
fn find_visible_spellbook_icon_in_view(
    registry: &WidgetRegistry,
    view_frame_id: u64,
) -> Option<u64> {
    let view_frame = registry.get(view_frame_id)?;
    if !view_frame.visible {
        return None;
    }

    view_frame
        .children
        .iter()
        .copied()
        .find_map(|item_id| find_visible_spellbook_icon_in_item(registry, item_id))
}

#[cfg(not(feature = "client-wrath"))]
fn find_visible_spellbook_icon_in_item(registry: &WidgetRegistry, item_id: u64) -> Option<u64> {
    let item = registry.get(item_id)?;
    if !item.visible || item.width <= 0.0 || item.height <= 0.0 {
        return None;
    }

    let button_id = *item.children_keys.get("Button")?;
    let button = registry.get(button_id)?;
    let icon_id = *button.children_keys.get("Icon")?;
    let icon = registry.get(icon_id)?;
    if !icon.visible || icon.width <= 0.0 || icon.height <= 0.0 {
        return None;
    }
    if icon.texture.is_some() || icon.texture_file_data_id.is_some() {
        return Some(icon_id);
    }

    None
}

// ── S → PlayerSpellsUtil.ToggleSpellBookFrame() ─────────────────────────

#[test]
fn keybind_s_opens_spellbook() {
    test_timeout! {
        let env = setup_env();
        press_spellbook_key(&env);
        if is_wrath() {
            env.exec(r#"
                assert(SpellBookFrame:IsShown(), "native spellbook should open")
                assert(SpellBookFrame:GetWidth() == 384 and SpellBookFrame:GetHeight() == 512)
                assert(SpellBookFrame:GetScale() == 1, "legacy book fits the viewport")
            "#).expect("native spellbook first-open surface");
            close_wrath_panel(&env, "S", "SpellBookFrame");
            return;
        }
        assert!(
            frame_is_shown(&env, "PlayerSpellsFrame"),
            "PlayerSpellsFrame should be shown after pressing S"
        );
        // ShowUIPanel should scale-to-fit; PlayerSpellsFrame keeps its default strata.
        let scale: f64 = env
            .eval("return PlayerSpellsFrame:GetScale()")
            .expect("GetScale failed");
        assert!(
            scale < 1.0,
            "1618px-wide frame at 1024px screen should be scaled down, got {scale}"
        );
        let strata: String = env
            .eval("return PlayerSpellsFrame:GetFrameStrata()")
            .expect("GetFrameStrata failed");
        assert_eq!(
            strata, "MEDIUM",
            "PlayerSpellsFrame should keep its default MEDIUM strata"
        );
    }
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn keybind_s_renders_spellbook_item_icon_quads_on_first_open() {
    test_timeout! {
        let env = setup_env();
        press_spellbook_key(&env);

        let batch = build_batch_for_root(&env, spellbook_root_name());
        let state = env.state().borrow();
        let registry = &state.widgets;
        let icon_id = find_first_visible_spellbook_icon_id(registry)
            .expect("Spellbook first-open path should expose a visible spell icon frame");
        let icon_rect = compute_frame_rect(registry, icon_id, 1024.0, 768.0);

        let icon_request = batch
            .texture_requests
            .iter()
            .find(|request| bounds_match_rect(quad_bounds(&batch, request), icon_rect))
            .unwrap_or_else(|| {
                panic!(
                    "Spellbook first-open path should emit a textured quad for the first visible spell icon at rect ({}, {}, {}x{}); batch had {} texture requests",
                    icon_rect.x,
                    icon_rect.y,
                    icon_rect.width,
                    icon_rect.height,
                    batch.texture_requests.len()
                )
            });

        assert!(
            icon_request.path.to_ascii_lowercase().contains("icons"),
            "First visible spellbook icon quad should come from an icon texture, got {}",
            icon_request.path
        );
        drop(state);
        close_wrath_panel(&env, "S", "SpellBookFrame");
    }
}

#[cfg(not(feature = "client-wrath"))]
#[test]
fn keybind_s_keeps_spellbook_backgrounds_below_icons_after_initial_bucket_build() {
    test_timeout! {
        let env = setup_env();
        {
            let mut state = env.state().borrow_mut();
            let _ = state.get_strata_buckets();
        }

        press_spellbook_key(&env);

        let batch = build_batch_for_root(&env, spellbook_root_name());
        let state = env.state().borrow();
        let registry = &state.widgets;
        let icon_id = find_first_visible_spellbook_icon_id(registry)
            .expect("Spellbook first-open path should expose a visible spell icon frame");
        let icon_rect = compute_frame_rect(registry, icon_id, 1024.0, 768.0);

        let icon_request = batch
            .texture_requests
            .iter()
            .find(|request| bounds_match_rect(quad_bounds(&batch, request), icon_rect))
            .expect("Spellbook first-open path should emit a textured quad for the first visible spell icon");

        let last_background_request = batch
            .texture_requests
            .iter()
            .filter(|request| {
                request
                    .path
                    .to_ascii_lowercase()
                    .contains(if is_wrath() {
                        "ui-spellbookpanel-"
                    } else {
                        "spellbookbackgroundevergreen"
                    })
            })
            .max_by_key(|request| request.vertex_start)
            .expect("Spellbook should emit evergreen parchment background quads");

        assert!(
            last_background_request.vertex_start < icon_request.vertex_start,
            "Spellbook parchment backgrounds must render before spell icons after LoD open with prebuilt strata buckets; last background vertex_start={} icon vertex_start={}",
            last_background_request.vertex_start,
            icon_request.vertex_start
        );
        drop(state);
        close_wrath_panel(&env, "S", "SpellBookFrame");
    }
}

#[cfg(feature = "client-wrath")]
#[test]
fn native_action_button_binding_tracks_down_and_up() {
    test_timeout! {
        let env = setup_env();
        env.exec("assert(SetBinding('1', 'ACTIONBUTTON1')); assert(GetBindingAction('1') == 'ACTIONBUTTON1'); assert(ActionButton1:GetButtonState() == 'NORMAL')")
            .expect("bind native action button and check initial state");
        record_native_binding_caller(&env, "ActionButtonDown");
        record_native_binding_caller(&env, "ActionButtonUp");

        for spell_id in [19750, 82326] {
            {
                let mut state = env.state().borrow_mut();
                state.action_bars.insert(1, spell_id);
                state.casting = None;
                state.gcd = None;
            }
            env.send_key_down("1", None).expect("press action key");
            assert_native_binding_caller(&env, 126);
            env.exec("assert(ActionButton1:GetButtonState() == 'PUSHED')")
                .expect("native key-down pushes action button");

            env.send_key_up("1");
            assert_native_binding_caller(&env, 128);
            env.exec("assert(ActionButton1:GetButtonState() == 'NORMAL')")
                .expect("native key-up releases action button");
            let state = env.state().borrow();
            assert_eq!(state.casting.as_ref().map(|cast| cast.spell_id), Some(spell_id));
        }
    }
}

// ── N → PlayerSpellsUtil.ToggleClassTalentFrame() ───────────────────────

#[cfg(not(feature = "client-wrath"))]
#[test]
fn keybind_n_opens_talents() {
    test_timeout! {
        let env = setup_env();
        env.send_key_press("N", None).expect("N keybind failed");
        assert!(
            frame_is_shown(&env, "PlayerSpellsFrame"),
            "PlayerSpellsFrame should be shown after pressing N (talents tab)"
        );
        assert!(
            !frame_is_shown(&env, "ClassTalentLoadoutImportDialog"),
            "ClassTalentLoadoutImportDialog should stay hidden until the Import action is clicked"
        );
        assert!(
            !frame_is_shown(&env, "ClassTalentLoadoutCreateDialog"),
            "ClassTalentLoadoutCreateDialog should stay hidden until the New Loadout action is clicked"
        );
        assert!(
            !frame_is_visible(&env, "ClassTalentLoadoutImportDialogImportControl"),
            "Import dialog content should not become visible when opening the talents tab"
        );
        assert!(
            !frame_is_visible(&env, "ClassTalentLoadoutImportDialogNameControl"),
            "Import dialog name control should not become visible when opening the talents tab"
        );
        assert!(
            !frame_is_visible(&env, "ClassTalentLoadoutCreateDialogNameControl"),
            "Create dialog content should not become visible when opening the talents tab"
        );
    }
}

post_wrath_ui_fixture! {
#[test]
fn hidden_talent_dialogs_do_not_emit_quads_after_opening_talents() {
    test_timeout! {
        let env = setup_env();
        wow_ui_sim::startup::settle_headless_startup(&env);
        env.send_key_press("N", None).expect("N keybind failed");
        wow_ui_sim::startup::run_extra_update_ticks(&env, 3);

        let import_batch = build_batch_for_root(&env, "ClassTalentLoadoutImportDialog");
        assert_eq!(
            import_batch.quad_count(),
            12,
            "hidden import dialog should not emit quads"
        );
        assert_eq!(
            import_batch.texture_requests.len(),
            1,
            "hidden import dialog should only contribute the tiled background"
        );

        let hero_batch = build_batch_for_root(&env, "HeroTalentsSelectionDialog");
        assert_eq!(
            hero_batch.quad_count(),
            12,
            "hidden hero talents dialog should not emit quads"
        );
        assert_eq!(
            hero_batch.texture_requests.len(),
            1,
            "hidden hero talents dialog should only contribute the tiled background"
        );
    }
}
}

// ── A → ToggleAchievementFrame() ────────────────────────────────────────

#[test]
fn keybind_a_opens_achievements() {
    test_timeout! {
        let env = setup_env();
        env.send_key_press("A", None).expect("A keybind failed");
        assert!(
            frame_is_shown(&env, "AchievementFrame"),
            "AchievementFrame should be shown after pressing A"
        );
    }
}

// ── L → PVEFrame_ToggleFrame() ──────────────────────────────────────────

#[test]
fn keybind_l_opens_group_finder() {
    test_timeout! {
        let env = setup_env();
        if is_wrath() {
            env.exec("assert(SetBinding('L', 'TOGGLELFGPARENT')); assert(GetBindingAction('L') == 'TOGGLELFGPARENT')").expect("bind native dungeon finder action");
            record_native_binding_caller(&env, "ToggleLFDParentFrame");
            assert!(!frame_is_shown(&env, "LFDParentFrame"), "dungeon finder starts closed");
            env.send_key_press("L", None).expect("L keybind failed");
            assert_native_binding_caller(&env, 695);
            assert!(frame_is_shown(&env, "LFDParentFrame"), "native dungeon finder should open");
            close_wrath_panel(&env, "L", "LFDParentFrame");
            return;
        }
        env.exec(
            r#"
            assert(C_AddOns.LoadAddOn("Blizzard_GroupFinder"))
            _G.__group_finder_triggered = false
            local original_toggle_group_finder = PVEFrame_ToggleFrame
            PVEFrame_ToggleFrame = function(...)
                _G.__group_finder_triggered = true
                return original_toggle_group_finder(...)
            end
            "#
        ).expect("Failed to wrap PVEFrame_ToggleFrame for L keybind test");
        env.send_key_press("L", None).expect("L keybind failed");
        let triggered: bool = env
            .eval("return _G.__group_finder_triggered == true")
            .expect("Failed to read L keybind trigger flag");
        assert!(
            triggered,
            "Pressing L should dispatch PVEFrame_ToggleFrame"
        );
        let premade_enabled: bool = env
            .eval(
                r#"
                return GroupFinderFrame ~= nil
                    and GroupFinderFrame.groupButton3 ~= nil
                    and GroupFinderFrame.groupButton3:IsEnabled() == true
                "#,
            )
            .expect("Failed to read Premade Groups button enabled state");
        assert!(
            premade_enabled,
            "Premade Groups should be enabled after opening Group Finder"
        );
    }
}

post_wrath_ui_fixture! {
#[test]
fn raid_finder_group_button_path_handles_empty_rf_dungeon_list() {
    test_timeout! {
        let env = setup_env();
        env.exec(
            r#"
            assert(C_AddOns.LoadAddOn("Blizzard_GroupFinder"))
            GroupFinderFrame_ShowGroupFrame(RaidFinderFrame)
            assert(RaidFinderFrame:IsShown(), "RaidFinderFrame should be shown")
            "#
        ).expect("Raid Finder group button path should not raise Lua errors");
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn premade_group_category_buttons_can_be_clicked() {
    test_timeout! {
        let env = setup_env();
        env.exec(
            r#"
            assert(C_AddOns.LoadAddOn("Blizzard_GroupFinder"))
            LFGListPVEStub_OnShow(LFGListPVEStub)
            LFGListFrame_SetActivePanel(LFGListFrame, LFGListFrame.CategorySelection)
            assert(LFGListFrame.CategorySelection:IsShown(), "category selection should be shown")

            local clicked = 0
            for _, button in ipairs(LFGListFrame.CategorySelection.CategoryButtons) do
                if button:IsShown() then
                    button:GetScript("OnClick")(button)
                    assert(
                        LFGListFrame.CategorySelection.selectedCategory == button.categoryID,
                        "click should select category " .. tostring(button.categoryID)
                    )
                    clicked = clicked + 1
                end
            end

            assert(clicked > 0, "expected at least one visible premade category button")
            "#
        ).expect("Premade Group category buttons should click without Lua errors");
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn premade_group_categories_can_start_search() {
    test_timeout! {
        let env = setup_env();
        env.exec(
            r#"
            assert(C_AddOns.LoadAddOn("Blizzard_GroupFinder"))
            LFGListPVEStub_OnShow(LFGListPVEStub)
            LFGListFrame_SetActivePanel(LFGListFrame, LFGListFrame.CategorySelection)

            C_LFGList.Search(2)
            local count, results = C_LFGList.GetSearchResults()
            assert(count > 0 and C_LFGList.GetSearchResultInfo(results[1]))
            LFGListSearchPanel_Clear(LFGListFrame.SearchPanel)
            assert(select(1, C_LFGList.GetSearchResults()) == 0)
            assert(not C_LFGList.GetSearchResultInfo(results[1]))
            C_LFGList.Search(2)
            assert(select(1, C_LFGList.GetSearchResults()) == count)

            for _, button in ipairs(LFGListFrame.CategorySelection.CategoryButtons) do
                if button:IsShown() then
                    LFGListFrame_SetActivePanel(LFGListFrame, LFGListFrame.CategorySelection)
                    button:GetScript("OnClick")(button)
                    LFGListFrame.CategorySelection.FindGroupButton:GetScript("OnClick")(
                        LFGListFrame.CategorySelection.FindGroupButton
                    )
                    assert(LFGListFrame.SearchPanel:IsShown(), "search panel should be shown")
                end
            end
            "#
        ).expect("Premade Group categories should start search without Lua errors");
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn premade_group_result_update_refreshes_selected_listing() {
    test_timeout! {
        let env = setup_env();
        env.exec(r#"
            assert(C_AddOns.LoadAddOn('Blizzard_GroupFinder'))
            LFGListPVEStub_OnShow(LFGListPVEStub)
            LFGListFrame_SetActivePanel(LFGListFrame, LFGListFrame.CategorySelection)
            local category
            for _, button in ipairs(LFGListFrame.CategorySelection.CategoryButtons) do
                if button:IsShown() then category = button; break end
            end
            assert(category, 'expected a visible category')
            category:GetScript('OnClick')(category)
            local find = LFGListFrame.CategorySelection.FindGroupButton
            find:GetScript('OnClick')(find)
            local panel = LFGListFrame.SearchPanel
            assert(LFGListFrame.activePanel == panel)
            local source = debug.getinfo(LFGListSearchPanel_OnEvent, 'S').source:gsub('\\', '/')
            assert(source:find('Blizzard_GroupFinder/', 1, true), source)
            C_LFGList.Search(2)
            assert(C_LFGList.GetSearchResultInfo(1).numMembers == 3)
            local count = select(1, C_LFGList.GetSearchResults())
            local events = 0
            local observer = CreateFrame('Frame')
            observer:RegisterEvent('LFG_LIST_SEARCH_RESULT_UPDATED')
            observer:SetScript('OnEvent', function(_, _, id)
                assert(id == 1)
                events = events + 1
            end)
            A_Admin.UpdatePremadeListing(1, 'numMembers', 5)
            assert(C_LFGList.GetSearchResultInfo(1).numMembers == 5)
            assert(events == 1, 'listing update event was not delivered')
            panel.selectedResult = 1
            A_Admin.UpdatePremadeListing(1, 'isDelisted', true)
            assert(C_LFGList.GetSearchResultInfo(1).isDelisted)
            assert(C_LFGList.HasSearchResultInfo(1), 'delisted result must remain in the set')
            assert(panel.selectedResult == nil, 'original OnEvent must clear a delisted selection')
            assert(events == 2, 'delisting update event was not delivered')
            A_Admin.AddPremadeListing('New group', 'new', 1195, 2, 5)
            assert(select(1, C_LFGList.GetSearchResults()) == count)
            assert(events == 2, 'a new catalog listing must not emit a result update')
        "#).expect("Premade result updates should reach the original search panel handler");
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn premade_group_search_text_filter_and_clear() {
    test_timeout! {
        let env = setup_env();
        env.exec(r#"
            assert(C_AddOns.LoadAddOn('Blizzard_GroupFinder'))
            LFGListPVEStub_OnShow(LFGListPVEStub)
            LFGListFrame_SetActivePanel(LFGListFrame, LFGListFrame.CategorySelection)
            local category
            for _, button in ipairs(LFGListFrame.CategorySelection.CategoryButtons) do
                if button:IsShown() then category = button; break end
            end
            assert(category)
            category:GetScript('OnClick')(category)
            local find = LFGListFrame.CategorySelection.FindGroupButton
            find:GetScript('OnClick')(find)
            local panel = LFGListFrame.SearchPanel
            local button = panel.AutoCompleteFrame.Results[1]
            assert(button and button:GetScript('OnClick'))
            button.activityID = 1195
            button:GetScript('OnClick')(button)
            local count, ids = C_LFGList.GetSearchResults()
            assert(count == 1 and ids[1] == 1, 'activity 1195 filter')
            button.activityID = 1188
            button:GetScript('OnClick')(button)
            count, ids = C_LFGList.GetSearchResults()
            assert(count == 1 and ids[1] == 2, 'activity 1188 filter')
            C_LFGList.SetSearchToQuestID(987654)
            C_LFGList.Search(2)
            assert(select(1, C_LFGList.GetSearchResults()) == 0, 'quest filter must replace activity filter')
            LFGListSearchPanel_Clear(panel)
            C_LFGList.Search(2)
            assert(select(1, C_LFGList.GetSearchResults()) == 8, 'clear must reset search text fields')
        "#).expect("Original GroupFinder search text controls should filter and clear results");
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn premade_group_search_result_tooltips_do_not_error() {
    test_timeout! {
        let env = setup_env();
        env.exec(
            r#"
            assert(C_AddOns.LoadAddOn("Blizzard_GroupFinder"))
            -- Blizzard_GroupFinder/Mainline/LFGList.lua:2817 starts the search before reading results.
            C_LFGList.Search(2)
            local _, results = C_LFGList.GetSearchResults()
            assert(#results > 0, "expected seeded premade search results")

            for _, resultID in ipairs(results) do
                LFGListUtil_SetSearchEntryTooltip(GameTooltip, resultID)
            end
            local low = C_ChallengeMode.GetDungeonScoreRarityColor(0)
            local high = C_ChallengeMode.GetDungeonScoreRarityColor(2200)
            assert(low.r ~= high.r or low.g ~= high.g or low.b ~= high.b, "score colors must vary")
            "#
        ).expect("Premade Group search result tooltips should not raise Lua errors");
    }
}
}

// ── O → ToggleFriendsFrame() ────────────────────────────────────────────

#[test]
fn keybind_o_opens_social() {
    test_timeout! {
        let env = setup_env();
        if is_wrath() {
            common::panel_fixtures::clear_recorded_lua_errors(&env);
        }
        env.send_key_press("O", None).expect("O keybind failed");
        if is_wrath() {
            let errors = common::panel_fixtures::recorded_lua_errors(&env);
            assert!(errors.is_empty(), "native friends keybind errors: {errors:?}");
        }
        assert!(
            frame_is_shown(&env, "FriendsFrame"),
            "FriendsFrame should be shown after pressing O"
        );
    }
}

#[test]
fn keybind_o_dispatches_toggle_friends_frame() {
    test_timeout! {
        let env = setup_env();
        if is_wrath() {
            common::panel_fixtures::clear_recorded_lua_errors(&env);
        }
        if !is_wrath() {
            env.exec(r#"assert(C_AddOns.LoadAddOn("Blizzard_FriendsFrame"))"#)
                .expect("modern FriendsFrame addon should load");
        }
        env.exec(
            r#"
            _G.__friends_keybind_triggered = false
            local original_toggle_friends_frame = ToggleFriendsFrame
            ToggleFriendsFrame = function(...)
                _G.__friends_keybind_triggered = true
                return original_toggle_friends_frame(...)
            end
            "#
        ).expect("Failed to wrap ToggleFriendsFrame for O keybind test");
        env.send_key_press("O", None).expect("O keybind failed");
        if is_wrath() {
            let errors = common::panel_fixtures::recorded_lua_errors(&env);
            assert!(errors.is_empty(), "native friends dispatch errors: {errors:?}");
        }
        let triggered: bool = env
            .eval("return _G.__friends_keybind_triggered == true")
            .expect("Failed to read O keybind trigger flag");
        assert!(
            triggered,
            "Pressing O should dispatch ToggleFriendsFrame"
        );
    }
}

// ── J → ToggleGuildFrame() ──────────────────────────────────────────────

#[test]
fn keybind_j_opens_guild() {
    test_timeout! {
        let env = setup_env();
        if is_wrath() {
            env.exec("assert(SetBinding('J', 'TOGGLEGUILDTAB')); assert(GetBindingAction('J') == 'TOGGLEGUILDTAB')").expect("bind native guild action");
            record_native_binding_caller(&env, "ToggleFriendsFrame");
            assert!(!frame_is_shown(&env, "FriendsFrame"), "friends starts closed");
            env.send_key_press("J", None).expect("J keybind failed");
            assert_native_binding_caller(&env, 686);
            assert!(frame_is_visible(&env, "GuildFrame"), "native guild tab should be visible");
            assert!(frame_is_shown(&env, "FriendsFrame"), "native guild tab belongs to FriendsFrame");
            env.exec("assert(FriendsFrame.selectedTab == 3, 'J selects the guild tab')")
                .expect("native Bindings.xml:685-687 guild selection");
            close_wrath_panel(&env, "J", "FriendsFrame");
            return;
        }
        env.exec(
            r#"
            _G.__guild_keybind_triggered = false
            local original_toggle_guild_frame = ToggleGuildFrame
            ToggleGuildFrame = function(...)
                _G.__guild_keybind_triggered = true
                return original_toggle_guild_frame(...)
            end
            "#
        ).expect("Failed to wrap ToggleGuildFrame for J keybind test");
        env.send_key_press("J", None).expect("J keybind failed");
        let triggered: bool = env
            .eval("return _G.__guild_keybind_triggered == true")
            .expect("Failed to read J keybind trigger flag");
        assert!(
            triggered,
            "Pressing J should dispatch ToggleGuildFrame"
        );
    }
}
