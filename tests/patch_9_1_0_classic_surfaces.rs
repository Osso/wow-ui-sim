//! Classic namespace lookup remains unaffected by retail retirement.
#![cfg(any(feature = "client-mists", feature = "client-wrath", feature = "client-era", feature = "client-anniversary"))]

#[test]
fn patch_9_1_0_classic_preserves_legacy_lookup() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(r#"
for _, entry in ipairs({
    {'C_BarberShop', 'OldBarberShopLoaded'},
    {'C_LegendaryCrafting', 'GetRuneforgePowersByClassAndSpec'},
    {'C_PetJournal', 'GetNumMaxPets'},
    {'C_PlayerChoice', 'GetPlayerChoiceInfo'},
    {'C_PlayerChoice', 'GetPlayerChoiceOptionInfo'},
    {'C_PlayerChoice', 'GetPlayerChoiceRewardInfo'},
    {'C_Soulbinds', 'GetConduitRankFromCollection'},
    {'C_Transmog', 'LoadSources'},
    {'C_Transmog', 'ValidateAllPending'},
    {'C_TransmogCollection', 'CanSetFavoriteInCategory'},
    {'C_TransmogCollection', 'GetIllusionFallbackWeaponSource'},
    {'C_TransmogCollection', 'GetIllusionSourceInfo'},
    {'C_TransmogCollection', 'GetInspectSources'},
    {'C_TransmogCollection', 'GetOutfitName'},
    {'C_TransmogCollection', 'GetOutfitSources'},
    {'C_TransmogCollection', 'GetShowMissingSourceInItemTooltips'},
    {'C_TransmogCollection', 'SaveOutfit'},
    {'C_TransmogCollection', 'SetShowMissingSourceInItemTooltips'},
    {'C_TransmogSets', 'GetSetSources'},
    {'C_TransmogSets', 'IsSetCollected'},
    {'C_TransmogSets', 'IsSetUsable'},
}) do
    assert(type(_G[entry[1]][entry[2]]) == "function", entry[1] .. "." .. entry[2])
end
"#).unwrap();
}
