//! Global popup surface for `Blizzard_AnimaDiversionUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::loader::BlizzardAddonOverride;

const ROOT: &str = "Blizzard_AnimaDiversionUI";
const IMPLICIT_DEPS: &[&str] = &["Blizzard_MapCanvas", "Blizzard_SharedMapDataProviders"];
const CLOSURE_OVERRIDES: &[BlizzardAddonOverride<'_>] = &[BlizzardAddonOverride {
    addon: ROOT,
    extra_roots: IMPLICIT_DEPS,
}];
const POPUP_SURFACE_PROBE: &str = r#"
local channel = StaticPopupDialogs["ANIMA_DIVERSION_CONFIRM_CHANNEL"]
local reinforce = StaticPopupDialogs["ANIMA_DIVERSION_CONFIRM_REINFORCE"]

local function popupSurface(dialog)
    return type(dialog) == "table",
           type(dialog and dialog.text),
           dialog and dialog.button1 == YES,
           dialog and dialog.button2 == CANCEL,
           type(dialog and dialog.OnAccept),
           type(dialog and dialog.OnShow),
           type(dialog and dialog.OnHide),
           dialog and dialog.hideOnEscape == 1
end

local channelIsTable,
      channelTextType,
      channelButton1IsYes,
      channelButton2IsCancel,
      channelOnAcceptType,
      channelOnShowType,
      channelOnHideType,
      channelHideOnEscape = popupSurface(channel)
local reinforceIsTable,
      reinforceTextType,
      reinforceButton1IsYes,
      reinforceButton2IsCancel,
      reinforceOnAcceptType,
      reinforceOnShowType,
      reinforceOnHideType,
      reinforceHideOnEscape = popupSurface(reinforce)

return channelIsTable,
       channelTextType,
       channelButton1IsYes,
       channelButton2IsCancel,
       channelOnAcceptType,
       channelOnShowType,
       channelOnHideType,
       channelHideOnEscape,
       type(channel and channel.GetExpirationText),
       reinforceIsTable,
       reinforceTextType,
       reinforceButton1IsYes,
       reinforceButton2IsCancel,
       reinforceOnAcceptType,
       reinforceOnShowType,
       reinforceOnHideType,
       reinforceHideOnEscape,
       type(reinforce and reinforce.GetExpirationText)
"#;
const UTIL_SURFACE_PROBE: &str = r#"
local state = Enum.AnimaDiversionNodeState
local originalGetNodes = C_AnimaDiversion.GetAnimaDiversionNodes

local function isAnyActive(nodes)
    C_AnimaDiversion.GetAnimaDiversionNodes = function()
        return nodes
    end
    return AnimaDiversionUtil.IsAnyNodeActive()
end

local unavailableActive = AnimaDiversionUtil.IsNodeActive(state.Unavailable)
local availableActive = AnimaDiversionUtil.IsNodeActive(state.Available)
local temporaryActive = AnimaDiversionUtil.IsNodeActive(state.SelectedTemporary)
local permanentActive = AnimaDiversionUtil.IsNodeActive(state.SelectedPermanent)
local cooldownActive = AnimaDiversionUtil.IsNodeActive(state.Cooldown)
local anyWithNoNodes = isAnyActive(nil)
local anyWithInactiveNodes = isAnyActive({
    { state = state.Unavailable },
    { state = state.Available },
    { state = state.Cooldown },
})
local anyWithTemporaryNode = isAnyActive({ { state = state.SelectedTemporary } })
local anyWithPermanentNode = isAnyActive({ { state = state.SelectedPermanent } })

C_AnimaDiversion.GetAnimaDiversionNodes = originalGetNodes

return type(AnimaDiversionUtil.IsNodeActive),
       type(AnimaDiversionUtil.IsAnyNodeActive),
       unavailableActive,
       availableActive,
       temporaryActive,
       permanentActive,
       cooldownActive,
       anyWithNoNodes,
       anyWithInactiveNodes,
       anyWithTemporaryNode,
       anyWithPermanentNode
"#;
const NODE_STATE_ENUM_PROBE: &str = r#"
local state = Enum.AnimaDiversionNodeState
return type(state),
       state.Unavailable,
       state.Available,
       state.SelectedTemporary,
       state.SelectedPermanent,
       state.Cooldown
"#;

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_ui_registers_static_popup_globals() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let surface: PopupSurface = env
            .eval(POPUP_SURFACE_PROBE)
            .expect("AnimaDiversionUI static popup surface probe must run cleanly");

        assert_popup_surface(surface);
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_util_predicates_match_node_state_semantics() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let surface: UtilSurface = env
            .eval(UTIL_SURFACE_PROBE)
            .expect("AnimaDiversionUtil predicate probe must run cleanly");

        assert_util_surface(surface);
    });
}

}

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_node_state_enum_exposes_published_values() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let surface: NodeStateEnumSurface = env
            .eval(NODE_STATE_ENUM_PROBE)
            .expect("AnimaDiversionNodeState enum surface probe must run cleanly");

        assert_node_state_enum_surface(surface);
    });
}

