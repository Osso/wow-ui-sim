#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::c_api::c_talking_head::{TalkingHeadAnimationInfo, TalkingHeadLine};
use wow_ui_sim::lua_api::WowLuaEnv;

fn sample_line() -> TalkingHeadLine {
    TalkingHeadLine {
        display_info: 123.0,
        camera_id: 456.0,
        voiceover: Some(789.0),
        duration: 2.5,
        line_number: 3.0,
        number_of_lines: 4.0,
        name: "Speaker".into(),
        text: "Hello.".into(),
        is_new: true,
        texture_kit: Some("TalkingHeads-Neutral".into()),
        animation: TalkingHeadAnimationInfo {
            animation_kit: Some(10.0),
            intro_animation: Some(11.0),
            loop_animation: Some(12.0),
            line_duration: Some(2.5),
        },
    }
}

const API_CONTRACT: &str = r##"
    assert(not C_TalkingHead.GetConversationsDeferred())
    C_TalkingHead.SetConversationsDeferred(true)
    assert(C_TalkingHead.GetConversationsDeferred())
    C_TalkingHead.IgnoreCurrentTalkingHead()
    assert(C_TalkingHead.IsCurrentTalkingHeadIgnored())
    local d, c, vo, duration, line, count, name, text, isNew, kit = C_TalkingHead.GetCurrentLineInfo()
    assert(select("#", C_TalkingHead.GetCurrentLineInfo()) == 10)
    assert(d == 123 and c == 456 and vo == 789 and duration == 2.5)
    assert(line == 3 and count == 4 and name == "Speaker" and text == "Hello.")
    assert(isNew and kit == "TalkingHeads-Neutral")
    local kitID, intro, loop, lineDuration = C_TalkingHead.GetCurrentLineAnimationInfo()
    assert(select("#", C_TalkingHead.GetCurrentLineAnimationInfo()) == 4)
    assert(kitID == 10 and intro == 11 and loop == 12 and lineDuration == 2.5)
    return "ok"
"##;

#[test]
fn retail_talking_head_apis_project_explicit_host_inputs() {
    let env = WowLuaEnv::new().unwrap();
    let empty_results: bool = env
        .eval(
            r##"return select("#", C_TalkingHead.GetCurrentLineInfo()) == 0
                and select("#", C_TalkingHead.GetCurrentLineAnimationInfo()) == 0"##,
        )
        .unwrap();
    assert!(empty_results);
    env.state().borrow_mut().talking_head.current_line = Some(sample_line());
    let result: String = env.eval(API_CONTRACT).unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn original_blizzard_talking_head_and_splash_consumers_use_the_model() {
    with_blizzard_addon_closure(
        &["Blizzard_FrameXML", "Blizzard_SplashFrame"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|addon| addon == "Blizzard_FrameXML"));
            assert!(loaded.iter().any(|addon| addon == "Blizzard_SplashFrame"));
            env.state().borrow_mut().talking_head.current_line = Some(TalkingHeadLine {
                display_info: 0.0,
                name: "Speaker".into(),
                text: "Hello.".into(),
                ..Default::default()
            });
            env.exec(
                r#"
                TalkingHeadFrameMixin:OnEvent(TalkingHeadFrame, "TALKINGHEAD_REQUESTED")
                TalkingHeadFrameModelMixin:SetupAnimations(TalkingHeadFrame.MainFrame.Model)
                SplashFrameMixin:OnShow(SplashFrame)
                assert(C_TalkingHead.GetConversationsDeferred())
                SplashFrameMixin:OnHide(SplashFrame)
                assert(not C_TalkingHead.GetConversationsDeferred())
                "#,
            )
            .expect("original Blizzard consumers should read and update talking-head state");
        },
    );
}
