#![cfg(all(
    feature = "gui",
    any(feature = "client-retail", feature = "client-ptr")
))]

mod tooltip_full_env_helpers;

use tooltip_full_env_helpers::setup_full_env;

#[test]
fn blizzard_tooltip_wrapped_content_uses_custom_width_as_cap() {
    let env = setup_full_env();
    env.exec(r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        GameTooltip:SetCustomWordWrapMinWidth(350)
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "Short", wrapText = true },
        } } }))
        assert(GameTooltip:GetWidth() < 150, "short wrapped content must not expand to the custom cap")
    "#).unwrap();
}
