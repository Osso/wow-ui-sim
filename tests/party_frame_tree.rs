//! Party-frame tree regression test.
//!
//! Captures the rendered PartyFrame tree for a 4-member group against the
//! reference dump taken on `master` at commit `322eba4a` (see
//! `docs/wiki/investigations/partyframe-tree.md`):
//!
//! ```text
//! PartyFrame          (120x244) visible LOW:2 x=22  y=147
//!   .MemberFrame1     (120x53)  visible LOW:2 x=22  y=147
//!   .MemberFrame2     (120x53)  visible LOW:2 x=22  y=210
//!   .MemberFrame3     (120x53)  visible LOW:2 x=22  y=273
//!   .MemberFrame4     (120x53)  visible LOW:2 x=22  y=336
//! ```
//!
//! The test suite now pins the remaining structural details that differed
//! from `master`: semantic child names and the EditMode selection subtree's
//! strata/level chain.

use crate::common;

use wow_ui_sim::dump::build_tree;
use wow_ui_sim::lua_api::WowLuaEnv;

const PARTY_FRAME_SELECTION_SIZE: &str = "120x244";

// PartyFrame itself has the shape `master` produces.
prefork_full_ui_case! {
fn party_frame_has_master_reference_shape(env: &WowLuaEnv) {
        env.exec("A_Admin.SetPartySize(4)").unwrap();
        env.exec(
            r#"
            if PartyFrame and PartyFrame.UpdatePartyFrames then
                pcall(PartyFrame.UpdatePartyFrames, PartyFrame)
            end
            "#,
        )
        .unwrap();

        let (exists, width, height, visible, x, _y): (bool, f64, f64, bool, f64, f64) = env
            .eval(
                r#"
                if not PartyFrame then return false, 0, 0, false, 0, 0 end
                local w, h = PartyFrame:GetSize()
                local x, y = PartyFrame:GetLeft() or 0, PartyFrame:GetTop() or 0
                return true, w, h, PartyFrame:IsVisible(), x, y
                "#,
            )
            .expect("eval PartyFrame");

        assert!(exists, "PartyFrame must be a global frame after addons load");
        assert!(visible, "PartyFrame must be IsVisible() after a 4-member party is set");
        // Master dump: size 120x244, top-left at (22, 147).
        assert_eq!(
            (width as i32, height as i32),
            (120, 244),
            "PartyFrame size must match master reference (got {width}x{height})",
        );
        assert_eq!(
            x as i32, 22,
            "PartyFrame left edge must be x=22 (got x={x})",
        );
}
}

prefork_full_ui_case! {
fn party_member_frame_hover_shows_unit_tooltip(env: &WowLuaEnv) {
        env.exec("A_Admin.SetPartySize(1)").unwrap();
        env.exec(
            r#"
            if PartyFrame and PartyFrame.UpdatePartyFrames then
                pcall(PartyFrame.UpdatePartyFrames, PartyFrame)
            end
            "#,
        )
        .unwrap();

        let result: String = env
            .eval(
                r#"
                local mf = PartyFrame and PartyFrame.MemberFrame1
                if not mf then
                    return "missing-member"
                end

                local handler = mf:GetScript("OnEnter")
                if type(handler) ~= "function" then
                    return "missing-enter"
                end

                local ok, err = pcall(handler, mf)
                if not ok then
                    return "error:" .. tostring(err)
                end

                local name, unit, guid = GameTooltip:GetUnit()
                return table.concat({
                    tostring(GameTooltip:IsVisible()),
                    tostring(GameTooltip:NumLines()),
                    tostring(name),
                    tostring(unit),
                    tostring(guid),
                }, "|")
                "#,
            )
            .expect("hover PartyFrame.MemberFrame1");

        assert_eq!(
            result,
            "true|6|Thrynn|party1|Player-0000-00000002",
            "PartyFrame.MemberFrame1 hover should show a unit tooltip for party1"
        );
}
}

