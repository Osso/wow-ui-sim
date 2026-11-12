//! Supplied Path points and modeled NONE/SMOOTH playback.
//! Catmull–Rom geometry is simulator policy, not native or historical parity.
#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::LayoutRect;
use wow_ui_sim::layout::compute_frame_rect;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn new_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        function near(actual, expected)
            assert(math.abs(actual - expected) < 0.00001,
                tostring(actual) .. " differs from " .. tostring(expected))
        end
        function point_offset(point, x, y)
            assert(select('#', point:GetOffset()) == 2)
            local actualX, actualY = point:GetOffset()
            near(actualX, x)
            near(actualY, y)
        end
        "#,
    )
    .unwrap();
    env
}

fn create_path_scene(env: &WowLuaEnv) {
    env.exec(
        r#"
        owner = CreateFrame('Frame', 'PathStageOwner', UIParent)
        owner:SetSize(120, 80)
        owner:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 100, -100)
        owner.Icon = owner:CreateTexture('PathStageIcon')
        owner.Icon:SetSize(20, 20)
        owner.Icon:SetPoint('TOPLEFT', owner, 'TOPLEFT', 40, -30)
        owner.Other = owner:CreateTexture('PathStageOther')
        owner.Other:SetSize(20, 20)
        owner.Other:SetPoint('TOPLEFT', owner, 'TOPLEFT', 70, -30)
        group = owner:CreateAnimationGroup()
        path = group:CreateAnimation('Path')
        path:SetChildKey('Icon')
        path:SetDuration(2)
        group:SetLooping('NONE')
        "#,
    )
    .unwrap();
}

fn frame_rect(env: &WowLuaEnv, name: &str) -> LayoutRect {
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name(name).unwrap();
    compute_frame_rect(&state.widgets, id, 1024.0, 768.0)
}

fn assert_shift(env: &WowLuaEnv, name: &str, baseline: LayoutRect, dx: f32, dy: f32) {
    let actual = frame_rect(env, name);
    let expected_x = baseline.x + dx;
    let expected_y = baseline.y + dy;
    assert!(
        (actual.x - expected_x).abs() < 0.0001,
        "{name}: x={} expected {expected_x}",
        actual.x
    );
    assert!(
        (actual.y - expected_y).abs() < 0.0001,
        "{name}: y={} expected {expected_y}",
        actual.y
    );
    assert_eq!(actual.width, baseline.width);
    assert_eq!(actual.height, baseline.height);
}

// Fixture injection only: existing pre-play presentation state, not a new public API.
fn seed_icon_offset(env: &WowLuaEnv, x: f32, y: f32) {
    let mut state = env.state().borrow_mut();
    let id = state.widgets.get_id_by_name("PathStageIcon").unwrap();
    let icon = state.widgets.get_mut(id).unwrap();
    icon.anim_offset_x = x;
    icon.anim_offset_y = y;
}

#[test]
fn path_supplied_point_order_and_offsets_survive_creation_and_mutation() {
    let env = new_env();
    create_path_scene(&env);
    env.exec(
        r#"
        local later = path:CreateControlPoint(nil, nil, 9)
        later:SetOffset(-30, 12)
        local earlier = path:CreateControlPoint(nil, nil, 2)
        earlier:SetOffset(14, -8)
        assert(later:GetOrder() == 9)
        assert(earlier:GetOrder() == 2)
        point_offset(later, -30, 12)
        point_offset(earlier, 14, -8)
        assert(select('#', path:GetControlPoints()) == 2)
        local first, second = path:GetControlPoints()
        assert(first == earlier and second == later)
        assert(path:GetMaxControlPointOrder() == 9)

        later:SetOrder(1)
        later:SetOffset(-6, 18)
        first, second = path:GetControlPoints()
        assert(first == later and second == earlier)
        assert(first:GetOrder() == 1 and second:GetOrder() == 2)
        assert(path:GetMaxControlPointOrder() == 2)
        point_offset(first, -6, 18)
        point_offset(second, 14, -8)
        "#,
    )
    .unwrap();
}

