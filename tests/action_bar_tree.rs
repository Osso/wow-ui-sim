//! ActionBar tree regression test.
//!
//! Captures the ActionButton1 shape master renders vs. rilua-migration. On
//! `master` at commit `322eba4a`, ActionButton1 has `Icon`, `HotKey`, and
//! `NormalTexture` children — and `ActionButton1Icon` carries a real spell
//! texture (`Interface\ICONS\Spell_Holy_FlashHeal` in the default seeded
//! layout). On `rilua-migration` those child textures never get attached, so
//! every action bar slot renders as an empty gray square.
//!
//! See `docs/wiki/investigations/partyframe-tree.md` — the same
//! `intern_string_static` registry-key mismatch that drops PartyFrame's
//! member enumeration also suppresses the button children that carry the
//! spell icon and keybind text.
//!
//! Master reference (dump-tree --filter ActionButton1):
//!
//! ```text
//! ActionButton1               [CheckButton] (45x45) visible MEDIUM:52 x=519 y=1110
//!   ActionButton1Icon         [Texture]     (45x45) visible MEDIUM:53 x=519 y=1110
//!     [texture] Interface\ICONS\Spell_Holy_FlashHeal
//!   ActionButton1Name         [FontString]  (36x0)  visible MEDIUM:53 x=523 y=1153
//!   ActionButton1NormalTexture[Texture]     (46x45) visible MEDIUM:53 x=519 y=1110
//!     [atlas] UI-HUD-ActionBar-IconFrame
//!     ActionButton1HotKey     [FontString]  (37x10) visible MEDIUM:54 x=523 y=1115 text="1"
//!     ActionButton1Count      [FontString]  (0x0)   visible MEDIUM:54 x=559 y=1150
//! ```

use crate::common;

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

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn action_bar_addons() -> &'static [&'static str] {
    ACTION_BAR_ADDONS
}

fn fire_startup_events(env: &WowLuaEnv) {
    let _ = env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("WoWUISim")]);
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    let _ = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[rilua::Val::Bool(true), rilua::Val::Bool(false)],
    );
    let _ = env.fire_edit_mode_layouts_updated();
    let _ = env.fire_event("ACTIONBAR_SHOWGRID");
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
}

fn load_settled_game_ui() -> common::LockedEnv {
    common::lock_env(|| {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");
        env.set_screen_size(1024.0, 768.0);
        env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];

        if !common::load_classic_game_startup_inventory(&env) {
            let ui = blizzard_ui_dir();
            for addon_name in action_bar_addons() {
                common::load_required_blizzard_addon(&env, &ui, addon_name);
            }
        }

        env.apply_post_load_workarounds();
        fire_startup_events(&env);
        env
    })
}

#[test]
fn action_button1_has_icon_hotkey_and_normal_texture() {
    wrath_action_case!(button_children);

    test_timeout! {
        let env = load_settled_game_ui();

        // The slot must exist with the frame itself visible at master's
        // geometry first — if ActionButton1 isn't there, nothing else is.
        let (ab1_exists, ab1_w, ab1_h, ab1_visible): (bool, f64, f64, bool) = env
            .eval(
                r#"
                if not ActionButton1 then return false, 0, 0, false end
                local w, h = ActionButton1:GetSize()
                return true, w, h, ActionButton1:IsVisible()
                "#,
            )
            .expect("eval ActionButton1");
        assert!(ab1_exists, "ActionButton1 must exist after addons load");
        assert!(ab1_visible, "ActionButton1 must be IsVisible()");
        assert_eq!(
            (ab1_w as i32, ab1_h as i32),
            (45, 45),
            "ActionButton1 size must be 45x45",
        );

        // Master has three critical child regions on every action button:
        //   * Icon (holds the spell texture)
        //   * HotKey (shows the keybind text "1"..="=")
        //   * NormalTexture (the gold action-bar frame overlay)
        // On rilua-migration none of these exist, so the slot renders empty.
        let (has_icon, has_hotkey, has_normal): (bool, bool, bool) = env
            .eval(
                r#"
                return
                    _G.ActionButton1Icon ~= nil,
                    _G.ActionButton1HotKey ~= nil,
                    _G.ActionButton1NormalTexture ~= nil
                "#,
            )
            .expect("eval action button children");

        assert!(has_icon, "ActionButton1Icon global must be defined");
        assert!(
            has_hotkey,
            "ActionButton1HotKey global must be defined — keybind text needs it",
        );
        assert!(
            has_normal,
            "ActionButton1NormalTexture global must be defined — gold frame overlay needs it",
        );
    }
}