type PopupSurface = (
    bool,
    String,
    bool,
    bool,
    String,
    String,
    String,
    bool,
    String,
    bool,
    String,
    bool,
    bool,
    String,
    String,
    String,
    bool,
    String,
);
type PopupFields = (bool, String, bool, bool, String, String, String, bool);
type UtilSurface = (
    String,
    String,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
);
type NodeStatePredicates = (bool, bool, bool, bool, bool);
type AnyNodePredicates = (bool, bool, bool, bool);
type NodeStateEnumSurface = (String, i64, i64, i64, i64, i64);

fn assert_popup_surface(surface: PopupSurface) {
    assert_common_popup_fields("ANIMA_DIVERSION_CONFIRM_CHANNEL", channel_fields(&surface));
    assert_eq!(
        surface.8, "function",
        "channel popup must expose `GetExpirationText`"
    );

    assert_common_popup_fields(
        "ANIMA_DIVERSION_CONFIRM_REINFORCE",
        reinforce_fields(&surface),
    );
    assert_eq!(
        surface.17, "nil",
        "reinforce popup must not expose `GetExpirationText`"
    );
}

fn channel_fields(surface: &PopupSurface) -> PopupFields {
    (
        surface.0,
        surface.1.clone(),
        surface.2,
        surface.3,
        surface.4.clone(),
        surface.5.clone(),
        surface.6.clone(),
        surface.7,
    )
}

fn reinforce_fields(surface: &PopupSurface) -> PopupFields {
    (
        surface.9,
        surface.10.clone(),
        surface.11,
        surface.12,
        surface.13.clone(),
        surface.14.clone(),
        surface.15.clone(),
        surface.16,
    )
}

fn assert_common_popup_fields(name: &str, fields: PopupFields) {
    let (
        is_table,
        text_type,
        button1_is_yes,
        button2_is_cancel,
        on_accept_type,
        on_show_type,
        on_hide_type,
        hide_on_escape,
    ) = fields;

    assert!(
        is_table,
        "`{name}` must be registered as a StaticPopup table"
    );
    assert_eq!(text_type, "string", "`{name}` must define popup text");
    assert!(
        button1_is_yes,
        "`{name}` must use YES as the primary button"
    );
    assert!(
        button2_is_cancel,
        "`{name}` must use CANCEL as the secondary button"
    );
    assert_eq!(on_accept_type, "function", "`{name}` must define OnAccept");
    assert_eq!(on_show_type, "function", "`{name}` must define OnShow");
    assert_eq!(on_hide_type, "function", "`{name}` must define OnHide");
    assert!(
        hide_on_escape,
        "`{name}` must opt into hide-on-escape behavior"
    );
}

fn assert_util_surface(surface: UtilSurface) {
    assert_eq!(
        surface.0, "function",
        "`AnimaDiversionUtil.IsNodeActive` must be a function"
    );
    assert_eq!(
        surface.1, "function",
        "`AnimaDiversionUtil.IsAnyNodeActive` must be a function"
    );
    assert_node_state_predicates(node_state_predicates(&surface));
    assert_any_node_predicates(any_node_predicates(&surface));
}

fn node_state_predicates(surface: &UtilSurface) -> NodeStatePredicates {
    (surface.2, surface.3, surface.4, surface.5, surface.6)
}

fn any_node_predicates(surface: &UtilSurface) -> AnyNodePredicates {
    (surface.7, surface.8, surface.9, surface.10)
}

fn assert_node_state_predicates(predicates: NodeStatePredicates) {
    let (unavailable_active, available_active, temporary_active, permanent_active, cooldown_active) =
        predicates;

    assert!(
        !unavailable_active,
        "Unavailable nodes must not be considered active"
    );
    assert!(
        !available_active,
        "Available nodes must not be considered active"
    );
    assert!(
        temporary_active,
        "SelectedTemporary nodes must be considered active"
    );
    assert!(
        permanent_active,
        "SelectedPermanent nodes must be considered active"
    );
    assert!(
        !cooldown_active,
        "Cooldown nodes must not be considered active"
    );
}

fn assert_any_node_predicates(predicates: AnyNodePredicates) {
    let (
        any_with_no_nodes,
        any_with_inactive_nodes,
        any_with_temporary_node,
        any_with_permanent_node,
    ) = predicates;

    assert!(
        !any_with_no_nodes,
        "`IsAnyNodeActive` must handle nil nodes"
    );
    assert!(
        !any_with_inactive_nodes,
        "`IsAnyNodeActive` must reject all-inactive node lists"
    );
    assert!(
        any_with_temporary_node,
        "`IsAnyNodeActive` must accept a temporary selected node"
    );
    assert!(
        any_with_permanent_node,
        "`IsAnyNodeActive` must accept a permanent selected node"
    );
}

fn assert_node_state_enum_surface(surface: NodeStateEnumSurface) {
    let (enum_type, unavailable, available, selected_temporary, selected_permanent, cooldown) =
        surface;

    assert_eq!(
        enum_type, "table",
        "`Enum.AnimaDiversionNodeState` must be a table"
    );
    assert_eq!(unavailable, 0, "`Unavailable` enum value changed");
    assert_eq!(available, 1, "`Available` enum value changed");
    assert_eq!(
        selected_temporary, 2,
        "`SelectedTemporary` enum value changed"
    );
    assert_eq!(
        selected_permanent, 3,
        "`SelectedPermanent` enum value changed"
    );
    assert_eq!(cooldown, 4, "`Cooldown` enum value changed");
}
