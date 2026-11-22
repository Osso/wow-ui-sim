#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

type VoteFrameState = (
    bool,
    String,
    bool,
    String,
    String,
    String,
    String,
    bool,
    f64,
    f64,
    f64,
    String,
);

#[test]
fn instance_abandon_vote_frame_uses_supplied_vote_state() {
    with_blizzard_addon_startup_shape(&["Blizzard_FrameXML"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_FrameXML"));
        env.exec(
            r#"
            A_Admin.SetInstanceAbandonVoteState(30, 18, 60, 45, true, {true, false})
            A_Admin.FireEvent("INSTANCE_ABANDON_VOTE_STARTED")
            "#,
        )
        .expect("supplied vote state should reach the Blizzard consumer");

        let state = read_vote_frame_state(env);

        assert!(state.0, "vote frame should be shown");
        assert!(!state.1.is_empty(), "vote requirement text should be set");
        assert!(state.2, "local vote response should be shown");
        assert_eq!(state.3, state.11);
        assert_eq!(state.4, "ui-lfg-roleicon-generic");
        assert_eq!(state.5, "ui-lfg-roleicon-generic");
        assert_eq!(state.6, "UI-LFG-RoleIcon-Generic-Disabled");
        assert!(state.7, "vote timer popup should be shown");
        assert_eq!((state.8, state.9, state.10), (30.0, 18.0, 2.0));
    });
}

fn read_vote_frame_state(env: &WowLuaEnv) -> VoteFrameState {
    env.eval(
        r#"
                local textures = InstanceAbandonFrame.StatusFrame.textures
                return InstanceAbandonFrame:IsShown(),
                       InstanceAbandonFrame.VoteText:GetText(),
                       InstanceAbandonFrame.ResponseText:IsShown(),
                       InstanceAbandonFrame.ResponseText:GetText(),
                       textures[1]:GetAtlas(),
                       textures[2]:GetAtlas(),
                       textures[3]:GetAtlas(),
                       InstanceAbandonPopup:IsShown(),
                       InstanceAbandonPopup.progressBarDuration,
                       InstanceAbandonPopup.timeleft,
                       C_PartyInfo.GetNumInstanceAbandonGroupVoteResponses(),
                       VOTE_TO_ABANDON_VOTED_YES
                "#,
    )
    .expect("vote frame state should be observable")
}
