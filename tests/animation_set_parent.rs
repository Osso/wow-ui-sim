//! Simulator ownership and timing policy; native/security equivalence is unverified.
use wow_ui_sim::lua_api::WowLuaEnv;

fn setup() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        left = CreateFrame("Frame", nil, UIParent)
        right = CreateFrame("Frame", nil, UIParent)
        old = left:CreateAnimationGroup()
        new = right:CreateAnimationGroup()
        function near(actual, expected)
            assert(math.abs(actual - expected) < 0.000001,
                tostring(actual) .. " ~= " .. tostring(expected))
        end
        function animation(group, duration, order)
            local result = group:CreateAnimation("Alpha")
            result:SetDuration(duration)
            result:SetOrder(order or 1)
            return result
        end
        "#,
    )
    .unwrap();
    env
}

#[test]
fn stopped_transfer_preserves_configuration_and_repairs_remaining_indexes() {
    let env = setup();
    env.exec(
        r#"
        first = animation(old, 1)
        moved = animation(old, 2, 7)
        last = animation(old, 3)
        existing = animation(new, 4, 2)
        left.Target = left:CreateTexture()
        right.Target = right:CreateTexture()
        moved:SetChildKey("Target")
        moved:SetFromAlpha(0.2)
        moved:SetToAlpha(0.8)
        moved:SetStartDelay(0.25)
        moved:SetEndDelay(0.5)
        moved:SetSmoothing("IN")
        assert(select('#', moved:SetParent(new)) == 0)
        assert(moved:GetParent() == new)
        assert(moved:GetRegionParent() == right and moved:GetTarget() == right.Target)
        assert(select('#', old:GetAnimations()) == 2, "old group retained transferred animation")
        local a, b = old:GetAnimations()
        assert(a == first and b == last)
        local c, d = new:GetAnimations()
        assert(c == existing and d == moved)
        assert(select('#', new:GetAnimations()) == 2)
        near(moved:GetDuration(), 2)
        assert(moved:GetOrder() == 7 and moved:GetSmoothing() == "IN")
        near(moved:GetFromAlpha(), 0.2)
        near(moved:GetToAlpha(), 0.8)
        near(moved:GetStartDelay(), 0.25)
        near(moved:GetEndDelay(), 0.5)
        last:SetDuration(5)
        first:SetDuration(6)
        near(last:GetDuration(), 5)
        near(first:GetDuration(), 6)
        near(moved:GetDuration(), 2)
        moved:SetParent(old, 3)
        local e, f, g = old:GetAnimations()
        assert(e == first and f == last and g == moved)
        assert(new:GetAnimations() == existing)
        assert(moved:GetOrder() == 3 and moved:GetParent() == old)
        "#,
    )
    .unwrap();
}

