//! Render invalidation for animated alpha: a pulse the player can see marks
//! its texture dirty, one under a fully transparent parent does not.

use super::*;

fn pulse_setup(parent_alpha: f32) -> String {
    format!(
        r#"
        PulseParent = CreateFrame("Frame", "PulseParent", UIParent)
        PulseParent:SetSize(100, 100)
        PulseParent:SetPoint("CENTER")
        PulseParent:SetAlpha({parent_alpha})
        PulseGlow = PulseParent:CreateTexture("PulseGlow", "ARTWORK")
        PulseGlow:SetAllPoints()
        local group = PulseGlow:CreateAnimationGroup()
        group:SetLooping("BOUNCE")
        local fade = group:CreateAnimation("Alpha")
        fade:SetFromAlpha(0.25)
        fade:SetToAlpha(0.5)
        fade:SetDuration(0.75)
        group:Play()
    "#
    )
}

struct PulseTick {
    dirtied: Vec<String>,
    alpha_before: f64,
    alpha_after: f64,
}

/// Advances animations one frame and reports which pulse frames it dirtied
/// and the glow alpha on either side of that frame.
fn advance_pulse_one_tick(t: &TestCtx) -> PulseTick {
    t.env.fire_on_update(0.016).unwrap();
    let _ = t.env.state().borrow().widgets.take_render_dirty_with_ids();
    let alpha_before: f64 = t.env.eval("return PulseGlow:GetAlpha()").unwrap();
    t.env.fire_on_update(0.1).unwrap();
    let alpha_after: f64 = t.env.eval("return PulseGlow:GetAlpha()").unwrap();
    let state = t.env.state().borrow();
    let (_, ids) = state.widgets.take_render_dirty_with_ids();
    let mut names: Vec<String> = ids
        .unwrap_or_default()
        .into_iter()
        .filter_map(|id| state.widgets.get(id).and_then(|f| f.name.clone()))
        .filter(|name| name.starts_with("Pulse"))
        .collect();
    names.sort();
    PulseTick {
        dirtied: names,
        alpha_before,
        alpha_after,
    }
}

#[test]
fn visible_alpha_animation_marks_its_texture_dirty() {
    let (t, _) = load_test_lua("anim-dirty-visible", &pulse_setup(1.0));
    let tick = advance_pulse_one_tick(&t);

    assert_eq!(tick.dirtied, vec!["PulseGlow"]);
    assert!(
        tick.alpha_after > tick.alpha_before,
        "{} -> {}",
        tick.alpha_before,
        tick.alpha_after
    );
}

#[test]
fn alpha_animation_under_transparent_parent_stays_clean() {
    let (t, _) = load_test_lua("anim-dirty-hidden", &pulse_setup(0.0));
    let tick = advance_pulse_one_tick(&t);

    assert_eq!(tick.dirtied, Vec::<String>::new());
    // The animation still runs; only the invisible redraw is skipped.
    assert!(
        tick.alpha_after > tick.alpha_before,
        "{} -> {}",
        tick.alpha_before,
        tick.alpha_after
    );
}
