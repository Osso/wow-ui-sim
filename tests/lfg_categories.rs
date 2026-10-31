use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().unwrap()
}

#[test]
fn get_available_categories_includes_dungeons_and_raids() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local cats = C_LFGList.GetAvailableCategories(0)
            if type(cats) ~= "table" then return "type=" .. type(cats) end
            local has2, has3 = false, false
            for _, id in ipairs(cats) do
                if id == 2 then has2 = true end
                if id == 3 then has3 = true end
            end
            if not has2 then return "missing_dungeons" end
            if not has3 then return "missing_raids" end
            if #cats < 2 then return "count=" .. #cats end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetAvailableCategories: {result}");
}

#[test]
fn get_lfg_category_info_dungeons() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local info = C_LFGList.GetLfgCategoryInfo(2)
            if type(info) ~= "table" then return "type=" .. type(info) end
            if info.name ~= "Dungeons" then return "name=" .. tostring(info.name) end
            if info.allowCrossFaction ~= true then return "allowCrossFaction=" .. tostring(info.allowCrossFaction) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetLfgCategoryInfo(2): {result}");
}

#[test]
fn get_lfg_category_info_unknown_returns_nil() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local info = C_LFGList.GetLfgCategoryInfo(99)
            return info == nil and "nil" or "not_nil"
            "#,
        )
        .unwrap();
    assert_eq!(result, "nil");
}

#[test]
fn get_available_activities_for_dungeons() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local acts = C_LFGList.GetAvailableActivities(2)
            if type(acts) ~= "table" then return "type=" .. type(acts) end
            if #acts < 2 then return "count=" .. #acts end
            local has1195, has1188 = false, false
            for _, id in ipairs(acts) do
                if id == 1195 then has1195 = true end
                if id == 1188 then has1188 = true end
            end
            if not has1195 then return "missing_1195" end
            if not has1188 then return "missing_1188" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetAvailableActivities(2): {result}");
}

#[test]
fn get_available_activity_groups_for_dungeons() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local groups = C_LFGList.GetAvailableActivityGroups(2)
            if type(groups) ~= "table" then return "type=" .. type(groups) end
            if #groups < 1 then return "empty" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetAvailableActivityGroups(2): {result}");
}

#[test]
fn get_activity_group_info() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local name, order = C_LFGList.GetActivityGroupInfo(295)
            if name ~= "The War Within Mythic+" then return "name=" .. tostring(name) end
            if type(order) ~= "number" then return "order_type=" .. type(order) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetActivityGroupInfo(295): {result}");
}

#[test]
fn get_activity_full_name() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local name = C_LFGList.GetActivityFullName(1195)
            if name ~= "Mists of Tirna Scithe (M+)" then return "name=" .. tostring(name) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetActivityFullName(1195): {result}");
}

#[test]
fn has_activity_list_returns_true() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            return C_LFGList.HasActivityList() and "true" or "false"
            "#,
        )
        .unwrap();
    assert_eq!(result, "true");
}

#[test]
fn has_active_entry_info_returns_false() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            return C_LFGList.HasActiveEntryInfo() and "true" or "false"
            "#,
        )
        .unwrap();
    assert_eq!(result, "false");
}

#[test]
fn get_active_entry_info_returns_nil() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local info = C_LFGList.GetActiveEntryInfo()
            return info == nil and "nil" or "not_nil"
            "#,
        )
        .unwrap();
    assert_eq!(result, "nil");
}

#[test]
fn get_available_roles_returns_true_true_true() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local tank, healer, dps = C_LFGList.GetAvailableRoles()
            if tank ~= true then return "tank=" .. tostring(tank) end
            if healer ~= true then return "healer=" .. tostring(healer) end
            if dps ~= true then return "dps=" .. tostring(dps) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetAvailableRoles: {result}");
}

#[test]
fn has_search_result_info() {
    let env = env();
    // Retail GroupFinder/Mainline/LFGList.lua:2817 searches before reading results.
    env.exec("C_LFGList.Search(2)").unwrap();
    let result: String = env
        .eval(
            r#"
            if not C_LFGList.HasSearchResultInfo(1) then return "1_missing" end
            if C_LFGList.HasSearchResultInfo(9999) then return "9999_found" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "HasSearchResultInfo: {result}");
}

#[test]
fn get_activity_info_table_for_seeded_activities() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local info = C_LFGList.GetActivityInfoTable(1195)
            if type(info) ~= "table" then return "type=" .. type(info) end
            if info.maxPlayers ~= 5 then return "max=" .. tostring(info.maxPlayers) end
            if info.fullName ~= "Mists of Tirna Scithe (M+)" then return "name=" .. tostring(info.fullName) end
            if info.categoryID ~= 2 then return "cat=" .. tostring(info.categoryID) end
            if info.isMythicPlusActivity ~= true then return "mplus=" .. tostring(info.isMythicPlusActivity) end
            local info2 = C_LFGList.GetActivityInfoTable(1296)
            if info2.maxPlayers ~= 20 then return "raid_max=" .. tostring(info2.maxPlayers) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "GetActivityInfoTable: {result}");
}