#[test]
fn action_button1_normal_texture_uses_iconframe_atlas() {
    wrath_action_case!(button_normal_texture);

    test_timeout! {
        let env = load_settled_game_ui();

        let (atlas, width, height, visible): (String, f64, f64, bool) = env
            .eval(
                r#"
                local nt = _G.ActionButton1NormalTexture
                if not nt then return "<missing>", 0, 0, false end
                local a = (nt.GetAtlas and nt:GetAtlas()) or ""
                local w, h = nt:GetSize()
                return a, w, h, nt:IsVisible()
                "#,
            )
            .expect("eval ActionButton1NormalTexture");

        assert_ne!(
            atlas, "<missing>",
            "ActionButton1NormalTexture must exist so the gold action-bar frame can render",
        );
        assert_eq!(
            atlas.to_ascii_lowercase(),
            "ui-hud-actionbar-iconframe",
            "ActionButton1NormalTexture must use UI-HUD-ActionBar-IconFrame (got {atlas:?})",
        );
        assert!(visible, "ActionButton1NormalTexture must be IsVisible()");
        // Master dump: 46x45 (NormalTexture is 1px wider than the button to
        // cover both edges of the gold frame).
        assert_eq!((width as i32, height as i32), (46, 45));
    }
}

#[test]
fn action_button1_hotkey_shows_slot_one() {
    wrath_action_case!(button_hotkey);

    test_timeout! {
        let env = load_settled_game_ui();

        let (text, visible): (String, bool) = env
            .eval(
                r#"
                local hk = _G.ActionButton1HotKey
                if not hk then return "<missing>", false end
                local txt = hk.GetText and hk:GetText() or ""
                return txt or "", hk:IsVisible()
                "#,
            )
            .expect("eval ActionButton1HotKey");

        assert_ne!(text, "<missing>", "ActionButton1HotKey must exist");
        assert!(visible, "ActionButton1HotKey must be IsVisible()");
        // Master default binding set labels slot 1 as "1". Accept either the
        // single-digit label or the full "ACTIONBUTTON1" keybind placeholder
        // Blizzard uses before the keybinding mapping finishes.
        assert!(
            text == "1" || text == "ACTIONBUTTON1",
            "ActionButton1HotKey text should be \"1\" (mapped) or \"ACTIONBUTTON1\" (unmapped), got {text:?}",
        );
    }
}

