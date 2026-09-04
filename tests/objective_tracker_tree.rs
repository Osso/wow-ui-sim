#![cfg(feature = "gui")]

//! ObjectiveTracker tree regression test.
//!
//! On master at commit `322eba4a` the tracker paints "All Objectives",
//! "Quests", and three quest blocks (The Lost Expedition, Defending the
//! Gates, Supply Run). On `rilua-migration` the same environment leaves
//! `ObjectiveTrackerFrame` `hidden` with no child Header, so the screenshot
//! shows nothing in the right-hand tracker area past the minimap.
//!
//! Reference dump (dump-tree --filter-key ObjectiveTrackerFrame):
//!
//! ```text
//! ObjectiveTrackerFrame  [Frame] (260x847) visible LOW:3 x=1335 y=260
//!   .Header              [Frame] (260x32)  visible LOW:4 x=1335 y=260
//!     .Text              [FontString]      text="All Objectives"
//!     .Header            [Frame] (260x26)  visible LOW:5
//!       .Text            [FontString]      text="Quests"
//!       .HeaderText      [FontString]      text="The Lost Expedition"
//!       .HeaderText      [FontString]      text="Defending the Gates"
//!       .HeaderText      [FontString]      text="Supply Run"
//! ```

use crate::common;

use wow_ui_sim::iced_app::{
    RegistryQuadBatchParams, build_quad_batch_for_registry_with_quest_blobs,
};
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::render::{GlyphAtlas, WowFontSystem};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

fn blizzard_ui_dir() -> std::path::PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be synced")

}

fn load_settled_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("Failed to load Blizzard addon {name}: {err}"));
    }

    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
}

