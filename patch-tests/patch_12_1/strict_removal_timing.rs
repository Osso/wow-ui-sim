use super::new_game_env;

/// Proves the PTR startup surface has no invented phase-retirement names.
#[test]
fn ptr_retirement_surfaces_match_native_absence() {
    let env = new_game_env();
    let surfaces_match: bool = env
        .eval(
            r#"
            return GetInventorySlotInfo == nil
                and type(C_PaperDollInfo.GetInventorySlotInfo) == "function"
                and C_Housing.IsInsideOwnHouse == nil
                and Enum.EditModeUnitFrameSetting.IconSize == nil
                and Enum.EditModeUnitFrameSetting.DebuffIconSize == 19
                and Enum.EditModeUnitFrameSetting.BigDefensiveIconSize == 21
                and Enum.EditModeUnitFrameSetting.BuffIconSize ~= nil
                and GetCVar("lastLockedDelvesCompanionAbilities") == nil
                and GetCVar("SlugSupersampling") == nil
            "#,
        )
        .expect("PTR native retirement-name probe succeeds");

    assert!(
        surfaces_match,
        "PTR uses the C_PaperDollInfo inventory namespace, omits the old enum member and aliases, and keeps removed CVars absent"
    );
}