#[test]
fn path_xml_declared_points_reach_public_queries_and_none_playback() {
    let env = new_env();
    let addon = tempfile::tempdir().unwrap();
    std::fs::write(
        addon.path().join("PathStageXml.toc"),
        "## Title: PathStageXml\nPathStageXml.xml\n",
    )
    .unwrap();
    std::fs::write(
        addon.path().join("PathStageXml.xml"),
        r#"<Ui>
        <Frame name="PathStageXmlOwner" parent="UIParent">
            <Size x="20" y="20"/>
            <Anchors><Anchor point="TOPLEFT" relativeTo="UIParent"
                relativePoint="TOPLEFT" x="100" y="-100"/></Anchors>
            <Animations><AnimationGroup parentKey="Motion" looping="NONE">
                <Path parentKey="Route" curve="NONE" duration="2" order="1">
                    <ControlPoints>
                        <ControlPoint name="PathStageXmlFirst" offsetX="12" offsetY="-6"/>
                        <ControlPoint name="PathStageXmlSecond" offsetX="-20" offsetY="10"/>
                    </ControlPoints>
                </Path>
            </AnimationGroup></Animations>
        </Frame>
        </Ui>"#,
    )
    .unwrap();
    load_addon(&env.loader_env(), &addon.path().join("PathStageXml.toc")).unwrap();
    env.exec(
        r#"
        local route = PathStageXmlOwner.Motion.Route
        assert(route:GetCurveType() == 'NONE')
        assert(select('#', route:GetControlPoints()) == 2)
        local first, second = route:GetControlPoints()
        assert(first:GetName() == 'PathStageXmlFirst')
        assert(second:GetName() == 'PathStageXmlSecond')
        assert(first:GetOrder() == 1 and second:GetOrder() == 2)
        assert(route:GetMaxControlPointOrder() == 2)
        point_offset(first, 12, -6)
        point_offset(second, -20, 10)
        "#,
    )
    .unwrap();
    let baseline = frame_rect(&env, "PathStageXmlOwner");
    env.exec("PathStageXmlOwner.Motion:Play()").unwrap();
    assert_shift(&env, "PathStageXmlOwner", baseline, 12.0, -6.0);
    env.fire_on_update(1.0).unwrap();
    assert_shift(&env, "PathStageXmlOwner", baseline, -4.0, 2.0);
    env.fire_on_update(1.0).unwrap();
    env.exec("assert(PathStageXmlOwner.Motion:IsDone())")
        .unwrap();
    assert_shift(&env, "PathStageXmlOwner", baseline, 0.0, 0.0);
}

#[test]
fn path_none_walks_sorted_supplied_points_without_moving_owner_or_sibling() {
    let env = new_env();
    create_path_scene(&env);
    env.exec(
        r#"
        path:SetCurveType('NONE')
        path:SetSmoothing('NONE')
        assert(path:GetCurveType() == 'NONE')
        assert(path:GetTarget() == owner.Icon)
        local last = path:CreateControlPoint(nil, nil, 9)
        last:SetOffset(-10, 20)
        local first = path:CreateControlPoint(nil, nil, 2)
        first:SetOffset(10, -4)
        local middle = path:CreateControlPoint(nil, nil, 5)
        middle:SetOffset(30, 8)
        "#,
    )
    .unwrap();
    let icon = frame_rect(&env, "PathStageIcon");
    let owner = frame_rect(&env, "PathStageOwner");
    let sibling = frame_rect(&env, "PathStageOther");
    env.exec("group:Play()").unwrap();
    assert_shift(&env, "PathStageIcon", icon, 10.0, -4.0);
    assert_shift(&env, "PathStageOwner", owner, 0.0, 0.0);
    assert_shift(&env, "PathStageOther", sibling, 0.0, 0.0);
    for (delta, x, y) in [(0.5, 20.0, 2.0), (0.5, 30.0, 8.0), (0.5, 10.0, 14.0)] {
        env.fire_on_update(delta).unwrap();
        env.exec("assert(group:IsPlaying())").unwrap();
        assert_shift(&env, "PathStageIcon", icon, x, y);
        assert_shift(&env, "PathStageOwner", owner, 0.0, 0.0);
        assert_shift(&env, "PathStageOther", sibling, 0.0, 0.0);
    }
}

