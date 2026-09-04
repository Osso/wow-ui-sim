#![cfg(all(feature = "profile-retail", feature = "retail-12-1-0"))]

use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{fire_one_on_update_tick, fire_startup_events, process_pending_timers};

const ACCOUNT_CACHE: &str = include_str!("fixtures/frame_position_replay/edit-mode-cache-account.txt");

#[test]
fn captured_ultrawide_configuration_reproduces_frame_anchors_and_rects() {
    test_timeout! {
        let env = load_captured_configuration();
        env.exec(r#"
            local function near(actual, expected, label)
                assert(math.abs(actual - expected) <= 1,
                       label .. ": expected " .. expected .. ", got " .. actual)
            end
            local function rect(frame, left, bottom, width, height)
                local x, y, w, h = frame:GetRect()
                local name = frame:GetName()
                near(x, left, name .. ".left")
                near(y, bottom, name .. ".bottom")
                near(w, width, name .. ".width")
                near(h, height, name .. ".height")
            end
            local function anchor(frame, point, relative, relativePoint, x, y)
                assert(frame:GetNumPoints() == 1, frame:GetName() .. " anchor count")
                local p, r, rp, ox, oy = frame:GetPoint(1)
                assert(p == point and r == relative and rp == relativePoint,
                       frame:GetName() .. " anchor relationship")
                near(ox, x, frame:GetName() .. ".offsetX")
                near(oy, y, frame:GetName() .. ".offsetY")
            end
            local layout = EditModeManagerFrame:GetActiveLayoutInfo()
            assert(layout.layoutName == "Ultrawide" and layout.layoutType == 1)
            local pw, ph = GetPhysicalScreenSize()
            assert(pw == 3440 and ph == 1440)
            assert(math.abs(UIParent:GetEffectiveScale() - 0.7999999523162842) < 0.00001)
            assert(not InCombatLockdown())
            assert(RaidWarningFrame:GetLowestMessage() == nil)
            assert(not DeadlyDebuffFrame:IsShown())
            rect(UIParent, 0, 0, 2293.333251953125, 960.0001220703125)
            rect(RaidWarningFrame, 746.6666870117188, 678.0000610351562, 800.0000610351562, 99.9999771118164)
            rect(PrivateRaidBossEmoteFrameAnchor, 746.6666870117188, 698.0000610351562, 800.0000610351562, 79.9999771118164)
            anchor(RaidWarningFrame, "TOP", UIParent, "TOP", 0, -182)
            anchor(PrivateRaidBossEmoteFrameAnchor, "TOP", RaidWarningFrame, "TOP", 0, 0)
            assert(UIParentRightManagedFrameContainer == nil)
            anchor(RightManagedFrameContainer, "TOPRIGHT", UIParent, "TOPRIGHT", -5, -260)
            rect(RightManagedFrameContainer, 2287.33349609375, 92.49992370605469, 0.9999628067016602, 607.5001220703125)
            assert(ObjectiveTrackerFrame:GetParent() == UIParent)
            anchor(ObjectiveTrackerFrame, "RIGHT", UIParent, "RIGHT", 0, -25.39999961853027)
            rect(ObjectiveTrackerFrame, 2033.333740234375, 179.6000518798828, 259.9999389648438, 550.0000610351562)
        "#).expect("Blizzard startup must reproduce captured output from captured configuration");
    }
}

fn load_captured_configuration() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create replay environment");
    env.set_display_size(3440.0, 1440.0)
        .expect("set physical display input");
    load_captured_layout(&env);
    load_replay_addons(&env);
    env.apply_post_load_workarounds();
    env.exec("SetCVar('uiScale', '0.79999995231628'); SetCVar('useUiScale', '1')")
        .expect("apply captured scale before login events");
    settle_captured_startup(&env);
    env
}

fn load_captured_layout(env: &WowLuaEnv) {
    let cache = ACCOUNT_CACHE.trim_end_matches('\n');
    env.exec(&format!(
        "C_EditMode.__LoadCache([=[{cache}]=] .. string.char(0), nil, 1, 'Ultrawide')"
    ))
    .expect("load captured account layout input");
}

fn load_replay_addons(env: &WowLuaEnv) {
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    for (name, path) in discover_blizzard_addons(&ui) {
        load_addon(&env.loader_env(), &path)
            .unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
}

fn settle_captured_startup(env: &WowLuaEnv) {
    fire_startup_events(env);
    env.apply_post_event_workarounds();
    env.state().borrow_mut().widgets.rebuild_anchor_index();
    process_pending_timers(env);
    fire_one_on_update_tick(env);
    std::thread::sleep(std::time::Duration::from_secs(2));
    for _ in 0..3 {
        env.state().borrow_mut().ensure_layout_rects();
        fire_one_on_update_tick(env);
        process_pending_timers(env);
    }
}
