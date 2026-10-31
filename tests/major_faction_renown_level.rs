//! `C_MajorFactions.GetCurrentRenownLevel` / `HasMaximumRenown` read supplied
//! faction state, observed through Blizzard's landing-page renown tooltip.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::c_api::c_major_factions::RenownRewardInfo;
use wow_ui_sim::lua_api::{MajorFactionData, WowLuaEnv};

const FACTION: i64 = 2507;

fn supply_faction(env: &WowLuaEnv, renown_level: i32, max_level: i32) {
    env.state().borrow_mut().major_factions.insert(
        FACTION,
        MajorFactionData {
            faction_id: FACTION,
            name: "Fixture faction".into(),
            max_level,
            renown_level,
            renown_reputation_earned: 100,
            renown_level_threshold: 2500,
            is_unlocked: true,
            ..Default::default()
        },
    );
}

fn reward_at_level_4() -> RenownRewardInfo {
    RenownRewardInfo {
        renown_reward_id: 9401,
        ui_order: 1,
        is_account_unlock: false,
        item_id: None,
        spell_id: None,
        mount_id: None,
        transmog_id: None,
        transmog_set_id: None,
        title_mask_id: None,
        transmog_illusion_source_id: None,
        icon: Some(134400),
        name: Some("Level four reward".into()),
        description: None,
        toast_description: None,
        reward_type: None,
        is_collected: None,
    }
}

/// Runs the original consumer and returns every tooltip line joined by `|`.
fn landing_page_tooltip(env: &WowLuaEnv) -> String {
    env.eval(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        RenownRewardUtil.AddMajorFactionLandingPageSummaryToTooltip(GameTooltip, 2507)
        local lines = {}
        for i = 1, GameTooltip:NumLines() do
            lines[#lines + 1] = _G["GameTooltipTextLeft" .. i]:GetText() or ""
        end
        GameTooltip:Hide()
        return table.concat(lines, "|")
        "#,
    )
    .expect("landing-page renown tooltip should build")
}

prefork_full_ui_case! {
fn renown_tooltip_follows_supplied_level_and_cap(env: &WowLuaEnv) {
    supply_faction(env, 3, 5);
    env.state()
        .borrow_mut()
        .major_faction_renown_rewards
        .insert((FACTION, 4), vec![reward_at_level_4()]);

    let below_cap = landing_page_tooltip(env);
    assert!(
        below_cap.contains("Level four reward"),
        "level 3 of 5 must show the level-4 reward: {below_cap}"
    );

    supply_faction(env, 5, 5);
    let at_cap = landing_page_tooltip(env);
    assert!(
        !at_cap.contains("Level four reward") && at_cap.contains("Fixture faction"),
        "a capped faction must skip the progress and next-reward block: {at_cap}"
    );
}
}

prefork_full_ui_case! {
fn renown_level_and_cap_direct_queries(env: &WowLuaEnv) {
    supply_faction(env, 3, 5);
    let below: (i32, bool) = env
        .eval("return C_MajorFactions.GetCurrentRenownLevel(2507), C_MajorFactions.HasMaximumRenown(2507)")
        .unwrap();
    assert_eq!(below, (3, false));

    supply_faction(env, 6, 5);
    let above: (i32, bool) = env
        .eval("return C_MajorFactions.GetCurrentRenownLevel(2507), C_MajorFactions.HasMaximumRenown(2507)")
        .unwrap();
    assert_eq!(above, (6, true));

    let unknown: (i32, bool, i32, bool) = env
        .eval(
            "return C_MajorFactions.GetCurrentRenownLevel(987654), \
                    C_MajorFactions.HasMaximumRenown(987654), \
                    C_MajorFactions.GetCurrentRenownLevel('x'), \
                    C_MajorFactions.HasMaximumRenown('x')",
        )
        .unwrap();
    assert_eq!(unknown, (0, false, 0, false), "unknown or non-number IDs return 0 and false");
}
}
