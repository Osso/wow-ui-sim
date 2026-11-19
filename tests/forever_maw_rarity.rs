#![cfg(all(feature = "client-wowforever", feature = "optional-tests"))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::settle_headless_startup;

const POWERS: [(u32, u32, &str); 2] = [
    (116, 2, "jailerstower-animapowerlist-powerborder-green"),
    (133, 3, "jailerstower-animapowerlist-powerborder-blue"),
];

#[test]
fn forever_maw_buffs_consume_modeled_rarity() {
    let env = WowLuaEnv::new().expect("bootstrap Forever");
    let legacy_native: bool = env
        .eval("return rawget(C_Spell, 'GetMawPowerBorderAtlasBySpellID') ~= nil")
        .unwrap();
    load_game(&env);
    seed_maw_powers(&env);
    assert_maw_buffs(&env);
    assert!(
        !legacy_native,
        "legacy atlas must not be installed at bootstrap"
    );
}

fn assert_maw_buffs(env: &WowLuaEnv) {
    let array = POWERS.map(|(spell, rarity, atlas)| format!("{{{spell}, {rarity}, '{atlas}'}}"));
    env.exec(&format!(
        r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_MawBuffs'))
        local source = debug.getinfo(MawBuffMixin.SetBuffInfo, 'S')
        assert(source.source:gsub('\\', '/'):find('Blizzard_MawBuffs/Blizzard_MawBuffs.lua', 1, true))
        assert(source.linedefined == 241)
        for _, fixture in ipairs({{{}, {}}}) do
            local spell, rarity, atlas = unpack(fixture)
            local buff = CreateFrame('Button', nil, UIParent, 'MawBuffTemplate')
            buff:SetBuffInfo({{spellID = spell, icon = 135846, count = 2, slot = 1}})
            assert(buff.Border:GetAtlas() == atlas, tostring(buff.Border:GetAtlas()))
            assert(buff:IsShown() and buff.Count:IsShown())
            local actualRarity, actualAtlas = C_Spell.GetMawPowerRarityInfoBySpellID(spell)
            assert(actualRarity == rarity and actualAtlas == atlas)
        end
        assert(select('#', C_Spell.GetMawPowerRarityInfoBySpellID(987654)) == 0)
        local wrapper = rawget(C_Spell, 'GetMawPowerBorderAtlasBySpellID')
        assert(debug.getinfo(wrapper, 'S').source:find('Deprecated_12_0_7.lua', 1, true))
        assert(wrapper({}) == '{}')
        "#,
        array[0], array[1], POWERS[0].0, POWERS[0].2,
    ))
    .expect("original MawBuffTemplate uses the modeled rarity and atlas");
}

fn load_game(env: &WowLuaEnv) {
    use wow_ui_sim::loader::{discover_all_blizzard_addons, register_load_with_addons};
    let ui = crate::common::blizzard_ui_dir();
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    crate::common::supply_empty_forever_spellbook_outfits(env);
    crate::common::load_game_startup_inventory(env);
    env.apply_post_load_workarounds();
    settle_headless_startup(env);
}

fn seed_maw_powers(_env: &WowLuaEnv) {
    // The stock Forever build has no host model; its unchanged consumer fails
    // on the missing C_Spell member before any rarity record could be read.
    #[cfg(feature = "maw-power-model")]
    {
        let mut state = _env.state().borrow_mut();
        for (spell, rarity, atlas) in POWERS {
            state.maw_powers.rarity_ids.insert(spell, rarity);
            state.maw_powers.border_atlases.insert(spell, atlas.into());
        }
    }
}
