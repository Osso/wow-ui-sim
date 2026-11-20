//! The original crafting order quality cell uses host-supplied recipe quality.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::c_trade_skill_quality::CraftingQualityInfo;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn original_order_quality_cell_displays_each_configured_atlas() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    {
        let mut sim = env.state().borrow_mut();
        let records = &mut sim.recipe_quality_inputs.item_quality;
        for (quality, icon) in [
            (1, "Professions-Icon-Quality-Tier1"),
            (2, "Professions-Icon-Quality-Tier2"),
        ] {
            records.insert(
                (164, quality),
                CraftingQualityInfo {
                    quality,
                    icon: icon.to_owned(),
                    ..Default::default()
                },
            );
        }
    }
    env.exec(
        r#"
        assert(C_AddOns.LoadAddOn('Blizzard_ProfessionsTemplates'))
        local cell = Mixin(CreateFrame('Frame'), ProfessionsCrafterTableCellQualityMixin)
        cell.Text = cell:CreateFontString(nil, 'ARTWORK', 'GameFontNormal')
        cell:Populate({ spellID = 164, quality = 1 }, 1)
        local first = cell.Text:GetText()
        cell:Populate({ spellID = 164, quality = 2 }, 2)
        local second = cell.Text:GetText()
        assert(first == CreateAtlasMarkup('Professions-Icon-Quality-Tier1', 25, 25))
        assert(second == CreateAtlasMarkup('Professions-Icon-Quality-Tier2', 25, 25))
        assert(first ~= second, 'substituting one constant icon must fail')
    "#,
    )
    .expect("original crafting order quality-cell population");
}
