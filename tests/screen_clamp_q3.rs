//! Native q3.1 propagation through the selected Retail UI; the host supplies
//! initial propagation states because loaded native owners are not assumed.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn host_propagation(env: &WowLuaEnv, name: &str, requested: bool) {
    let mut sim = env.state().borrow_mut();
    let id = sim.widgets.get_id_by_name(name).unwrap();
    sim.widgets.get_mut(id).unwrap().anchor_propagation = requested;
    sim.widgets.set_screen_clamp_suppressed(id, requested);
}

fn assert_clamp(env: &WowLuaEnv, target: &str, expected: bool, phase: &str) {
    env.exec(&format!(
        "assert({target}:IsClampedToScreen() == {expected}, '{phase}')"
    ))
    .unwrap();
}

#[cfg(all(feature = "client-retail", feature = "optional-tests"))]
prefork_full_ui_case! {
fn original_tooltip_inherits_distinct_owner_and_coordinate_root_states(env: &WowLuaEnv) {
    env.exec(r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_SharedXML'))
        assert(C_AddOns.IsAddOnLoaded('Blizzard_GameTooltip'))
        local source = debug.getinfo(GameTooltip_ShowSimpleTooltip, 'S').source
        assert(source:gsub('\\', '/'):find('Blizzard_SharedXML/SharedTooltipTemplates.lua', 1, true))
        ClampQ3CursorOwner = CreateFrame('Frame', 'ClampQ3CursorOwner', UIParent)
        ClampQ3CursorOwner:SetSize(100, 50)
        ClampQ3CursorOwner:SetPoint('CENTER')
        ClampQ3RightOwner = CreateFrame('Frame', 'ClampQ3RightOwner', UIParent)
        ClampQ3RightOwner:SetSize(50, 20)
        ClampQ3RightOwner:SetPoint('CENTER')
        GameTooltip:ClearAllPoints()
        GameTooltip:SetClampedToScreen(true)
    "#).unwrap();
    env.state().borrow_mut().set_mouse_position(Some((75.0, 90.0)));
    host_propagation(env, "GameTooltip", false);
    host_propagation(env, "ClampQ3CursorOwner", true);
    host_propagation(env, "ClampQ3RightOwner", true);
    host_propagation(env, "UIParent", true);

    env.exec(r#"
        GameTooltip_ShowSimpleTooltip(GameTooltip, 'Cursor', nil, false,
            ClampQ3CursorOwner, 'ANCHOR_CURSOR')
    "#).unwrap();
    assert_clamp(env, "GameTooltip", true, "cursor must use guarded false coordinate root");

    env.exec(r#"
        GameTooltip_ShowSimpleTooltip(GameTooltip, 'Owner right', nil, false,
            ClampQ3RightOwner, 'ANCHOR_RIGHT')
    "#).unwrap();
    assert_clamp(env, "GameTooltip", false, "right must inherit true owner propagation");

    env.exec("GameTooltip:ClearAllPoints()").unwrap();
    assert_clamp(env, "GameTooltip", true, "clear must request false");
    env.exec("GameTooltip:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 8, -8)").unwrap();
    assert_clamp(env, "GameTooltip", false, "final point must inherit true UIParent propagation");

    // INFERRED conditional walkthrough: supplied false owners and UIParent
    // leave all four suppression results false. Not a native startup claim.
    host_propagation(env, "ClampQ3CursorOwner", false);
    host_propagation(env, "ClampQ3RightOwner", false);
    host_propagation(env, "UIParent", false);
    env.exec("GameTooltip:ClearAllPoints()").unwrap();
    for (call, phase) in [
        ("GameTooltip_ShowSimpleTooltip(GameTooltip, 'Cursor', nil, false, ClampQ3CursorOwner, 'ANCHOR_CURSOR')", "conditional cursor"),
        ("GameTooltip_ShowSimpleTooltip(GameTooltip, 'Right', nil, false, ClampQ3RightOwner, 'ANCHOR_RIGHT')", "conditional right"),
        ("GameTooltip:ClearAllPoints()", "conditional clear"),
        ("GameTooltip:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 8, -8)", "conditional final point"),
    ] {
        env.exec(call).unwrap();
        assert_clamp(env, "GameTooltip", true, phase);
    }
}
}

#[test]
fn ordinary_movement_uses_coordinate_root_instead_of_propagating_parent() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(400.0, 300.0);
    env.exec(
        r#"
        ClampQ3MovingParent = CreateFrame('Frame', 'ClampQ3MovingParent', UIParent)
        ClampQ3MovingParent:SetSize(150, 100)
        ClampQ3MovingParent:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 25, -15)
        ClampQ3Moving = CreateFrame('Frame', 'ClampQ3Moving', ClampQ3MovingParent)
        ClampQ3Moving:SetSize(20, 30)
        ClampQ3Moving:SetMovable(true)
        ClampQ3Moving:SetClampedToScreen(true)
    "#,
    )
    .unwrap();
    host_propagation(&env, "ClampQ3MovingParent", true);
    env.exec("ClampQ3Moving:SetPoint('TOPLEFT', ClampQ3MovingParent, 'TOPLEFT', 12, -2)")
        .unwrap();
    assert_clamp(
        &env,
        "ClampQ3Moving",
        false,
        "initial point must inherit true propagation",
    );
    env.exec("ClampQ3Moving:StartMoving()").unwrap();
    assert_clamp(
        &env,
        "ClampQ3Moving",
        true,
        "StartMoving must use coordinate root",
    );
    env.exec("ClampQ3Moving:SetPoint('TOPLEFT', ClampQ3MovingParent, 'TOPLEFT', 12, -2)")
        .unwrap();
    assert_clamp(
        &env,
        "ClampQ3Moving",
        false,
        "new attached dependency is true",
    );
    env.exec("ClampQ3Moving:StopMovingOrSizing()").unwrap();
    assert_clamp(
        &env,
        "ClampQ3Moving",
        true,
        "ordinary settlement must use coordinate root",
    );
}

fn set_host_frame(env: &WowLuaEnv, name: &str, set: impl FnOnce(&mut wow_ui_sim::widget::Frame)) {
    let mut sim = env.state().borrow_mut();
    let id = sim.widgets.get_id_by_name(name).unwrap();
    set(sim.widgets.get_mut(id).unwrap());
}

fn inspect_frame<T>(
    env: &WowLuaEnv,
    name: &str,
    inspect: impl FnOnce(&wow_ui_sim::widget::Frame) -> T,
) -> T {
    let sim = env.state().borrow();
    let id = sim.widgets.get_id_by_name(name).unwrap();
    inspect(sim.widgets.get(id).unwrap())
}

#[test]
fn clamp_propagation_stops_at_guard_or_equal_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ClampQ3Driver = CreateFrame('Frame','ClampQ3Driver',UIParent)
        ClampQ3Equal = CreateFrame('Frame','ClampQ3Equal',UIParent)
        ClampQ3Guard = CreateFrame('Frame','ClampQ3Guard',UIParent)
        ClampQ3Leaf = CreateFrame('Frame','ClampQ3Leaf',UIParent)
        ClampQ3GuardLeaf = CreateFrame('Frame','ClampQ3GuardLeaf',UIParent)
        ClampQ3Equal:SetPoint('CENTER',ClampQ3Driver)
        ClampQ3Leaf:SetPoint('CENTER',ClampQ3Equal)
        ClampQ3Guard:SetPoint('TOP',ClampQ3Driver)
        ClampQ3GuardLeaf:SetPoint('CENTER',ClampQ3Guard)
    "#,
    )
    .unwrap();
    host_propagation(&env, "UIParent", true);
    set_host_frame(&env, "ClampQ3Equal", |frame| {
        frame.anchor_propagation = true
    });
    set_host_frame(&env, "ClampQ3Guard", |frame| {
        frame.anchor_propagation_guard = true
    });
    env.exec("ClampQ3Driver:SetPoint('TOPLEFT',UIParent,'TOPLEFT',5,-5)")
        .unwrap();
    assert!(inspect_frame(&env, "ClampQ3Driver", |f| f.screen_clamp_suppressed));
    for name in [
        "ClampQ3Equal",
        "ClampQ3Leaf",
        "ClampQ3Guard",
        "ClampQ3GuardLeaf",
    ] {
        assert!(
            !inspect_frame(&env, name, |f| f.screen_clamp_suppressed),
            "{name}"
        );
    }
    env.exec("ClampQ3Driver:ClearAllPoints()").unwrap();
    assert!(!inspect_frame(&env, "ClampQ3Driver", |f| f.screen_clamp_suppressed));
    assert!(!inspect_frame(&env, "ClampQ3Equal", |f| f.anchor_propagation));
}

