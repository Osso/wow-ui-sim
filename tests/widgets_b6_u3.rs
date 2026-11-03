use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
fn wrath() -> bool {
    ACTIVE == ClientProfile::Wrath
}
#[path = "common/wrath_label_fixtures.rs"]
mod originals;
use originals::{lfr_env, tooltip_env};

#[test]
fn b6_u3_original_lfr_and_nonempty_right_disables_wrap() {
    if !wrath() {
        return;
    }
    let env = lfr_env();
    env.exec(
        r#"
        assert(B6Tip:GetRightLine(B6Tip:NumLines()):GetText()=='Friend')
        B6Tip:ClearLines()
        B6Tip:AddDoubleLine(string.rep('long left ',40),'R',1,1,1,1,1,1,true)
        assert(not B6Tip:GetLeftLine(1):CanWordWrap(),'right text disables wrap')
    "#,
    )
    .unwrap();
}
#[cfg(feature = "gui")]
#[test]
fn b6_u3_height_depends_on_left_not_right() {
    if !wrath() {
        return;
    }
    let env = tooltip_env();
    let mut fonts = wow_ui_sim::render::WowFontSystem::new_without_casc();
    assert!(
        fonts.font_system.db().faces().next().is_some(),
        "SETUP: fonts required"
    );
    let measure = |right: &str, fonts: &mut wow_ui_sim::render::WowFontSystem| {
        env.exec(&format!(
            "B6Tip:ClearLines(); B6Tip:AddDoubleLine('left',{right},1,1,1,1,1,1,true); B6Tip:Show()"
        ))
        .unwrap();
        wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut env.state().borrow_mut(), fonts);
        env.eval::<f64>("return B6Tip:GetHeight()").unwrap()
    };
    let short = measure("'R'", &mut fonts);
    let tall = measure("'R\\nR\\nR\\nR'", &mut fonts);
    assert!(
        (short - tall).abs() < 0.01,
        "right string must not add row height"
    );
}

#[cfg(feature = "gui")]
#[test]
fn b6_u3_right_text_wrap_flag_does_not_change_measured_size() {
    if !wrath() {
        return;
    }
    let env = tooltip_env();
    let mut fonts = wow_ui_sim::render::WowFontSystem::new_without_casc();
    assert!(
        fonts.font_system.db().faces().next().is_some(),
        "SETUP: fonts required"
    );
    let measure = |wrap: bool, fonts: &mut wow_ui_sim::render::WowFontSystem| {
        env.exec(&format!("B6Tip:ClearLines(); B6Tip:AddLine('header'); B6Tip:AddDoubleLine(string.rep('word ',80),'R',1,1,1,1,1,1,{wrap}); B6Tip:Show()")).unwrap();
        wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut env.state().borrow_mut(), fonts);
        env.eval::<(f64, f64)>("return B6Tip:GetWidth(),B6Tip:GetHeight()")
            .unwrap()
    };
    let off = measure(false, &mut fonts);
    let on = measure(true, &mut fonts);
    assert!((off.0 - on.0).abs() < 0.01 && (off.1 - on.1).abs() < 0.01);
}

#[cfg(feature = "gui")]
#[test]
fn b6_u3_wrath_refinement_is_monotone_without_back_propagation() {
    if !wrath() {
        return;
    }
    let env = tooltip_env();
    let mut fonts = wow_ui_sim::render::WowFontSystem::new_without_casc();
    assert!(
        fonts.font_system.db().faces().next().is_some(),
        "SETUP: fonts required"
    );
    env.exec("B6Tip:AddLine('base');B6Tip:Show()").unwrap();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut env.state().borrow_mut(), &mut fonts);
    let base: f64 = env.eval("return B6Tip:GetWidth()").unwrap();
    env.exec("B6Tip:AddLine(string.rep('wide words ',30),1,1,1,true);B6Tip:Show()")
        .unwrap();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut env.state().borrow_mut(), &mut fonts);
    let first: (f64, f64) = env
        .eval("return B6Tip:GetWidth(),B6Tip:GetLeftLine(2):GetWidth()")
        .unwrap();
    assert!(
        first.0 > base,
        "wrapped candidate grows narrow shared width"
    );
    env.exec("B6Tip:AddLine(string.rep('WIDER differently segmented words ',40),1,1,1,true);B6Tip:Show()").unwrap();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut env.state().borrow_mut(), &mut fonts);
    let later: (f64, f64) = env
        .eval("return B6Tip:GetWidth(),B6Tip:GetLeftLine(2):GetWidth()")
        .unwrap();
    assert!(
        later.0 + 0.01 >= first.0,
        "shared width grows monotonically"
    );
    assert!(
        (later.1 - first.1).abs() < 0.01,
        "later refinement cannot back-propagate into earlier left row"
    );
}
#[cfg(feature = "gui")]
#[test]
fn b6_u3_local_double_column_gap_only_when_both_strings_shown() {
    if !wrath() {
        return;
    }
    let env = lfr_env();
    let mut fonts = wow_ui_sim::render::WowFontSystem::new_without_casc();
    assert!(
        fonts.font_system.db().faces().next().is_some(),
        "SETUP: fonts required"
    );
    let measure = |left: &str, right: &str, fonts: &mut wow_ui_sim::render::WowFontSystem| {
        env.exec(&format!("B6Tip:ClearLines(); B6Tip:AddLine('h'); B6Tip:AddDoubleLine({left},{right}); B6Tip:Show()")).unwrap();
        wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut env.state().borrow_mut(), fonts);
        env.eval::<f64>("return B6Tip:GetWidth()").unwrap()
    };
    let right_text = "RRRRRRRRRRRR";
    let right_width = fonts.measure_text_width(right_text, None, 12.0) as f64;
    let left_only = measure("'LLLLLLLLLLLL'", "nil", &mut fonts);
    let both = measure("'LLLLLLLLLLLL'", "'RRRRRRRRRRRR'", &mut fonts);
    assert!((both - left_only - right_width - 38.400002).abs() < 0.01);
    let right_only = measure("''", "'RRRRRRRRRRRR'", &mut fonts);
    let same_as_left = measure("'RRRRRRRRRRRR'", "nil", &mut fonts);
    assert!(
        (right_only - same_as_left).abs() < 0.01,
        "hidden left string must not add column gap"
    );
}

#[cfg(feature = "gui")]
#[test]
fn b6_u3_already_wide_shared_width_bypasses_refinement() {
    if !wrath() {
        return;
    }
    let env = tooltip_env();
    let mut fonts = wow_ui_sim::render::WowFontSystem::new_without_casc();
    assert!(
        fonts.font_system.db().faces().next().is_some(),
        "SETUP: fonts required"
    );
    env.exec(
        r#"
        B6Tip:AddLine(string.rep('W',200))
        B6Tip:AddLine('short wrapped words',1,1,1,true)
        B6Tip:GetLeftLine(2):SetWidth(70)
        B6PreservedLeftWidth=B6Tip:GetLeftLine(2):GetWidth()
        B6Tip:Show()
    "#,
    )
    .unwrap();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut env.state().borrow_mut(), &mut fonts);
    env.exec("assert(B6Tip:GetLeftLine(2):GetWidth()==B6PreservedLeftWidth,'already wide shared width must bypass assignment')").unwrap();
    // 70 is a supplied fixture width, not a native numeric sizing oracle.
    // No assertion of absolute final tooltip width or final rendered line count.
}
