//! Frame-shape probes for `Blizzard_ActionBarController`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ActionBarController";

#[cfg(feature = "optional-tests")]
#[test]
fn action_bar_controller_frame_is_parented_to_uiparent_and_wires_scripts() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let (exists, parent_name, has_on_load, has_on_event): (bool, String, bool, bool) = env
            .eval(
                r#"
                local frame = ActionBarController
                local parent = frame and frame:GetParent()
                return frame ~= nil,
                    parent and parent:GetName() or "<nil>",
                    type(frame and frame:GetScript("OnLoad")) == "function",
                    type(frame and frame:GetScript("OnEvent")) == "function"
                "#,
            )
            .expect("ActionBarController frame probe must run cleanly");

        assert!(
            exists,
            "ActionBarController frame must exist after XML load"
        );
        assert_eq!(
            parent_name, "UIParent",
            "ActionBarController XML declares parent=\"UIParent\""
        );
        assert!(
            has_on_load,
            "ActionBarController XML must wire the OnLoad script handler"
        );
        assert!(
            has_on_event,
            "ActionBarController XML must wire the OnEvent script handler"
        );
    });
}
