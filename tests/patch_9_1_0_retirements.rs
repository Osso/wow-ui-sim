//! Removed members remain absent on ordinary and repeated namespace lookup.
#![cfg(feature = "client-retail")]

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
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
    local namespace, member = _G[entry[1]], entry[2]
    assert(rawget(namespace, member) == nil, entry[1] .. "." .. member)
    assert(namespace[member] == nil, entry[1] .. "." .. member)
    assert(namespace[member] == nil, entry[1] .. "." .. member)
end
"#;

#[test]
fn patch_9_1_0_unused_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