#[test]
fn ordinary_movement_equal_request_and_special_context_do_not_rewrite_suppression() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ClampQ3MoveContext = CreateFrame('Frame','ClampQ3MoveContext',UIParent)
        ClampQ3MoveContext:SetSize(80,80)
        ClampQ3MoveContext:SetPoint('CENTER',UIParent)
        ClampQ3MoveSubject = CreateFrame('Frame','ClampQ3MoveSubject',ClampQ3MoveContext)
        ClampQ3MoveSubject:SetSize(25,25)
        ClampQ3MoveSubject:SetPoint('CENTER',ClampQ3MoveContext)
        ClampQ3MoveSubject:SetMovable(true)
        ClampQ3MoveSubject:SetClampedToScreen(true)
    "#,
    )
    .unwrap();
    set_host_frame(&env, "ClampQ3MoveSubject", |frame| {
        frame.screen_clamp_suppressed = true
    });
    env.exec("ClampQ3MoveSubject:StartMoving()").unwrap();
    assert_clamp(
        &env,
        "ClampQ3MoveSubject",
        false,
        "matching false must not repair selected suppression",
    );

    host_propagation(&env, "ClampQ3MoveContext", true);
    set_host_frame(&env, "ClampQ3MoveContext", |frame| {
        frame.clamp_special_movement_context = true
    });
    env.exec(
        r#"
        ClampQ3MoveSubject:SetPoint('TOPLEFT',ClampQ3MoveContext,'TOPLEFT',2,-2)
        ClampQ3MoveSubject:StopMovingOrSizing()
    "#,
    )
    .unwrap();
    assert_clamp(
        &env,
        "ClampQ3MoveSubject",
        false,
        "supplied special context skips settlement",
    );
    assert!(inspect_frame(&env, "ClampQ3MoveSubject", |f| f.anchor_propagation));
}

