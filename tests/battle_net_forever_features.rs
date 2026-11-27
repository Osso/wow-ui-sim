//! Forever 1.60.1 BattleNetDocumentation.lua declares the five account feature
//! queries. Friend tags load disabled (F4 Q1); the rest follow supplied values.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

const FEATURES: &str = r#"
    local function read()
        return {
            C_BattleNet.AreFriendTagsEnabled(),
            C_BattleNet.AreTitleFriendCustomNamesEnabled(),
            C_BattleNet.AreTitleFriendsEnabled(),
            C_BattleNet.IsBattleNetFriendsListEnabled(),
            C_BattleNet.IsBattleNetFriendsListSupported(),
        }
    end
    assert(read()[1] == false, "friend tags load disabled")
    for _, supplied in ipairs({
        { true, false, true, false, true },
        { false, true, false, true, false },
    }) do
        A_Admin.SetBattleNetAccountFeatures(unpack(supplied))
        local observed = read()
        for index = 1, 5 do
            assert(observed[index] == supplied[index], "feature " .. index)
        end
    end
"#;

#[test]
fn forever_battle_net_features_follow_supplied_values() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(FEATURES).expect("feature queries follow their inputs");
}
