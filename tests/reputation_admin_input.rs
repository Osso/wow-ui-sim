//! `A_Admin.SetReputationFactions` validation and watched-row input
//! (docs/admin-api/reputation.md).
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const WATCHED_ROWS_LUA: &str = r#"
    local updates = 0
    local listener = CreateFrame("Frame")
    listener:RegisterEvent("UPDATE_FACTION")
    listener:SetScript("OnEvent", function() updates = updates + 1 end)
    A_Admin.SetReputationFactions({
        { factionID = 1, name = "Group", reaction = 0, currentReactionThreshold = 0,
          nextReactionThreshold = 0, currentStanding = 0, isHeader = true },
        { factionID = 2590, name = "Council of Dornogal", reaction = 5,
          currentReactionThreshold = 3000, nextReactionThreshold = 9000,
          currentStanding = 4500, isChild = true, isWatched = true },
    })
    local watched = C_Reputation.GetWatchedFactionData()
    return updates, C_Reputation.GetNumFactions(), watched.factionID, watched.currentStanding
"#;

#[test]
fn supplied_rows_replace_the_list_select_the_watched_row_and_fire_update() {
    let env = WowLuaEnv::new().expect("Lua environment");
    let result: (i32, i32, i32, i32) = env.eval(WATCHED_ROWS_LUA).unwrap();
    assert_eq!(result, (1, 2, 2590, 4500));
}

#[test]
fn invalid_rows_are_rejected_with_the_field_name() {
    let env = WowLuaEnv::new().expect("Lua environment");
    for (row, field) in [
        ("{ factionID = 'x', reaction = 5, currentReactionThreshold = 0, nextReactionThreshold = 1, currentStanding = 0 }", "factionID"),
        ("{ factionID = 7, currentReactionThreshold = 0, nextReactionThreshold = 1, currentStanding = 0 }", "reaction"),
        ("{ factionID = 7, reaction = 5, currentReactionThreshold = 0, nextReactionThreshold = 1, currentStanding = 0, isHeader = 1 }", "isHeader"),
    ] {
        let error = env
            .exec(&format!("A_Admin.SetReputationFactions({{ {row} }})"))
            .unwrap_err()
            .to_string();
        assert!(error.contains(&format!("invalid {field}")), "{error}");
    }
}

#[test]
fn only_one_row_may_be_watched() {
    let env = WowLuaEnv::new().expect("Lua environment");
    let row = "reaction = 5, currentReactionThreshold = 0, nextReactionThreshold = 1, currentStanding = 0, isWatched = true";
    let error = env
        .exec(&format!(
            "A_Admin.SetReputationFactions({{ {{ factionID = 7, {row} }}, {{ factionID = 8, {row} }} }})"
        ))
        .unwrap_err()
        .to_string();
    assert!(error.contains("invalid isWatched"), "{error}");
}