#[test]
fn clamp_insets_use_native_single_precision_round_trip_and_arity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ClampQ3Units = CreateFrame('Frame','ClampQ3Units',UIParent)
        ClampQ3Units:SetScale(1.75)
        assert(select('#',ClampQ3Units:SetClampRectInsets('0.1',-0.1,0.3,123.456789)) == 0)
        assert(select('#',ClampQ3Units:GetClampRectInsets()) == 4)
    "#,
    )
    .unwrap();
    let input = [0.1_f32, -0.1, 0.3, 123.456789];
    let expected_stored = input.map(|number| (number * ((1.0_f32 / 1024.0) / 1.0)) * 0.8);
    let stored = inspect_frame(&env, "ClampQ3Units", |f| f.clamp_rect_insets);
    assert_eq!(<[f32; 4]>::from(stored), expected_stored);
    let public: (f64, f64, f64, f64) = env
        .eval("return ClampQ3Units:GetClampRectInsets()")
        .unwrap();
    let wanted = expected_stored.map(|inset| f64::from(((1.0_f32 * 1024.0) * inset) / 0.8));
    assert_eq!(<[f64; 4]>::from(public), wanted);
    assert_ne!(public.0, f64::from(input[0]));
}

#[test]
fn clamp_rounds_point_displacements_and_explicit_dimensions_but_not_target_edges() {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(1024.0, 768.0);
    env.exec(
        r#"
        local parent = CreateFrame('Frame',nil,UIParent)
        parent:SetSize(30.25,40)
        parent:SetPoint('TOPLEFT',UIParent,'TOPLEFT',20.25,-20.25)
        local child = CreateFrame('Frame',nil,parent)
        child:SetSize(0.25,0.25)
        child:SetPoint('TOPLEFT',parent,'TOPLEFT',0.5,-0.5)
        child:SetRoundLayoutToNearestPixel(true)
        local x,y,w,h=child:GetBoundsRect()
        assert(x==21.25 and y==745.75 and w==1 and h==1, 'rounded point and dimension minimum')
        child:ClearAllPoints()
        child:SetPoint('TOPLEFT',parent,'TOPLEFT',0,0)
        child:SetPoint('BOTTOMRIGHT',parent,'BOTTOMRIGHT',0,0)
        x,y,w,h=child:GetBoundsRect()
        assert(x==20.25 and w==30.25 and h==40, 'range from original target edges')

        child:ClearAllPoints()
        child:SetSize(10,10)
        child:SetPoint('TOPLEFT',UIParent,'TOPLEFT',-20,-20)
        child:SetClampRectInsets(0.3,0,0,0)
        child:SetClampedToScreen(true)
        x=child:GetBoundsRect()
        assert(math.abs(x+0.3)<0.000001, 'pre-shaped clamp result not snapped again')
    "#,
    )
    .unwrap();
}

