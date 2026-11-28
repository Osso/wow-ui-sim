#![cfg(all(feature = "gui", not(feature = "client-wrath")))]
// Wrath FrameXML/SpellBookFrame.xml:83-105,195,404 uses fixed SpellButtonTemplate
// children, not Mainline PagedSpellsFrame or SpellBookFrameTutorialsMixin
// (Blizzard_PlayerSpells/SpellBook/Blizzard_SpellBookFrame.lua:34,51-52).
//! Tests for spellbook geometry, layout locking, and tooltip rendering.

use crate::common;
#[path = "spellbook/common.rs"]
mod spellbook_common;

use std::path::PathBuf;

use spellbook_common::*;
use wow_ui_sim::iced_app::{
    RegistryQuadBatchParams, build_quad_batch_for_registry, compute_frame_rect,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::WidgetRegistry;

type PassiveItemGeometry = (
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
);

struct ExpectedRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn spellbook_tutorials_lua_path() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PlayerSpells/SpellBook/Blizzard_SpellBookFrameTutorials.lua")
}

fn restore_spellbook_tutorials(env: &WowLuaEnv) {
    let source = std::fs::read_to_string(spellbook_tutorials_lua_path())
        .expect("Failed to read spellbook tutorials source");
    env.exec(&source)
        .expect("Failed to restore spellbook tutorials mixin");
}

#[cfg(not(feature = "client-wrath"))]
fn hover_first_spell_button(env: &WowLuaEnv) -> (String, f32, f32, f32, f32) {
    env.eval(
        r#"
        local paged = PlayerSpellsFrame and PlayerSpellsFrame.SpellBookFrame and PlayerSpellsFrame.SpellBookFrame.PagedSpellsFrame
        assert(paged, "PagedSpellsFrame should exist")

        for _, frame in paged:EnumerateFrames() do
            if frame
                and frame:IsShown()
                and frame.HasValidData
                and frame:HasValidData()
                and frame.spellBookItemInfo
                and frame.spellBookItemInfo.spellID
                and frame.Button
                and frame.Button:IsShown()
            then
                local onEnter = frame.Button:GetScript("OnEnter")
                assert(onEnter, "Spellbook button should have an OnEnter handler")
                onEnter(frame.Button)
                return frame.spellBookItemInfo.name, frame.Button:GetLeft(), frame.Button:GetBottom(), frame.Button:GetRight(), frame.Button:GetTop()
            end
        end

        local entries = {}
        for _, frame in paged:EnumerateFrames() do
            local info = frame.spellBookItemInfo
            entries[#entries + 1] = string.format('shown=%s valid=%s spell=%s button=%s',
                tostring(frame:IsShown()), tostring(frame.HasValidData and frame:HasValidData()),
                tostring(info and info.spellID), tostring(frame.Button and frame.Button:IsShown()))
        end
        error("No visible spellbook spell with tooltip data; " .. table.concat(entries, '; ')
            .. '; first=' .. tostring(C_SpellBook.GetSpellBookItemInfo(1, Enum.SpellBookSpellBank.Player))
            .. '; pageShown=' .. tostring(paged:IsShown()))
        "#,
    )
    .expect("Failed to hover a visible spellbook button")
}

fn build_quads_with_textures(env: &WowLuaEnv) -> (usize, Vec<String>) {
    let buckets = build_strata_buckets(env);
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(RegistryQuadBatchParams::new(
        &state.widgets,
        (1024.0, 768.0),
        &buckets,
    ));
    let textures: Vec<String> = batch
        .texture_requests
        .iter()
        .map(|r| r.path.clone())
        .collect();
    (batch.quad_count(), textures)
}

fn find_first_passive_spell_item(
    registry: &WidgetRegistry,
    item_ids: &[u64],
) -> Option<(u64, u64, u64, u64)> {
    item_ids.iter().find_map(|&item_id| {
        let item = registry.get(item_id)?;
        let &button_id = item.children_keys.get("Button")?;
        let button = registry.get(button_id)?;
        let &icon_id = button.children_keys.get("Icon")?;
        let &border_id = button.children_keys.get("Border")?;
        let &mask_id = button.children_keys.get("IconMask")?;
        let border = registry.get(border_id)?;
        (border.atlas.as_deref() == Some("talents-node-circle-gray"))
            .then_some((button_id, icon_id, border_id, mask_id))
    })
}

fn check_rect(registry: &WidgetRegistry, name: &str, sw: f32, sh: f32, expected: ExpectedRect) {
    let id = registry
        .get_id_by_name(name)
        .unwrap_or_else(|| panic!("Frame '{name}' not found"));
    let rect = compute_frame_rect(registry, id, sw, sh);
    let tol = 2.0;
    assert!(
        (rect.x - expected.x).abs() <= tol
            && (rect.y - expected.y).abs() <= tol
            && (rect.width - expected.width).abs() <= tol
            && (rect.height - expected.height).abs() <= tol,
        "{name}: expected ({}, {}, {}x{}), got ({}, {}, {}x{})",
        expected.x,
        expected.y,
        expected.width,
        expected.height,
        rect.x,
        rect.y,
        rect.width,
        rect.height
    );
}

