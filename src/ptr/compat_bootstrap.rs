#[cfg(not(feature = "client-ptr"))]
const PATCH_12_1_COMPAT_BOOTSTRAP_LUA: &str = concat!(
    "Enum.TieredEntranceTypeMeta = { MinValue = 0, MaxValue = 4, NumValues = 5 }\n",
    include_str!("../lua_api/workarounds/temporary/securecopy.lua"),
    "\n",
    include_str!("compat_bootstrap.lua"),
);
#[cfg(feature = "client-ptr")]
const PATCH_12_1_COMPAT_BOOTSTRAP_LUA: &str = concat!(
    include_str!("../lua_api/workarounds/temporary/securecopy.lua"),
    "\n",
    include_str!("tiered_entrance_type.lua"),
    "\n",
    include_str!("fragment_id.lua"),
    "\n",
    include_str!("transmog_illusion_flags.lua"),
    "\n",
    include_str!("bonus_stat_index.lua"),
    "\n",
    include_str!("weather_type.lua"),
    "\n",
    include_str!("player_data_flags.lua"),
    "\n",
    include_str!("intl_enums.lua"),
    "\n",
    include_str!("compat_bootstrap.lua"),
    "\n",
    include_str!("tooltip_data_line_type.lua"),
    "\n",
    include_str!("seconds_formatter_abbreviation.lua"),
);
const PATCH_12_1_STRICT_REMOVALS_LUA: &str = include_str!("strict_removals.lua");

pub fn init(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(PATCH_12_1_COMPAT_BOOTSTRAP_LUA)?;
    Ok(())
}

pub fn apply_post_load(env: &crate::lua_api::WowLuaEnv) {
    if let Err(err) = env.exec(PATCH_12_1_COMPAT_BOOTSTRAP_LUA) {
        eprintln!("patch 12.1 compat bootstrap failed after load: {err}");
    }
}

pub fn apply_strict_removals(env: &crate::lua_api::WowLuaEnv) {
    if let Err(err) = env.exec(PATCH_12_1_STRICT_REMOVALS_LUA) {
        eprintln!("patch 12.1 strict removals failed after startup events: {err}");
    }
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn patch_12_1_post_load_restores_dynamic_difficulty_color_delegates() {
        let env = WowLuaEnv::new().expect("env");
        env.exec(
            r#"
            DifficultyUtil = {}
            local function sentinel(name)
                return function(...)
                    local arguments = {...}
                    for index, value in ipairs(arguments) do
                        arguments[index] = tostring(value)
                    end
                    return name .. ":" .. table.concat(arguments, ","), name .. "-highlight"
                end
            end
            GetCreatureDifficultyColor = sentinel("creature")
            GetDifficultyColor = sentinel("difficulty")
            GetQuestDifficultyColor = sentinel("quest")
            GetRelativeDifficultyColor = sentinel("relative")
            GetScalingQuestDifficultyColor = sentinel("scaling")
            "#,
        )
        .expect("install sentinel globals");

        super::apply_post_load(&env);

        let result: String = env
            .eval(
                r#"
                local cases = {
                    { "GetCreatureDifficultyColor", { 71 }, "creature:71", "creature-highlight" },
                    { "GetDifficultyColor", { 4 }, "difficulty:4", "difficulty-highlight" },
                    { "GetQuestDifficultyColor", { 72, true, 99 }, "quest:72,true,99", "quest-highlight" },
                    { "GetRelativeDifficultyColor", { 10, 15 }, "relative:10,15", "relative-highlight" },
                    { "GetScalingQuestDifficultyColor", { 73 }, "scaling:73", "scaling-highlight" },
                }
                for _, case in ipairs(cases) do
                    local color, highlight = DifficultyUtil[case[1]](unpack(case[2]))
                    if color ~= case[3] or highlight ~= case[4] then return case[1] end
                end

                GetRelativeDifficultyColor = function(referenceLevel, targetLevel)
                    return targetLevel - referenceLevel, "replaced"
                end
                local color, highlight = DifficultyUtil.GetRelativeDifficultyColor(10, 15)
                if color ~= 5 or highlight ~= "replaced" then return "hot-swap" end

                GetCreatureDifficultyColor = nil
                local ok = pcall(DifficultyUtil.GetCreatureDifficultyColor, 70)
                if ok then return "missing-global" end

                DifficultyUtil.GetDifficultyColor = function() return "existing" end
                return "ok"
                "#,
            )
            .expect("difficulty delegates should run");

        assert_eq!(result, "ok");

        super::apply_post_load(&env);
        let existing: String = env
            .eval("return DifficultyUtil.GetDifficultyColor()")
            .expect("existing namespace member should remain installed");
        assert_eq!(existing, "existing");
    }

    #[test]
    fn patch_12_1_post_load_reapplies_epoch_enums_after_generated_docs_reset() {
        let env = WowLuaEnv::new().expect("env");
        env.exec("Enum.OnUpdateMode = nil; Enum.ClubStreamType.Discord = nil")
            .expect("reset enums");

        super::apply_post_load(&env);

        let (on_update_mode, discord): (String, String) = env
            .eval(
                r#"
                return Enum.OnUpdateMode.Disabled,
                    type(Enum.ClubStreamType.Discord)
                "#,
            )
            .expect("patch 12.1 enums");
        assert_eq!(on_update_mode, "Disabled");
        assert_eq!(discord, "number");
    }
}
