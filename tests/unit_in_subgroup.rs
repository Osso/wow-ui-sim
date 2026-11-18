#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

#[test]
fn unit_position_frame_uses_raid_appearance_without_subgroup_roster() {
    with_blizzard_addon_closure(&["Blizzard_UIPanels_Game"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
        {
            let mut state = env.state().borrow_mut();
            state.party_group_active = true;
            state.party_members.truncate(1);
        }
        env.exec(
            r#"
            local frame = {}
            Mixin(frame, UnitPositionFrameMixin, UnitPositionFrameUpdateSecureMixin)
            frame.needsFullUpdate = true
            frame.capturedTexture = nil
            function frame:ClearUnits() end
            function frame:AddUnit(_unit, texture) self.capturedTexture = texture end
            function frame:FinalizeUnits() end
            frame:SetPinTexture("party", "party-subgroup")
            frame:SetPinTexture("raid", "other-subgroup")
            frame:UpdateFull(0)
            assert(frame.capturedTexture == "other-subgroup",
                "the original unit-position consumer must use raid appearance when no subgroup owner exists")
            "#,
        )
        .expect("original UnitPositionFrameTemplates consumer must classify the party unit");
    });
}