#[test]
fn restricted_clamp_enable_raises_without_writing_and_preserves_normal_arities() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ClampQ3Restricted = CreateFrame('Frame','ClampQ3Restricted',UIParent)
        ClampQ3Restricted:SetPoint('CENTER',UIParent)
        ClampQ3Restricted:SetSize(30,30)
        ClampQ3Restricted:GetBoundsRect()
    "#,
    )
    .unwrap();
    set_host_frame(&env, "ClampQ3Restricted", |frame| {
        frame.clamp_restricted_region = true
    });
    env.exec(r#"
        local f=ClampQ3Restricted
        assert(select('#', f:SetClampedToScreen(false))==0, 'unchanged request')
        local function call()
            f:SetClampedToScreen(true)
        end
        local prefix="ClampQ3Restricted:SetClampedToScreen(): Action[ClampedToScreen] failed because[Can't clamp restricted regions]: attempted from: Frame:SetClampedToScreen."
        local ok, e=pcall(call)
        assert(not ok and tostring(e):find(prefix..'\nNo Lua Taint',1,true), tostring(e))
        debug.setobjecttaint(call, 'Q3Probe')
        ok,e=pcall(call)
        assert(not ok and tostring(e):find(prefix..'\nLua Taint: Q3Probe',1,true), tostring(e))
        assert(not f:IsClampedToScreen())
    "#).unwrap();
    assert!(!inspect_frame(&env, "ClampQ3Restricted", |f| f.clamped_to_screen));
    set_host_frame(&env, "ClampQ3Restricted", |frame| {
        frame.clamped_to_screen = true
    });
    env.exec(
        r#"
        assert(select('#',ClampQ3Restricted:SetClampedToScreen(true))==0)
        assert(select('#',ClampQ3Restricted:SetClampedToScreen(false))==0)
        assert(not ClampQ3Restricted:IsClampedToScreen())
    "#,
    )
    .unwrap();
}