#[test]
fn path_smooth_uses_declared_catmull_rom_geometry_through_supplied_points() {
    let env = new_env();
    create_path_scene(&env);
    env.exec(
        r#"
        path:SetCurveType('SMOOTH')
        assert(path:GetCurveType() == 'SMOOTH')
        for order, offset in ipairs({{0, 0}, {10, 10}, {20, 0}}) do
            local point = path:CreateControlPoint(nil, nil, order)
            point:SetOffset(offset[1], offset[2])
        end
        "#,
    )
    .expect("SMOOTH configuration, not a NONE substitution");
    let icon = frame_rect(&env, "PathStageIcon");
    let owner = frame_rect(&env, "PathStageOwner");
    let sibling = frame_rect(&env, "PathStageOther");
    env.exec("group:Play()").unwrap();
    assert_shift(&env, "PathStageIcon", icon, 0.0, 0.0);
    for (delta, x, y) in [(0.5, 4.375, 5.625), (0.5, 10.0, 10.0), (0.5, 15.625, 5.625)] {
        env.fire_on_update(delta).unwrap();
        assert_shift(&env, "PathStageIcon", icon, x, y);
        assert_shift(&env, "PathStageOwner", owner, 0.0, 0.0);
        assert_shift(&env, "PathStageOther", sibling, 0.0, 0.0);
    }
    env.exec("group:Stop()").unwrap();
    assert_shift(&env, "PathStageIcon", icon, 0.0, 0.0);
}