// All four member frames populate with the 63px vertical stride master uses.
prefork_full_ui_case! {
fn party_frame_member_frames_render_at_master_offsets(env: &WowLuaEnv) {
        env.exec("A_Admin.SetPartySize(4)").unwrap();
        env.exec(
            r#"
            if PartyFrame and PartyFrame.UpdatePartyFrames then
                pcall(PartyFrame.UpdatePartyFrames, PartyFrame)
            end
            "#,
        )
        .unwrap();

        // Member 1..=4: (120x53) visible with y = 147, 210, 273, 336 (63px
        // stride). `GetTop` returns the bottom-up coordinate of the top edge.
        let results: Vec<(String, bool, f64, f64, f64, f64)> = (1..=4)
            .map(|i| {
                let key = format!("MemberFrame{i}");
                let eval_src = format!(
                    r#"
                    local mf = PartyFrame and PartyFrame.{key}
                    if not mf then return "{key}|missing", false, 0, 0, 0, 0 end
                    local w, h = mf:GetSize()
                    local x, y = mf:GetLeft() or 0, mf:GetTop() or 0
                    return "{key}|ok", mf:IsVisible(), w, h, x, y
                    "#
                );
                env.eval::<(String, bool, f64, f64, f64, f64)>(&eval_src)
                    .expect("eval MemberFrame")
            })
            .collect();

        let expected_y = [147.0, 210.0, 273.0, 336.0];
        for (idx, (tag, visible, w, h, x, y)) in results.iter().enumerate() {
            assert!(
                tag.ends_with("|ok"),
                "MemberFrame{} missing from PartyFrame ({tag})",
                idx + 1,
            );
            assert!(
                *visible,
                "MemberFrame{} must be IsVisible() with a 4-member party",
                idx + 1,
            );
            assert_eq!(
                (*w as i32, *h as i32),
                (120, 53),
                "MemberFrame{} size mismatch (got {w}x{h})",
                idx + 1,
            );
            assert_eq!(*x as i32, 22, "MemberFrame{} x mismatch", idx + 1);
            // GetTop uses WoW's Y-up coordinates, so equivalent layouts can
            // differ by sign depending on which edge the dump compared. Match
            // the absolute stride from the first member.
            let baseline_y = results[0].5;
            let rel = (*y - baseline_y).abs();
            let expected_rel = expected_y[idx] - expected_y[0];
            assert!(
                (rel - expected_rel).abs() < 1.0,
                "MemberFrame{} y offset mismatch: got {}, expected {} (baseline {})",
                idx + 1,
                rel,
                expected_rel,
                baseline_y,
            );
        }
}
}

prefork_full_ui_case! {
fn player_and_party_portraits_use_current_class_texture_identity(env: &WowLuaEnv) {
        env.exec("A_Admin.SetPartySize(4)").unwrap();
        env.exec(
            r#"
            if PartyFrame and PartyFrame.UpdatePartyFrames then
                pcall(PartyFrame.UpdatePartyFrames, PartyFrame)
            end
            if PlayerFrame_Update then
                pcall(PlayerFrame_Update)
            end
            "#,
        )
        .unwrap();

        let (player_atlas, party_atlas, player_texture, party_texture): (
            String,
            String,
            String,
            String,
        ) = env
            .eval(
                r#"
                local playerPortrait = PlayerFrame and PlayerFrame.PlayerFrameContainer and PlayerFrame.PlayerFrameContainer.PlayerPortrait
                local partyPortrait = PartyFrame and PartyFrame.MemberFrame1 and PartyFrame.MemberFrame1.Portrait
                return string.lower((playerPortrait and playerPortrait:GetAtlas()) or ""),
                       string.lower((partyPortrait and partyPortrait:GetAtlas()) or ""),
                       tostring(playerPortrait and playerPortrait:GetTexture()),
                       tostring(partyPortrait and partyPortrait:GetTexture())
                "#,
            )
            .expect("eval portrait fallback");

        assert_eq!(
            player_atlas, "",
            "player portrait should expose its authored texture identity without an atlas"
        );
        assert_eq!(
            player_texture, "237669",
            "player portrait should expose the current class-circle texture fileDataID"
        );
        assert_eq!(
            party_atlas, "",
            "party1 portrait should expose its authored texture identity without an atlas"
        );
        assert_eq!(
            party_texture, "237669",
            "party1 portrait should expose the current class-circle texture fileDataID"
        );
}
}

