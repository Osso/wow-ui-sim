use crate::common;
use crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env;

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn env_with_restricted_environment() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create env");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    for (name, toc_path) in discover_blizzard_addons(&ui) {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("{name} should load before group header test: {error}"));
        if name == "Blizzard_RestrictedAddOnEnvironment" {
            break;
        }
    }

    env
}

#[test]
fn secure_raid_group_header_spawns_raid_unit_children() {
    test_timeout! {
        let env = env_with_restricted_environment();

        let units: String = env.eval(
            r#"
            A_Admin.SetPartySize(7)
            local header = CreateFrame("Frame", "TestRaidHeader", UIParent, "SecureGroupHeaderTemplate")
            header:SetAttribute("showRaid", true)
            header:SetAttribute("template", "SecureUnitButtonTemplate")
            header:SetAttribute("point", "TOP")
            header:SetAttribute("xOffset", 0)
            header:SetAttribute("yOffset", -12)
            header:SetAttribute("unitsPerColumn", 4)
            header:SetAttribute("maxColumns", 2)
            header:SetAttribute("columnAnchorPoint", "LEFT")
            header:SetAttribute("columnSpacing", 20)
            header:Show()

            local out = {}
            for index = 1, 8 do
                local child = header:GetAttribute("child" .. index)
                out[index] = child and tostring(child:GetAttribute("unit")) or "nil"
            end

            local child2 = header:GetAttribute("child2")
            local child5 = header:GetAttribute("child5")
            local p2, _, rp2, x2, y2 = child2:GetPoint(1)
            local p5, _, rp5, x5, y5 = child5:GetPoint(1)
            local layout = table.concat({p2, rp2, tostring(x2), tostring(y2), p5, rp5, tostring(x5), tostring(y5)}, ":")
            return table.concat(out, ",") .. ";" .. layout
            "#,
        ).expect("raid header should update through Blizzard native Lua");

        assert_eq!(
            units,
            "raid1,raid2,raid3,raid4,raid5,raid6,raid7,raid8;TOP:BOTTOM:0:-12:LEFT:RIGHT:20:0"
        );
    }
}

#[test]
fn secure_group_pet_header_spawns_pet_child() {
    test_timeout! {
        let env = env_with_restricted_environment();

        let unit: String = env.eval(
            r#"
            A_Admin.SetPartySize(0)
            local header = CreateFrame("Frame", "TestPetHeader", UIParent, "SecureGroupPetHeaderTemplate")
            header:SetAttribute("showSolo", true)
            header:SetAttribute("template", "SecureUnitButtonTemplate")
            header:Show()

            local child1 = header:GetAttribute("child1")
            local child2 = header:GetAttribute("child2")
            return tostring(child1 and child1:GetAttribute("unit")) .. "," .. tostring(child2)
            "#,
        ).expect("pet header should update through Blizzard native Lua");

        assert_eq!(unit, "pet,nil");
    }
}

fn assert_auto_hide_no_lua_errors(env: &WowLuaEnv, stage: &str) {
    // Counts only: do not dump cached-addon errors or arbitrary Lua payloads.
    assert_eq!(
        env.state().borrow().lua_errors.len(),
        0,
        "{stage}: Lua error count"
    );
}

fn load_auto_hide_fixture() -> (WowLuaEnv, (f32, f32), (f32, f32)) {
    let env = load_auto_hide_real_dependencies();
    prepare_auto_hide_real_frame_and_caller(&env);
    let (interior, exterior) = observe_auto_hide_resolved_cursor_points(&env);
    (env, interior, exterior)
}

