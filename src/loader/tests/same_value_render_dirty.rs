//! Render invalidation for repeated setter calls: writing the value a region
//! already has must not mark it dirty. Addons such as ClickableRaidBuffs
//! re-apply the same icon state from timers many times per second.

use super::*;

const SETUP: &str = r#"
    SameParent = CreateFrame("Frame", "SameParent", UIParent)
    SameParent:SetSize(40, 40)
    SameParent:SetPoint("CENTER")
    SameIcon = SameParent:CreateTexture("SameIcon", "ARTWORK")
    SameIcon:SetAllPoints()
    SameIcon:SetTexture("Interface\\ICONS\\SPELL_HOLY_DEVOTIONAURA")
    SameIcon:SetVertexColor(1, 1, 1, 1)
    SameIcon:SetDesaturated(false)
    SameIcon:SetAlpha(1)
    SameLabel = SameParent:CreateFontString("SameLabel", "OVERLAY", "GameFontNormal")
    SameLabel:SetPoint("CENTER")
    SameLabel:SetText("")
    SameLabel:SetTextColor(1, 1, 1, 1)
    SameHidden = SameParent:CreateTexture("SameHidden", "ARTWORK")
    SameHidden:Hide()
"#;

/// Same-value calls replayed from ClickableRaidBuffs' idle icon refresh.
const SAME_VALUE_CALLS: &[&str] = &[
    "SameIcon:SetTexture(\"Interface\\\\ICONS\\\\SPELL_HOLY_DEVOTIONAURA\")",
    "SameIcon:SetVertexColor(1, 1, 1, 1)",
    "SameIcon:SetDesaturated(false)",
    "SameIcon:SetAlpha(1)",
    "SameIcon:Show()",
    "SameHidden:Hide()",
    "SameParent:SetWidth(40)",
    "SameParent:SetHeight(40)",
    "SameLabel:SetText(\"\")",
    "SameLabel:SetTextColor(1, 1, 1, 1)",
];

fn dirty_names_after(t: &TestCtx, lua: &str) -> Vec<String> {
    {
        let mut state = t.env.state().borrow_mut();
        state.ensure_layout_rects();
        let _ = state.widgets.take_render_dirty_with_ids();
    }
    t.env.exec(lua).unwrap();
    let mut state = t.env.state().borrow_mut();
    state.ensure_layout_rects();
    let (_, ids) = state.widgets.take_render_dirty_with_ids();
    let mut names: Vec<String> = ids
        .unwrap_or_default()
        .into_iter()
        .filter_map(|id| state.widgets.get(id).and_then(|f| f.name.clone()))
        .filter(|name| name.starts_with("Same"))
        .collect();
    names.sort();
    names
}

#[test]
fn repeating_current_values_does_not_dirty_render_state() {
    let (t, _) = load_test_lua("same-value-dirty", SETUP);

    let dirtying_calls: Vec<String> = SAME_VALUE_CALLS
        .iter()
        .filter_map(|call| {
            let dirtied = dirty_names_after(&t, call);
            (!dirtied.is_empty()).then(|| format!("{call} -> {dirtied:?}"))
        })
        .collect();

    assert_eq!(dirtying_calls, Vec::<String>::new());
}

#[test]
fn changing_values_still_dirties_render_state() {
    let (t, _) = load_test_lua("changed-value-dirty", SETUP);

    assert_eq!(
        dirty_names_after(&t, "SameIcon:SetDesaturated(true)"),
        vec!["SameIcon"]
    );
    assert_eq!(
        dirty_names_after(&t, "SameLabel:SetTextColor(1, 0, 0, 1)"),
        vec!["SameLabel"]
    );
}