#[test]
fn path_actual_cached_smooth_xml_configures_points_and_restores_playback() {
    let env = new_env();
    // PTR TOC Blizzard_MainMenuBarBagButtons.toc:14 selects this Mainline template XML.
    let source_path = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("active profile cache required")
        .join("Blizzard_MainMenuBarBagButtons/Mainline/MainMenuBarBagButtonTemplates.xml");
    let source = std::fs::read_to_string(source_path).expect("actual cached bag-button XML");
    // Preserve the actual first Path element unchanged; unrelated template dependencies
    // are not this failure boundary. Assertions below observe configuration/layout.
    let template = source
        .find("<ItemButton name=\"BaseBagSlotButtonTemplate\"")
        .expect("cached bag-slot template");
    let template_end = template + source[template..].find("</ItemButton>").unwrap();
    let path_start = source[template..template_end]
        .find("<Path ")
        .expect("bag-slot Path fixture");
    let start = template + path_start;
    let end = start
        + source[start..]
            .find("</Path>")
            .expect("closed Path element")
        + "</Path>".len();
    let path_xml = &source[start..end];
    let addon = tempfile::tempdir().unwrap();
    std::fs::write(
        addon.path().join("SmoothVendor.toc"),
        "## Title: SmoothVendor\nSmoothVendor.xml\n",
    )
    .unwrap();
    std::fs::write(
        addon.path().join("SmoothVendor.xml"),
        format!(r#"<Ui><Frame name="SmoothVendorOwner" parent="UIParent">
        <Size x="20" y="20"/>
        <Anchors><Anchor point="TOPLEFT" relativeTo="UIParent" relativePoint="TOPLEFT" x="100" y="-100"/></Anchors>
        <Layers><Layer level="ARTWORK"><Texture name="SmoothVendorIcon" parentKey="AnimIcon" setAllPoints="true"/></Layer></Layers>
        <Animations><AnimationGroup parentKey="Motion" looping="NONE">{path_xml}</AnimationGroup></Animations>
        </Frame></Ui>"#),
    ).unwrap();
    load_addon(&env.loader_env(), &addon.path().join("SmoothVendor.toc")).unwrap();
    env.exec(
        r#"
        local route = SmoothVendorOwner.Motion:GetAnimations()
        assert(route:GetCurveType() == 'SMOOTH', 'actual XML curve survives setup')
        assert(select('#', route:GetControlPoints()) == 2, 'setup reaches both points')
        local first, second = route:GetControlPoints()
        point_offset(first, -15, 30)
        point_offset(second, -75, 60)
        assert(first:GetOrder() == 1 and second:GetOrder() == 2)
        "#,
    )
    .expect("actual SMOOTH XML must not abort configuration before its points");
    let icon = frame_rect(&env, "SmoothVendorIcon");
    env.exec("SmoothVendorOwner.Motion:Play()").unwrap();
    assert_shift(&env, "SmoothVendorIcon", icon, -15.0, 30.0);
    env.fire_on_update(0.25).unwrap();
    assert_shift(&env, "SmoothVendorIcon", icon, -27.1875, 36.09375);
    env.fire_on_update(0.25).unwrap();
    assert_shift(&env, "SmoothVendorIcon", icon, -45.0, 45.0);
    env.fire_on_update(0.5).unwrap();
    assert_shift(&env, "SmoothVendorIcon", icon, 0.0, 0.0);
}

#[test]
fn path_stop_and_natural_finish_restore_each_playbacks_existing_offset() {
    let env = new_env();
    create_path_scene(&env);
    env.exec(
        r#"
        path:SetCurveType('NONE')
        path:SetSmoothing('NONE')
        local first = path:CreateControlPoint(nil, nil, 1)
        first:SetOffset(10, -4)
        local last = path:CreateControlPoint(nil, nil, 2)
        last:SetOffset(30, 8)
        "#,
    )
    .unwrap();
    let anchored = frame_rect(&env, "PathStageIcon");
    seed_icon_offset(&env, 3.0, -5.0);
    assert_shift(&env, "PathStageIcon", anchored, 3.0, -5.0);
    env.exec("group:Play()").unwrap();
    assert_shift(&env, "PathStageIcon", anchored, 13.0, -9.0);
    env.fire_on_update(1.0).unwrap();
    assert_shift(&env, "PathStageIcon", anchored, 23.0, -3.0);
    env.exec("group:Stop(); assert(group:IsDone() and not group:IsPlaying())")
        .unwrap();
    assert_shift(&env, "PathStageIcon", anchored, 3.0, -5.0);

    seed_icon_offset(&env, -7.0, 11.0);
    env.exec("group:Play()").unwrap();
    assert_shift(&env, "PathStageIcon", anchored, 3.0, 7.0);
    env.fire_on_update(1.0).unwrap();
    assert_shift(&env, "PathStageIcon", anchored, 13.0, 13.0);
    env.fire_on_update(1.0).unwrap();
    env.exec("assert(group:IsDone() and not group:IsPlaying())")
        .unwrap();
    assert_shift(&env, "PathStageIcon", anchored, -7.0, 11.0);
}

#[test]
fn path_hiding_owner_subtree_stops_playback_and_restores_existing_offset() {
    let env = new_env();
    create_path_scene(&env);
    env.exec(
        r#"
        path:SetCurveType('NONE')
        path:SetSmoothing('NONE')
        local first = path:CreateControlPoint(nil, nil, 1)
        first:SetOffset(10, -4)
        local last = path:CreateControlPoint(nil, nil, 2)
        last:SetOffset(30, 8)
        assert(owner:IsVisible() and owner.Icon:IsVisible())
        "#,
    )
    .unwrap();
    let anchored = frame_rect(&env, "PathStageIcon");
    seed_icon_offset(&env, 3.0, -5.0);
    env.exec("group:Play()").unwrap();
    env.fire_on_update(1.0).unwrap();
    env.exec("assert(group:IsPlaying())").unwrap();
    assert_shift(&env, "PathStageIcon", anchored, 23.0, -3.0);
    env.exec(
        r#"
        owner:Hide()
        assert(not owner:IsShown())
        assert(not owner:IsVisible() and not owner.Icon:IsVisible())
        assert(group:IsDone() and not group:IsPlaying())
        "#,
    )
    .unwrap();
    assert_shift(&env, "PathStageIcon", anchored, 3.0, -5.0);
    env.exec(
        r#"
        group:Stop()
        assert(not owner:IsVisible() and not owner.Icon:IsVisible())
        assert(group:IsDone() and not group:IsPlaying())
        "#,
    )
    .unwrap();
    assert_shift(&env, "PathStageIcon", anchored, 3.0, -5.0);
}

#[test]
fn path_unknown_curve_token_is_rejected_without_changing_curve() {
    let env = new_env();
    create_path_scene(&env);
    env.exec(
        r#"
        path:SetCurveType('SMOOTH')
        local prior = path:GetCurveType()
        assert(prior == 'SMOOTH')
        local ok = pcall(function()
            path:SetCurveType('UNSUPPORTED_CURVE')
        end)
        assert(ok == false)
        assert(path:GetCurveType() == prior)
        "#,
    )
    .unwrap();
}