#[test]
fn main_action_bar_layout_is_locked() {
    wrath_action_case!(main_layout);

    test_timeout! {
        let env = load_settled_game_ui();

        let endcap_texture_access = if cfg!(feature = "client-ptr") {
            ".Texture"
        } else {
            ""
        };
        let script = r#"
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

            local function rect(path, frame)
                local l = frame:GetLeft()
                local r = frame:GetRight()
                local t = frame:GetTop()
                local b = frame:GetBottom()
                local w = frame:GetWidth()
                local h = frame:GetHeight()
                assert(type(l) == "number", path .. ":GetLeft() missing")
                assert(type(r) == "number", path .. ":GetRight() missing")
                assert(type(t) == "number", path .. ":GetTop() missing")
                assert(type(b) == "number", path .. ":GetBottom() missing")
                assert(type(w) == "number", path .. ":GetWidth() missing")
                assert(type(h) == "number", path .. ":GetHeight() missing")
                return l, r, t, b, w, h
            end

            local function center(path, frame)
                local x, y = frame:GetCenter()
                assert(type(x) == "number", path .. ":GetCenter() x missing")
                assert(type(y) == "number", path .. ":GetCenter() y missing")
                return x, y
            end

            local bar = require_frame("MainActionBar", MainActionBar)
            local endCaps = require_frame("MainActionBar.EndCaps", bar.EndCaps)
            local leftEndCap = require_frame("MainActionBar.EndCaps.LeftEndCap", endCaps.LeftEndCap)
            local rightEndCap = require_frame("MainActionBar.EndCaps.RightEndCap", endCaps.RightEndCap)
            local page = require_frame("MainActionBar.ActionBarPageNumber", bar.ActionBarPageNumber)
            local pageUp = require_frame("MainActionBar.ActionBarPageNumber.UpButton", page.UpButton)
            local pageDown = require_frame("MainActionBar.ActionBarPageNumber.DownButton", page.DownButton)
            local pageText = require_frame("MainActionBar.ActionBarPageNumber.Text", page.Text)

            local bL, bR, bT, bB, bW, bH = rect("MainActionBar", bar)
            assert(bar:IsVisible(), "MainActionBar must be visible")
            assert(approx(bW, 562, 0.1), "MainActionBar width changed: " .. tostring(bW))
            assert(approx(bH, 45, 0.1), "MainActionBar height changed: " .. tostring(bH))
            assert(approx(bar:GetAlpha() or -1, 1, 0.01), "MainActionBar alpha changed")

            local eL, eR, eT, eB, eW, eH = rect("MainActionBar.EndCaps", endCaps)
            assert(endCaps:IsShown(), "MainActionBar.EndCaps must be shown")
            assert(approx(eL, bL), "EndCaps left drifted")
            assert(approx(eR, bR), "EndCaps right drifted")
            assert(approx(eT, bT), "EndCaps top drifted")
            assert(approx(eB, bB), "EndCaps bottom drifted")
            assert(approx(eW, bW, 0.1), "EndCaps width changed: " .. tostring(eW))
            assert(approx(eH, bH, 0.1), "EndCaps height changed: " .. tostring(eH))

            local leL, leR, _, leB, leW, leH = rect("LeftEndCap", leftEndCap)
            local reL, reR, _, reB, reW, reH = rect("RightEndCap", rightEndCap)
            local leftTexture = require_frame(
                "MainActionBar.EndCaps.LeftEndCap__ENDCAP_TEXTURE_ACCESS__",
                leftEndCap__ENDCAP_TEXTURE_ACCESS__
            )
            local rightTexture = require_frame(
                "MainActionBar.EndCaps.RightEndCap__ENDCAP_TEXTURE_ACCESS__",
                rightEndCap__ENDCAP_TEXTURE_ACCESS__
            )
            assert((leftTexture:GetAtlas() or ""):lower() == "ui-hud-actionbar-gryphon-left", "LeftEndCap atlas changed")
            assert((rightTexture:GetAtlas() or ""):lower() == "ui-hud-actionbar-gryphon-right", "RightEndCap atlas changed")
            assert(leW > 0 and leH > 0, "LeftEndCap size must be non-zero")
            assert(reW > 0 and reH > 0, "RightEndCap size must be non-zero")
            assert(approx(leW, reW, 0.1), "EndCap widths diverged")
            assert(approx(leH, reH, 0.1), "EndCap heights diverged")
            assert(approx(leR, bL + 9), "LeftEndCap right anchor offset changed")
            assert(approx(reL, bR - 8), "RightEndCap left anchor offset changed")
            assert(approx(leB, bB - 22), "LeftEndCap bottom anchor offset changed")
            assert(approx(reB, bB - 22), "RightEndCap bottom anchor offset changed")

            local pL, pR, _, pB, pW, pH = rect("ActionBarPageNumber", page)
            assert(page:IsVisible(), "ActionBarPageNumber must be visible")
            assert(approx(pW, 18, 0.1) and approx(pH, 34, 0.1), "ActionBarPageNumber size changed")
            assert(approx(pR, bL - 4), "ActionBarPageNumber right anchor offset changed")
            assert(approx(pB, bB + 9), "ActionBarPageNumber bottom anchor offset changed")

            local _, upCY = center("UpButton", pageUp)
            local _, downCY = center("DownButton", pageDown)
            local _, pageCY = center("ActionBarPageNumber", page)
            assert(pageUp:IsVisible(), "ActionBar page up button must be visible")
            assert(pageDown:IsVisible(), "ActionBar page down button must be visible")
            assert(approx(math.abs(upCY - pageCY), 10), "ActionBar page up button offset changed")
            assert(approx(math.abs(downCY - pageCY), 10), "ActionBar page down button offset changed")
            local pageNumberText = pageText:GetText() or ""
            assert(pageNumberText ~= "", "ActionBar page text must be non-empty")

            -- Lock all 12 main bar button container slots to 47px stride
            -- (45px button + 2px gap) from MainActionBar's left edge.
            for i = 1, 12 do
                local container = require_frame("MainActionBarButtonContainer" .. i, _G["MainActionBarButtonContainer" .. i])
                local button = require_frame("ActionButton" .. i, _G["ActionButton" .. i])
                local icon = require_frame("ActionButton" .. i .. "Icon", _G["ActionButton" .. i .. "Icon"])
                local normal = require_frame("ActionButton" .. i .. "NormalTexture", _G["ActionButton" .. i .. "NormalTexture"])

                local cL, _, _, cB, cW, cH = rect("MainActionBarButtonContainer" .. i, container)
                local expectedLeft = bL + (i - 1) * 47
                assert(approx(cL, expectedLeft), "button container " .. i .. " left drifted")
                assert(approx(cB, bB), "button container " .. i .. " bottom drifted")
                assert(approx(cW, 45, 0.1) and approx(cH, 45, 0.1), "button container " .. i .. " size changed")

                local b2L, b2R, b2T, b2B, b2W, b2H = rect("ActionButton" .. i, button)
                assert(button:IsVisible(), "ActionButton" .. i .. " must be visible")
                assert(approx(b2L, cL) and approx(b2B, cB), "ActionButton" .. i .. " drifted from container")
                assert(approx(b2W, 45, 0.1) and approx(b2H, 45, 0.1), "ActionButton" .. i .. " size changed")
                assert((button:GetAlpha() or -1) > 0.95, "ActionButton" .. i .. " alpha changed")

                local iL, iR, iT, iB = icon:GetLeft(), icon:GetRight(), icon:GetTop(), icon:GetBottom()
                assert(iL and iR and iT and iB, "ActionButton" .. i .. "Icon bounds missing")
                assert(approx(iL, b2L) and approx(iR, b2R), "ActionButton" .. i .. "Icon x drifted")
                assert(approx(iT, b2T) and approx(iB, b2B), "ActionButton" .. i .. "Icon y drifted")

                local nAtlas = normal.GetAtlas and normal:GetAtlas() or ""
                assert(nAtlas:lower() == "ui-hud-actionbar-iconframe", "ActionButton" .. i .. "NormalTexture atlas changed")
            end

            local hk = require_frame("ActionButton1HotKey", ActionButton1HotKey)
            assert(hk:IsVisible(), "ActionButton1HotKey must be visible")
            local hkText = hk:GetText() or ""
            assert(hkText == "1" or hkText == "ACTIONBUTTON1", "ActionButton1HotKey text changed: " .. tostring(hkText))

            local a1 = require_frame("ActionButton1", ActionButton1)
            local a2 = require_frame("ActionButton2", ActionButton2)
            local a12 = require_frame("ActionButton12", ActionButton12)
            local a1L, a1R = a1:GetLeft(), a1:GetRight()
            local a2L, _ = a2:GetLeft(), a2:GetRight()
            local _, a12R = a12:GetLeft(), a12:GetRight()
            assert(a1L and a1R and a2L and a12R, "action button bounds missing for spacing checks")
            assert(approx(a2L - a1L, 47), "ActionButton1->2 spacing changed")
            assert(approx(a12R, bR), "ActionButton12 right edge no longer matches MainActionBar")
            "#;
        let script = script.replace("__ENDCAP_TEXTURE_ACCESS__", endcap_texture_access);
        env.exec(&script)
            .expect("main action bar layout lock assertions");
    }
}
