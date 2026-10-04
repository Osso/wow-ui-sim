//! Retail 12.1.0 Roleset System: frame tags filtered by `C_Roleset`.
#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn eval_ok(env: &WowLuaEnv, code: &str) {
    let result: String = env.eval(code).unwrap();
    assert_eq!(result, "ok");
}

fn in_strata_buckets(env: &WowLuaEnv, name: &str) -> bool {
    let mut state = env.state().borrow_mut();
    let id = state.widgets.get_id_by_name(name).unwrap();
    state
        .get_strata_buckets()
        .is_some_and(|buckets| buckets.iter().any(|bucket| bucket.contains(&id)))
}

#[test]
fn inactive_roleset_hides_frame_and_children_while_shown() {
    let env = WowLuaEnv::new().unwrap();
    eval_ok(
        &env,
        r#"
        RolesetChat = CreateFrame("Frame", "RolesetChat", UIParent)
        RolesetChat:SetSize(40, 40)
        RolesetChat:SetPoint("CENTER")
        RolesetChat:SetRolesets("chat, microMenu")
        RolesetChatChild = CreateFrame("Frame", nil, RolesetChat)
        RolesetPlain = CreateFrame("Frame", "RolesetPlain", UIParent)
        local names = RolesetChat:GetRolesetNames()
        if table.concat(names, ",") ~= "chat,microMenu" then return "names" end
        if RolesetPlain:GetRolesetNames()[1] ~= "roleless" then return "roleless" end
        if RolesetChat:IsRolesetFiltered() or not RolesetChat:IsVisible() then return "initial" end

        C_Roleset.ApplyRolesetFilters({ "chat" }, {})
        if not RolesetChat:IsRolesetFiltered() then return "blocked-filtered" end
        if RolesetChat:IsVisible() then return "blocked-visible" end
        if not RolesetChat:IsShown() then return "blocked-shown" end
        if RolesetChatChild:IsVisible() then return "child-visible" end
        if RolesetChatChild:IsRolesetFiltered() then return "child-own-filter" end
        if not RolesetPlain:IsVisible() then return "untagged-blocked" end
        return "ok"
        "#,
    );
    assert!(!in_strata_buckets(&env, "RolesetChat"));

    eval_ok(
        &env,
        r#"
        -- Show/Hide keep working on the shown state while filtered.
        RolesetChat:Hide()
        RolesetChat:Show()
        if RolesetChat:IsVisible() or not RolesetChat:IsShown() then return "toggle" end

        -- Allowlist: a frame with none of the allowed rolesets is filtered.
        C_Roleset.ApplyRolesetFilters({}, { "unitFrames" })
        if not RolesetChat:IsRolesetFiltered() then return "allow-miss" end
        if not RolesetPlain:IsVisible() then return "untagged-allow" end
        C_Roleset.ApplyRolesetFilters({}, { "microMenu" })
        if RolesetChat:IsRolesetFiltered() or not RolesetChat:IsVisible() then return "allow-hit" end

        -- Changing the frame's own tags reevaluates it immediately.
        C_Roleset.ApplyRolesetFilters({ "bags" }, {})
        RolesetChat:AddRoleset("bags")
        if not RolesetChat:IsRolesetFiltered() then return "add" end
        RolesetChat:RemoveRoleset("bags")
        if RolesetChat:IsRolesetFiltered() then return "remove" end

        C_Roleset.ApplyRolesetFilters({}, {})
        if RolesetChat:IsRolesetFiltered() or not RolesetChatChild:IsVisible() then return "cleared" end
        return "ok"
        "#,
    );
    assert!(in_strata_buckets(&env, "RolesetChat"));
}

#[test]
fn always_blocked_roleset_never_shows_without_filters() {
    let env = WowLuaEnv::new().unwrap();
    eval_ok(
        &env,
        r#"
        local frame = CreateFrame("Frame", nil, UIParent)
        frame:AddRoleset("alwaysBlocked")
        if #C_Roleset.GetActiveBlockedRolesets() ~= 0 then return "filters-active" end
        if not frame:IsRolesetFiltered() or frame:IsVisible() then return "blocked" end
        if not frame:IsShown() then return "shown" end
        frame:SetRolesets(nil)
        if frame:IsRolesetFiltered() or not frame:IsVisible() then return "cleared" end
        return "ok"
        "#,
    );
}

#[test]
fn xml_roleset_attribute_tags_frame() {
    let env = WowLuaEnv::new().unwrap();
    let directory = tempfile::tempdir().expect("create fixture directory");
    let toc = directory.path().join("RolesetFixture.toc");
    std::fs::write(&toc, "## Title: RolesetFixture\nfixture.xml\n").unwrap();
    std::fs::write(
        directory.path().join("fixture.xml"),
        r#"<Ui>
            <Frame name="RolesetXmlTemplate" virtual="true" roleset="buffs"/>
            <Frame name="RolesetXmlBars" parent="UIParent" roleset="actionBars"/>
            <Frame name="RolesetXmlBuffs" parent="UIParent" inherits="RolesetXmlTemplate"/>
        </Ui>"#,
    )
    .unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).expect("load roleset fixture");
    eval_ok(
        &env,
        r#"
        if RolesetXmlBars:GetRolesetNames()[1] ~= "actionBars" then return "attr" end
        if RolesetXmlBuffs:GetRolesetNames()[1] ~= "buffs" then return "template" end
        C_Roleset.ApplyRolesetFilters({ "actionBars" }, {})
        if RolesetXmlBars:IsVisible() or not RolesetXmlBuffs:IsVisible() then return "filter" end
        return "ok"
        "#,
    );
}