#[test]
fn playing_transfer_resets_local_time_and_moves_finished_callback_ownership() {
    let env = setup();
    env.exec(
        r#"
        moved = animation(old, 2)
        retained = animation(old, 1)
        existing = animation(new, 2)
        movedFinished, retainedFinished, oldFinished, newFinished = 0, 0, 0, 0
        moved:SetScript("OnFinished", function(self)
            assert(self == moved and self:GetParent() == new)
            movedFinished = movedFinished + 1
        end)
        retained:SetScript("OnFinished", function() retainedFinished = retainedFinished + 1 end)
        old:SetScript("OnFinished", function() oldFinished = oldFinished + 1 end)
        new:SetScript("OnFinished", function() newFinished = newFinished + 1 end)
        old:Play()
        new:Play()
        "#,
    )
    .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec(
        r#"
        near(moved:GetElapsed(), 0.5)
        moved:SetParent(new)
        near(moved:GetElapsed(), 0)
        near(moved:GetProgress(), 0)
        near(old:GetElapsed(), 0.5)
        near(new:GetElapsed(), 0.5)
        assert(old:IsPlaying() and new:IsPlaying())
        assert(movedFinished == 0 and retainedFinished == 0)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec(
        r#"
        near(moved:GetElapsed(), 1)
        near(moved:GetProgress(), 0.5)
        assert(oldFinished == 1 and retainedFinished == 1)
        assert(movedFinished == 0 and newFinished == 0)
        "#,
    )
    .unwrap();
    env.fire_on_update(1.0).unwrap();
    env.exec("assert(movedFinished == 1 and newFinished == 1 and retainedFinished == 1)")
        .unwrap();
}

#[test]
fn sequence_order_is_not_insertion_position_and_same_parent_does_not_reset() {
    let env = setup();
    env.exec(
        r#"
        moved = animation(old, 1, 9)
        first = animation(new, 1, 1)
        last = animation(new, 1, 3)
        moved:SetParent(new, 2)
        local a, b, c = new:GetAnimations()
        assert(a == first and b == last and c == moved, "order changed insertion position")
        assert(moved:GetOrder() == 2)
        new:Play()
        "#,
    )
    .unwrap();
    env.fire_on_update(1.25).unwrap();
    env.exec(
        r#"
        near(first:GetElapsed(), 1)
        near(moved:GetElapsed(), 0.25)
        near(last:GetElapsed(), 0)
        moved:SetParent(new)
        near(moved:GetElapsed(), 0.25)
        assert(moved:GetOrder() == 2)
        moved:SetParent(new, 4)
        near(moved:GetElapsed(), 0.25)
        assert(moved:GetOrder() == 4)
        assert(select('#', new:GetAnimations()) == 3)
        local a, b, c = new:GetAnimations()
        assert(a == first and b == last and c == moved)
        "#,
    )
    .unwrap();
    env.fire_on_update(1.0).unwrap();
    env.exec("near(last:GetElapsed(), 1); near(moved:GetElapsed(), 0.25)")
        .unwrap();
}

#[test]
fn transfer_to_stopped_owner_stays_idle_until_destination_is_played() {
    let env = setup();
    env.exec(
        r#"
        moved = animation(old, 2)
        retained = animation(old, 3)
        finished = 0
        moved:SetScript("OnFinished", function() finished = finished + 1 end)
        old:Play()
        "#,
    )
    .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec("moved:SetParent(new); near(moved:GetElapsed(), 0); assert(not moved:IsPlaying())")
        .unwrap();
    env.fire_on_update(3.0).unwrap();
    env.exec("near(moved:GetElapsed(), 0); assert(finished == 0); new:Play()")
        .unwrap();
    env.fire_on_update(2.0).unwrap();
    env.exec("assert(finished == 1); near(moved:GetElapsed(), 2)")
        .unwrap();
}

#[test]
fn stopped_animation_joins_destination_timeline_without_restarting_it() {
    let env = setup();
    env.exec(
        r#"
        moved = animation(old, 1)
        existing = animation(new, 2)
        new:Play()
        "#,
    )
    .unwrap();
    env.fire_on_update(0.5).unwrap();
    env.exec(
        r#"
        moved:SetParent(new, 2)
        assert(not old:IsPlaying() and new:IsPlaying() and moved:IsPlaying())
        near(new:GetElapsed(), 0.5)
        near(moved:GetElapsed(), 0)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec("near(existing:GetElapsed(), 0.75); near(moved:GetElapsed(), 0)")
        .unwrap();
    env.fire_on_update(1.5).unwrap();
    env.exec("near(existing:GetElapsed(), 2); near(moved:GetElapsed(), 0.25)")
        .unwrap();
}

#[test]
fn invalid_target_and_order_leave_ownership_unchanged() {
    let env = setup();
    env.exec(
        r#"
        moved = animation(old, 1, 7)
        another = animation(new, 1)
        for _, invalid in ipairs({left, another, {}, false, "new"}) do
            assert(not pcall(moved.SetParent, moved, invalid))
            assert(moved:GetParent() == old and old:GetAnimations() == moved)
        end
        assert(not pcall(moved.SetParent, moved, nil))
        for _, invalid in ipairs({-1, math.huge, 0/0, "bad", {}, false}) do
            assert(not pcall(moved.SetParent, moved, new, invalid))
            assert(moved:GetParent() == old and old:GetAnimations() == moved)
            assert(moved:GetOrder() == 7)
        end
        assert(new:GetAnimations() == another)
        moved:SetParent(new, 2.75)
        assert(moved:GetOrder() == 2)
        moved:SetParent(new, 0)
        assert(moved:GetOrder() == 0)
        "#,
    )
    .unwrap();
}

#[test]
fn ordinary_frame_and_region_parenting_remains_independent() {
    let env = setup();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", nil, left)
        local texture = left:CreateTexture()
        frame:SetParent(right)
        texture:SetParent(right)
        assert(frame:GetParent() == right and texture:GetParent() == right)
        frame:SetParent(right)
        assert(right:GetNumChildren() == 2) -- its animation group plus the frame
        frame:SetParent(nil)
        texture:SetParent(nil)
        assert(frame:GetParent() == nil and texture:GetParent() == nil)
        assert(old:GetParent() == left and new:GetParent() == right)
        "#,
    )
    .unwrap();
}
