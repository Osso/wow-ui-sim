//! Frame-shaped surface pinned by `Blizzard_AccessibilityTemplates`.
//!
//! Three XML files in this addon publish entries into the global template
//! registries, and downstream addons rely on every entry resolving by
//! name:
//!
//! | Template name                  | Defining file (under `Blizzard_AccessibilityTemplates/`)  | Registry                  |
//! |--------------------------------|------------------------------------------------------------|---------------------------|
//! | `UIThemeContainerFrame`        | `AccessibilityIntrinsics.xml` (`<Frame intrinsic="true">`) | unified template registry |
//! | `UserScaledFrameTemplate`      | `UserScaledElementTemplates.xml` (`<Frame virtual="true">`)| unified template registry |
//! | `UserScaledFontStringTemplate` | `UserScaledElementTemplates.xml` (`<FontString virtual>`)  | font-string registry      |
//! | `UserScaledSliderTemplate`     | `UserScaledSliderTemplates.xml` (`<Frame virtual="true">`) | unified template registry |
//!
//! Why this matters: `UIPanelButtonUserScaledTemplate` (Blizzard_SharedXML),
//! `TextToSpeechCheckButtonTemplate` (Blizzard_ChatFrame) and the WhoFrame /
//! AddFriend dialog buttons (Blizzard_FriendsFrame) all inherit
//! `UserScaledFrameTemplate`, while `MovieFrame`, `InstanceAbandon` and
//! `WhoFrameTotals` rely on `UserScaledFontStringTemplate` for the
//! user-scale font binding. If any of these four entries fails to register,
//! the inherits-chain walker silently drops the parent and the children
//! lose their auto-scale wiring at runtime.
//!
//! `<FontString virtual>` templates live in a separate registry from
//! `<Frame virtual>` / `<Frame intrinsic>` (see `register_font_string_template`
//! in `src/xml/template.rs`) because the unified registry only stores
//! `FrameXml`. The test queries each template through the registry that
//! actually owns it.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccessibilityTemplates";

const FRAME_TEMPLATES: &[&str] = &[
    "UIThemeContainerFrame",
    "UserScaledFrameTemplate",
    "UserScaledSliderTemplate",
];

const FONT_STRING_TEMPLATE: &str = "UserScaledFontStringTemplate";

#[cfg(feature = "optional-tests")]
#[test]
fn accessibility_templates_registers_intrinsic_and_virtual_templates() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |_env, _loaded| {
        for template_name in FRAME_TEMPLATES {
            assert!(
                wow_ui_sim::xml::get_template(template_name).is_some(),
                "Frame template `{template_name}` MUST be present in the unified template \
                 registry after `{ROOT}` loads. If a regression drops the registration here, \
                 every downstream addon that inherits the missing name (e.g. \
                 `UIPanelButtonUserScaledTemplate`, `TextToSpeechCheckButtonTemplate`) will \
                 load with a broken inherit chain — verify the XML loader still calls \
                 `register_virtual_or_intrinsic` (preparation.rs:68) for both `<Frame virtual>` \
                 AND `<Frame intrinsic>`."
            );
        }
        assert!(
            wow_ui_sim::xml::get_font_string_template(FONT_STRING_TEMPLATE).is_some(),
            "FontString template `{FONT_STRING_TEMPLATE}` MUST be present in the font-string \
             template registry after `{ROOT}` loads. The `<FontString name=\"{FONT_STRING_TEMPLATE}\" \
             mixin=\"UserScaledElementMixin\" virtual=\"true\">` declaration in \
             `UserScaledElementTemplates.xml` is what `MovieFrame.Title`, `InstanceAbandon.VoteText`, \
             and `WhoFrameTotals` rely on for the user-scale font binding. If this assertion \
             regresses, verify `create_fontstring_from_xml` (xml_fontstring.rs) still calls \
             `register_font_string_template` before returning early on the virtual path."
        );
    });
}