#[test]
fn inset_notification_respects_saved_access_prevention_and_prior_marker() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ClampQ3Notice = CreateFrame('GameTooltip','ClampQ3Notice',UIParent)
        ClampQ3Notice:SetSize(30,30)
        ClampQ3Notice:SetPoint('CENTER',UIParent)
        ClampQ3Notice:GetBoundsRect()
        assert(select('#',ClampQ3Notice:SetClampRectInsets(1,2,3,4))==0)
    "#,
    )
    .unwrap();
    assert!(!inspect_frame(&env, "ClampQ3Notice", |f| f.clamp_secret_notified));
    env.state().borrow_mut().clamp_inset_saved_access = 1;
    set_host_frame(&env, "ClampQ3Notice", |frame| {
        frame.prevent_secret_values = true
    });
    env.exec(r#"
        local f=ClampQ3Notice
        local ok, e=pcall(f.SetClampRectInsets, f, 5,6,7,8)
        assert(not ok and tostring(e):find('Action[SetSecretValues] failed because[Attempt to set secret values on an object that prevents secret values]',1,true), tostring(e))
        local l,r,t,b=f:GetClampRectInsets()
        assert(l==1 and r==2 and t==3 and b==4, 'prevention must skip all stores')
    "#).unwrap();
    assert!(!inspect_frame(&env, "ClampQ3Notice", |f| f.clamp_secret_notified));
    set_host_frame(&env, "ClampQ3Notice", |frame| {
        frame.prevent_secret_values = false
    });
    env.exec("ClampQ3Notice:SetClampRectInsets(5,6,7,8)")
        .unwrap();
    assert!(inspect_frame(&env, "ClampQ3Notice", |f| f
        .clamp_secret_notified
        && f.clamp_geometry_notified));
    set_host_frame(&env, "ClampQ3Notice", |frame| {
        frame.clamp_geometry_notified = false
    });
    env.exec("ClampQ3Notice:SetClampRectInsets(9,10,11,12)")
        .unwrap();
    assert!(!inspect_frame(&env, "ClampQ3Notice", |f| f.clamp_geometry_notified));
    env.exec("assert(ClampQ3Notice:GetClampRectInsets()==9)")
        .unwrap();
}

#[test]
fn clamp_geometry_notification_stops_marked_subtrees_and_respects_ancestor_gate() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ClampQ3Ancestor = CreateFrame('Frame','ClampQ3Ancestor',UIParent)
        ClampQ3Boundary = CreateFrame('Frame','ClampQ3Boundary',ClampQ3Ancestor)
        ClampQ3Geometry = CreateFrame('GameTooltip','ClampQ3Geometry',ClampQ3Boundary)
        ClampQ3Cut = CreateFrame('Frame','ClampQ3Cut',UIParent)
        ClampQ3BelowCut = CreateFrame('Frame','ClampQ3BelowCut',UIParent)
        ClampQ3Sibling = CreateFrame('Frame','ClampQ3Sibling',UIParent)
        ClampQ3Geometry:SetPoint('CENTER',UIParent)
        ClampQ3Cut:SetPoint('CENTER',ClampQ3Geometry)
        ClampQ3BelowCut:SetPoint('CENTER',ClampQ3Cut)
        ClampQ3Sibling:SetPoint('CENTER',ClampQ3Geometry)
        ClampQ3Geometry:GetBoundsRect()
    "#,
    )
    .unwrap();
    env.state().borrow_mut().clamp_inset_saved_access = 1;
    set_host_frame(&env, "ClampQ3Boundary", |frame| {
        frame.clamp_notification_boundary = true
    });
    set_host_frame(&env, "ClampQ3Cut", |frame| {
        frame.clamp_geometry_notified = true
    });
    set_host_frame(&env, "ClampQ3Geometry", |frame| {
        frame.clamp_notification_ancestor_eligible = true
    });
    env.exec("ClampQ3Geometry:SetClampRectInsets(1,2,3,4)")
        .unwrap();
    assert!(inspect_frame(&env, "ClampQ3Sibling", |f| f.clamp_geometry_notified));
    assert!(!inspect_frame(&env, "ClampQ3BelowCut", |f| f.clamp_geometry_notified));
    assert!(!inspect_frame(&env, "ClampQ3Ancestor", |f| f.clamp_ancestor_notified));
    set_host_frame(&env, "ClampQ3Geometry", |frame| {
        frame.clamp_secret_notified = false;
        frame.clamp_geometry_notified = false;
        frame.clamp_notification_ancestor_gate = true;
    });
    env.exec("ClampQ3Geometry:SetClampRectInsets(4,3,2,1)")
        .unwrap();
    assert!(inspect_frame(&env, "ClampQ3Ancestor", |f| f.clamp_ancestor_notified));
    assert!(!inspect_frame(&env, "ClampQ3Boundary", |f| f.clamp_ancestor_notified));
    assert!(!inspect_frame(&env, "UIParent", |f| f.clamp_ancestor_notified));
}