prefork_full_ui_case! {
fn party_frame_member_name_uses_master_font_size(env: &WowLuaEnv) {
        env.exec("A_Admin.SetPartySize(4)").unwrap();
        env.exec(
            r#"
            if PartyFrame and PartyFrame.UpdatePartyFrames then
                pcall(PartyFrame.UpdatePartyFrames, PartyFrame)
            end
            "#,
        )
        .unwrap();

        let (font_path, font_size, flags): (String, f64, String) = env
            .eval(
                r#"
                local name = PartyFrame and PartyFrame.MemberFrame1 and PartyFrame.MemberFrame1.Name
                if not name then
                    return "", 0, ""
                end
                local path, size, outline = name:GetFont()
                return path or "", size or 0, outline or ""
                "#,
            )
            .expect("eval PartyFrame.MemberFrame1.Name font");

        let normalized_font_path = font_path.replace('/', "\\");
        assert_eq!(
            normalized_font_path, "Fonts\\FRIZQT__.TTF",
            "party member names should inherit the master font path",
        );
        assert_eq!(
            font_size, 10.0,
            "party member names should inherit GameFontNormalSmall size, got {font_size} with flags {flags}",
        );
}
}

// Structural sanity: the four decorative templates master emits
// (Selection + Background + Selection.MouseOverHighlight.Center) are
// present on the branch too.
prefork_full_ui_case! {
fn party_frame_has_background_and_selection_children(env: &WowLuaEnv) {
        env.exec("A_Admin.SetPartySize(4)").unwrap();
        env.exec(
            r#"
            if PartyFrame and PartyFrame.UpdatePartyFrames then
                pcall(PartyFrame.UpdatePartyFrames, PartyFrame)
            end
            "#,
        )
        .unwrap();

        let (has_selection, has_background, selection_w, background_w): (
            bool,
            bool,
            f64,
            f64,
        ) = env
            .eval(
                r#"
                if not PartyFrame then return false, false, 0, 0 end
                local sel = PartyFrame.Selection
                local bg = PartyFrame.Background
                local sw = sel and sel.GetWidth and sel:GetWidth() or 0
                local bw = bg and bg.GetWidth and bg:GetWidth() or 0
                return sel ~= nil, bg ~= nil, sw, bw
                "#,
            )
            .expect("eval PartyFrame decorations");

        assert!(has_selection, "PartyFrame.Selection must be attached");
        assert!(has_background, "PartyFrame.Background must be attached");
        // Master: Selection 120, Background 144.
        assert_eq!(
            selection_w as i32, 120,
            "PartyFrame.Selection width mismatch (got {selection_w})",
        );
        assert_eq!(
            background_w as i32, 144,
            "PartyFrame.Background width mismatch (got {background_w})",
        );
}
}

