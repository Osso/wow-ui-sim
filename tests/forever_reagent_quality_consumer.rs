#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::c_trade_skill_quality::CraftingQualityInfo;

#[path = "common/blizzard_addon_closure.rs"]
mod closure;

#[test]
fn professions_consumer_observes_two_supplied_reagent_qualities() {
    let ui = closure::blizzard_ui_dir();
    let (env, loaded) =
        closure::build_blizzard_addon_closure_env(&ui, &["Blizzard_ProfessionsTemplates"], &[]);
    assert!(
        loaded
            .iter()
            .any(|name| name == "Blizzard_ProfessionsTemplates")
    );
    for (item, quality, icon) in [(900001, 1, "first-chat"), (900002, 2, "second-chat")] {
        env.state()
            .borrow_mut()
            .recipe_quality_inputs
            .reagent_item_quality
            .insert(
                item,
                CraftingQualityInfo {
                    quality,
                    icon_chat: icon.into(),
                    ..Default::default()
                },
            );
    }
    env.exec(
        r#"
        local source = debug.getinfo(Professions.GetReagentQualityInfo, 'S').source
        assert(source:find('Blizzard_Professions.lua', 1, true), source)
        local first = Professions.GetReagentQualityInfo({itemID = 900001})
        local second = Professions.GetReagentQualityInfo({itemID = 900002})
        assert(first.quality == 1 and first.iconChat == 'first-chat')
        assert(second.quality == 2 and second.iconChat == 'second-chat')
    "#,
    )
    .expect("original professions consumer must read supplied quality records");
}