post_wrath_ui_fixture! {
const PASSIVE_ITEM_GEOMETRY: &str = r#"
                local paged = assert(PlayerSpellsFrame and PlayerSpellsFrame.SpellBookFrame and PlayerSpellsFrame.SpellBookFrame.PagedSpellsFrame, "missing paged spells frame")
                for _, frame in paged:EnumerateFrames() do
                    if frame
                        and frame:IsShown()
                        and frame.HasValidData
                        and frame:HasValidData()
                        and frame.spellBookItemInfo
                        and frame.spellBookItemInfo.isPassive
                        and frame.Button
                        and frame.Button.Icon
                        and frame.Button.Border
                        and frame.Button.IconMask
                    then
                        local _, _, bw, bh = frame.Button:GetRect()
                        local _, _, iw, ih = frame.Button.Icon:GetRect()
                        local _, _, rw, rh = frame.Button.Border:GetRect()
                        local _, _, mw, mh = frame.Button.IconMask:GetRect()
                        local _, _, sbw, sbh = frame.Button:GetScaledRect()
                        local _, _, siw, sih = frame.Button.Icon:GetScaledRect()
                        local _, _, srw, srh = frame.Button.Border:GetScaledRect()
                        local _, _, smw, smh = frame.Button.IconMask:GetScaledRect()
                        return bw, bh, iw, ih, rw, rh, mw, mh, sbw, sbh, siw, sih, srw, srh, smw, smh
                    end
                end
                error("no passive item found")
                "#;

fn open_passive_spell_page(env: &WowLuaEnv) {
    if cfg!(feature = "client-wowforever") {
        env.exec("SetCVar('spellBookHidePassives', false)").unwrap();
    }
    open_spellbook(env);
    if cfg!(feature = "client-wowforever") {
        env.exec(include_str!("fixtures/forever/spellbook_passive_page.lua"))
            .expect("original Camelot category and paging buttons must reach the passive spells");
    }
}

fn assert_passive_lua_geometry(geometry: PassiveItemGeometry) -> [(f32, f32); 4] {
    // Lua returns button, icon, border and mask pairs, first logical then scaled.
    // Shared SpellBookItem.xml:107-154,201-210 and SpellBookItem.lua:704-722.
    for (label, width, height, expected) in [
        ("button", geometry.0, geometry.1, 40.0),
        ("icon", geometry.2, geometry.3, 36.0),
        ("border", geometry.4, geometry.5, 40.0),
        ("mask", geometry.6, geometry.7, 36.0),
    ] {
        assert!(
            (width - expected).abs() <= 0.1,
            "Passive {label} GetRect width should be {expected}, got {width}"
        );
        assert!(
            (height - expected).abs() <= 0.1,
            "Passive {label} GetRect height should be {expected}, got {height}"
        );
    }
    [
        (geometry.8, geometry.9),
        (geometry.10, geometry.11),
        (geometry.12, geometry.13),
        (geometry.14, geometry.15),
    ]
}

fn passive_item_render_rects(registry: &WidgetRegistry) -> [wow_ui_sim::LayoutRect; 4] {
    let item_ids = find_spell_item_ids(registry);
    assert!(!item_ids.is_empty(), "Should have spell items");
    let (button, icon, border, mask) = find_first_passive_spell_item(registry, &item_ids)
        .expect("Should find a passive spellbook item with the circle border art set");
    [button, icon, border, mask].map(|id| compute_frame_rect(registry, id, 1024.0, 768.0))
}

fn assert_passive_render_dimensions(rects: &[wow_ui_sim::LayoutRect; 4], scaled: [(f32, f32); 4]) {
    for ((label, rect), (width, height)) in ["button", "icon", "border", "mask"]
        .into_iter()
        .zip(rects)
        .zip(scaled)
    {
        assert!(
            (rect.width - width).abs() <= 0.1,
            "Rendered passive {label} width should match GetScaledRect: render={} lua={width}",
            rect.width
        );
        assert!(
            (rect.height - height).abs() <= 0.1,
            "Rendered passive {label} height should match GetScaledRect: render={} lua={height}",
            rect.height
        );
    }
}

fn request_at_rect<'a>(
    batch: &wow_ui_sim::render::QuadBatch,
    requests: &'a [wow_ui_sim::render::TextureRequest],
    rect: wow_ui_sim::LayoutRect,
    label: &str,
) -> &'a wow_ui_sim::render::TextureRequest {
    requests
        .iter()
        .find(|request| bounds_match_rect(quad_bounds(batch, request), rect))
        .unwrap_or_else(|| panic!("Passive {label} should emit a quad matching its layout rect"))
}

fn assert_passive_texture_requests(
    batch: &wow_ui_sim::render::QuadBatch,
    rects: [wow_ui_sim::LayoutRect; 4],
) {
    let [_, icon, border, mask] = rects;
    let border_request = request_at_rect(batch, &batch.texture_requests, border, "border");
    let icon_request = request_at_rect(batch, &batch.texture_requests, icon, "icon");
    let mask_request = request_at_rect(batch, &batch.mask_texture_requests, mask, "icon mask");
    assert!(
        border_request
            .path
            .contains(r"Interface\talentframe\talents"),
        "Passive border should come from the talents atlas, got {}",
        border_request.path
    );
    assert!(
        mask_request
            .path
            .contains(r"Interface\talentframe\talentsmasknodecircle"),
        "Passive mask should come from the circle mask texture, got {}",
        mask_request.path
    );
    assert_ne!(
        icon_request.path, border_request.path,
        "Passive icon and border should not collapse onto the same textured quad request"
    );
}

