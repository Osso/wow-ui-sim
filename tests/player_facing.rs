#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn player_facing_tracks_nullable_state_independently_of_model_widgets() {
    let env = WowLuaEnv::new().unwrap();
    let other = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(GetPlayerFacing) == 'function', 'GetPlayerFacing missing')
        assert(GetPlayerFacing() == nil)
        local model = CreateFrame('PlayerModel')
        model:SetFacing(2)
        assert(GetPlayerFacing() == nil)
        A_Admin.SetPlayerFacing(0.25)
        assert(GetPlayerFacing() == 0.25)
        model:SetFacing(3)
        assert(GetPlayerFacing() == 0.25)
        A_Admin.SetPlayerFacing(7.25)
        assert(GetPlayerFacing() == 7.25, 'facing must not be implicitly wrapped')
        assert(model:GetFacing() == 3)
        A_Admin.SetPlayerFacing(-0.75)
        assert(GetPlayerFacing() == -0.75)
        A_Admin.SetPlayerFacing(nil)
        assert(GetPlayerFacing() == nil)
        "#,
    )
    .unwrap();
    assert!(
        other
            .eval::<bool>("return GetPlayerFacing() == nil")
            .unwrap()
    );
}

#[test]
fn invalid_player_facing_inputs_preserve_previous_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetPlayerFacing(1.5)
        for _, value in ipairs({math.huge, -math.huge, 0/0, true, '1.25', {}}) do
            assert(not pcall(A_Admin.SetPlayerFacing, value), 'invalid facing accepted')
            assert(GetPlayerFacing() == 1.5, 'invalid facing mutated state')
        end
        "#,
    )
    .unwrap();
}

#[test]
fn player_facing_drives_texture_rotation_through_on_update() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        Arrow = CreateFrame('Frame', nil, UIParent)
        Arrow.texture = Arrow:CreateTexture()
        Arrow.texture:SetRotation(0)
        Arrow:SetScript('OnUpdate', function(self)
            local facing = GetPlayerFacing()
            if facing == nil then return end
            if facing ~= self.facing then
                self.texture:SetRotation(facing)
                self.facing = facing
            end
        end)
        "#,
    )
    .unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec("assert(Arrow.texture:GetRotation() == 0)")
        .unwrap();
    for angle in [0.25, 1.5] {
        env.exec(&format!("A_Admin.SetPlayerFacing({angle})"))
            .unwrap();
        env.fire_on_update(0.016).unwrap();
        let observed: f64 = env.eval("return Arrow.texture:GetRotation()").unwrap();
        assert!((observed - angle).abs() < 0.000001);
    }
    env.exec("A_Admin.SetPlayerFacing(nil)").unwrap();
    env.fire_on_update(0.016).unwrap();
    assert_eq!(
        env.eval::<f64>("return Arrow.texture:GetRotation()")
            .unwrap(),
        1.5
    );
    assert!(env.state().borrow().lua_errors.is_empty());
}
