//! Frame surface for `Blizzard_AccountSaveUI.xml`.
//!
//! Source (`Interface/BlizzardUI/Blizzard_AccountSaveUI/
//! Blizzard_AccountSaveUI.xml`, lines 4-83):
//!
//! ```xml
//! <Frame name="AccountSaveFrame" mixin="AccountSaveFrameMixin"
//!        parent="CharacterSelect" frameStrata="DIALOG" toplevel="true"
//!        enableMouse="true">
//!     <Frames>
//!         <Frame      parentKey="Border"        inherits="DialogBorderOpaqueTemplate"/>
//!         <Frame      parentKey="ContentInsets" .../>
//!         <Frame      parentKey="AlertIcon"     .../>
//!         <SimpleHTML parentKey="Text"          .../>
//!         <EditBox    parentKey="LockEditBox"   letters="32" .../>
//!         <Button     parentKey="SaveButton"    inherits="UIPanelButtonTemplate" .../>
//!     </Frames>
//! ```
//!
//! The frame is parsed during XML load — these assertions catch
//! regressions in the XML loader (parent resolution, frameStrata
//! propagation, parentKey wiring) without exercising any
//! `AccountSaveFrameMixin` method. Method behavior is pinned by the
//! dedicated behavior fixtures.

use crate::common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape;

const ROOT: &str = "Blizzard_AccountSaveUI";
const PARENT_KEYS: &[&str] = &[
    "Border",
    "ContentInsets",
    "AlertIcon",
    "Text",
    "LockEditBox",
    "SaveButton",
];

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_frame_parent_and_strata() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let (frame_type, parent_name, strata) = env
            .eval::<(String, String, String)>(
                r#"
                assert(AccountSaveFrame, "AccountSaveFrame global must exist after XML load")
                local parent = AccountSaveFrame:GetParent()
                return type(AccountSaveFrame),
                       parent and parent:GetName() or "<nil>",
                       AccountSaveFrame:GetFrameStrata()
                "#,
            )
            .expect("AccountSaveFrame must expose GetParent / GetFrameStrata after XML load");
        assert_eq!(
            frame_type, "table",
            "AccountSaveFrame must register as a table-typed handle (FrameRef reports as \
             `table` via __metatable). Got type = `{frame_type}`."
        );
        assert_eq!(
            parent_name, "CharacterSelect",
            "AccountSaveFrame's `parent=\"CharacterSelect\"` XML attribute must resolve \
             to the CharacterSelect frame at parse time. If this regresses, either the \
             XML attribute parser dropped the parent reference or CharacterSelect failed \
             to materialise from Blizzard_GlueXML's closure. Got parent: `{parent_name}`."
        );
        assert_eq!(
            strata, "DIALOG",
            "AccountSaveFrame's `frameStrata=\"DIALOG\"` XML attribute must propagate to \
             the runtime strata string returned by GetFrameStrata. Got: `{strata}`."
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_frame_exposes_all_parent_keys() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for key in PARENT_KEYS {
            let probe = format!("return type(AccountSaveFrame.{key})");
            let kind = env
                .eval::<String>(&probe)
                .unwrap_or_else(|err| panic!("AccountSaveFrame.{key} probe raised: {err}"));
            assert_eq!(
                kind, "table",
                "AccountSaveFrame.{key} must be wired as a child frame via parentKey. \
                 The XML declares `<... parentKey=\"{key}\">` inside <Frames>; if this \
                 regresses, either the parentKey attribute was dropped during XML parse \
                 or the child element type stopped being recognised by the loader. Got \
                 type = `{kind}` for key `{key}`."
            );
        }
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_frame_text_is_a_simple_html_widget() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let object_type = env
            .eval::<String>("return AccountSaveFrame.Text:GetObjectType()")
            .expect("AccountSaveFrame.Text must support GetObjectType");
        assert_eq!(
            object_type, "SimpleHTML",
            "AccountSaveFrame.Text must be a SimpleHTML widget — XML declares it as \
             `<SimpleHTML parentKey=\"Text\" resizeToFitContents=\"true\">` (lines 28-40 \
             of Blizzard_AccountSaveUI.xml). The widget receives `<FontString>`, \
             `<FontStringHeader1>` and `<FontStringHeader2>` children that the kick-flow \
             dialog body relies on; if this drops to a plain Frame, the rich-text dialog \
             body would render as a single un-styled run. Got: `{object_type}`."
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_frame_lock_edit_box_caps_at_32_letters() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let (object_type, max_letters) = env
            .eval::<(String, i64)>(
                "return AccountSaveFrame.LockEditBox:GetObjectType(),
                        AccountSaveFrame.LockEditBox:GetMaxLetters()",
            )
            .expect("AccountSaveFrame.LockEditBox must support GetObjectType and GetMaxLetters");
        assert_eq!(
            object_type, "EditBox",
            "AccountSaveFrame.LockEditBox must be an EditBox widget — XML declares \
             `<EditBox parentKey=\"LockEditBox\" letters=\"32\" historyLines=\"1\">` \
             (lines 41-65). Got: `{object_type}`."
        );
        assert_eq!(
            max_letters, 32,
            "AccountSaveFrame.LockEditBox must cap input at 32 letters — the \
             `letters=\"32\"` XML attribute maps to SetMaxLetters(32). The kick-flow \
             confirmation phrase is exactly 32 characters; reducing or removing this \
             cap would let the player type extra characters and silently fail the \
             confirmation match. Got: max_letters = {max_letters}."
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_frame_save_button_inherits_ui_panel_button_template() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let template_registered = wow_ui_sim::xml::get_template("UIPanelButtonTemplate").is_some();
        assert!(
            template_registered,
            "UIPanelButtonTemplate must be present in the unified template registry \
             before AccountSaveFrame's <Button parentKey=\"SaveButton\" \
             inherits=\"UIPanelButtonTemplate\" ...> entry can resolve its inherits \
             chain. If this regresses, either Blizzard_SharedXML failed to load \
             alongside Blizzard_GlueXML or the inheritance walker stopped registering \
             SecureUIPanelTemplates.xml's virtual buttons."
        );

        let (object_type, has_left, has_right, has_middle) = env
            .eval::<(String, bool, bool, bool)>(
                "return AccountSaveFrame.SaveButton:GetObjectType(),
                        AccountSaveFrame.SaveButton.Left   ~= nil,
                        AccountSaveFrame.SaveButton.Right  ~= nil,
                        AccountSaveFrame.SaveButton.Middle ~= nil",
            )
            .expect(
                "AccountSaveFrame.SaveButton must expose GetObjectType plus Left / Right / \
                 Middle parentKey children inherited from UIPanelButtonTemplate",
            );
        assert_eq!(
            object_type, "Button",
            "AccountSaveFrame.SaveButton must be a Button widget — XML declares \
             `<Button parentKey=\"SaveButton\" ... inherits=\"UIPanelButtonTemplate\">`. \
             Got: `{object_type}`."
        );
        assert!(
            has_left && has_right && has_middle,
            "AccountSaveFrame.SaveButton must expose `Left`, `Right`, `Middle` \
             child textures inherited from UIPanelButtonTemplate → \
             UIPanelButtonNoTooltipTemplate (SecureUIPanelTemplates.xml lines 210-242). \
             These three textures are the only template-specific surface attached \
             to every UIPanelButtonTemplate instance — if any goes missing, the \
             button's inheritance chain stopped being walked for THIS instance. \
             Got: Left={has_left}, Right={has_right}, Middle={has_middle}."
        );
    });
}
