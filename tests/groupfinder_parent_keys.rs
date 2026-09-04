use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn groupfinder_queue_frames_keep_party_backfill_parent_keys_after_startup(env: &WowLuaEnv) {

    let result: String = env
        .eval(
            r#"
            local checks = {
                { "LFDQueueFrame", "LFDQueueFramePartyBackfill" },
                { "RaidFinderQueueFrame", "RaidFinderQueueFramePartyBackfill" },
                { "ScenarioQueueFrame", "ScenarioQueueFramePartyBackfill" },
            }

            for _, check in ipairs(checks) do
                local parent = _G[check[1]]
                local child = _G[check[2]]
                if parent == nil then
                    return "missing_parent:" .. check[1]
                end
                if child == nil then
                    return "missing_child:" .. check[2]
                end
                if parent.PartyBackfill == nil then
                    return "missing_parent_key:" .. check[1]
                end
                if parent.PartyBackfill ~= child then
                    return "wrong_child:" .. check[1]
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "queue frames should expose PartyBackfill via the parentKey Lua field after startup"
    );
}
}

prefork_full_ui_case! {
fn groupfinder_backfill_update_uses_real_party_backfill_frames_after_startup(env: &WowLuaEnv) {

    let result: String = env
        .eval(
            r#"
            local checks = {
                { "LFDQueueFrame", false },
                { "RaidFinderQueueFrame", true },
                { "ScenarioQueueFrame", true },
            }

            for _, check in ipairs(checks) do
                local parent = _G[check[1]]
                if parent == nil then
                    return "missing_parent:" .. check[1]
                end

                local backfill = parent.PartyBackfill
                if backfill == nil then
                    return "missing_backfill:" .. check[1]
                end

                local ok, err = pcall(LFGBackfillCover_Update, backfill, check[2])
                if not ok then
                    return "update_failed:" .. check[1] .. ":" .. tostring(err)
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "real LFGBackfillCover_Update calls should work without the removed nil-self workaround"
    );
}
}
