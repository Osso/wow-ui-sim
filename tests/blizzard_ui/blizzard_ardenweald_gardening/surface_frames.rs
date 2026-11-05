//! Frame factory surface for `Blizzard_ArdenwealdGardening`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ArdenwealdGardening";

#[cfg(feature = "optional-tests")]
#[test]
fn ardenweald_gardening_create_returns_panel_with_template_children() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let surface: FrameSurfaceProbe = env
            .eval(
                r#"
                local parent = CreateFrame("Frame", "ArdenwealdGardeningTestParent", UIParent)
                local panel = ArdenwealdGardening.Create(parent)

                return panel:GetObjectType(),
                       panel:GetParent() == parent,
                       panel:GetWidth(),
                       panel:GetHeight(),
                       panel.Background:GetObjectType(),
                       panel.Label:GetObjectType(),
                       type(panel.Button),
                       ArdenwealdGardeningButtonTemplate:GetObjectType(),
                       ArdenwealdGardeningButtonTemplate:GetParent() == panel,
                       panel.Label:GetText(),
                       GARDENWEALD_LANDING_HEADER
                "#,
            )
            .expect("Ardenweald Gardening frame surface probe must run cleanly");

        assert_frame_surface(surface);
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn ardenweald_gardening_button_template_exposes_texture_children_and_scripts() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let surface: ButtonSurfaceProbe = env
            .eval(
                r#"
                local parent = CreateFrame("Frame", "ArdenwealdGardeningButtonParent", UIParent)
                local panel = ArdenwealdGardening.Create(parent)
                local button = ArdenwealdGardeningButtonTemplate
                local mask = button.Icon:GetMaskTexture(1)
                button.Highlight:Hide()
                button.Icon2:Hide()
                button:GetScript("OnEnter")(button)
                local onEnterShowsHoverArt = button.Highlight:IsShown() and button.Icon2:IsShown()
                button:GetScript("OnLeave")(button)
                local onLeaveHidesHoverArt = not button.Highlight:IsShown() and not button.Icon2:IsShown()

                return button:GetObjectType(),
                       button:GetParent() == panel,
                       button.Icon:GetObjectType(),
                       button.Icon2:GetObjectType(),
                       button.Border:GetObjectType(),
                       button.Highlight:GetObjectType(),
                       type(button.Mask),
                       button.Icon:GetNumMaskTextures(),
                       button.Icon2:GetNumMaskTextures(),
                       mask and mask:GetObjectType() or "nil",
                       mask and mask:GetAtlas() or "nil",
                       type(button:GetScript("OnEnter")),
                       type(button:GetScript("OnLeave")),
                       onEnterShowsHoverArt,
                       onLeaveHidesHoverArt,
                       button.OnEnter == ArdenwealdGardeningButtonMixin.OnEnter,
                       button.OnLeave == ArdenwealdGardeningButtonMixin.OnLeave
                "#,
            )
            .expect("Ardenweald Gardening button surface probe must run cleanly");

        assert_button_surface(surface);
    });
}

type FrameSurfaceProbe = (
    String,
    bool,
    f64,
    f64,
    String,
    String,
    String,
    String,
    bool,
    String,
    String,
);

type ButtonSurfaceProbe = (
    String,
    bool,
    String,
    String,
    String,
    String,
    String,
    f64,
    f64,
    String,
    String,
    String,
    String,
    bool,
    bool,
    bool,
    bool,
);

fn assert_frame_surface(surface: FrameSurfaceProbe) {
    let (
        panel_type,
        parent_matches,
        panel_width,
        panel_height,
        background_type,
        label_type,
        panel_button_type,
        named_button_type,
        named_button_parent_matches,
        label_text,
        expected_label_text,
    ) = surface;

    assert_panel_shape(panel_type, parent_matches, panel_width, panel_height);
    assert_panel_children(
        background_type,
        label_type,
        panel_button_type,
        named_button_type,
        named_button_parent_matches,
    );
    assert_eq!(
        label_text, expected_label_text,
        "`panel.Label` text must resolve from `GARDENWEALD_LANDING_HEADER`"
    );
}