fn build_text_quad_batch(env: &WowLuaEnv) -> wow_ui_sim::render::QuadBatch {
    let mut font_sys = WowFontSystem::new();
    let mut glyph_atlas = GlyphAtlas::new();
    let buckets = {
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let state = env.state().borrow();
    build_quad_batch_for_registry_with_quest_blobs(
        RegistryQuadBatchParams::new(&state.widgets, (1024.0, 768.0), &buckets)
            .text_ctx(Some((&mut font_sys, &mut glyph_atlas)))
            .message_frames(Some(&state.message_frames))
            .quest_blobs(Some(&state.quest_blobs)),
    )
}

prefork_full_ui_case! {
fn objective_tracker_frame_is_visible_after_startup(env: &WowLuaEnv) {
    let (exists, shown, is_visible, width, height): (bool, bool, bool, f64, f64) = env
        .eval(
            r#"
            if not ObjectiveTrackerFrame then return false, false, false, 0, 0 end
            local w, h = ObjectiveTrackerFrame:GetSize()
            return true,
                ObjectiveTrackerFrame:IsShown(),
                ObjectiveTrackerFrame:IsVisible(),
                w, h
            "#,
        )
        .expect("eval ObjectiveTrackerFrame");

    assert!(exists, "ObjectiveTrackerFrame global must exist");
    assert!(
        shown,
        "ObjectiveTrackerFrame:IsShown() must be true after startup (currently hidden on rilua-migration)",
    );
    assert!(
        is_visible,
        "ObjectiveTrackerFrame:IsVisible() must be true — dump-tree shows it as `hidden` on the branch",
    );
    // Master dump: 260x847 post-EditMode layout.
    assert_eq!(width as i32, 260, "ObjectiveTrackerFrame width mismatch");
    assert!(
        height > 100.0,
        "ObjectiveTrackerFrame height must at least cover its header (got {height})",
    );
}
}

prefork_full_ui_case! {
fn objective_tracker_header_shows_all_objectives_text(env: &WowLuaEnv) {
    // Master mounts a `.Header` child with a `.Text` FontString reading
    // "All Objectives". Without the header, the rest of the tracker
    // (quests list, category sub-headers, objectives) never paints.
    let (header_exists, header_visible, text, text_visible): (bool, bool, String, bool) = env
        .eval(
            r#"
            local tracker = ObjectiveTrackerFrame
            if not tracker then return false, false, "<no tracker>", false end
            local header = tracker.Header
            if not header then return false, false, "<no header>", false end
            local text_region = header.Text
            local txt = text_region and text_region.GetText and text_region:GetText() or ""
            return true,
                header:IsVisible(),
                txt or "",
                text_region and text_region:IsVisible() or false
            "#,
        )
        .expect("eval ObjectiveTracker header");

    assert!(header_exists, "ObjectiveTrackerFrame.Header must exist");
    assert!(
        header_visible,
        "ObjectiveTrackerFrame.Header must be IsVisible() so it paints above the quest list",
    );
    assert!(
        text_visible,
        "ObjectiveTrackerFrame.Header.Text must be IsVisible() — the 'All Objectives' title",
    );
    assert_eq!(
        text, "All Objectives",
        "Header text must be \"All Objectives\" per master dump (got {text:?})",
    );
}
}

prefork_full_ui_case! {
fn objective_tracker_quest_module_header_shows_quests_text(env: &WowLuaEnv) {
    let (
        header_exists,
        header_visible,
        text,
        text_visible,
        text_alpha,
        text_r,
        text_g,
        text_b,
        text_color_a,
        quest_block_count,
    ): (bool, bool, String, bool, f64, f64, f64, f64, f64, i32) = env
        .eval(
            r#"
            local module = QuestObjectiveTracker
            if not module then return false, false, "<no module>", false end
            local header = module.Header
            if not header then return false, false, "<no header>", false end
            local text_region = header.Text
            local txt = text_region and text_region.GetText and text_region:GetText() or ""
            local alpha = text_region and text_region.GetAlpha and text_region:GetAlpha() or -1
            local r, g, b, a = 0, 0, 0, 0
            if text_region and text_region.GetTextColor then
                r, g, b, a = text_region:GetTextColor()
            end
            local count = 0
            if module.usedBlocks then
                for _, blocks in pairs(module.usedBlocks) do
                    if type(blocks) == "table" then
                        for _ in pairs(blocks) do
                            count = count + 1
                        end
                    end
                end
            end
            return true,
                header:IsVisible(),
                txt or "",
                text_region and text_region:IsVisible() or false,
                alpha,
                r or 0,
                g or 0,
                b or 0,
                a or 0,
                count
            "#,
        )
        .expect("eval QuestObjectiveTracker header");

    assert!(header_exists, "QuestObjectiveTracker.Header must exist");
    assert!(
        quest_block_count > 0,
        "QuestObjectiveTracker must contain at least one quest block in seeded startup state",
    );
    assert!(
        header_visible,
        "QuestObjectiveTracker.Header must be visible so the module title paints",
    );
    assert!(
        text_visible,
        "QuestObjectiveTracker.Header.Text must be visible so the 'Quests' label paints",
    );
    assert_eq!(
        text, "Quests",
        "Quest module header text must be \"Quests\" (got {text:?})",
    );
    assert!(
        text_alpha > 0.0,
        "QuestObjectiveTracker.Header.Text alpha must be > 0 so title is not fully transparent",
    );
    // Header text should be readable (gold/yellow-ish), not blacked out.
    assert!(
        text_r > 0.2 || text_g > 0.2 || text_b > 0.2,
        "QuestObjectiveTracker.Header.Text color must not be black (r={text_r}, g={text_g}, b={text_b})",
    );
    assert!(
        text_color_a > 0.0,
        "QuestObjectiveTracker.Header.Text color alpha must be > 0 (got {text_color_a})",
    );
}
}

prefork_full_ui_case! {
fn objective_tracker_quest_module_header_recovers_after_runtime_corruption(env: &WowLuaEnv) {
    env.exec(
        r#"
        local module = QuestObjectiveTracker
        assert(module and module.Header and module.Header.Text, "QuestObjectiveTracker header missing")
        local text = module.Header.Text
        text:SetText("")
        text:Hide()
        text:SetAlpha(0)
        text:SetTextColor(1, 1, 0, 0)
        if ObjectiveTrackerManager and ObjectiveTrackerManager.UpdateAll then
            ObjectiveTrackerManager:UpdateAll()
        end
        "#,
    )
    .expect("corrupt + update quest header text");

    let (text, text_visible, text_alpha, text_r, text_g, text_b, text_color_a): (String, bool, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local text_region = QuestObjectiveTracker and QuestObjectiveTracker.Header and QuestObjectiveTracker.Header.Text
            if not text_region then
                return "<missing>", false, -1, 0, 0, 0, 0
            end
            local r, g, b, a = text_region:GetTextColor()
            return text_region:GetText() or "",
                text_region:IsVisible(),
                text_region:GetAlpha() or -1,
                r or 0,
                g or 0,
                b or 0,
                a or 0
            "#,
        )
        .expect("eval restored quest header");

    assert_eq!(text, "Quests", "quest module header text must be restored to 'Quests'");
    assert!(text_visible, "quest module header text must be visible after recovery");
    assert!(text_alpha > 0.0, "quest module header text alpha must be restored (> 0)");
    assert!(
        text_r > 0.2 || text_g > 0.2 || text_b > 0.2,
        "quest module header text color must be readable after recovery (r={text_r}, g={text_g}, b={text_b})",
    );
    assert!(
        text_color_a > 0.0,
        "quest module header text color alpha must recover (> 0), got {text_color_a}",
    );
}
}

prefork_full_ui_case! {
fn objective_tracker_quest_module_header_emits_glyph_quads(env: &WowLuaEnv) {
    let (exists, left, right, top, bottom): (bool, f64, f64, f64, f64) = env
        .eval(
            r#"
            local text_region = QuestObjectiveTracker and QuestObjectiveTracker.Header and QuestObjectiveTracker.Header.Text
            if not text_region then
                return false, 0, 0, 0, 0
            end
            return true,
                text_region:GetLeft() or 0,
                text_region:GetRight() or 0,
                text_region:GetTop() or 0,
                text_region:GetBottom() or 0
            "#,
        )
        .expect("eval quest header text bounds");

    assert!(exists, "QuestObjectiveTracker.Header.Text must exist");

    let ui_scale = wow_ui_sim::render::texture::UI_SCALE;
    let min_x = left.min(right) as f32 * ui_scale - 2.0;
    let max_x = left.max(right) as f32 * ui_scale + 2.0;
    // GetTop/GetBottom are in WoW coordinates (origin bottom-left), while
    // quad vertices are in renderer coordinates (origin top-left).
    let screen_height = 768.0_f32;
    let top_y = screen_height - top as f32 * ui_scale;
    let bottom_y = screen_height - bottom as f32 * ui_scale;
    let min_y = top_y.min(bottom_y) - 2.0;
    let max_y = top_y.max(bottom_y) + 2.0;

    let batch = build_text_quad_batch(&env);
    let glyph_tex_index = wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX;
    let glyph_vertices_in_bounds = batch
        .vertices
        .iter()
        .filter(|v| v.tex_index == glyph_tex_index)
        .filter(|v| {
            let x = v.position[0];
            let y = v.position[1];
            x >= min_x && x <= max_x && y >= min_y && y <= max_y
        })
        .count();

    assert!(
        glyph_vertices_in_bounds > 0,
        "QuestObjectiveTracker.Header.Text should emit glyph vertices in bounds; got 0",
    );
}
}

#[test]
fn objective_tracker_frame_layout_is_locked() {
    test_timeout! {
        let env = load_settled_game_ui();

        env.exec(
            r#"
            local EPS = 0.75

            local function approx(actual, expected, eps)
                if type(actual) ~= "number" or type(expected) ~= "number" then
                    return false
                end
                return math.abs(actual - expected) <= (eps or EPS)
            end

            local function require_frame(path, frame)
                assert(type(frame) == "table", path .. " missing")
                return frame
            end

            local function get_rect(path, frame)
                local left = frame:GetLeft()
                local right = frame:GetRight()
                local top = frame:GetTop()
                local bottom = frame:GetBottom()
                local width = frame:GetWidth()
                local height = frame:GetHeight()
                assert(type(left) == "number", path .. ":GetLeft() missing")
                assert(type(right) == "number", path .. ":GetRight() missing")
                assert(type(top) == "number", path .. ":GetTop() missing")
                assert(type(bottom) == "number", path .. ":GetBottom() missing")
                assert(type(width) == "number", path .. ":GetWidth() missing")
                assert(type(height) == "number", path .. ":GetHeight() missing")
                return left, right, top, bottom, width, height
            end

            local function get_center(path, frame)
                local x, y = frame:GetCenter()
                assert(type(x) == "number", path .. ":GetCenter() x missing")
                assert(type(y) == "number", path .. ":GetCenter() y missing")
                return x, y
            end

            local tracker = require_frame("ObjectiveTrackerFrame", ObjectiveTrackerFrame)
            local trackerHeader = require_frame("ObjectiveTrackerFrame.Header", tracker.Header)
            local trackerHeaderText = require_frame("ObjectiveTrackerFrame.Header.Text", trackerHeader.Text)
            local module = require_frame("QuestObjectiveTracker", QuestObjectiveTracker)
            local header = require_frame("QuestObjectiveTracker.Header", module.Header)
            local background = require_frame("QuestObjectiveTracker.Header.Background", header.Background)
            local shine = require_frame("QuestObjectiveTracker.Header.Shine", header.Shine)
            local glow = require_frame("QuestObjectiveTracker.Header.Glow", header.Glow)
            local text = require_frame("QuestObjectiveTracker.Header.Text", header.Text)
            local minimize = require_frame("QuestObjectiveTracker.Header.MinimizeButton", header.MinimizeButton)
            local contents = require_frame("QuestObjectiveTracker.ContentsFrame", module.ContentsFrame)

            local tL, tR, tT, tB, tW, _ = get_rect("ObjectiveTrackerFrame", tracker)
            assert(approx(tW, 260, 0.1), "ObjectiveTrackerFrame width changed: " .. tostring(tW))

            local thL, thR, thT, thB, thW, thH = get_rect("ObjectiveTrackerFrame.Header", trackerHeader)
            assert(approx(thL, tL), "ObjectiveTrackerFrame.Header left drifted")
            assert(approx(thR, tR), "ObjectiveTrackerFrame.Header right drifted")
            assert(approx(thT, tT), "ObjectiveTrackerFrame.Header top drifted")
            assert(approx(thW, 260, 0.1), "ObjectiveTrackerFrame.Header width changed: " .. tostring(thW))
            assert(approx(thH, 32, 0.1), "ObjectiveTrackerFrame.Header height changed: " .. tostring(thH))

            local aotText = trackerHeaderText:GetText()
            assert(aotText == "All Objectives", "ObjectiveTrackerFrame.Header.Text changed: " .. tostring(aotText))

            local mL, mR, mT, mB, mW, _ = get_rect("QuestObjectiveTracker", module)
            assert(approx(mL, tL), "QuestObjectiveTracker left drifted from ObjectiveTrackerFrame")
            assert(approx(mR, tR), "QuestObjectiveTracker right drifted from ObjectiveTrackerFrame")
            assert(approx(mW, 260, 0.1), "QuestObjectiveTracker width changed: " .. tostring(mW))
            assert(approx(mT, tT - 38, 0.1), "QuestObjectiveTracker top offset changed")

            local hL, hR, hT, hB, hW, hH = get_rect("QuestObjectiveTracker.Header", header)
            assert(approx(hL, mL), "QuestObjectiveTracker.Header left drifted")
            assert(approx(hR, mR), "QuestObjectiveTracker.Header right drifted")
            assert(approx(hT, mT), "QuestObjectiveTracker.Header top drifted")
            assert(approx(hW, 260, 0.1), "QuestObjectiveTracker.Header width changed: " .. tostring(hW))
            assert(approx(hH, 26, 0.1), "QuestObjectiveTracker.Header height changed: " .. tostring(hH))

            local hCX, hCY = get_center("QuestObjectiveTracker.Header", header)
            local bgCX, bgCY = get_center("QuestObjectiveTracker.Header.Background", background)
            local shCX, shCY = get_center("QuestObjectiveTracker.Header.Shine", shine)
            local glCX, glCY = get_center("QuestObjectiveTracker.Header.Glow", glow)
            local minCX, minCY = get_center("QuestObjectiveTracker.Header.MinimizeButton", minimize)

            local _, _, _, _, bgW, bgH = get_rect("QuestObjectiveTracker.Header.Background", background)
            local _, _, _, _, shW, shH = get_rect("QuestObjectiveTracker.Header.Shine", shine)
            local _, _, _, _, glW, glH = get_rect("QuestObjectiveTracker.Header.Glow", glow)
            local minL, minR, _, _, minW, minH = get_rect("QuestObjectiveTracker.Header.MinimizeButton", minimize)
            local txL, _, _, _, txW, _ = get_rect("QuestObjectiveTracker.Header.Text", text)

            assert(approx(bgCX, hCX), "Header.Background center-x drifted")
            assert(approx(bgCY, hCY), "Header.Background center-y drifted")
            assert(approx(bgW, 300, 0.1), "Header.Background width changed: " .. tostring(bgW))
            assert(approx(bgH, 30, 0.1), "Header.Background height changed: " .. tostring(bgH))

            assert(approx(shCX, hCX - 150), "Header.Shine center-x offset changed")
            assert(approx(shCY, hCY + 1), "Header.Shine center-y offset changed")
            assert(approx(shW, 154, 0.1), "Header.Shine width changed: " .. tostring(shW))
            assert(approx(shH, 23, 0.1), "Header.Shine height changed: " .. tostring(shH))

            assert(approx(glCX, hCX - 120), "Header.Glow center-x offset changed")
            assert(approx(glCY, hCY + 1), "Header.Glow center-y offset changed")
            assert(approx(glW, 187, 0.1), "Header.Glow width changed: " .. tostring(glW))
            assert(approx(glH, 28, 0.1), "Header.Glow height changed: " .. tostring(glH))

            assert(approx(minR, hR + 1), "Header.MinimizeButton right anchor offset changed")
            assert(approx(minCY, hCY), "Header.MinimizeButton center-y drifted")
            assert(approx(minW, 16, 0.1), "Header.MinimizeButton width changed: " .. tostring(minW))
            assert(approx(minH, 16, 0.1), "Header.MinimizeButton height changed: " .. tostring(minH))
            assert(approx(minL, minR - 16, 0.1), "Header.MinimizeButton width/left mismatch")

            assert(approx(txL, hL + 7), "Header.Text left anchor offset changed")
            assert(approx(txW, 200, 0.1), "Header.Text width changed: " .. tostring(txW))
            local moduleText = text:GetText()
            assert(moduleText == "Quests", "QuestObjectiveTracker.Header.Text changed: " .. tostring(moduleText))

            local _, _, cT, cB, cW, _ = get_rect("QuestObjectiveTracker.ContentsFrame", contents)
            assert(approx(cT, hB), "QuestObjectiveTracker.ContentsFrame top no longer tracks header bottom")
            assert(approx(cW, 260, 0.1), "QuestObjectiveTracker.ContentsFrame width changed: " .. tostring(cW))
            local cL, cR = contents:GetLeft(), contents:GetRight()
            assert(approx(cL, mL), "QuestObjectiveTracker.ContentsFrame left drifted")
            assert(approx(cR, mR), "QuestObjectiveTracker.ContentsFrame right drifted")
            assert(approx(cB, mB), "QuestObjectiveTracker.ContentsFrame bottom drifted")

            -- Lock down quest block placement in the objective frame.
            local blockCount = 0
            local blockTops = {}
            if type(module.usedBlocks) == "table" then
                for _, blocksById in pairs(module.usedBlocks) do
                    if type(blocksById) == "table" then
                        for _, block in pairs(blocksById) do
                            if type(block) == "table" and block.GetTop and block:IsShown() then
                                local bL, bR, bT, bB = block:GetLeft(), block:GetRight(), block:GetTop(), block:GetBottom()
                                assert(bL and bR and bT and bB, "quest block bounds missing")
                                local bMinX, bMaxX = math.min(bL, bR), math.max(bL, bR)
                                local bMaxY = math.max(bT, bB)
                                local cMinX, cMaxX = math.min(cL, cR), math.max(cL, cR)
                                assert(bMinX >= cMinX - EPS and bMaxX <= cMaxX + EPS, "quest block escaped contents frame horizontally")
                                table.insert(blockTops, bMaxY)
                                blockCount = blockCount + 1
                            end
                        end
                    end
                end
            end
            assert(blockCount >= 3, "expected >=3 visible quest blocks, got " .. tostring(blockCount))
            table.sort(blockTops, function(a, b) return a > b end)
            for i = 2, #blockTops do
                assert(blockTops[i] < blockTops[i - 1] - 0.1, "quest block ordering changed or collapsed")
            end

            -- Lock down quest header surface state to catch shifted/overbright background regressions.
            assert((background:GetAlpha() or -1) > 0.95, "header background alpha must settle to 1")
            assert((shine:GetAlpha() or 1) < 0.05, "header shine alpha must settle to 0")
            assert((glow:GetAlpha() or 1) < 0.05, "header glow alpha must settle to 0")
            assert((minimize:GetAlpha() or -1) > 0.95, "header minimize alpha must settle to 1")
            "#,
        )
        .expect("objective tracker layout lock assertions");
    }
}
