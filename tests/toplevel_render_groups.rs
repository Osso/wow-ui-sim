//! Native 12.1.0.69587 controlled layering capture (2026-09-08).
//! Case 1's HIGH child stays inside its LOW top-level group. Independent
//! HIGH, DIALOG, plain TOOLTIP and GameTooltip controls remain above MEDIUM.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::FrameStrata;

fn create_controls() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local function frame(kind, name, parent, strata, index)
            local f = CreateFrame(kind, name, parent)
            f:SetFrameStrata(strata)
            f:SetSize(120, 80)
            f:SetPoint('BOTTOMLEFT', UIParent, 'BOTTOMLEFT',
                100 + ((index - 1) % 3) * 240, 500 - math.floor((index - 1) / 3) * 200)
            return f
        end
        local strata = { 'HIGH', 'HIGH', 'DIALOG', 'TOOLTIP', 'TOOLTIP' }
        NativeLayerCases = {}
        for i = 1, 5 do
            local parent = UIParent
            if i == 1 then
                parent = frame('Frame', 'NativeLayerLowParent', UIParent, 'LOW', i)
                parent:SetToplevel(true)
            end
            local red = frame(i == 5 and 'GameTooltip' or 'Frame', 'NativeLayerRed' .. i, parent, strata[i], i)
            local redTexture = red:CreateTexture('NativeLayerRedTexture' .. i, 'ARTWORK')
            redTexture:SetAllPoints()
            redTexture:SetColorTexture(1, 0, 0, 1)
            local blue = frame('Frame', 'NativeLayerBlue' .. i, UIParent, 'MEDIUM', i)
            blue:SetToplevel(true)
            local blueTexture = blue:CreateTexture('NativeLayerBlueTexture' .. i, 'ARTWORK')
            blueTexture:SetAllPoints()
            blueTexture:SetColorTexture(0, 0, 1, 1)
            if i == 5 then
                red:SetOwner(blue, 'ANCHOR_NONE')
                assert(red:GetParent() == UIParent)
            end
            NativeLayerCases[i] = { red = red, blue = blue }
        end
        "#,
    )
    .unwrap();
    env
}

fn assert_control_order(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.ensure_layout_rects();
    let buckets = state.get_strata_buckets().unwrap().clone();
    let order: Vec<u64> = buckets.iter().flatten().copied().collect();
    for case in 1..=5 {
        let red = state
            .widgets
            .get_id_by_name(&format!("NativeLayerRedTexture{case}"))
            .unwrap();
        let blue = state
            .widgets
            .get_id_by_name(&format!("NativeLayerBlueTexture{case}"))
            .unwrap();
        let red_index = order.iter().position(|&id| id == red).unwrap();
        let blue_index = order.iter().position(|&id| id == blue).unwrap();
        assert_eq!(
            red_index < blue_index,
            case == 1,
            "native control {case}: red={red_index}, blue={blue_index}"
        );
        let expected = if case == 1 {
            FrameStrata::Low
        } else {
            state
                .widgets
                .get(red)
                .unwrap()
                .parent_id
                .and_then(|id| state.widgets.get(id))
                .unwrap()
                .frame_strata
        };
        assert!(buckets[expected.as_index()].contains(&red));
        assert_eq!(order.iter().filter(|&&id| id == red).count(), 1);
    }
}

#[test]
fn native_controls_keep_unraised_and_raised_groups_in_owner_strata() {
    let env = create_controls();
    assert_control_order(&env);
    env.exec("for _, c in ipairs(NativeLayerCases) do c.blue:Hide(); c.blue:Show() end")
        .unwrap();
    assert_control_order(&env);
    env.exec("for _, c in ipairs(NativeLayerCases) do c.blue:Raise() end")
        .unwrap();
    assert_control_order(&env);
    env.exec(
        "assert(NativeLayerRed1:GetFrameStrata() == 'HIGH'); \
         assert(NativeLayerRed1:GetFrameLevel() == 2); \
         assert(NativeLayerBlue1:GetFrameStrata() == 'MEDIUM'); \
         assert(NativeLayerBlue1:GetFrameLevel() == 1)",
    )
    .unwrap();
}

