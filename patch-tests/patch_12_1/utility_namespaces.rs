use super::{load_full_game_ui_with_all_lod, missing_runtime_functions};

const PTR_METHODS: &[&str] = &[
    "ChatFrameUtil.DiscordNameColorize",
    "ChatFrameUtil.FormatDiscordMessage",
    "ChatFrameUtil.GetNameForDiscordMessage",
    "CooldownViewerUtil.AddSoundAlertRadio",
    "CooldownViewerUtil.BuildSoundMenus",
    "CooldownViewerUtil.GetSoundTypeSoundKit",
    "CooldownViewerUtil.GetSoundTypeText",
    "HousingFramesUtil.IsBlueprintCollectionAvailable",
    "HousingFramesUtil.IsBlueprintOperationInProgress",
    "HousingFramesUtil.ShowBlueprintExport",
    "HousingFramesUtil.ShowBlueprintImport",
    "HousingFramesUtil.ShowBlueprintRoomExport",
    "HousingFramesUtil.TryOpenBlueprintCollection",
    "ItemUtil.DisplayEquipSlotTooltip",
    "ItemUtil.GetEmptyEquipSlotTooltipForSlotName",
    "ItemUtil.GetEmptyEquipSlotTooltip",
    "ItemUtil.GetEquipSlotTexture",
    "ItemUtil.GetValidatedItemLocation",
    "PingUtil.SendMacroPing",
    "PingUtil.TogglePingTarget",
    "RegionUtil.GetTopLeftMost",
    "RegionUtil.SortByTopLeft",
    "TimeUtil.BetterDate",
    "TimeUtil.GetRecentTimeDate",
    "UIModeUtil.CreateExtendedBlocklist",
    "UIModeUtil.CreateModifiedBlocklist",
    "UIModeUtil.IsModeActive",
    "UIModeUtil.RegisterMode",
    "UIModeUtil.SetModeActive",
];

/// Proves current PTR utility methods are available after their LoD owners load.
#[test]
fn ptr_utility_methods_are_published_by_the_loaded_owners() {
    let env = load_full_game_ui_with_all_lod();
    let missing = missing_runtime_functions(&env, PTR_METHODS);
    assert_eq!(
        missing, "",
        "pinned PTR utility methods must remain published"
    );
}

/// Proves the native-absent contextual ping member is not callable on PTR.
#[test]
fn contextual_ping_helpers_are_absent_on_ptr() {
    let env = load_full_game_ui_with_all_lod();
    let absent: bool = env
        .eval(
            r#"
            return C_Ping.GetContextualPingTypeForUnit == nil
                and (PingUtil == nil or PingUtil.GetContextualPingTypeForUnit == nil)
                and not pcall(function()
                    return C_Ping.GetContextualPingTypeForUnit("player")
                end)
            "#,
        )
        .expect("contextual ping absence probe succeeds");

    assert!(absent);
}
