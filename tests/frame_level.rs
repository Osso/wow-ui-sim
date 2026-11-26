//! Tests for frame level behavior: SetFrameLevel, SetFixedFrameLevel, SetParent interaction.

use wow_ui_sim::iced_app::frame_collect::collect_hittable_frames;
use wow_ui_sim::lua_api::WowLuaEnv;

const FRAME_LEVEL_DELTA_SCENARIO: &str = r#"
local parent = CreateFrame("Frame", "FrameLevelDeltaParent", UIParent)
parent:SetFrameLevel(100)
local fixed = CreateFrame("Frame", "FrameLevelDeltaFixed", parent)
fixed:SetFrameLevel(150)
fixed:SetFixedFrameLevel(true)
local child = CreateFrame("Frame", "FrameLevelDeltaChild", parent)
local grandchild = CreateFrame("Frame", "FrameLevelDeltaGrandchild", child)
local other = CreateFrame("Frame", "FrameLevelDeltaOtherStrata", parent)
other:SetFrameStrata("HIGH")
local otherChild = CreateFrame("Frame", "FrameLevelDeltaOtherChild", other)
parent:SetFrameLevel(125)
assert(fixed:GetFrameLevel() == 175)
assert(child:GetFrameLevel() == 126 and grandchild:GetFrameLevel() == 127)
assert(other:GetFrameLevel() == 101 and otherChild:GetFrameLevel() == 102)
"#;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().unwrap()
}

#[test]
fn set_frame_level_shifts_same_strata_descendants_by_delta() {
    env().exec(FRAME_LEVEL_DELTA_SCENARIO).unwrap();
}

#[test]
fn test_raise_lower_do_not_change_lua_visible_raised_frame_level() {
    let env = env();
    let (
        before_low_level,
        before_high_level,
        before_low_raised,
        before_high_raised,
        after_raise_low_level,
        after_raise_high_level,
        after_raise_low_raised,
        after_raise_high_raised,
        after_lower_low_level,
        after_lower_high_level,
        after_lower_low_raised,
        after_lower_high_raised,
    ): (i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32) = env
        .eval(
            r#"
        local parent = CreateFrame("Frame", "RaiseLowerLuaVisibleParent", UIParent)
        parent:SetSize(100, 100)
        parent:Show()

        local low = CreateFrame("Frame", "RaiseLowerLuaVisibleLow", parent)
        low:SetFrameLevel(1)
        low:Show()

        local high = CreateFrame("Frame", "RaiseLowerLuaVisibleHigh", parent)
        high:SetFrameLevel(10)
        high:Show()

        local beforeLowLevel = low:GetFrameLevel()
        local beforeHighLevel = high:GetFrameLevel()
        local beforeLowRaised = low:GetRaisedFrameLevel()
        local beforeHighRaised = high:GetRaisedFrameLevel()

        low:Raise()

        local afterRaiseLowLevel = low:GetFrameLevel()
        local afterRaiseHighLevel = high:GetFrameLevel()
        local afterRaiseLowRaised = low:GetRaisedFrameLevel()
        local afterRaiseHighRaised = high:GetRaisedFrameLevel()

        high:Lower()

        return beforeLowLevel,
            beforeHighLevel,
            beforeLowRaised,
            beforeHighRaised,
            afterRaiseLowLevel,
            afterRaiseHighLevel,
            afterRaiseLowRaised,
            afterRaiseHighRaised,
            low:GetFrameLevel(),
            high:GetFrameLevel(),
            low:GetRaisedFrameLevel(),
            high:GetRaisedFrameLevel()
    "#,
        )
        .unwrap();

    assert_eq!((before_low_level, before_high_level), (1, 10));
    assert_eq!((before_low_raised, before_high_raised), (0, 0));
    assert_eq!((after_raise_low_level, after_raise_high_level), (1, 10));
    assert_eq!((after_raise_low_raised, after_raise_high_raised), (0, 0));
    assert_eq!((after_lower_low_level, after_lower_high_level), (1, 10));
    assert_eq!((after_lower_low_raised, after_lower_high_raised), (0, 0));
}