fn load_auto_hide_real_dependencies() -> WowLuaEnv {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("auto-hide fixture requires the active profile Blizzard UI cache");
    let (env, loaded) =
        build_blizzard_addon_closure_env(&ui, &["Blizzard_RestrictedAddOnEnvironment"], &[]);
    for prerequisite in ["Blizzard_FrameXML", "Blizzard_RestrictedAddOnEnvironment"] {
        assert!(
            loaded.iter().any(|name| name == prerequisite),
            "missing {prerequisite}"
        );
        assert!(
            env.state()
                .borrow()
                .addons
                .iter()
                .any(|addon| { addon.folder_name == prerequisite && addon.loaded }),
            "{prerequisite} did not finish loading"
        );
    }
    assert_auto_hide_no_lua_errors(&env, "cached dependency closure");
    env
}

fn prepare_auto_hide_real_frame_and_caller(env: &WowLuaEnv) {
    prepare_auto_hide_real_frame(env);
    define_auto_hide_tainted_callers(env);
    assert_auto_hide_no_lua_errors(env, "frame and caller preparation");
}

fn prepare_auto_hide_real_frame(env: &WowLuaEnv) {
    assert!(env.exec(r#"
        assert(type(RegisterAutoHide) == 'function', 'registration unavailable')
        assert(type(UnregisterAutoHide) == 'function', 'unregistration unavailable')
        assert(SecureHoverDriverManager, 'vendor manager unavailable')
        assert(SecureHoverDriverManager:GetScript('OnUpdate'), 'vendor update unavailable')
        assert(SecureHoverDriverManager:GetScript('OnAttributeChanged'), 'vendor request handler unavailable')

        AutoHideRuntimeTarget = CreateFrame('Frame', nil, UIParent)
        AutoHideRuntimeTarget:SetScale(1)
        AutoHideRuntimeTarget:SetSize(120, 80)
        AutoHideRuntimeTarget:SetPoint('BOTTOMLEFT', UIParent, 'BOTTOMLEFT', 200, 200)
        AutoHideRuntimeTarget:Hide()
        AutoHideRuntimeTarget:Show()
        assert(AutoHideRuntimeTarget:IsShown() and AutoHideRuntimeTarget:IsVisible(), 'target not visible')
    "#).is_ok(), "prepare actual vendor auto-hide fixture");
}

fn define_auto_hide_tainted_callers(env: &WowLuaEnv) {
    assert!(
        env.exec(
            r#"
        local function register()
            assert(not issecure(), 'registration requires the fixture addon caller')
            RegisterAutoHide(AutoHideRuntimeTarget, 1.0)
        end
        local function unregister()
            assert(not issecure(), 'unregistration requires the fixture addon caller')
            UnregisterAutoHide(AutoHideRuntimeTarget)
        end
        debug.setobjecttaint(register, 'AutoHideRuntimeFixture')
        debug.setobjecttaint(unregister, 'AutoHideRuntimeFixture')
        AutoHideRuntimeRegister = register
        AutoHideRuntimeUnregister = unregister
    "#
        )
        .is_ok(),
        "prepare actual vendor auto-hide fixture"
    );
}

fn observe_auto_hide_resolved_cursor_points(env: &WowLuaEnv) -> ((f32, f32), (f32, f32)) {
    // GetRect resolves dirty layout. Normalize the observed rectangle exactly
    // as the vendor does; do not assume the requested anchor was resolved.
    let (left, right, bottom, top): (f64, f64, f64, f64) = env.eval(r#"
        local left, bottom, width, height = AutoHideRuntimeTarget:GetRect()
        local scale = AutoHideRuntimeTarget:GetEffectiveScale()
        assert(left and bottom and width > 0 and height > 0 and scale > 0, 'target geometry unavailable')
        return left * scale, (left + width) * scale, bottom * scale, (bottom + height) * scale
    "#).unwrap_or_else(|_| panic!("observe resolved auto-hide target geometry"));
    assert!(
        [left, right, bottom, top]
            .iter()
            .all(|edge| edge.is_finite()),
        "finite target rectangle"
    );
    assert!(right > left && top > bottom, "nonempty target rectangle");
    let interior = (((left + right) / 2.0) as f32, ((bottom + top) / 2.0) as f32);
    let exterior = ((right + 32.0) as f32, (top + 32.0) as f32);
    (interior, exterior)
}

fn tick_auto_hide_at(env: &WowLuaEnv, cursor: (f32, f32), elapsed: f64) {
    {
        let mut state = env.state().borrow_mut();
        let renderer_y = state.screen_height - cursor.1;
        state.set_mouse_position(Some((cursor.0, renderer_y)));
    }
    assert!(
        env.fire_on_update(elapsed).is_ok(),
        "auto-hide runtime tick failed"
    );
    assert_auto_hide_no_lua_errors(env, "registered OnUpdate dispatch");
}

fn register_auto_hide_from_addon(env: &WowLuaEnv) {
    assert!(
        env.exec("AutoHideRuntimeRegister()").is_ok(),
        "actual addon registration failed"
    );
    assert_auto_hide_no_lua_errors(env, "registration attribute dispatch");
    let manager_visible: bool = env
        .eval("return SecureHoverDriverManager:IsVisible()")
        .unwrap_or_else(|_| panic!("observe vendor manager visibility"));
    assert!(
        manager_visible,
        "registration must activate the vendor manager"
    );
}

fn assert_auto_hide_target_shown(env: &WowLuaEnv, expected: bool, stage: &str) {
    let shown: bool = env
        .eval("return AutoHideRuntimeTarget:IsShown()")
        .unwrap_or_else(|_| panic!("{stage}: observe target visibility"));
    assert_eq!(shown, expected, "{stage}: target shown");
}

fn prepare_auto_hide_child_and_caller(env: &WowLuaEnv) {
    env.exec(r#"
        assert(type(AddToAutoHide) == 'function', 'child registration unavailable')
        AutoHideRuntimeChild = CreateFrame('Frame', nil, UIParent)
        AutoHideRuntimeChild:SetScale(1)
        AutoHideRuntimeChild:SetSize(80, 60)
        AutoHideRuntimeChild:SetPoint('BOTTOMLEFT', UIParent, 'BOTTOMLEFT', 500, 200)
        AutoHideRuntimeChild:Show()
        assert(AutoHideRuntimeChild:IsShown() and AutoHideRuntimeChild:IsVisible(), 'child not visible')

        local function addChild()
            assert(not issecure(), 'child registration requires the fixture addon caller')
            AddToAutoHide(AutoHideRuntimeTarget, AutoHideRuntimeChild)
        end
        debug.setobjecttaint(addChild, 'AutoHideRuntimeFixture')
        AutoHideRuntimeAddChild = addChild
    "#).expect("prepare child rectangle and tainted normal caller");
    assert_auto_hide_no_lua_errors(env, "child and caller preparation");
}

fn observe_auto_hide_child_cursor_points(env: &WowLuaEnv) -> [(f32, f32); 3] {
    let (target_x, target_y, child_x, child_y, outside_x, outside_y):
        (f64, f64, f64, f64, f64, f64) = env.eval(r#"
        local function rectangle(frame)
            local left, bottom, width, height = frame:GetRect()
            local scale = frame:GetEffectiveScale()
            assert(left and bottom and width > 0 and height > 0 and scale > 0, 'geometry unavailable')
            return left * scale, (left + width) * scale, bottom * scale, (bottom + height) * scale
        end
        local tl, tr, tb, tt = rectangle(AutoHideRuntimeTarget)
        local cl, cr, cb, ct = rectangle(AutoHideRuntimeChild)
        assert(tr < cl or cr < tl or tt < cb or ct < tb, 'target and child rectangles must be disjoint')
        return (tl + tr) / 2, (tb + tt) / 2,
            (cl + cr) / 2, (cb + ct) / 2,
            math.max(tr, cr) + 32, math.max(tt, ct) + 32
    "#).expect("observe disjoint resolved target and child rectangles");
    let points = [
        (target_x as f32, target_y as f32),
        (child_x as f32, child_y as f32),
        (outside_x as f32, outside_y as f32),
    ];
    assert!(points.iter().all(|(x, y)| x.is_finite() && y.is_finite()));
    assert_auto_hide_no_lua_errors(env, "resolved child geometry");
    points
}

fn assert_auto_hide_child_boundary(env: &WowLuaEnv, expected: bool, stage: &str) {
    assert_auto_hide_target_shown(env, expected, stage);
    assert_auto_hide_no_lua_errors(env, stage);
}

#[test]
fn cached_auto_hide_child_rectangle_extends_hover_set() {
    test_timeout! {
        for include_child in [true, false] {
            let case = if include_child { "treatment" } else { "control" };
            let env = load_auto_hide_real_dependencies();
            prepare_auto_hide_real_frame_and_caller(&env);
            prepare_auto_hide_child_and_caller(&env);
            let [target, child, exterior] = observe_auto_hide_child_cursor_points(&env);

            register_auto_hide_from_addon(&env);
            if include_child {
                // The cached driver accepts children only before the first update.
                env.exec("AutoHideRuntimeAddChild()")
                    .expect("tainted normal caller adds child immediately after registration");
            }
            assert_auto_hide_child_boundary(&env, true, &format!("{case}: registration"));

            for (cursor, elapsed, shown, stage) in [
                (target, 0.05, true, "target dwell"),
                (child, 0.05, true, "child-only entry"),
                (child, 0.75, true, "child-only dwell before TTL"),
                (child, 0.30, include_child, "child-only dwell past TTL"),
            ] {
                tick_auto_hide_at(&env, cursor, elapsed);
                assert_auto_hide_child_boundary(&env, shown, &format!("{case}: {stage}"));
            }

            if include_child {
                // Leaving both rectangles starts a fresh TTL in the treatment.
                for (elapsed, shown, stage) in [
                    (0.05, true, "outside both entry"),
                    (0.75, true, "outside both before TTL"),
                    (0.30, false, "outside both past TTL"),
                ] {
                    tick_auto_hide_at(&env, exterior, elapsed);
                    assert_auto_hide_child_boundary(&env, shown, stage);
                }
            }
        }
    }
}

#[test]
fn cached_auto_hide_enter_leave_expires_after_duration() {
    test_timeout! {
        let (env, interior, exterior) = load_auto_hide_fixture();
        register_auto_hide_from_addon(&env);
        tick_auto_hide_at(&env, interior, 0.05);
        assert_auto_hide_target_shown(&env, true, "cursor entered");
        // The leave tick starts the vendor TTL; subsequent ticks consume it.
        tick_auto_hide_at(&env, exterior, 0.05);
        assert_auto_hide_target_shown(&env, true, "cursor left");
        tick_auto_hide_at(&env, exterior, 0.75);
        assert_auto_hide_target_shown(&env, true, "before duration 1.0");
        tick_auto_hide_at(&env, exterior, 0.30);
        assert_auto_hide_target_shown(&env, false, "strictly after duration 1.0");
    }
}

#[test]
fn cached_auto_hide_unregister_cancels_pending_expiry() {
    test_timeout! {
        let (env, interior, exterior) = load_auto_hide_fixture();
        register_auto_hide_from_addon(&env);
        tick_auto_hide_at(&env, interior, 0.05);
        tick_auto_hide_at(&env, exterior, 0.05);
        tick_auto_hide_at(&env, exterior, 0.25);
        assert_auto_hide_target_shown(&env, true, "pending expiry before unregister");
        assert!(env.exec("AutoHideRuntimeUnregister()").is_ok(), "actual addon unregistration failed");
        assert_auto_hide_no_lua_errors(&env, "unregistration attribute dispatch");
        tick_auto_hide_at(&env, exterior, 1.25);
        assert_auto_hide_target_shown(&env, true, "past cancelled deadline");
    }
}
