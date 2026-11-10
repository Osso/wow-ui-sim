use super::{load_full_game_ui_with_all_lod, missing_runtime_functions};

const PTR_METHODS: &[&str] = &[
    "NarrationUtil.CreateNarrationInfo",
    "NarrationUtil.GetCheckboxContext",
    "NarrationUtil.MakeIndexInfo",
    "NarrationUtil.MakeNarrationStringForMoney",
    "NarrationUtil.MakeNarrationStringFromIndexInfo",
    "NarrationUtil.MakeNarrationStringFromInfo",
    "NarrationUtil.MakeNarrationString",
    "NarrationUtil.NarrateCurrentScreen",
    "NarrationUtil.RegionToNarrationInfo",
    "NarrationUtil.ResolveForwardedRegion",
    "NarrationUtil.SetStaticDescription",
    "NarrationUtil.SetStaticName",
    "NarrationUtil.ShouldBeEnabled",
    "NarrationUtil.ShouldRegionNavigationSkipTooltips",
];

/// Proves the pinned PTR NarrationUtil methods are published after their LoD owners load.
#[test]
fn ptr_narration_methods_are_published_by_the_loaded_owner() {
    let env = load_full_game_ui_with_all_lod();
    let missing = missing_runtime_functions(&env, PTR_METHODS);
    assert_eq!(missing, "", "pinned PTR methods must remain published");
}