#[test]
fn test_raise_lower_do_not_cross_raw_frame_levels_for_hit_order() {
    let env = env();
    env.exec(
        r#"
        local low = CreateFrame("Frame", "RaiseLowerHitLow", UIParent)
        low:SetAllPoints(UIParent)
        low:SetFrameStrata("DIALOG")
        low:SetFrameLevel(1)
        low:EnableMouse(true)
        low:Show()

        local high = CreateFrame("Frame", "RaiseLowerHitHigh", UIParent)
        high:SetAllPoints(UIParent)
        high:SetFrameStrata("DIALOG")
        high:SetFrameLevel(10)
        high:EnableMouse(true)
        high:Show()
    "#,
    )
    .unwrap();

    assert_eq!(top_hittable_probe_frame(&env), "RaiseLowerHitHigh");

    env.exec("RaiseLowerHitLow:Raise()").unwrap();
    assert_eq!(top_hittable_probe_frame(&env), "RaiseLowerHitHigh");

    env.exec("RaiseLowerHitHigh:Lower()").unwrap();
    assert_eq!(top_hittable_probe_frame(&env), "RaiseLowerHitHigh");
}

fn top_hittable_probe_frame(env: &WowLuaEnv) -> String {
    let mut state = env.state().borrow_mut();
    state.ensure_layout_rects();
    let strata_buckets = state
        .get_strata_buckets()
        .expect("strata buckets should be available")
        .clone();
    let collected = collect_hittable_frames(&state.widgets, &strata_buckets);

    collected
        .hittable
        .iter()
        .rev()
        .find_map(|(id, _, _)| {
            let frame = state.widgets.get(*id)?;
            let name = frame.name.as_deref()?;
            name.starts_with("RaiseLowerHit").then(|| name.to_string())
        })
        .expect("probe frame should be hittable")
}

// ============================================================================
// SetFixedFrameLevel + SetParent interaction
// ============================================================================

#[test]
fn test_fixed_frame_level_preserved_on_reparent() {
    let env = env();
    // Fixed level should be preserved when reparenting
    let (f_level, g_level): (i32, i32) = env
        .eval(
            r#"
        local h = CreateFrame("Frame")
        local f = CreateFrame("Frame")
        local g = CreateFrame("Frame", nil, f)
        f:SetParent(nil)
        f:SetFrameLevel(42)
        f:SetFixedFrameLevel(true)
        f:SetParent(h)
        return f:GetFrameLevel(), g:GetFrameLevel()
    "#,
        )
        .unwrap();
    assert_eq!(
        f_level, 42,
        "Fixed frame level should be preserved on reparent"
    );
    assert_eq!(g_level, 43, "Child should inherit from fixed parent level");
}

#[test]
fn test_frame_level_same_parent_no_recalc() {
    let env = env();
    // After disabling fixed level, re-assigning same parent should NOT recalculate
    let (f_level, g_level): (i32, i32) = env
        .eval(
            r#"
        local h = CreateFrame("Frame")
        local f = CreateFrame("Frame")
        local g = CreateFrame("Frame", nil, f)
        f:SetParent(nil)
        f:SetFrameLevel(42)
        f:SetFixedFrameLevel(true)
        f:SetParent(h)
        f:SetFixedFrameLevel(false)
        f:SetParent(h)
        return f:GetFrameLevel(), g:GetFrameLevel()
    "#,
        )
        .unwrap();
    assert_eq!(
        f_level, 42,
        "Same-parent reparent should not recalculate level"
    );
    assert_eq!(
        g_level, 43,
        "Child should keep inherited level on same-parent reparent"
    );
}

#[test]
fn test_same_parent_set_parent_preserves_child_custom_frame_level() {
    let env = env();
    let child_level: i32 = env
        .eval(
            r#"
        local host = CreateFrame("Frame")
        local parent = CreateFrame("Frame")
        local child = CreateFrame("Frame", nil, parent)

        parent:SetParent(host)
        parent:SetFrameLevel(100)
        child:SetFrameLevel(7)

        parent:SetParent(host)
        return child:GetFrameLevel()
    "#,
        )
        .unwrap();
    assert_eq!(
        child_level, 7,
        "same-parent SetParent should not clobber explicitly assigned child frame levels",
    );
}

#[test]
fn test_frame_level_recalc_after_nil_reparent() {
    let env = env();
    // After going through nil parent, reparenting should recalculate
    let (f_level, g_level): (i32, i32) = env
        .eval(
            r#"
        local h = CreateFrame("Frame")
        local f = CreateFrame("Frame")
        local g = CreateFrame("Frame", nil, f)
        f:SetParent(nil)
        f:SetFrameLevel(42)
        f:SetFixedFrameLevel(true)
        f:SetParent(h)
        f:SetFixedFrameLevel(false)
        f:SetParent(nil)
        f:SetParent(h)
        return f:GetFrameLevel(), g:GetFrameLevel()
    "#,
        )
        .unwrap();
    assert_eq!(
        f_level, 1,
        "Level should recalculate after nil->parent transition"
    );
    assert_eq!(
        g_level, 2,
        "Child should recalculate after parent's nil->parent transition"
    );
}
