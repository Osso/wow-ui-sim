//! Tests for forbidden frame proxy behavior.
//!
//! Forbidden frames use a proxy table instead of LightUserData. The proxy must
//! delegate both __index and __newindex to the underlying LightUserData so that
//! children_keys and __frame_fields stay in sync.

use crate::common;

use common::env_with_shared_xml;

/// Forbidden proxy __newindex must delegate to the underlying LightUserData.
/// Without this, assignments on the proxy table are invisible when the same frame
/// is later accessed as LightUserData (e.g., via children_keys lookup).
///
/// The template code pattern:
/// 1. `parent["Track"] = CreateFrame(...)` → lud stored in children_keys ✓
/// 2. `_G["__tpl_XXX"]` returns proxy table for Track
/// 3. `proxy["Thumb"] = CreateFrame(...)` → rawset on proxy, NOT synced ✗
/// 4. `parent.Track.Thumb` → children_keys returns lud → lud.Thumb = nil
#[test]
fn test_forbidden_frame_newindex_syncs_children_keys() {
    let env = env_with_shared_xml();

    // Step 1: Create a normal (non-forbidden) parent frame
    env.exec("TestParent = CreateFrame('Frame', 'TestParent', UIParent)")
        .unwrap();

    // Step 2: Enable forbidden, create named child. CreateFrame returns LUD but
    // _G["TestChild"] gets the proxy (set inside create_frame_userdata).
    // Assign the return LUD to parent via parentKey (goes through LUD __newindex).
    env.state().borrow_mut().loading_forbidden = true;
    env.exec("TestParent.Child = CreateFrame('Frame', 'TestChild', TestParent)")
        .unwrap();

    // Step 3: Access child via _G["TestChild"] (proxy), assign a grandchild on it.
    // This mimics template code: _G["__tpl_XXX"] returns proxy, proxy["Key"] = lud.
    env.exec(
        r#"
        local proxy = _G["TestChild"]
        proxy.GC = CreateFrame("Frame", nil, proxy)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().loading_forbidden = false;

    // Step 4: Access via children_keys path.
    // TestParent.Child returns LUD (from children_keys), then .GC uses LUD __index.
    let result: String = env
        .eval(
            r#"
        local child_lud = TestParent.Child
        return tostring(child_lud ~= nil) .. "," .. tostring(child_lud.GC ~= nil)
    "#,
        )
        .unwrap();
    assert_eq!(
        result, "true,true",
        "property set on proxy should be visible via LightUserData __index"
    );
}

/// Template children created inside a forbidden scope should have their parentKey
/// accessible. This reproduces the ScrollBar.lua:30 error where Track.Thumb is nil
/// because Track was a forbidden frame and __newindex didn't sync to children_keys.
#[test]
fn test_forbidden_scrollbar_track_has_thumb() {
    let env = env_with_shared_xml();

    // Set forbidden loading context before creating the scrollbar
    env.state().borrow_mut().loading_forbidden = true;

    let result: String = env
        .eval(
            r#"
        local sb = CreateFrame("EventFrame", nil, UIParent, "MinimalScrollBar")
        local track = sb.Track
        local thumb = track and track.Thumb or nil
        return tostring(track ~= nil) .. "," .. tostring(thumb ~= nil)
    "#,
        )
        .unwrap();

    env.state().borrow_mut().loading_forbidden = false;

    assert_eq!(
        result, "true,true",
        "Forbidden MinimalScrollBar's Track.Thumb should be accessible"
    );
}

#[test]
fn has_access_constraints_reports_forbidden_frames_without_restrictions() {
    let env = env_with_shared_xml();
    env.exec("local normal = CreateFrame('Frame', 'AccessConstraintNormal', UIParent)")
        .unwrap();

    env.state().borrow_mut().loading_forbidden = true;
    env.exec("local forbidden = CreateFrame('Frame', 'AccessConstraintForbidden', UIParent)")
        .unwrap();
    env.state().borrow_mut().loading_forbidden = false;

    let result: (String, bool, bool) = env
        .eval(
            "return type(AccessConstraintNormal:HasAccessConstraints()), \
             AccessConstraintNormal:HasAccessConstraints(), \
             AccessConstraintForbidden:HasAccessConstraints()",
        )
        .unwrap();

    assert_eq!(result, ("boolean".to_string(), false, true));
}

#[cfg(feature = "forbidden-aspects")]
#[test]
fn access_restrictions_accumulate_masks_and_support_filtered_queries() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(r#"
        local frame = CreateFrame('Frame')
        local auraRestriction = Enum.ScriptObjectAccessRestriction.DenyTaintedAccessWhenAurasAreSecret
        assert(auraRestriction == 1)
        assert(frame:GetAccessRestrictions() == 0)
        assert(not frame:HasAnyAccessRestrictions())
        assert(not frame:HasAnyAccessRestrictions(nil))
        assert(not frame:HasAccessConstraints())

        frame:AddAccessRestrictions(auraRestriction)
        frame:AddAccessRestrictions(4)
        frame:AddAccessRestrictions(auraRestriction)
        frame:AddAccessRestrictions(0)
        assert(frame:GetAccessRestrictions() == 5, 'additions accumulate rather than replace')
        assert(frame:HasAnyAccessRestrictions())
        assert(frame:HasAnyAccessRestrictions(nil))
        assert(frame:HasAnyAccessRestrictions(auraRestriction))
        assert(frame:HasAnyAccessRestrictions(6), 'any intersecting bit matches')
        assert(not frame:HasAnyAccessRestrictions(2))
        assert(not frame:HasAnyAccessRestrictions(0))
        assert(frame:HasAccessConstraints())
        assert(not frame:IsForbidden(), 'conditional restrictions do not mark the frame forbidden')
    "#).expect("restriction masks and optional query masks");
}

#[cfg(feature = "forbidden-aspects")]
#[test]
fn access_restrictions_are_per_frame_and_combine_with_forbidden_state() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local parent = CreateFrame('Frame')
        local child = CreateFrame('Frame', nil, parent)
        local sibling = CreateFrame('Frame', nil, parent)
        local mask = Enum.ScriptObjectAccessRestriction.DenyTaintedAccessWhenAurasAreSecret
        parent:AddAccessRestrictions(mask)
        assert(parent:GetAccessRestrictions() == mask)
        assert(child:GetAccessRestrictions() == 0 and sibling:GetAccessRestrictions() == 0)
        assert(not child:HasAccessConstraints() and not sibling:HasAccessConstraints())

        child:SetForbidden()
        assert(child:IsForbidden() and child:HasAccessConstraints())
        assert(not child:HasAnyAccessRestrictions())
        assert(child:GetAccessRestrictions() == 0)
        child:AddAccessRestrictions(mask)
        assert(child:IsForbidden() and child:HasAccessConstraints())
        assert(child:HasAnyAccessRestrictions(mask))
        assert(sibling:GetAccessRestrictions() == 0 and not sibling:HasAccessConstraints())
    "#,
    )
    .expect("frame isolation and forbidden/restriction union");
}

