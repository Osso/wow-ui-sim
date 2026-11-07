#![cfg(feature = "gui")]
// Every case here is profile-specific: Mainline owns `SetCustomWordWrapMinWidth` and
// Wrath 12340 owns only `SetMinimumWidth`.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wrath"
))]

use wow_ui_sim::lua_api::WowLuaEnv;

const WRAPPED_ONLY_BODY: &str = "This is a wrapped-only tooltip body that should honor the custom minimum width instead of collapsing down to padding width.";

/// `SetCustomWordWrapMinWidth` is a Mainline method; the Wrath 3.3.5 surface has
/// only `SetMinimumWidth` (see the `wrath` method baseline surplus diff), so the
/// custom-wrap-minimum case is Mainline-only.
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn test_custom_word_wrap_min_width_expands_wrapped_only_tooltip_width() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(&format!(
        r#"
        local owner = CreateFrame("Frame", "WrapMinWidthOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:SetCustomWordWrapMinWidth(350)
        GameTooltip:AddLine("{WRAPPED_ONLY_BODY}", 1, 1, 1, true)
        GameTooltip:Show()
    "#
    ))
    .unwrap();

    prepare_tooltip_layout(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    // WM63 L23:16-22: the custom width caps the candidate; fitting may shrink it.
    let content_width = frame.width - 20.48;
    let line_id = state.tooltips[&gt_id].left_line_ids[0];
    let line_width = state.widgets.get(line_id).unwrap().width;
    assert!(content_width > 0.0 && content_width <= 350.0);
    assert!((line_width - content_width).abs() < 0.01);
}

/// Wrath 12340 has no custom word-wrap minimum. `SetMinimumWidth` sets the
/// CONTENT width floor, which a wrapped-only row starts from and cannot grow
/// past when the wrap candidate is smaller
/// (handoff/laptop-labels-answers-20261018/tooltip-addtexture-12340.md).
#[cfg(feature = "client-wrath")]
#[test]
fn test_minimum_width_sets_wrapped_only_content_width() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(&format!(
        r#"
        local owner = CreateFrame("Frame", "WrapMinWidthOwner", UIParent)
        local probe = CreateFrame("GameTooltip", "WrapMinWidthProbe")
        probe:SetOwner(owner, "ANCHOR_NONE")
        probe:AddLine("Short")
        probe:Show()
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:SetMinimumWidth(350)
        GameTooltip:AddLine("{WRAPPED_ONLY_BODY}", 1, 1, 1, true)
        GameTooltip:Show()
        local second = CreateFrame("GameTooltip", "WrapMinWidthTooltip2")
        second:SetOwner(owner, "ANCHOR_NONE")
        second:SetMinimumWidth(500)
        second:AddLine("{WRAPPED_ONLY_BODY}", 1, 1, 1, true)
        second:Show()
    "#
    ))
    .unwrap();

    prepare_tooltip_layout(&env);

    // The text inset comes from a known unwrapped row, not a hard-coded border size.
    let mut fonts = wow_ui_sim::render::font::WowFontSystem::new();
    let inset = {
        let state = env.state().borrow();
        let id = state.widgets.get_id_by_name("WrapMinWidthProbe").unwrap();
        state.widgets.get(id).unwrap().width
    } - fonts.measure_text_width("Short", None, 14.0);

    let state = env.state().borrow();
    for (name, expected) in [("GameTooltip", 350.0), ("WrapMinWidthTooltip2", 500.0)] {
        let id = state.widgets.get_id_by_name(name).expect("tooltip frame");
        let frame = state.widgets.get(id).expect("tooltip geometry");
        let content_width = frame.width - inset;
        assert!(
            (content_width - expected).abs() < 0.01,
            "{name} content width should be the SetMinimumWidth floor {expected}, got {content_width}"
        );
    }
}

fn prepare_tooltip_layout(env: &WowLuaEnv) {
    use wow_ui_sim::render::font::WowFontSystem;

    let mut font_sys = WowFontSystem::new();
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut state, &mut font_sys);
}
