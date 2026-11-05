//! Frame-surface probes for `Blizzard_AnimaDiversionUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::loader::BlizzardAddonOverride;

const ROOT: &str = "Blizzard_AnimaDiversionUI";
const IMPLICIT_DEPS: &[&str] = &["Blizzard_MapCanvas", "Blizzard_SharedMapDataProviders"];
const CLOSURE_OVERRIDES: &[BlizzardAddonOverride<'_>] = &[BlizzardAddonOverride {
    addon: ROOT,
    extra_roots: IMPLICIT_DEPS,
}];
const ANIMA_DIVERSION_FRAME_SURFACE_PROBE: &str = r#"
return type(AnimaDiversionFrame),
       AnimaDiversionFrame:GetObjectType(),
       AnimaDiversionFrame:GetParent() == UIParent,
       AnimaDiversionFrame:IsToplevel(),
       AnimaDiversionFrame:IsShown(),
       AnimaDiversionFrame.OnLoad == AnimaDiversionFrameMixin.OnLoad,
       AnimaDiversionFrame.OnShow == AnimaDiversionFrameMixin.OnShow,
       type(AnimaDiversionFrame.dataProviders),
       type(AnimaDiversionFrame.pinFrameLevelsManager),
       type(AnimaDiversionFrame.pinFrameLevelsManager and AnimaDiversionFrame.pinFrameLevelsManager.definitions)
"#;
const ANIMA_DIVERSION_FRAME_CHILDREN_PROBE: &str = r#"
local frame = AnimaDiversionFrame
local currencyFrame = frame.AnimaDiversionCurrencyFrame
local nestedCurrencyFrame = currencyFrame and currencyFrame.CurrencyFrame
local quantity = nestedCurrencyFrame and nestedCurrencyFrame.Quantity

return type(frame.NineSlice),
       type(frame.BorderFrame),
       type(frame.ScrollContainer),
       type(frame.CloseButton),
       type(currencyFrame),
       type(frame.ReinforceProgressFrame),
       type(frame.ReinforceInfoFrame),
       type(nestedCurrencyFrame),
       type(quantity),
       quantity and quantity:GetObjectType()
"#;
const REINFORCE_INFO_FRAME_CHILDREN_PROBE: &str = r#"
local frame = AnimaDiversionFrame.ReinforceInfoFrame
local title = frame and frame.Title

return type(frame.TitleShadow),
       type(title),
       title and title:GetObjectType(),
       title and title:GetText(),
       ANIMA_DIVERSION_REINFORCE_READY,
       type(frame.AnimaNodeReinforceButton),
       frame.AnimaNodeReinforceButton and frame.AnimaNodeReinforceButton:GetObjectType()
"#;

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_frame_loads_as_hidden_map_canvas_panel() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let surface: AnimaDiversionFrameSurface = env
            .eval(ANIMA_DIVERSION_FRAME_SURFACE_PROBE)
            .expect("AnimaDiversionFrame surface probe must run cleanly");

        assert_anima_diversion_frame_surface(surface);
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_frame_exposes_expected_children() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let surface: AnimaDiversionFrameChildren = env
            .eval(ANIMA_DIVERSION_FRAME_CHILDREN_PROBE)
            .expect("AnimaDiversionFrame child probe must run cleanly");

        assert_anima_diversion_frame_children(surface);
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn reinforce_info_frame_exposes_expected_children_and_ready_title() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let surface: ReinforceInfoFrameChildren = env
            .eval(REINFORCE_INFO_FRAME_CHILDREN_PROBE)
            .expect("ReinforceInfoFrame child probe must run cleanly");

        assert_reinforce_info_frame_children(surface);
    });
}

type AnimaDiversionFrameSurface = (
    String,
    String,
    bool,
    bool,
    bool,
    bool,
    bool,
    String,
    String,
    String,
);
type AnimaDiversionFrameChildren = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
);
type ReinforceInfoFrameChildren = (
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    String,
    Option<String>,
);

fn assert_anima_diversion_frame_surface(surface: AnimaDiversionFrameSurface) {
    let (
        frame_type,
        object_type,
        parent_is_ui_parent,
        is_toplevel,
        is_shown,
        on_load_matches_mixin,
        on_show_matches_mixin,
        data_providers_type,
        pin_frame_levels_manager_type,
        pin_frame_level_definitions_type,
    ) = surface;

    assert_frame_identity(frame_type, object_type, parent_is_ui_parent);
    assert_frame_state(is_toplevel, is_shown);
    assert_frame_mixin(on_load_matches_mixin, on_show_matches_mixin);
    assert_map_canvas_state(
        data_providers_type,
        pin_frame_levels_manager_type,
        pin_frame_level_definitions_type,
    );
}

fn assert_frame_identity(frame_type: String, object_type: String, parent_is_ui_parent: bool) {
    assert_eq!(frame_type, "table", "`AnimaDiversionFrame` must exist");
    assert_eq!(
        object_type, "Frame",
        "`AnimaDiversionFrame` must be a Frame object"
    );
    assert!(
        parent_is_ui_parent,
        "`AnimaDiversionFrame` must be parented to UIParent"
    );
}

fn assert_frame_state(is_toplevel: bool, is_shown: bool) {
    assert!(
        is_toplevel,
        "`AnimaDiversionFrame` must honor XML `toplevel=true`"
    );
    assert!(
        !is_shown,
        "`AnimaDiversionFrame` must be hidden immediately after load"
    );
}

