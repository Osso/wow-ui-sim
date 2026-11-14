//! R88n.5r VERIFIED geometry state with explicit host suppression inputs.
//! These API selectors do not claim an unknown StartMoving suppression predicate.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture() -> (WowLuaEnv, u64) {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(300.0, 200.0);
    env.exec(
        r#"
        ClampProbe=CreateFrame('Frame','ClampProbe',UIParent)
        ClampProbe:SetSize(50,50)
        ClampProbe:SetPoint('TOPLEFT',UIParent,'TOPLEFT',-20,20)
        ClampProbe:SetClampedToScreen(true)
        local x,y,w,h=ClampProbe:GetBoundsRect()
        assert(x==0 and y==150 and w==50 and h==50)
    "#,
    )
    .unwrap();
    let id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("ClampProbe")
        .unwrap();
    (env, id)
}

#[test]
fn suppression_effective_state_and_unchanged_request() {
    let (env, id) = fixture();
    env.state()
        .borrow_mut()
        .widgets
        .set_screen_clamp_suppressed(id, true);
    env.exec(
        r#"
        assert(ClampProbe:IsClampedToScreen()==false)
        local x,y=ClampProbe:GetBoundsRect(); assert(x==-20 and y==170)
        assert(select('#',ClampProbe:SetClampedToScreen(false))==0)
    "#,
    )
    .unwrap();
    {
        let sim = env.state().borrow();
        assert!(sim.widgets.get(id).unwrap().clamped_to_screen);
        assert!(!sim.widgets.is_rect_dirty_self(id));
    }
    env.state()
        .borrow_mut()
        .widgets
        .set_screen_clamp_suppressed(id, false);
    env.exec("assert(ClampProbe:IsClampedToScreen()); local x,y=ClampProbe:GetBoundsRect(); assert(x==0 and y==150)").unwrap();
    env.exec("ClampProbe:SetClampedToScreen(true)").unwrap();
    assert!(!env.state().borrow().widgets.is_rect_dirty_self(id));
}

#[test]
fn user_placement_does_not_change_suppression() {
    let (env, id) = fixture();
    env.state()
        .borrow_mut()
        .widgets
        .set_screen_clamp_suppressed(id, true);
    env.exec("ClampProbe:SetUserPlaced(true); assert(not ClampProbe:IsClampedToScreen())")
        .unwrap();
    env.state()
        .borrow_mut()
        .widgets
        .set_screen_clamp_suppressed(id, false);
    env.exec("ClampProbe:SetUserPlaced(false); assert(ClampProbe:IsClampedToScreen())")
        .unwrap();
}

#[test]
fn clamp_setters_require_readable_arguments_and_return_native_arity() {
    let (env, _) = fixture();
    env.exec(r#"
        local f=ClampProbe
        for _,call in ipairs({function() f:SetClampedToScreen() end,
            function() f:SetClampedToScreen(nil) end}) do
            local ok,e=pcall(call); assert(not ok and tostring(e):find('Usage: self:SetClampedToScreen(clampedToScreen)',1,true))
        end
        for _,call in ipairs({function() f:SetClampRectInsets() end,
            function() f:SetClampRectInsets(1) end,function() f:SetClampRectInsets(1,2) end,
            function() f:SetClampRectInsets(1,2,3) end}) do
            local ok,e=pcall(call); assert(not ok and tostring(e):find('Usage: self:SetClampRectInsets(left, right, top, bottom)',1,true))
        end
        assert(select('#',f:SetClampedToScreen(0))==0 and f:IsClampedToScreen())
        assert(select('#',f:SetClampRectInsets('1.5',2.5,-3.5,4.5))==0)
        assert(select('#',f:GetClampRectInsets())==4)
        local l,r,t,b=f:GetClampRectInsets(); assert(l==1.5 and r==2.5 and t==-3.5 and b==4.5)
    "#).unwrap();
}

#[test]
fn already_dirty_clamp_root_invalidates_clean_anchor_dependent() {
    let (env, id) = fixture();
    env.exec(
        r#"
        ClampDependent=CreateFrame('Frame',nil,UIParent)
        ClampDependent:SetSize(10,10)
        ClampDependent:SetPoint('TOPLEFT',ClampProbe,'TOPLEFT',10,0)
        assert(ClampDependent:GetLeft()==10)
        ClampProbe:SetPoint('TOPLEFT',UIParent,'TOPLEFT',-40,20)
    "#,
    )
    .unwrap();
    assert!(env.state().borrow().widgets.is_rect_dirty_self(id));
    env.state()
        .borrow_mut()
        .widgets
        .set_screen_clamp_suppressed(id, true);
    env.exec("assert(ClampDependent:GetLeft()==-30, 'query dependent before dirty root')")
        .unwrap();
}
