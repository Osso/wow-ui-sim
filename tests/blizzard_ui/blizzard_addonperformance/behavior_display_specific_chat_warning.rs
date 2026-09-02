//! AddOnPerformance chat-warning display behavior for `Blizzard_AddOnPerformance`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AddOnPerformance";

#[test]
fn display_specific_chat_warning_formats_and_forwards_system_message() {
    with_blizzard_addon_smoke_shape(&["Blizzard_ChatFrameBase", ROOT], &[], |env, _loaded| {
        let expected_message: String = env
            .eval(
                r#"
                _G.addOnPerformanceSystemMessages = {}
                DEFAULT_CHAT_FRAME = {
                    AddMessage = function(_, text)
                        table.insert(_G.addOnPerformanceSystemMessages, text)
                    end,
                }

                local addOnName = "ChatWarningPerformanceProbe"
                local expected = string.format(ADDON_PERFORMANCE_SPECIFIC_WARNING_TEXT, addOnName)
                AddOnPerformance:DisplayMessage({
                    type = Enum.AddOnPerformanceMessageType.SpecificAddOnChatWarning,
                    addOnName = addOnName,
                })
                return expected
                "#,
            )
            .expect("AddOnPerformance chat-warning display probe must run cleanly");
        let captured_messages: Vec<String> = env
            .eval("return _G.addOnPerformanceSystemMessages")
            .expect("system chat message recorder must be readable");

        assert_eq!(
            captured_messages,
            vec![expected_message],
            "`SpecificAddOnChatWarning` must forward the formatted warning through Blizzard ChatFrameUtil.AddSystemMessage"
        );
    });
}