fn assert_frame_mixin(on_load_matches_mixin: bool, on_show_matches_mixin: bool) {
    assert!(
        on_load_matches_mixin,
        "`AnimaDiversionFrame` must mix in `AnimaDiversionFrameMixin.OnLoad`"
    );
    assert!(
        on_show_matches_mixin,
        "`AnimaDiversionFrame` must mix in `AnimaDiversionFrameMixin.OnShow`"
    );
}

fn assert_map_canvas_state(
    data_providers_type: String,
    pin_frame_levels_manager_type: String,
    pin_frame_level_definitions_type: String,
) {
    assert_eq!(
        data_providers_type, "table",
        "`MapCanvasMixin.OnLoad` must initialize dataProviders"
    );
    assert_eq!(
        pin_frame_levels_manager_type, "table",
        "`MapCanvasMixin.OnLoad` must initialize pinFrameLevelsManager"
    );
    assert_eq!(
        pin_frame_level_definitions_type, "table",
        "`MapCanvasPinFrameLevelsManagerMixin:Initialize` must seed definitions"
    );
}

fn assert_anima_diversion_frame_children(children: AnimaDiversionFrameChildren) {
    let (
        nine_slice_type,
        border_frame_type,
        scroll_container_type,
        close_button_type,
        currency_frame_type,
        reinforce_progress_frame_type,
        reinforce_info_frame_type,
        nested_currency_frame_type,
        quantity_type,
        quantity_object_type,
    ) = children;

    assert_main_child_frames(
        nine_slice_type,
        border_frame_type,
        scroll_container_type,
        close_button_type,
        currency_frame_type,
        reinforce_progress_frame_type,
        reinforce_info_frame_type,
    );
    assert_currency_quantity(
        nested_currency_frame_type,
        quantity_type,
        quantity_object_type,
    );
}

fn assert_main_child_frames(
    nine_slice_type: String,
    border_frame_type: String,
    scroll_container_type: String,
    close_button_type: String,
    currency_frame_type: String,
    reinforce_progress_frame_type: String,
    reinforce_info_frame_type: String,
) {
    assert_child_frame("NineSlice", nine_slice_type);
    assert_child_frame("BorderFrame", border_frame_type);
    assert_child_frame("ScrollContainer", scroll_container_type);
    assert_child_frame("CloseButton", close_button_type);
    assert_child_frame("AnimaDiversionCurrencyFrame", currency_frame_type);
    assert_child_frame("ReinforceProgressFrame", reinforce_progress_frame_type);
    assert_child_frame("ReinforceInfoFrame", reinforce_info_frame_type);
}

fn assert_currency_quantity(
    nested_currency_frame_type: String,
    quantity_type: String,
    quantity_object_type: Option<String>,
) {
    assert_child_frame(
        "AnimaDiversionCurrencyFrame.CurrencyFrame",
        nested_currency_frame_type,
    );
    assert_child_frame(
        "AnimaDiversionCurrencyFrame.CurrencyFrame.Quantity",
        quantity_type,
    );
    assert_eq!(
        quantity_object_type.as_deref(),
        Some("FontString"),
        "`AnimaDiversionCurrencyFrame.CurrencyFrame.Quantity` must be a FontString"
    );
}

fn assert_child_frame(parent_key: &str, child_type: String) {
    assert_eq!(
        child_type, "table",
        "`AnimaDiversionFrame.{parent_key}` must exist as a frame child"
    );
}

fn assert_reinforce_info_frame_children(children: ReinforceInfoFrameChildren) {
    let (
        title_shadow_type,
        title_type,
        title_object_type,
        title_text,
        ready_text,
        reinforce_button_type,
        reinforce_button_object_type,
    ) = children;

    assert_reinforce_info_child_frames(
        title_shadow_type,
        title_type,
        title_object_type,
        reinforce_button_type,
        reinforce_button_object_type,
    );
    assert_reinforce_title_text(title_text, ready_text);
}

fn assert_reinforce_info_child_frames(
    title_shadow_type: String,
    title_type: String,
    title_object_type: Option<String>,
    reinforce_button_type: String,
    reinforce_button_object_type: Option<String>,
) {
    assert_child_frame("ReinforceInfoFrame.TitleShadow", title_shadow_type);
    assert_child_frame("ReinforceInfoFrame.Title", title_type);
    assert_child_frame(
        "ReinforceInfoFrame.AnimaNodeReinforceButton",
        reinforce_button_type,
    );
    assert_eq!(
        title_object_type.as_deref(),
        Some("FontString"),
        "`AnimaDiversionFrame.ReinforceInfoFrame.Title` must be a FontString"
    );
    assert_eq!(
        reinforce_button_object_type.as_deref(),
        Some("Button"),
        "`AnimaDiversionFrame.ReinforceInfoFrame.AnimaNodeReinforceButton` must be a Button"
    );
}

fn assert_reinforce_title_text(title_text: Option<String>, ready_text: String) {
    assert_eq!(
        title_text.as_deref(),
        Some(ready_text.as_str()),
        "`ReinforceInfoFrame.Title` must resolve `ANIMA_DIVERSION_REINFORCE_READY`"
    );
    assert_eq!(
        ready_text, "Select a location to Reinforce",
        "`ANIMA_DIVERSION_REINFORCE_READY` must match the en-US global string"
    );
}