prefork_full_ui_case! {
fn party_frame_selection_tracks_parent_size_in_registry(env: &WowLuaEnv) {
        env.exec("A_Admin.SetPartySize(4)").unwrap();
        env.exec(
            r#"
            if PartyFrame and PartyFrame.UpdatePartyFrames then
                pcall(PartyFrame.UpdatePartyFrames, PartyFrame)
            end
            "#,
        )
        .unwrap();
        let selection_width_before_ensure: f64 = env
            .eval(
                r#"
                if not PartyFrame or not PartyFrame.Selection then return 0 end
                return PartyFrame.Selection:GetWidth()
                "#,
            )
            .expect("read PartyFrame.Selection width before ensure_layout_rects");

        let state = env.state();
        let sim = state.borrow();
        let party_id = sim
            .widgets
            .get_id_by_name("PartyFrame")
            .expect("PartyFrame id");
        let party = sim.widgets.get(party_id).expect("PartyFrame widget");
        let selection_id = *party
            .children_keys
            .get("Selection")
            .expect("PartyFrame.Selection child id");
        let selection = sim.widgets.get(selection_id).expect("Selection widget");

        assert_eq!(
            selection.parent_id,
            Some(party_id),
            "PartyFrame.Selection must stay parented to PartyFrame",
        );
        assert!(
            party.children.contains(&selection_id),
            "PartyFrame.Selection must stay in PartyFrame.children",
        );
        assert_eq!(
            selection.anchors.len(),
            2,
            "PartyFrame.Selection must keep TOPLEFT/BOTTOMRIGHT anchors",
        );
        assert!(
            sim.widgets.is_rect_dirty(selection_id) || selection.layout_rect.is_some(),
            "PartyFrame.Selection must either be dirty or already have a layout rect",
        );
        drop(sim);

        state.borrow_mut().ensure_layout_rects();
        let sim = state.borrow();
        let party = sim.widgets.get(party_id).expect("PartyFrame widget");
        let selection = sim.widgets.get(selection_id).expect("Selection widget");
        let party_rect = party.layout_rect.expect("PartyFrame layout rect");
        let selection_rect = selection.layout_rect.expect("Selection layout rect");
        assert_eq!(
            selection_width_before_ensure as i32,
            selection_rect.width as i32,
            "Lua GetWidth() must agree with the resolved registry width",
        );
        assert_eq!(
            selection_rect.width as i32,
            party_rect.width as i32,
            "PartyFrame.Selection cached width must track PartyFrame width (selection={selection_rect:?}, party={party_rect:?})",
        );
        assert_eq!(
            selection_rect.height as i32,
            party_rect.height as i32,
            "PartyFrame.Selection cached height must track PartyFrame height (selection={selection_rect:?}, party={party_rect:?})",
        );
}
}

