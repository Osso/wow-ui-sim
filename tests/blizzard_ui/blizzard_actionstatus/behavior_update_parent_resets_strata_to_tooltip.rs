//! `UpdateParent` strata-reset behavior for `Blizzard_ActionStatus`.

#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary",
    feature = "client-wowforever"
))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ActionStatus";

timerunning_util_panel_fixture! {
#[test]
fn update_parent_resets_frame_strata_to_tooltip() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let (strata_after_mutation, strata_after_update): (String, String) = env
            .eval(
                r#"
                local parent = CreateFrame("Frame", "ActionStatusStrataParentProbe", UIParent)
                parent:Show()

                GetAppropriateTopLevelParent = function()
                    return parent
                end

                ActionStatus:SetFrameStrata("LOW")
                local strataAfterMutation = ActionStatus:GetFrameStrata()

                ActionStatus:UpdateParent()

                return strataAfterMutation, ActionStatus:GetFrameStrata()
                "#,
            )
            .expect("ActionStatus strata-reset behavior probe must run cleanly");

        assert_eq!(
            strata_after_mutation, "LOW",
            "test setup must lower ActionStatus frame strata before UpdateParent()"
        );
        assert_eq!(
            strata_after_update, "TOOLTIP",
            "`UpdateParent()` must reset ActionStatus frame strata to TOOLTIP"
        );
    });
}
}