#[test]
fn spellbook_passive_item_border_and_icon_match_master_geometry() {
    test_timeout! {
        let env = setup_full_ui();
        open_passive_spell_page(&env);
        let geometry: PassiveItemGeometry = env.eval(PASSIVE_ITEM_GEOMETRY)
            .expect("Passive spell item geometry should be queryable from Lua");
        let scaled = assert_passive_lua_geometry(geometry);
        let buckets = build_strata_buckets(&env);
        let state = env.state().borrow();
        let registry = &state.widgets;
        let rects = passive_item_render_rects(registry);
        assert_passive_render_dimensions(&rects, scaled);
        let batch = build_quad_batch_for_registry(RegistryQuadBatchParams::new(
            registry, (1024.0, 768.0), &buckets,
        ));
        assert_passive_texture_requests(&batch, rects);
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn spellbook_texture_requests_match_between_opens() {
    test_timeout! {
        let env = setup_full_ui();
        open_spellbook(&env);

        let (q1, tex1) = build_quads_with_textures(&env);
        let icon_tex1: Vec<_> = tex1.iter()
            .filter(|t| t.to_lowercase().contains("icons"))
            .collect();

        // Close and reopen
        env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()").unwrap();
        let _ = env.process_timers();
        env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()").unwrap();
        let _ = env.process_timers();

        let (q2, tex2) = build_quads_with_textures(&env);
        let icon_tex2: Vec<_> = tex2.iter()
            .filter(|t| t.to_lowercase().contains("icons"))
            .collect();

        eprintln!("First open: {} quads, {} textures, {} icon textures",
            q1, tex1.len(), icon_tex1.len());
        eprintln!("Second open: {} quads, {} textures, {} icon textures",
            q2, tex2.len(), icon_tex2.len());

        // Show icon textures unique to second open
        let set1: std::collections::HashSet<_> = icon_tex1.iter().collect();
        let new_icons: Vec<_> = icon_tex2.iter()
            .filter(|t| !set1.contains(t))
            .collect();
        if !new_icons.is_empty() {
            eprintln!("NEW icon textures on second open: {:?}", &new_icons[..new_icons.len().min(5)]);
        }

        assert_eq!(icon_tex1.len(), icon_tex2.len(),
            "Should have same icon texture count between opens");
    }
}
}

post_wrath_ui_fixture! {
fn expected_spellbook_panel_rect(sw: f32, sh: f32) -> ExpectedRect {
    if cfg!(feature = "client-wowforever") {
        // Camelot/Blizzard_PlayerSpellsFrame.xml:5-12 and .lua:37-54,94-110;
        // Blizzard_PlayerSpellsRegistration.lua:4-11 and SharedXMLBase/FrameUtil.lua:169-178.
        let scale = 1.0_f32.min(sw / (577.0 + 200.0)).min(sh / (720.0 + 140.0));
        let height = 720.0 * scale;
        // UIParentPanelManager.lua:244-258,656-681,1007-1020; UIPanelLayoutFrame.lua:1-9.
        // The first center-slot panel changes its area to left during auto-minimize.
        let y = (116.0_f32 - 75.0).min(sh - 140.0 - height).max(10.0);
        return ExpectedRect {
            x: 16.0 + 32.0,
            y,
            width: 577.0 * scale,
            height,
        };
    }
    ExpectedRect {
        x: 56.3,
        y: 41.0,
        width: 911.3,
        height: 497.4,
    }
}

fn assert_forever_spellbook_child_rect(
    registry: &WidgetRegistry,
    panel: &wow_ui_sim::widget::Frame,
    book: &wow_ui_sim::widget::Frame,
    screen: (f32, f32),
) {
    // Camelot/Blizzard_PlayerSpellsFrame.xml:51-59: 574x702 minimized,
    // BOTTOMLEFT +4. The shared mixin changes only the width.
    assert_eq!((book.width, book.height), (574.0, 702.0));
    let panel_rect = compute_frame_rect(registry, panel.id, screen.0, screen.1);
    let book_rect = compute_frame_rect(registry, book.id, screen.0, screen.1);
    let scale = panel_rect.width / 577.0;
    assert!((book_rect.x - panel_rect.x).abs() <= 1.0);
    assert!((book_rect.y - (panel_rect.y + (720.0 - 702.0 - 4.0) * scale)).abs() <= 1.0);
    assert!((book_rect.width - 574.0 * scale).abs() <= 1.0);
    assert!((book_rect.height - 702.0 * scale).abs() <= 1.0);
}

fn assert_spellbook_child_size(
    registry: &WidgetRegistry,
    panel: &wow_ui_sim::widget::Frame,
    sw: f32,
    sh: f32,
) {
    let sb = registry.get(panel.children_keys["SpellBookFrame"]).unwrap();
    if cfg!(feature = "client-wowforever") {
        assert_forever_spellbook_child_rect(registry, panel, sb, (sw, sh));
        return;
    }
    assert!(
        sb.width > 900.0,
        "SpellBookFrame stored width {} should be > 900",
        sb.width
    );
    assert!(
        sb.height > 500.0,
        "SpellBookFrame stored height {} should be > 500",
        sb.height
    );
}

fn assert_spellbook_border(
    registry: &WidgetRegistry,
    panel: &wow_ui_sim::widget::Frame,
    sw: f32,
    sh: f32,
) {
    let nine_id = panel.children_keys["NineSlice"];
    let nine_rect = compute_frame_rect(registry, nine_id, sw, sh);
    let psf_rect = compute_frame_rect(registry, panel.id, sw, sh);
    assert!(
        (nine_rect.x - psf_rect.x).abs() <= 1.0,
        "NineSlice x should match PlayerSpellsFrame"
    );
    assert!(
        (nine_rect.width - psf_rect.width).abs() <= 1.0,
        "NineSlice width should match"
    );
}

fn assert_spellbook_tabs(
    registry: &WidgetRegistry,
    panel: &wow_ui_sim::widget::Frame,
    sw: f32,
    sh: f32,
) {
    if cfg!(feature = "client-wowforever") {
        // Camelot/Blizzard_PlayerSpellsFrame.lua:57-62: no outer tab system.
        let tabs = registry.get(panel.children_keys["TabSystem"]).unwrap();
        assert!(!tabs.visible, "Camelot outer TabSystem should be hidden");
        return;
    }
    if let Some(&tab_id) = panel.children_keys.get("tabSystem") {
        let psf_rect = compute_frame_rect(registry, panel.id, sw, sh);
        let tab_rect = compute_frame_rect(registry, tab_id, sw, sh);
        assert!(
            tab_rect.y > psf_rect.y + psf_rect.height - 50.0,
            "tabSystem y={} should be near bottom of PlayerSpellsFrame (bottom={})",
            tab_rect.y,
            psf_rect.y + psf_rect.height
        );
        assert!(
            tab_rect.width > 100.0,
            "tabSystem should have width > 100, got {}",
            tab_rect.width
        );
    }
}

#[test]
fn spellbook_frame_positions() {
    test_timeout! {
        let env = setup_full_ui();
        open_spellbook(&env);
        let state = env.state().borrow();
        let registry = &state.widgets;
        let (sw, sh) = (1024.0, 768.0);
        check_rect(registry, "PlayerSpellsFrame", sw, sh, expected_spellbook_panel_rect(sw, sh));
        let panel = registry.get_by_name("PlayerSpellsFrame").expect("PlayerSpellsFrame exists");
        assert_spellbook_child_size(registry, panel, sw, sh);
        assert_spellbook_border(registry, panel, sw, sh);
        assert_spellbook_tabs(registry, panel, sw, sh);
    }
}
}

post_wrath_ui_fixture! {
const RETAIL_SPELLBOOK_LAYOUT_LOCK: &str = r#"
                local EPS = 1.5

                local function approx(actual, expected, eps)
                    if type(actual) ~= "number" or type(expected) ~= "number" then
                        return false
                    end
                    return math.abs(actual - expected) <= (eps or EPS)
                end

                local function rect(frame, tag)
                    if type(frame) ~= "table" then
                        return nil, tag .. "_missing"
                    end
                    local l, b, w, h = frame:GetRect()
                    if not (l and b and w and h) then
                        return nil, tag .. "_missing_rect"
                    end
                    return { l = l, b = b, w = w, h = h, r = l + w, t = b + h }, nil
                end

                local playerSpells = PlayerSpellsFrame
                local spellbook = playerSpells and playerSpells.SpellBookFrame
                local paged = spellbook and spellbook.PagedSpellsFrame
                if not playerSpells then return "player_spells_missing" end
                if not spellbook then return "spellbook_frame_missing" end
                if not paged then return "paged_spells_missing" end
                if not playerSpells:IsShown() then return "player_spells_hidden" end
                if not spellbook:IsShown() then return "spellbook_hidden" end
                if not paged:IsShown() then return "paged_spells_hidden" end

                local psRect, psErr = rect(playerSpells, "player_spells")
                if not psRect then return psErr end
                local sbRect, sbErr = rect(spellbook, "spellbook")
                if not sbRect then return sbErr end
                local pagedRect, pagedErr = rect(paged, "paged")
                if not pagedRect then return pagedErr end

                if sbRect.w < 900 then return "spellbook_width=" .. tostring(sbRect.w) end
                if sbRect.h < 500 then return "spellbook_height=" .. tostring(sbRect.h) end

                if pagedRect.l < sbRect.l - 2 then return "paged_left_outside=" .. tostring(pagedRect.l) end
                if pagedRect.b < sbRect.b - 2 then return "paged_bottom_outside=" .. tostring(pagedRect.b) end
                if pagedRect.r > sbRect.r + 2 then return "paged_right_outside=" .. tostring(pagedRect.r) end
                if pagedRect.t > sbRect.t + 2 then return "paged_top_outside=" .. tostring(pagedRect.t) end
                if pagedRect.w < sbRect.w * 0.65 then return "paged_width_too_small=" .. tostring(pagedRect.w) end
                if pagedRect.h < sbRect.h * 0.65 then return "paged_height_too_small=" .. tostring(pagedRect.h) end

                local bgLeft = spellbook.BookBGLeft
                local bgRight = spellbook.BookBGRight
                local bgTop = spellbook.TopBar
                local corner = spellbook.BookCornerFlipbook
                if not bgLeft then return "book_bg_left_missing" end
                if not bgRight then return "book_bg_right_missing" end
                if not bgTop then return "book_topbar_missing" end
                if not corner then return "book_corner_missing" end

                local bgLeftRect, bgLeftErr = rect(bgLeft, "book_bg_left")
                if not bgLeftRect then return bgLeftErr end
                local bgRightRect, bgRightErr = rect(bgRight, "book_bg_right")
                if not bgRightRect then return bgRightErr end
                local bgTopRect, bgTopErr = rect(bgTop, "book_topbar")
                if not bgTopRect then return bgTopErr end
                local cornerRect, cornerErr = rect(corner, "book_corner")
                if not cornerRect then return cornerErr end

                if not bgLeft:IsShown() then return "book_bg_left_hidden" end
                if not bgRight:IsShown() then return "book_bg_right_hidden" end
                if not bgTop:IsShown() then return "book_topbar_hidden" end
                if not corner:IsShown() then return "book_corner_hidden" end

                if not approx(bgLeftRect.w, bgRightRect.w, 1.5) then
                    return "book_bg_halves_width_mismatch=" .. tostring(bgLeftRect.w) .. "," .. tostring(bgRightRect.w)
                end
                if not approx(bgLeftRect.h, bgRightRect.h, 1.5) then
                    return "book_bg_halves_height_mismatch=" .. tostring(bgLeftRect.h) .. "," .. tostring(bgRightRect.h)
                end
                if not approx(bgLeftRect.r, bgRightRect.l, 2.0) then
                    return "book_bg_gap=" .. tostring(bgLeftRect.r - bgRightRect.l)
                end
                if not approx(bgLeftRect.l, sbRect.l, 2.0) then
                    return "book_bg_left_edge=" .. tostring(bgLeftRect.l)
                end
                if not approx(bgRightRect.r, sbRect.r, 2.0) then
                    return "book_bg_right_edge=" .. tostring(bgRightRect.r)
                end

                if cornerRect.w < 120 or cornerRect.h < 120 then
                    return "book_corner_size=" .. tostring(cornerRect.w) .. "x" .. tostring(cornerRect.h)
                end
                if cornerRect.r > sbRect.r then return "book_corner_right_outside=" .. tostring(cornerRect.r) end
                if cornerRect.b < sbRect.b then return "book_corner_bottom_outside=" .. tostring(cornerRect.b) end

                local paging = paged.PagingControls
                local prev = paging and paging.PrevPageButton
                local next = paging and paging.NextPageButton
                local pageText = paging and paging.PageText
                if not paging then return "paging_controls_missing" end
                if not prev then return "paging_prev_missing" end
                if not next then return "paging_next_missing" end
                if not pageText then return "paging_text_missing" end
                if not paging:IsShown() then return "paging_hidden" end
                if not prev:IsShown() then return "paging_prev_hidden" end
                if not next:IsShown() then return "paging_next_hidden" end
                if not pageText:IsShown() then return "paging_text_hidden" end

                local pagingRect, pagingErr = rect(paging, "paging")
                if not pagingRect then return pagingErr end
                local prevRect, prevErr = rect(prev, "paging_prev")
                if not prevRect then return prevErr end
                local nextRect, nextErr = rect(next, "paging_next")
                if not nextRect then return nextErr end
                local pageTextRect, pageTextErr = rect(pageText, "paging_text")
                if not pageTextRect then return pageTextErr end

                if not approx(pagingRect.w, 148, 1.0) or not approx(pagingRect.h, 32, 1.0) then
                    return "paging_size=" .. tostring(pagingRect.w) .. "x" .. tostring(pagingRect.h)
                end
                if pagingRect.r > pagedRect.r + 2 then return "paging_right_outside=" .. tostring(pagingRect.r) end
                if pagingRect.b < pagedRect.b then return "paging_bottom_outside=" .. tostring(pagingRect.b) end
                if not approx(prevRect.w, 32, 1.0) or not approx(prevRect.h, 32, 1.0) then
                    return "paging_prev_size=" .. tostring(prevRect.w) .. "x" .. tostring(prevRect.h)
                end
                if not approx(nextRect.w, 32, 1.0) or not approx(nextRect.h, 32, 1.0) then
                    return "paging_next_size=" .. tostring(nextRect.w) .. "x" .. tostring(nextRect.h)
                end
                if nextRect.l <= prevRect.l then
                    return "paging_button_order_invalid"
                end
                if pageTextRect.r >= prevRect.l then
                    return "paging_text_overlap_prev"
                end
                local pageLabel = pageText:GetText() or ""
                if pageLabel == "" then return "paging_text_empty" end
                if string.find(pageLabel, "%%") then return "paging_text_unformatted=" .. pageLabel end
                if not string.find(pageLabel, "%d") then return "paging_text_no_digits=" .. pageLabel end

                local firstSpell
                for _, frame in paged:EnumerateFrames() do
                    if frame
                        and frame:IsShown()
                        and frame.HasValidData
                        and frame:HasValidData()
                        and frame.Button
                        and frame.Button:IsShown()
                    then
                        firstSpell = frame
                        break
                    end
                end
                if not firstSpell then return "no_visible_spell_item" end

                local button = firstSpell.Button
                local icon = button.Icon
                local border = button.Border
                local mask = button.IconMask
                if not icon then return "spell_icon_missing" end
                if not border then return "spell_border_missing" end
                if not mask then return "spell_mask_missing" end

                local buttonRect, buttonErr = rect(button, "spell_button")
                if not buttonRect then return buttonErr end
                local iconRect, iconErr = rect(icon, "spell_icon")
                if not iconRect then return iconErr end
                local borderRect, borderErr = rect(border, "spell_border")
                if not borderRect then return borderErr end
                local maskRect, maskErr = rect(mask, "spell_mask")
                if not maskRect then return maskErr end

                if not approx(buttonRect.w, 40, 1.0) or not approx(buttonRect.h, 40, 1.0) then
                    return "spell_button_size=" .. tostring(buttonRect.w) .. "x" .. tostring(buttonRect.h)
                end
                if not approx(iconRect.w, 36, 1.0) or not approx(iconRect.h, 36, 1.0) then
                    return "spell_icon_size=" .. tostring(iconRect.w) .. "x" .. tostring(iconRect.h)
                end
                if borderRect.w < buttonRect.w or borderRect.h < buttonRect.h then
                    return "spell_border_smaller_than_button=" .. tostring(borderRect.w) .. "x" .. tostring(borderRect.h)
                end
                if maskRect.w <= 0 or maskRect.h <= 0 then
                    return "spell_mask_size=" .. tostring(maskRect.w) .. "x" .. tostring(maskRect.h)
                end
                if iconRect.l < buttonRect.l or iconRect.r > buttonRect.r then
                    return "spell_icon_outside_button_x"
                end
                if iconRect.b < buttonRect.b or iconRect.t > buttonRect.t then
                    return "spell_icon_outside_button_y"
                end
                if maskRect.w > borderRect.w + 1 or maskRect.h > borderRect.h + 1 then
                    return "spell_mask_larger_than_border=" .. tostring(maskRect.w) .. "x" .. tostring(maskRect.h)
                end
                if buttonRect.l < pagedRect.l or buttonRect.r > pagedRect.r then
                    return "spell_button_outside_paged_x"
                end
                if buttonRect.b < pagedRect.b or buttonRect.t > pagedRect.t then
                    return "spell_button_outside_paged_y"
                end

                return "ok"
                "#;

#[test]
fn spellbook_layout_stays_locked() {
    test_timeout! {
        let env = setup_full_ui();
        open_spellbook(&env);
        let lock = if cfg!(feature = "client-wowforever") {
            include_str!("fixtures/forever/spellbook_layout.lua")
        } else {
            RETAIL_SPELLBOOK_LAYOUT_LOCK
        };
        let result: String = env.eval(lock).unwrap();
        assert_eq!(result, "ok", "Spellbook layout should remain locked: {result}");
    }
}
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_spellbook_paging_click_updates_page_text() {
    let env = setup_full_ui();
    env.exec(
        r#"for _, id in ipairs({35395, 19750, 85673, 853, 275779, 465, 1022, 1044, 642, 633, 190784, 96231, 10326, 213644, 6940, 31884, 375576, 115750, 32223, 317920, 183435, 5502, 121183, 137026, 385125, 31935, 53595, 26573, 53600, 31850, 86659, 62124, 498, 327193, 378974, 387174, 204019, 85043, 152261, 76671, 280373}) do A_Admin.LearnSpell(id) end"#,
    )
    .expect("Paladin class and Protection catalog spells should be supplied");
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()")
        .expect("the original spellbook should open");
    env.exec("PlayerSpellsFrame.SpellBookFrame.CategoryTabSystem:GetTabButton(3):Click()")
        .expect("the real Protection category tab should accept the click");
    let first_page: String = env
        .eval("return PlayerSpellsFrame.SpellBookFrame.PagedSpellsFrame.PagingControls.PageText:GetText()")
        .expect("the Protection category page text should be readable");
    assert_eq!(first_page, "Page 1/2", "the Protection catalog supplies two pages");
    env.exec("PlayerSpellsFrame.SpellBookFrame.PagedSpellsFrame.PagingControls.NextPageButton:Click()")
        .expect("the real next-page button should accept the click");
    let second_page: String = env
        .eval("return PlayerSpellsFrame.SpellBookFrame.PagedSpellsFrame.PagingControls.PageText:GetText()")
        .expect("Protection page two text should be readable");
    assert_eq!(second_page, "Page 2/2", "the real next-page click should show page two");
}

#[test]
fn spellbook_spell_items_have_nonzero_rect() {
    test_timeout! {
        let env = setup_full_ui();
        open_spellbook(&env);

        let state = env.state().borrow();
        let registry = &state.widgets;
        let item_ids = find_spell_item_ids(registry);
        assert!(!item_ids.is_empty(), "Should have spell items");

        let zero_rect: Vec<_> = item_ids
            .iter()
            .filter_map(|&id| {
                let rect = compute_frame_rect(registry, id, 1024.0, 768.0);
                if rect.width <= 0.0 || rect.height <= 0.0 {
                    let f = registry.get(id)?;
                    let name = f.name.as_deref().unwrap_or("(anon)");
                    Some(format!(
                        "{}[{}] rect={:?} fw={} fh={} anchors={}",
                        name, id, rect, f.width, f.height, f.anchors.len()
                    ))
                } else {
                    None
                }
            })
            .collect();

        assert!(
            zero_rect.is_empty(),
            "All visible spell items should have non-zero layout rects.\n\
             Zero-rect items ({}):\n{}",
            zero_rect.len(),
            zero_rect.join("\n")
        );
    }
}

#[test]
fn spellbook_first_open_is_stable_with_real_tutorial_logic_restored() {
    test_timeout! {
        let env = setup_full_ui();
        restore_spellbook_tutorials(&env);
        {
            let mut state = env.state().borrow_mut();
            state.lua_errors.clear();
            state.lua_error_records.clear();
            state.lua_error_counts.clear();
        }

        open_spellbook(&env);

        let errors = env.state().borrow().lua_errors.clone();
        assert!(
            errors.is_empty(),
            "Restoring the real tutorial logic should not break first-open spellbook startup: {errors:?}"
        );

        let item_ids = {
            let state = env.state().borrow();
            find_spell_item_ids(&state.widgets)
        };
        assert!(
            !item_ids.is_empty(),
            "Spellbook items should still be visible without tutorial suppression"
        );

        let quads = build_quads(&env);
        assert!(
            quads > 0,
            "Spellbook should still render quads when the real tutorial logic runs"
        );
    }
}

click_binding_ui_fixture! {
#[test]
fn spellbook_hover_shows_spell_tooltip() {
    test_timeout! {
        let env = setup_full_ui();
        open_spellbook(&env);

        let (expected_name, _button_left, _button_bottom, _button_right, _button_top) =
            hover_first_spell_button(&env);

        let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
        let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
        assert!(visible, "GameTooltip should be visible after spellbook hover");
        assert!(
            num_lines >= 1,
            "Spellbook hover should populate spell tooltip lines, got {num_lines}"
        );

        let state = env.state().borrow();
        let tooltip_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let tooltip = state
            .tooltips
            .get(&tooltip_id)
            .expect("tooltip data should exist after spellbook hover");
        assert_eq!(tooltip.lines[0].left_text, expected_name);
    }
}
}

fn lua_frame_id(env: &WowLuaEnv, name: &str) -> u64 {
    env.loader_env()
        .with_state(|state| {
            let key = state.gc.intern_string(name.as_bytes());
            let global = state.global;
            let value = state
                .gc
                .tables
                .get(global)
                .expect("Lua globals table")
                .get_str(key, &state.gc.string_arena);
            let rilua::Val::Table(frame_ref) = value else {
                return Err("named global must be a Lua frame");
            };
            let (index, generation) = state
                .gc
                .tables
                .get(frame_ref)
                .and_then(|frame| frame.backing())
                .ok_or("named global must have a frame identity")?;
            Ok((u64::from(generation) << 32) | u64::from(index))
        })
        .expect("named global lookup must succeed")
}

click_binding_ui_fixture! {
#[test]
fn create_frame_replaces_builtin_game_tooltip_placeholder() {
    test_timeout! {
        let env = WowLuaEnv::new().unwrap();
        let placeholder_id = {
            let state = env.state().borrow();
            let id = state.widgets.get_id_by_name("GameTooltip").unwrap();
            assert!(state.widgets.get(id).unwrap().is_simulator_placeholder);
            id
        };
        env.exec("CreateFrame('GameTooltip', 'GameTooltip', UIParent)").unwrap();
        let registry_id = env.state().borrow().widgets.get_id_by_name("GameTooltip").unwrap();
        assert_ne!(placeholder_id, registry_id);
        assert_eq!(registry_id, lua_frame_id(&env, "GameTooltip"));
    }
}
}

click_binding_ui_fixture! {
#[test]
fn game_tooltip_registry_name_matches_lua_global_after_ui_fixture() {
    test_timeout! {
        let env = setup_full_ui();
        let registry_id = env.state().borrow().widgets.get_id_by_name("GameTooltip").unwrap();
        let global_id = lua_frame_id(&env, "GameTooltip");
        let state = env.state().borrow();
        let registry_frame = state.widgets.get(registry_id).unwrap();
        let global_frame = state.widgets.get(global_id).unwrap();
        assert_eq!(
            registry_id,
            global_id,
            "registry placeholder={} owner={:?}; global placeholder={} owner={:?}",
            registry_frame.is_simulator_placeholder,
            registry_frame.owner_addon,
            global_frame.is_simulator_placeholder,
            global_frame.owner_addon
        );
    }
}
}

click_binding_ui_fixture! {
#[test]
fn item_ref_tooltip_placeholder_replacement_keeps_global_identity() {
    test_timeout! {
        let env = setup_full_ui();
        let state = env.state().borrow();
        let registry_id = state.widgets.get_id_by_name("ItemRefTooltip").unwrap();
        assert!(!state.widgets.get(registry_id).unwrap().is_simulator_placeholder);
        drop(state);
        assert_eq!(registry_id, lua_frame_id(&env, "ItemRefTooltip"));
    }
}
}

click_binding_ui_fixture! {
#[test]
fn spellbook_hover_tooltip_is_sized_and_on_screen() {
    test_timeout! {
        let env = setup_full_ui();
        open_spellbook(&env);

        let (_spell_name, _button_left, _button_bottom, _button_right, _button_top) =
            hover_first_spell_button(&env);

        let mut font_sys = wow_ui_sim::render::font::WowFontSystem::new();
        {
            let mut state = env.state().borrow_mut();
            let _ = state.widgets.take_render_dirty();
            wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut state, &mut font_sys);
        }

        let tooltip_id = {
            let state = env.state().borrow();
            state.widgets.get_id_by_name("GameTooltip").unwrap()
        };

        let (tooltip_rect, tooltip_size) = {
            let state = env.state().borrow();
            let tooltip_frame = state.widgets.get(tooltip_id).unwrap();
            (
                compute_frame_rect(&state.widgets, tooltip_id, 1024.0, 768.0),
                (tooltip_frame.width, tooltip_frame.height),
            )
        };

        assert!(tooltip_size.0 > 0.0, "Tooltip width should be > 0 after spellbook hover");
        assert!(tooltip_size.1 > 0.0, "Tooltip height should be > 0 after spellbook hover");
        assert!(tooltip_rect.width > 0.0, "Tooltip rect width should be > 0");
        assert!(tooltip_rect.height > 0.0, "Tooltip rect height should be > 0");
        assert!(
            tooltip_rect.x >= 0.0 && tooltip_rect.x < 1024.0,
            "Tooltip x={} should be on screen",
            tooltip_rect.x
        );
        assert!(
            tooltip_rect.y >= 0.0 && tooltip_rect.y < 768.0,
            "Tooltip y={} should be on screen",
            tooltip_rect.y
        );

        let buckets = build_strata_buckets(&env);
        let state = env.state().borrow();
        let tooltip_data = wow_ui_sim::iced_app::tooltip::collect_tooltip_data(&state);
        assert!(
            tooltip_data.contains_key(&tooltip_id),
            "Tooltip render data should include GameTooltip after spellbook hover"
        );

        let mut glyph_atlas = wow_ui_sim::render::glyph::GlyphAtlas::new();
        let batch = build_quad_batch_for_registry(
            RegistryQuadBatchParams::new(&state.widgets, (1024.0, 768.0), &buckets)
                .text_ctx(Some((&mut font_sys, &mut glyph_atlas)))
                .message_frames(Some(&state.message_frames))
                .tooltip_data(Some(&tooltip_data)),
        );

        assert!(
            batch.vertices.len() > 100,
            "Quad batch should include tooltip geometry after spellbook hover"
        );
    }
}
}

#[test]
fn spellbook_texture_templates_preserve_half_page_anchors() {
    let env = WowLuaEnv::new().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let toc = directory.path().join("BookAnchors.toc");
    std::fs::write(&toc, "## Title: BookAnchors\nbook.xml\n").unwrap();
    // PTR 70493 Mainline/SpellBook/Blizzard_SpellBookTemplates.xml:41-51.
    std::fs::write(directory.path().join("book.xml"), r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Texture name="LeftPageTemplate" virtual="true">
                <Anchors><Anchor point="TOPLEFT" y="-51"/><Anchor point="BOTTOMRIGHT" relativePoint="BOTTOM"/></Anchors>
            </Texture>
            <Texture name="RightPageTemplate" virtual="true">
                <Anchors><Anchor point="TOPLEFT" relativePoint="TOP" y="-51"/><Anchor point="BOTTOMRIGHT"/></Anchors>
            </Texture>
            <Frame name="BookAnchorTemplate" virtual="true">
                <Size x="1000" y="600"/>
                <Anchors><Anchor point="BOTTOMLEFT" relativeTo="UIParent"/></Anchors>
                <Layers><Layer level="BACKGROUND">
                    <Texture parentKey="Left" inherits="LeftPageTemplate"/>
                    <Texture parentKey="Right" inherits="RightPageTemplate"/>
                    <Texture parentKey="Override" inherits="LeftPageTemplate">
                        <Anchors><Anchor point="TOPLEFT"/><Anchor point="BOTTOMRIGHT"/></Anchors>
                    </Texture>
                </Layer></Layers>
            </Frame>
            <Frame name="LoadedBook" inherits="BookAnchorTemplate"/>
        </Ui>
    "#).unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    env.exec(r#"
        local runtime = CreateFrame("Frame", "RuntimeBook", UIParent, "BookAnchorTemplate")
        for _, book in ipairs({LoadedBook, runtime}) do
            local l, b, w, h = book:GetRect()
            assert(w == 1000 and h == 600)
            local ll, lb, lw, lh = book.Left:GetRect()
            local rl, rb, rw, rh = book.Right:GetRect()
            assert(lw == 500 and rw == 500, "inherited backgrounds must be half-page width")
            assert(lh == 549 and rh == 549, "inherited top inset must survive")
            assert(ll == l and rl == ll + lw and lb == b and rb == b, "book halves must meet without a gap")
            local _, _, ow, oh = book.Override:GetRect()
            assert(ow == 1000 and oh == 600, "instance anchors must override template anchors")
        end
    "#).unwrap();
}