#[cfg(feature = "forbidden-aspects")]
#[test]
fn access_restrictions_native_aura_consumer_defers_until_world_entry_then_applies_immediately() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_AuraContainer"],
        &[],
    );
    env.set_logged_in(false);
    env.exec_maybe_secure(
        r#"
        RestrictionDeferred = CreateFrame('Frame')
        RestrictionUnchanged = CreateFrame('Frame')
        local mask = Enum.ScriptObjectAccessRestriction.DenyTaintedAccessWhenAurasAreSecret
        assert(not IsLoggedIn())
        AuraContainerUtil.ApplyAccessRestrictions(RestrictionDeferred, mask)
        assert(RestrictionDeferred:GetAccessRestrictions() == 0,
            'pre-login native application must wait for world entry')
        "#,
        true,
    )
    .unwrap();

    env.set_logged_in(true);
    env.fire_event("PLAYER_LOGIN").unwrap();
    env.exec_maybe_secure(
        r#"
        assert(IsLoggedIn())
        assert(RestrictionDeferred:GetAccessRestrictions() == 0,
            'PLAYER_LOGIN must not apply the deferred restriction')
        RestrictionImmediate = CreateFrame('Frame')
        local mask = Enum.ScriptObjectAccessRestriction.DenyTaintedAccessWhenAurasAreSecret
        AuraContainerUtil.ApplyAccessRestrictions(RestrictionImmediate, mask)
        assert(RestrictionImmediate:GetAccessRestrictions() == mask,
            'post-login native application must not wait for world entry')
        "#,
        true,
    )
    .unwrap();

    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    env.exec_maybe_secure(
        r#"
        local mask = Enum.ScriptObjectAccessRestriction.DenyTaintedAccessWhenAurasAreSecret
        assert(RestrictionDeferred:GetAccessRestrictions() == mask)
        assert(RestrictionImmediate:GetAccessRestrictions() == mask)
        assert(RestrictionUnchanged:GetAccessRestrictions() == 0)
        assert(RestrictionDeferred:HasAccessConstraints())
        assert(not RestrictionDeferred:IsForbidden())
        "#,
        true,
    )
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn test_forbidden_proxy_method_lookup_does_not_exhaust_aux_stack() {
    let env = env_with_shared_xml();
    env.state().borrow_mut().loading_forbidden = true;
    env.exec("CreateFrame('Frame', 'ForbiddenLoopFrame', UIParent)")
        .unwrap();
    env.state().borrow_mut().loading_forbidden = false;

    env.exec(
        r#"
        local proxy = _G["ForbiddenLoopFrame"]
        for i = 1, 9000 do
            local fn = proxy.GetName
            assert(type(fn) == "function")
            assert(fn(proxy) == "ForbiddenLoopFrame")
        end
    "#,
    )
    .unwrap();
}

#[test]
fn test_set_point_accepts_forbidden_proxy_relative_to() {
    let env = env_with_shared_xml();
    env.state().borrow_mut().loading_forbidden = true;

    let result: String = env
        .eval(
            r#"
        local parent = CreateFrame("Frame", "ForbiddenAnchorParent", UIParent)
        parent:SetSize(80, 35)
        parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 10, -10)

        local child = parent:CreateTexture("ForbiddenAnchorChild", "BACKGROUND")
        child:SetSize(20, 35)
        child:SetPoint("TOPLEFT", parent, "TOPLEFT", 0, 0)
        child:SetPoint("BOTTOMLEFT", parent, "BOTTOMLEFT", 0, 0)

        local _, _, parentW, parentH = parent:GetRect()
        local _, _, childW, childH = child:GetRect()
        return string.format("%.0f,%.0f,%.0f,%.0f", parentW or 0, parentH or 0, childW or 0, childH or 0)
    "#,
        )
        .unwrap();

    env.state().borrow_mut().loading_forbidden = false;

    assert_eq!(
        result, "80,35,20,35",
        "SetPoint should unwrap forbidden proxy relativeTo instead of anchoring to the screen"
    );
}
