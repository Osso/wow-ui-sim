//! Render invalidation for anchor edits: net no-op re-anchoring stays clean,
//! real moves (including through a moved anchor target) mark frames dirty.

use super::*;

const SETUP: &str = r#"
    AnchorTarget = CreateFrame("Frame", "AnchorTarget", UIParent)
    AnchorTarget:SetSize(40, 40)
    AnchorTarget:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
    AnchorFollower = CreateFrame("Frame", "AnchorFollower", UIParent)
    AnchorFollower:SetSize(20, 20)
    AnchorFollower:SetPoint("TOPLEFT", AnchorTarget, "BOTTOMLEFT", 0, -5)
"#;

fn dirty_ids_after(t: &TestCtx, lua: &str, resolve_layout_before_read: bool) -> Vec<String> {
    {
        let mut state = t.env.state().borrow_mut();
        state.ensure_layout_rects();
        let _ = state.widgets.take_render_dirty_with_ids();
    }
    t.env.exec(lua).unwrap();
    let mut state = t.env.state().borrow_mut();
    if resolve_layout_before_read {
        state.ensure_layout_rects();
    }
    let (_, ids) = state.widgets.take_render_dirty_with_ids();
    let mut names: Vec<String> = ids
        .unwrap_or_default()
        .into_iter()
        .filter_map(|id| state.widgets.get(id).and_then(|f| f.name.clone()))
        .filter(|name| name.starts_with("Anchor"))
        .collect();
    names.sort();
    names
}

#[test]
fn reanchoring_to_the_same_point_does_not_dirty_render_state() {
    let (t, _) = load_test_lua("anchor-dirty-noop", SETUP);

    for resolve_layout in [false, true] {
        let dirty = dirty_ids_after(
            &t,
            r#"
            AnchorFollower:ClearAllPoints()
            AnchorFollower:SetPoint("TOPLEFT", AnchorTarget, "BOTTOMLEFT", 0, -5)
        "#,
            resolve_layout,
        );
        assert!(
            dirty.is_empty(),
            "same-anchor re-anchor (layout resolved first: {resolve_layout}) dirtied {dirty:?}"
        );
    }
}

#[test]
fn moving_an_anchor_dirties_render_state() {
    let (t, _) = load_test_lua("anchor-dirty-move", SETUP);

    let dirty = dirty_ids_after(
        &t,
        r#"AnchorFollower:SetPoint("TOPLEFT", AnchorTarget, "BOTTOMLEFT", 0, -9)"#,
        false,
    );

    assert_eq!(dirty, ["AnchorFollower"]);
}

#[test]
fn clearing_anchors_without_reanchoring_dirties_render_state() {
    let (t, _) = load_test_lua("anchor-dirty-clear", SETUP);

    let dirty = dirty_ids_after(&t, "AnchorFollower:ClearAllPoints()", false);

    assert_eq!(dirty, ["AnchorFollower"]);
}

#[test]
fn same_anchor_reapplied_after_its_target_moved_dirties_render_state() {
    let (t, _) = load_test_lua("anchor-dirty-target-moved", SETUP);

    for resolve_layout in [false, true] {
        let dirty = dirty_ids_after(
            &t,
            &format!(
                r#"
                AnchorTarget:SetPoint("TOPLEFT", UIParent, "TOPLEFT", {}, -100)
                AnchorFollower:ClearAllPoints()
                AnchorFollower:SetPoint("TOPLEFT", AnchorTarget, "BOTTOMLEFT", 0, -5)
            "#,
                if resolve_layout { 160 } else { 130 }
            ),
            resolve_layout,
        );
        assert_eq!(
            dirty,
            ["AnchorFollower", "AnchorTarget"],
            "layout resolved first: {resolve_layout}"
        );
    }
}