fn assert_panel_shape(
    panel_type: String,
    parent_matches: bool,
    panel_width: f64,
    panel_height: f64,
) {
    assert_eq!(
        panel_type, "Frame",
        "`ArdenwealdGardening.Create(parent)` must return a Frame"
    );
    assert!(
        parent_matches,
        "created Ardenweald gardening panel must be parented to the supplied parent"
    );
    assert_eq!(
        panel_width, 350.0,
        "`ArdenwealdGardeningPanelTemplate` width must be applied"
    );
    assert_eq!(
        panel_height, 200.0,
        "`ArdenwealdGardeningPanelTemplate` height must be applied"
    );
}

fn assert_panel_children(
    background_type: String,
    label_type: String,
    panel_button_type: String,
    named_button_type: String,
    named_button_parent_matches: bool,
) {
    assert_eq!(
        background_type, "Texture",
        "panel must expose `Background` texture by parentKey"
    );
    assert_eq!(
        label_type, "FontString",
        "panel must expose `Label` font string by parentKey"
    );
    assert_eq!(
        panel_button_type, "nil",
        "`ArdenwealdGardeningPanelTemplate` does not declare a `Button` parentKey"
    );
    assert_eq!(
        named_button_type, "Button",
        "`ArdenwealdGardeningButtonTemplate` named child must be instantiated"
    );
    assert!(
        named_button_parent_matches,
        "`ArdenwealdGardeningButtonTemplate` named child must be parented to the created panel"
    );
}

fn assert_button_surface(surface: ButtonSurfaceProbe) {
    assert_button_texture_children(surface.2, surface.3, surface.4, surface.5);
    assert_button_mask(surface.6, surface.7, surface.8, surface.9, surface.10);
    assert_button_scripts(
        surface.11, surface.12, surface.13, surface.14, surface.15, surface.16,
    );
    assert_button_identity(surface.0, surface.1);
}

fn assert_button_identity(button_type: String, parent_matches: bool) {
    assert_eq!(
        button_type, "Button",
        "`ArdenwealdGardeningButtonTemplate` child must be a Button"
    );
    assert!(
        parent_matches,
        "`ArdenwealdGardeningButtonTemplate` child must be parented to the created panel"
    );
}

fn assert_button_texture_children(
    icon_type: String,
    icon2_type: String,
    border_type: String,
    highlight_type: String,
) {
    assert_eq!(icon_type, "Texture", "button must expose `Icon` texture");
    assert_eq!(icon2_type, "Texture", "button must expose `Icon2` texture");
    assert_eq!(
        border_type, "Texture",
        "button must expose `Border` texture"
    );
    assert_eq!(
        highlight_type, "Texture",
        "button must expose `Highlight` texture"
    );
}

fn assert_button_mask(
    mask_field_type: String,
    icon_mask_count: f64,
    icon2_mask_count: f64,
    mask_type: String,
    mask_atlas: String,
) {
    assert_eq!(
        mask_field_type, "nil",
        "the XML MaskTexture has no `parentKey`, so `button.Mask` must stay nil"
    );
    assert_eq!(
        icon_mask_count, 1.0,
        "`Icon` must have the CircleMaskScalable mask attached"
    );
    assert_eq!(
        icon2_mask_count, 1.0,
        "`Icon2` must have the CircleMaskScalable mask attached"
    );
    assert_eq!(
        mask_type, "MaskTexture",
        "attached mask must be a MaskTexture"
    );
    assert_eq!(
        mask_atlas, "CircleMaskScalable",
        "attached mask must use the CircleMaskScalable atlas"
    );
}

fn assert_button_scripts(
    on_enter_script_type: String,
    on_leave_script_type: String,
    on_enter_shows_hover_art: bool,
    on_leave_hides_hover_art: bool,
    on_enter_method_matches_mixin: bool,
    on_leave_method_matches_mixin: bool,
) {
    assert_eq!(
        on_enter_script_type, "function",
        "OnEnter script must exist"
    );
    assert_eq!(
        on_leave_script_type, "function",
        "OnLeave script must exist"
    );
    assert!(
        on_enter_shows_hover_art,
        "OnEnter script must dispatch through the mixin path and show hover art"
    );
    assert!(
        on_leave_hides_hover_art,
        "OnLeave script must dispatch through the mixin path and hide hover art"
    );
    assert!(
        on_enter_method_matches_mixin,
        "button.OnEnter must come from ArdenwealdGardeningButtonMixin"
    );
    assert!(
        on_leave_method_matches_mixin,
        "button.OnLeave must come from ArdenwealdGardeningButtonMixin"
    );
}