prefork_full_ui_case! {
fn party_frame_dump_tree_excludes_builtin_ghost_frame(env: &WowLuaEnv) {
        env.exec("A_Admin.SetPartySize(4)").unwrap();
        env.exec(
            r#"
            if PartyFrame and PartyFrame.UpdatePartyFrames then
                pcall(PartyFrame.UpdatePartyFrames, PartyFrame)
            end
            "#,
        )
        .unwrap();

        let state = env.state();
        let sim = state.borrow();
        let addon_names: Vec<String> = sim.addons.iter().map(|a| a.folder_name.clone()).collect();
        let lines = build_tree(
            &sim.widgets,
            &addon_names,
            None,
            Some("PartyFrame"),
            true,
            false,
            1024.0,
            768.0,
        );
        let party_roots: Vec<&String> = lines
            .iter()
            .filter(|line| line.contains("PartyFrame [Frame]"))
            .collect();

        assert_eq!(
            party_roots.len(),
            1,
            "dump tree should expose exactly one visible PartyFrame root, got:\n{}",
            party_roots
                .iter()
                .map(|line| line.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        );
        assert!(
            party_roots[0].contains("@Blizzard_UnitFrame"),
            "visible PartyFrame root must be owned by Blizzard_UnitFrame, got:\n{}",
            party_roots[0],
        );
}
}

prefork_full_ui_case! {
fn party_frame_member_frame1_uses_semantic_child_names(env: &WowLuaEnv) {
        env.exec("A_Admin.SetPartySize(4)").unwrap();
        env.exec(
            r#"
            if PartyFrame and PartyFrame.UpdatePartyFrames then
                pcall(PartyFrame.UpdatePartyFrames, PartyFrame)
            end
            "#,
        )
        .unwrap();

        let draw_layers: (String, i32, String, i32, String, i32) = env
            .eval(
                r#"
                local member = assert(PartyFrame and PartyFrame.MemberFrame1)
                local portraitLayer, portraitSublevel = member.Portrait:GetDrawLayer()
                local flashLayer, flashSublevel = member.Flash:GetDrawLayer()
                local nameLayer, nameSublevel = member.Name:GetDrawLayer()
                return portraitLayer, portraitSublevel,
                       flashLayer, flashSublevel,
                       nameLayer, nameSublevel
                "#,
            )
            .expect("query MemberFrame1 region draw layers");
        assert_eq!(
            draw_layers,
            (
                "BACKGROUND".to_string(),
                0,
                "ARTWORK".to_string(),
                0,
                "ARTWORK".to_string(),
                0,
            ),
            "MemberFrame1 regions must retain their current retail XML draw layers",
        );

        let state = env.state();
        let sim = state.borrow();
        let addon_names: Vec<String> = sim.addons.iter().map(|a| a.folder_name.clone()).collect();
        let lines = build_tree(
            &sim.widgets,
            &addon_names,
            None,
            Some("PartyFrame"),
            false,
            false,
            1024.0,
            768.0,
        );
        let dump = lines.join("\n");

        assert!(
            dump.contains(&format!(
                ".Selection [Frame] ({PARTY_FRAME_SELECTION_SIZE}) [stored=1x1] hidden LOW:1000"
            )),
            "PartyFrame.Selection must use the current retail frame level 1000, got:\n{dump}",
        );
        assert!(
            dump.contains(&format!(
                ".MouseOverHighlight [Frame] ({PARTY_FRAME_SELECTION_SIZE}) hidden LOW:1001"
            )),
            "PartyFrame.Selection.MouseOverHighlight must inherit level 1001, got:\n{dump}",
        );
        assert!(
            dump.contains(".TopLeftCorner [Texture] (16x16) hidden LOW:1002"),
            "PartyFrame.Selection.MouseOverHighlight corners must inherit level 1002, got:\n{dump}",
        );
        assert!(
            dump.contains(".MemberFrame1 [Button] (120x53) visible LOW:2"),
            "MemberFrame1 must be present in the dump, got:\n{dump}",
        );
        assert!(
            dump.contains(".Portrait [Texture] (37x37) visible"),
            "MemberFrame1 portrait must keep its Blizzard parentKey, type, size, and visibility, got:\n{dump}",
        );
        assert!(
            dump.contains(".Flash [Texture] (114x47) hidden"),
            "MemberFrame1 flash must keep its Blizzard parentKey, type, size, and visibility, got:\n{dump}",
        );
        assert!(
            dump.contains(".Name [FontString] (57x12) visible"),
            "MemberFrame1 name must keep its Blizzard parentKey, type, size, and visibility, got:\n{dump}",
        );
        assert!(
            dump.contains(".PowerBarAlt [Frame] (0x0) hidden LOW:3"),
            "MemberFrame1.PowerBarAlt must be present, got:\n{dump}",
        );
        for expected in [
            ".background [Texture] (0x0) hidden LOW:4",
            ".fill [Texture] (0x0) hidden LOW:4",
            ".frame [Texture] (0x0) hidden LOW:4",
            ".spark [Texture] (0x0) hidden LOW:4",
            ".BG [Texture] (16x64) hidden LOW:5",
            ".BGL [Texture] (32x64) hidden LOW:5",
            ".BGR [Texture] (32x64) hidden LOW:5",
        ] {
            assert!(
                dump.contains(expected),
                "MemberFrame1.PowerBarAlt must expose semantic child `{expected}`, got:\n{dump}",
            );
        }
}
}
