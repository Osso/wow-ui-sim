//! Frame-surface probes for `Blizzard_AdventureMap`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_AdventureMap";

// Wrath has no AdventureMap vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn adventure_map_frame_loads_as_map_canvas_panel() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let surface: AdventureMapFrameSurface = env
            .eval(
                r#"
                return type(AdventureMapFrame),
                       AdventureMapFrame and AdventureMapFrame:GetFrameStrata() or nil,
                       AdventureMapFrame and AdventureMapFrame:GetParent():GetName() or nil,
                       type(AdventureMapFrame and AdventureMapFrame.dataProviders),
                       type(AdventureMapFrame and AdventureMapFrame.pinFrameLevelsManager),
                       type(AdventureMapFrame and AdventureMapFrame.pinFrameLevelsManager and AdventureMapFrame.pinFrameLevelsManager.definitions)
                "#,
            )
            .expect("AdventureMapFrame surface probe must run cleanly");

        assert_adventure_map_frame_surface(surface);
    });
}
}

type AdventureMapFrameSurface = (String, String, String, String, String, String);

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn adventure_map_quest_choice_dialog_loads_with_mixin_and_children() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let surface: QuestChoiceDialogSurface = env
            .eval(
                r#"
                return type(AdventureMapQuestChoiceDialog),
                       AdventureMapQuestChoiceDialog.OnLoad == AdventureMapQuestChoiceDialogMixin.OnLoad,
                       AdventureMapQuestChoiceDialog.OnShow == AdventureMapQuestChoiceDialogMixin.OnShow,
                       type(AdventureMapQuestChoiceDialog.Portrait),
                       type(AdventureMapQuestChoiceDialog.Background),
                       type(AdventureMapQuestChoiceDialog.Details),
                       type(AdventureMapQuestChoiceDialog.Rewards),
                       type(AdventureMapQuestChoiceDialog.RewardsHeader),
                       type(AdventureMapQuestChoiceDialog.FadeIn)
                "#,
            )
            .expect("AdventureMapQuestChoiceDialog surface probe must run cleanly");

        assert_quest_choice_dialog_surface(surface);
    });
}
}

type QuestChoiceDialogSurface = (
    String,
    bool,
    bool,
    String,
    String,
    String,
    String,
    String,
    String,
);

fn assert_adventure_map_frame_surface(surface: AdventureMapFrameSurface) {
    let (
        frame_type,
        frame_strata,
        parent_name,
        data_providers_type,
        pin_frame_levels_manager_type,
        pin_frame_level_definitions_type,
    ) = surface;

    assert_eq!(frame_type, "table", "`AdventureMapFrame` must exist");
    assert_eq!(
        frame_strata, "DIALOG",
        "`AdventureMapFrame` must load in the DIALOG strata"
    );
    assert_eq!(
        parent_name, "UIParent",
        "`AdventureMapFrame` must be parented to UIParent"
    );
    assert_eq!(
        data_providers_type, "table",
        "`AdventureMapFrame` must expose MapCanvas `dataProviders` state"
    );
    assert_eq!(
        pin_frame_levels_manager_type, "table",
        "`AdventureMapFrame` must expose MapCanvas `pinFrameLevelsManager` state"
    );
    assert_eq!(
        pin_frame_level_definitions_type, "table",
        "`MapCanvasPinFrameLevelsManagerMixin:Initialize` must seed frame-level definitions"
    );
}

fn assert_quest_choice_dialog_surface(surface: QuestChoiceDialogSurface) {
    let (
        dialog_type,
        on_load_matches_mixin,
        on_show_matches_mixin,
        portrait_type,
        background_type,
        details_type,
        rewards_type,
        rewards_header_type,
        fade_in_type,
    ) = surface;

    assert_quest_choice_dialog_frame(dialog_type, on_load_matches_mixin, on_show_matches_mixin);
    assert_quest_choice_dialog_children([
        ("Portrait", portrait_type),
        ("Background", background_type),
        ("Details", details_type),
        ("Rewards", rewards_type),
        ("RewardsHeader", rewards_header_type),
        ("FadeIn", fade_in_type),
    ]);
}

fn assert_quest_choice_dialog_frame(
    dialog_type: String,
    on_load_matches_mixin: bool,
    on_show_matches_mixin: bool,
) {
    assert_eq!(
        dialog_type, "table",
        "`AdventureMapQuestChoiceDialog` must exist"
    );
    assert!(
        on_load_matches_mixin,
        "`AdventureMapQuestChoiceDialog` must copy `OnLoad` from its mixin"
    );
    assert!(
        on_show_matches_mixin,
        "`AdventureMapQuestChoiceDialog` must copy `OnShow` from its mixin"
    );
}

fn assert_quest_choice_dialog_children(child_types: [(&str, String); 6]) {
    for (child_name, child_type) in child_types {
        assert_eq!(
            child_type, "table",
            "`AdventureMapQuestChoiceDialog.{child_name}` must be exposed"
        );
    }
}
