//! One layout pass with several dirty roots must leave every stored rect equal
//! to a fresh computation, including frames reachable from more than one root.

use super::*;

const SETUP: &str = r#"
    MultiRootParent = CreateFrame("Frame", "MultiRootParent", UIParent)
    MultiRootParent:SetSize(100, 100)
    MultiRootParent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
    MultiRootTarget = CreateFrame("Frame", "MultiRootTarget", UIParent)
    MultiRootTarget:SetSize(50, 50)
    MultiRootTarget:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 400, -300)
    -- Child of one root, anchored to the other root.
    MultiRootShared = CreateFrame("Frame", "MultiRootShared", MultiRootParent)
    MultiRootShared:SetSize(20, 20)
    MultiRootShared:SetPoint("TOPLEFT", MultiRootTarget, "BOTTOMRIGHT", 5, -5)
    MultiRootGrandchild = CreateFrame("Frame", "MultiRootGrandchild", MultiRootShared)
    MultiRootGrandchild:SetPoint("TOPLEFT", MultiRootShared, "TOPLEFT", 2, -2)
    MultiRootGrandchild:SetPoint("BOTTOMRIGHT", MultiRootShared, "BOTTOMRIGHT", -2, 2)
"#;

const NAMES: [&str; 4] = [
    "MultiRootParent",
    "MultiRootTarget",
    "MultiRootShared",
    "MultiRootGrandchild",
];

fn assert_stored_rects_match_fresh_layout(t: &TestCtx) {
    let state = t.env.state().borrow();
    for name in NAMES {
        let id = state.widgets.get_id_by_name(name).unwrap();
        let stored = state.widgets.get(id).and_then(|frame| frame.layout_rect);
        let fresh = crate::layout::compute_frame_rect(
            &state.widgets,
            id,
            state.screen_width,
            state.screen_height,
        );
        assert_eq!(stored, Some(fresh), "{name} stored rect is stale");
    }
}

#[test]
fn moving_parent_and_anchor_target_in_one_pass_updates_shared_descendants() {
    let (t, _) = load_test_lua("layout-multi-root", SETUP);
    t.env.state().borrow_mut().ensure_layout_rects();

    t.env
        .exec(
            r#"
            MultiRootTarget:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 250, -150)
            MultiRootParent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 30, -40)
            MultiRootShared:SetSize(40, 30)
        "#,
        )
        .unwrap();
    t.env.state().borrow_mut().ensure_layout_rects();

    assert_stored_rects_match_fresh_layout(&t);
    let shared_x = t
        .env
        .eval::<f64>("return MultiRootShared:GetLeft()")
        .unwrap();
    assert_eq!(
        shared_x, 305.0,
        "shared frame follows the moved anchor target"
    );
}
