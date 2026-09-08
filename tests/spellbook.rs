#![cfg(feature = "gui")]
//! Tests for spellbook first-load rendering.
//!
//! Bug: Spellbook shows blank on first open, spells appear only on second open.
//! Lua state is correct on first open (IsVisible=true, anchors set, dimensions correct)
//! but the rendering pipeline skips the spell item frames.

use crate::common;
#[path = "spellbook/common.rs"]
mod spellbook_common;

use spellbook_common::*;
use wow_ui_sim::iced_app::{
    RegistryQuadBatchParams, build_quad_batch_for_registry, compute_frame_rect,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::{Frame, WidgetRegistry};

// Native 12.1.0.69587 capture: the HIGH no-portrait child stays at raised
// level zero while the MEDIUM SpellBook and its descendants share a positive
// raised level. The observed global ordinals (18/20) are not fixture inputs.
fn create_native_raised_spellbook_fixture() -> WowLuaEnv {
    let env = setup_full_ui();
    env.set_screen_size(1906.0, 960.0);
    env.exec(
        r#"
        assert(PlayerFrame.noPortraitMode == nil)
        local overlay = CreateFrame("Frame")
        overlay:SetParent(PlayerFrame)
        overlay:SetFrameStrata("HIGH")
        overlay:SetAllPoints(PlayerFrame)
        PlayerFrame.noPortraitMode = overlay
        overlay.Texture = overlay:CreateTexture(nil, "OVERLAY")
        overlay.Texture:SetAllPoints()
        overlay.Texture:SetColorTexture(1, 0, 0, 1)
        "#,
    )
    .expect("reproduce BetterBlizzFrames no-portrait creation order");
    open_spellbook(&env);
    env.exec(
        r#"
        local paper = PlayerSpellsFrame.SpellBookFrame.BookBGLeft
        assert(paper:IsVisible(), "expanded SpellBook paper must be visible")
        local x, y = paper:GetCenter()
        local scale = paper:GetEffectiveScale() / UIParent:GetEffectiveScale()
        PlayerFrame:ClearAllPoints()
        PlayerFrame:SetPoint("CENTER", UIParent, "BOTTOMLEFT", x * scale, y * scale)
        "#,
    )
    .expect("place only the test unit-frame geometry inside the actual book");
    wow_ui_sim::startup::run_extra_update_ticks(&env, 2);
    env.exec(
        r#"
        assert(PlayerFrame:IsVisible() and PlayerFrame:IsToplevel())
        assert(PlayerFrame:GetFrameStrata() == "LOW" and PlayerFrame:GetFrameLevel() == 1)
        local overlay = PlayerFrame.noPortraitMode
        assert(overlay:IsVisible() and overlay:GetParent() == PlayerFrame)
        assert(overlay:GetFrameStrata() == "HIGH" and overlay:GetFrameLevel() == 2)
        assert(PlayerSpellsFrame:IsVisible() and PlayerSpellsFrame:IsToplevel())
        assert(PlayerSpellsFrame:GetFrameStrata() == "MEDIUM" and PlayerSpellsFrame:GetFrameLevel() == 1)
        local book = PlayerSpellsFrame.SpellBookFrame
        assert(book:IsVisible() and book:GetFrameStrata() == "MEDIUM" and book:GetFrameLevel() == 100)
        "#,
    )
    .expect("native raw strata and levels remain unchanged by normal book opening");
    env
}

#[test]
fn native_raised_spellbook_shares_positive_level_without_raising_unit_overlay() {
    test_timeout! {
        let env = create_native_raised_spellbook_fixture();
        let (unit, overlay, panel, book): (u32, u32, u32, u32) = env.eval(
            "return PlayerFrame:GetRaisedFrameLevel(), PlayerFrame.noPortraitMode:GetRaisedFrameLevel(), \
             PlayerSpellsFrame:GetRaisedFrameLevel(), PlayerSpellsFrame.SpellBookFrame:GetRaisedFrameLevel()",
        ).unwrap();
        assert_eq!((unit, overlay), (0, 0), "ordinary unit subtree must remain unraised");
        assert!(panel > 0, "shown native SpellBook needs a positive raised level; panel={panel}, child={book}");
        assert_eq!(book, panel, "raw-level-100 child must share the panel's raised level");
    }
}

#[test]
fn native_raised_spellbook_paper_renders_after_high_strata_unit_overlay() {
    test_timeout! {
        let env = create_native_raised_spellbook_fixture();
        let buckets = {
            let mut state = env.state().borrow_mut();
            state.ensure_layout_rects();
            state.get_strata_buckets().unwrap().clone()
        };
        let state = env.state().borrow();
        let registry = &state.widgets;
        let player = registry.get_id_by_name("PlayerFrame").unwrap();
        let overlay = registry.get(player).unwrap().children_keys["noPortraitMode"];
        let marker = registry.get(overlay).unwrap().children_keys["Texture"];
        let panel = registry.get_id_by_name("PlayerSpellsFrame").unwrap();
        let book = registry.get(panel).unwrap().children_keys["SpellBookFrame"];
        let paper = registry.get(book).unwrap().children_keys["BookBGLeft"];
        let marker_rect = compute_frame_rect(registry, marker, 1906.0, 960.0);
        let paper_rect = compute_frame_rect(registry, paper, 1906.0, 960.0);
        assert!(
            marker_rect.x >= paper_rect.x && marker_rect.y >= paper_rect.y
                && marker_rect.x + marker_rect.width <= paper_rect.x + paper_rect.width
                && marker_rect.y + marker_rect.height <= paper_rect.y + paper_rect.height,
            "unit marker must lie inside opaque book paper: marker={marker_rect:?}, paper={paper_rect:?}",
        );
        let paper_frame = registry.get(paper).unwrap();
        assert_eq!(paper_frame.effective_alpha, 1.0, "native book paper must be opaque");
        let paper_path = paper_frame.texture.as_ref().expect("native book paper texture");
        let batch = build_quad_batch_for_registry(RegistryQuadBatchParams::new(
            registry, (1906.0, 960.0), &buckets,
        ));
        let markers: Vec<_> = batch.vertices.chunks_exact(4).enumerate()
            .filter(|(_, vertices)| vertices.iter().all(|v| v.color == [1.0, 0.0, 0.0, 1.0])
                && bounds_match_rect(quad_bounds_from_vertices(vertices), marker_rect))
            .map(|(quad, _)| (quad * 4) as u32).collect();
        assert_eq!(markers.len(), 1, "actual unit overlay must emit one marker quad");
        let paper_request = batch.texture_requests.iter().find(|request|
            request.path == *paper_path && bounds_match_rect(quad_bounds(&batch, request), paper_rect)
        ).expect("actual book paper must emit its textured quad");
        let overlay_draw = batch.indices.iter().position(|&vertex| vertex == markers[0]).unwrap();
        let paper_draw = batch.indices.iter().position(|&vertex| vertex == paper_request.vertex_start).unwrap();
        assert!(overlay_draw < paper_draw,
            "native raised SpellBook must cover the HIGH unit overlay: overlay draw={overlay_draw}, paper draw={paper_draw}");
    }
}

fn find_first_visible_spell_item_button_children(
    registry: &WidgetRegistry,
    item_ids: &[u64],
) -> Option<(u64, u64, u64)> {
    item_ids.iter().find_map(|&item_id| {
        let item = registry.get(item_id)?;
        let &button_id = item.children_keys.get("Button")?;
        let button = registry.get(button_id)?;
        let &icon_id = button.children_keys.get("Icon")?;
        let &mask_id = button.children_keys.get("IconMask")?;
        Some((button_id, icon_id, mask_id))
    })
}

/// Check that a frame is reachable from root via the children chain.
/// Returns (reachable, detail_string).
fn check_frame_reachability(registry: &WidgetRegistry, frame_id: u64) -> (bool, String) {
    let mut id = frame_id;
    let mut path = Vec::new();

    loop {
        let Some(frame) = registry.get(id) else {
            return missing_frame_result(id);
        };
        let name = frame_display_name(frame);
        path.push(format!("{}[{}]", name, id));

        let Some(parent_id) = frame.parent_id else {
            break;
        };
        if !parent_lists_child(registry, parent_id, id) {
            return broken_parent_link_result(registry, name, id, parent_id, &path);
        }
        id = parent_id;
    }
    path.reverse();
    (true, path.join(" -> "))
}

fn missing_frame_result(frame_id: u64) -> (bool, String) {
    (false, format!("Frame {} not found", frame_id))
}

fn frame_display_name(frame: &Frame) -> &str {
    frame.name.as_deref().unwrap_or("(anon)")
}

fn parent_lists_child(registry: &WidgetRegistry, parent_id: u64, child_id: u64) -> bool {
    registry
        .get(parent_id)
        .is_some_and(|parent| parent.children.contains(&child_id))
}

fn broken_parent_link_result(
    registry: &WidgetRegistry,
    frame_name: &str,
    frame_id: u64,
    parent_id: u64,
    path: &[String],
) -> (bool, String) {
    let parent_name = registry
        .get(parent_id)
        .map(frame_display_name)
        .unwrap_or("?");
    (
        false,
        format!(
            "BREAK: {}[{}] parent={}[{}] but NOT in children list. Path: {}",
            frame_name,
            frame_id,
            parent_name,
            parent_id,
            path.join(" -> ")
        ),
    )
}

/// Log the ancestor chain of a frame for debugging.
fn log_ancestor_chain(registry: &WidgetRegistry, frame_id: u64) {
    let mut id = frame_id;
    loop {
        let Some(frame) = registry.get(id) else {
            eprintln!("  Frame {} not found!", id);
            break;
        };
        let name = frame.name.as_deref().unwrap_or("(anon)");
        eprintln!(
            "  {} [{}]: visible={}, children={}, anchors={}",
            name,
            id,
            frame.visible,
            frame.children.len(),
            frame.anchors.len()
        );
        match frame.parent_id {
            Some(pid) => id = pid,
            None => break,
        }
    }
}

/// Check which items are missing from ancestor-visible set and log details.
/// Uses effective_alpha > 0 as the visibility check (requires get_strata_buckets called first).
fn diagnose_missing_items(env: &WowLuaEnv, item_ids: &[u64]) {
    // Propagate effective_alpha
    {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
    }

    let state = env.state().borrow();
    let registry = &state.widgets;

    let mut in_set = 0;
    let mut missing = 0;
    for &item_id in item_ids {
        let is_visible = registry
            .get(item_id)
            .is_some_and(|f| f.effective_alpha > 0.0);
        if is_visible {
            in_set += 1;
        } else {
            missing += 1;
            let (ok, detail) = check_frame_reachability(registry, item_id);
            eprintln!(
                "Item {} NOT ancestor-visible: ok={}, {}",
                item_id, ok, detail
            );
            log_ancestor_chain(registry, item_id);
        }
    }
    eprintln!("Ancestor-visible: {} in, {} missing", in_set, missing);
}

#[test]
#[ignore = "diagnostic"]
fn debug_spellbook_first_open_state() {
    test_timeout! {
        let env = setup_full_ui();
        open_spellbook(&env);

        let report: String = env
            .eval(
                r##"
                local lines = {}

                local function push(label, value)
                    table.insert(lines, label .. "=" .. tostring(value))
                end

                local sb = PlayerSpellsFrame and PlayerSpellsFrame.SpellBookFrame
                local paged = sb and sb.PagedSpellsFrame
                push("SpellBookFrame", sb)
                push("PagedSpellsFrame", paged)

                if not sb or not paged then
                    return table.concat(lines, "\n")
                end

                push("sb:IsShown()", sb:IsShown())
                push("sb:IsVisible()", sb:IsVisible())
                push("sb:GetTab()", sb.GetTab and sb:GetTab() or "missing")
                push("categoryMixins", sb.categoryMixins and #sb.categoryMixins or "nil")

                if sb.categoryMixins then
                    for i, category in ipairs(sb.categoryMixins) do
                        local groups = category.spellGroups and #category.spellGroups or "nil"
                        local name = category.GetName and category:GetName() or ("category_" .. tostring(i))
                        local available = category.IsAvailable and category:IsAvailable() or "missing"
                        table.insert(lines, string.format(
                            "category[%d]=%s available=%s tab=%s groups=%s",
                            i,
                            tostring(name),
                            tostring(available),
                            tostring(category.GetTabID and category:GetTabID() or "missing"),
                            tostring(groups)
                        ))
                        if category.spellGroups then
                            for groupIndex, group in ipairs(category.spellGroups) do
                                table.insert(lines, string.format(
                                    "category[%d].group[%d]=offset:%s num:%s ordered:%s",
                                    i,
                                    groupIndex,
                                    tostring(group.slotIndexOffset),
                                    tostring(group.numSpellBookItems),
                                    tostring(group.orderedSpellBookItemSlotIndices and #group.orderedSpellBookItemSlotIndices or "nil")
                                ))
                            end
                        end
                    end
                end

                local activeCategory = sb.GetActiveCategoryMixin and sb:GetActiveCategoryMixin()
                push("activeCategory", activeCategory and activeCategory:GetName() or "nil")
                if activeCategory then
                    local data = activeCategory:GetSpellBookItemData(true, sb:GetSpellBookItemFilterInstance())
                    push("activeCategoryData.groups", data and #data or "nil")
                    if data then
                        local total = 0
                        for groupIndex, group in ipairs(data) do
                            local elements = group.elements and #group.elements or 0
                            total = total + elements
                            table.insert(lines, string.format(
                                "data.group[%d]=header:%s elements:%s",
                                groupIndex,
                                tostring(group.header and group.header.templateKey or "nil"),
                                tostring(elements)
                            ))
                        end
                        push("activeCategoryData.totalElements", total)
                    end
                end

                push("paged.frames", paged.GetFrames and #paged:GetFrames() or "missing")
                push("paged.currentPage", paged.PagingControls and paged.PagingControls:GetCurrentPage() or "missing")
                push("paged.maxPages", paged.PagingControls and paged.PagingControls:GetMaxPages() or "missing")
                push("paged.viewDataList", paged.viewDataList and #paged.viewDataList or "nil")
                if paged.viewDataList then
                    for i, viewData in ipairs(paged.viewDataList) do
                        table.insert(lines, string.format("viewData[%d]=%s", i, #viewData))
                    end
                end

                if paged.ViewFrames then
                    for i, viewFrame in ipairs(paged.ViewFrames) do
                        local children = { viewFrame:GetChildren() }
                        table.insert(lines, string.format(
                            "ViewFrame[%d]=shown:%s visible:%s children:%s",
                            i,
                            tostring(viewFrame:IsShown()),
                            tostring(viewFrame:IsVisible()),
                            tostring(#children)
                        ))
                    end
                end

                local enumCount = 0
                for _ in paged:EnumerateFrames() do
                    enumCount = enumCount + 1
                end
                push("paged.enumerateFrames", enumCount)

                return table.concat(lines, "\n")
                "##,
            )
            .expect("diagnostic spellbook state should return");

        let errors = env.state().borrow().lua_errors.clone();
        panic!("{report}\nlua_errors={errors:#?}");
    }
}

#[test]
fn spellbook_spells_visible_on_first_open() {
    test_timeout! {
        let env = setup_full_ui();
        open_spellbook(&env);

        let sb_visible: bool = env
            .eval(
                "return PlayerSpellsFrame and PlayerSpellsFrame.SpellBookFrame \
                 and PlayerSpellsFrame.SpellBookFrame:IsVisible() or false",
            )
            .unwrap();
        assert!(sb_visible, "SpellBookFrame should be visible after toggle");

        let item_ids = {
            let state = env.state().borrow();
            find_spell_item_ids(&state.widgets)
        };
        assert!(!item_ids.is_empty(), "Should have visible spell items");
        eprintln!("{} visible spell items on first open", item_ids.len());

        diagnose_missing_items(&env, &item_ids);

        let first_quads = build_quads(&env);
        eprintln!("First open quad count: {}", first_quads);

        // Close and reopen
        env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()").unwrap();
        let _ = env.process_timers();
        env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()").unwrap();
        let _ = env.process_timers();

        let second_quads = build_quads(&env);
        eprintln!("Second open quad count: {}", second_quads);
        assert!(
            second_quads > 0,
            "Second open should also produce quads after closing and reopening"
        );
    }
}

#[test]
fn spellbook_spell_items_in_ancestor_visible() {
    test_timeout! {
        let env = setup_full_ui();
        open_spellbook(&env);

        // Propagate effective_alpha
        {
            let mut state = env.state().borrow_mut();
            let _ = state.get_strata_buckets();
        }

        let state = env.state().borrow();
        let item_ids = find_spell_item_ids(&state.widgets);
        assert!(!item_ids.is_empty(), "Should have spell items");

        let missing: Vec<_> = item_ids
            .iter()
            .filter(|&&id| !state.widgets.get(id).is_some_and(|f| f.effective_alpha > 0.0))
            .map(|&id| {
                let name = state.widgets.get(id)
                    .and_then(|f| f.name.as_deref())
                    .unwrap_or("(anon)");
                format!("{}[{}]", name, id)
            })
            .collect();

        assert!(
            missing.is_empty(),
            "All visible spell items should have effective_alpha > 0.\n\
             Missing {} items: {:?}",
            missing.len(),
            &missing[..missing.len().min(10)]
        );
    }
}

#[test]
fn spellbook_icon_textures_in_ancestor_visible() {
    test_timeout! {
        let env = setup_full_ui();
        open_spellbook(&env);

        // Propagate effective_alpha
        {
            let mut state = env.state().borrow_mut();
            let _ = state.get_strata_buckets();
        }

        let state = env.state().borrow();
        let registry = &state.widgets;
        let item_ids = find_spell_item_ids(registry);
        assert!(!item_ids.is_empty(), "Should have spell items");

        // For each spell item, find its Button child, then Icon texture child
        let mut icons_found = 0u32;
        let mut icons_missing = 0u32;
        for &item_id in &item_ids {
            let Some(item) = registry.get(item_id) else { continue };
            let Some(&btn_id) = item.children_keys.get("Button") else { continue };
            let Some(btn) = registry.get(btn_id) else { continue };
            let Some(&icon_id) = btn.children_keys.get("Icon") else { continue };
            let Some(icon) = registry.get(icon_id) else { continue };

            if icon.effective_alpha > 0.0 {
                icons_found += 1;
            } else {
                icons_missing += 1;
                if icons_missing <= 3 {
                    let (ok, detail) = check_frame_reachability(registry, icon_id);
                    eprintln!("Icon {icon_id} NOT ancestor-visible: ok={ok} {detail}");
                    eprintln!("  icon: vis={} tex={:?} w={} h={} alpha={}",
                        icon.visible, icon.texture, icon.width, icon.height,
                        icon.effective_alpha);
                    eprintln!("  btn {btn_id}: vis={} children={:?}",
                        btn.visible, btn.children);
                    eprintln!("  item {item_id}: vis={} children={:?}",
                        item.visible, item.children);
                }
            }
        }
        eprintln!("Icons: found={icons_found} missing={icons_missing}");
        assert_eq!(icons_missing, 0,
            "All spell icon textures should have effective_alpha > 0");
    }
}

#[test]
fn spellbook_first_visible_item_emits_icon_mask_quad_on_first_open() {
    test_timeout! {
        let env = setup_full_ui();
        open_spellbook(&env);

        let buckets = build_strata_buckets(&env);
        let state = env.state().borrow();
        let registry = &state.widgets;
        let item_ids = find_spell_item_ids(registry);
        assert!(!item_ids.is_empty(), "Should have spell items");

        let (_button_id, icon_id, mask_id) = find_first_visible_spell_item_button_children(registry, &item_ids)
            .expect("Should find the first visible spellbook item icon and mask");

        let icon = registry.get(icon_id).expect("first visible icon frame");
        let icon_masks = icon.mask_textures.len();
        assert_eq!(
            icon_masks,
            1,
            "First visible spellbook icon should already have exactly one mask wired on first open"
        );

        let icon_rect = compute_frame_rect(registry, icon_id, 1024.0, 768.0);
        let mask_rect = compute_frame_rect(registry, mask_id, 1024.0, 768.0);

        let batch = build_quad_batch_for_registry(RegistryQuadBatchParams::new(
            registry,
            (1024.0, 768.0),
            &buckets,
        ));

        let icon_request = batch
            .texture_requests
            .iter()
            .find(|request| bounds_match_rect(quad_bounds(&batch, request), icon_rect))
            .expect("First visible spellbook icon should emit a textured quad");
        let mask_request = batch
            .mask_texture_requests
            .iter()
            .find(|request| bounds_match_rect(quad_bounds(&batch, request), mask_rect))
            .expect("First visible spellbook icon mask should emit a mask quad on first open");

        assert!(
            icon_request.path.to_ascii_lowercase().contains("icons"),
            "First visible spellbook icon should render from an icon texture, got {}",
            icon_request.path
        );
        assert!(
            mask_request
                .path
                .to_ascii_lowercase()
                .contains("spellbookelementsiconmask"),
            "First visible spellbook icon mask should render from the spellbook icon mask texture, got {}",
            mask_request.path
        );
    }
}