#[test]
fn screen_roots_do_not_capture_independent_render_groups() {
    let env = create_controls();
    env.exec(
        "UIParent:SetToplevel(true); UIParent:Raise(); \
         WorldFrame:SetToplevel(true); WorldFrame:Raise()",
    )
    .unwrap();
    assert_control_order(&env);
    env.exec(
        "assert(NativeLayerRed2:GetRaisedFrameLevel() == 0); \
         assert(NativeLayerRed5:GetRaisedFrameLevel() == 0)",
    )
    .unwrap();
}

#[test]
fn enabling_toplevel_on_created_frames_does_not_allocate_raised_level() {
    let env = create_controls();
    env.exec(
        r#"
        for _, c in ipairs(NativeLayerCases) do
            assert(c.blue:IsToplevel() and c.blue:IsShown())
            assert(c.blue:GetRaisedFrameLevel() == 0)
            assert(c.red:GetRaisedFrameLevel() == 0)
        end
        assert(NativeLayerLowParent:GetRaisedFrameLevel() == 0)
        "#,
    )
    .unwrap();
}

#[test]
fn native_show_and_raise_update_parent_derived_level_not_tooltip_owner() {
    let env = create_controls();
    env.exec(
        r#"
        local child = CreateFrame('Frame', 'NativeRaisedChild', NativeLayerBlue1)
        child:SetFrameLevel(100)
        child:SetFrameStrata('HIGH')
        local previous = 0
        for _, c in ipairs(NativeLayerCases) do
            c.blue:Hide()
            c.blue:Show()
            local level = c.blue:GetRaisedFrameLevel()
            assert(level > previous, 'shown top-level frames need ordered positive raised levels')
            previous = level
            assert(c.red:GetRaisedFrameLevel() == 0, 'unrelated and tooltip-owner controls stay unraised')
        end
        assert(child:GetRaisedFrameLevel() == NativeLayerBlue1:GetRaisedFrameLevel())
        for _, c in ipairs(NativeLayerCases) do
            c.blue:Raise()
            local level = c.blue:GetRaisedFrameLevel()
            assert(level > previous, 'explicit Raise must advance shown top-level order')
            previous = level
            assert(c.blue:GetFrameLevel() == 1 and c.blue:GetFrameStrata() == 'MEDIUM')
        end
        assert(child:GetRaisedFrameLevel() == NativeLayerBlue1:GetRaisedFrameLevel())
        assert(child:GetFrameLevel() == 100 and child:GetFrameStrata() == 'HIGH')
        NativeLayerRed2:Raise()
        NativeLayerRed2:Lower()
        assert(NativeLayerRed2:GetRaisedFrameLevel() == 0)
        assert(NativeLayerRed5:GetRaisedFrameLevel() == 0)
        "#,
    )
    .unwrap();
}

#[test]
fn grouped_visibility_repairs_preserve_local_strata_and_complete_subtrees() {
    let env = create_controls();
    env.exec(
        r#"
        local low = CreateFrame('Frame', 'NativeLowGrandchild', NativeLayerRed1)
        low:SetFrameStrata('LOW')
        low:SetAllPoints()
        local marker = low:CreateTexture('NativeLowGrandchildTexture', 'ARTWORK')
        marker:SetAllPoints()
        marker:SetColorTexture(0, 1, 0, 1)
        "#,
    )
    .unwrap();
    for script in [
        "",
        "NativeLayerRed1:Hide()",
        "NativeLayerRed1:Show()",
        "NativeLayerRedTexture1:Hide()",
        "NativeLayerRedTexture1:Show()",
    ] {
        env.exec(script).unwrap();
        let mut state = env.state().borrow_mut();
        let buckets = state.get_strata_buckets().unwrap().clone();
        let red = state
            .widgets
            .get_id_by_name("NativeLayerRedTexture1")
            .unwrap();
        let low = state
            .widgets
            .get_id_by_name("NativeLowGrandchildTexture")
            .unwrap();
        for id in [red, low] {
            let visible = state.widgets.is_ancestor_visible(id);
            assert_eq!(
                buckets
                    .iter()
                    .flatten()
                    .filter(|&&entry| entry == id)
                    .count(),
                usize::from(visible)
            );
            assert_eq!(buckets[FrameStrata::Low.as_index()].contains(&id), visible);
        }
        let group = &buckets[FrameStrata::Low.as_index()];
        if let (Some(low), Some(high)) = (
            group.iter().position(|&id| id == low),
            group.iter().position(|&id| id == red),
        ) {
            assert!(
                low < high,
                "raw LOW must remain before raw HIGH within the group"
            );
        }
    }
}
