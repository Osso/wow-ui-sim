//! Collect at the host DTO's actual Lua color-callback boundary.
use wow_ui_sim::lua_api::WowLuaEnv;

fn collect_in_colors(env: &WowLuaEnv) {
    env.exec(&format!(
        "D56 = {}",
        matches!(
            wow_ui_sim::client_profile::ACTIVE,
            wow_ui_sim::client_profile::ClientProfile::Retail
                | wow_ui_sim::client_profile::ClientProfile::Ptr
        )
    ))
    .unwrap();
    env.exec(
        r#"
        colorCalls = 0
        local original = CreateColor
        CreateColor = function(...)
            colorCalls = colorCalls + 1
            collectgarbage('collect')
            return original(...)
        end
        function CheckTip(tip, name)
            assert(type(tip) == 'table' and type(tip.lines) == 'table')
            assert(tip.lines[1].leftText == name)
            assert(colorCalls > 0, 'must exercise collecting callback')
        end
    "#,
    )
    .unwrap();
}

#[test]
fn spell_parent_survives_color_collection() {
    let env = WowLuaEnv::new().unwrap();
    collect_in_colors(&env);
    env.exec("local t = C_TooltipInfo.GetSpellByID(19750); CheckTip(t, 'Flash of Light'); assert(t.id == 19750 and t.wordWrapMinWidth == 240)").unwrap();
}

#[test]
fn action_parent_survives_color_collection() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().action_bars.insert(5, 19750);
    collect_in_colors(&env);
    env.exec(
        r#"
        local original = CreateColor
        HIGHLIGHT_FONT_COLOR = {r=0.25, g=0.75, b=0.5}
        CreateColor = function(r, g, b, a)
            -- R81 Q3: binding is removed; collect at the remaining highlight color boundary.
            if r == 0.25 and g == 0.75 and b == 0.5 then
                return original(r, g, b, a)
            end
            return {r=r, g=g, b=b, a=a}
        end
        local t = C_TooltipInfo.GetAction(5)
        CheckTip(t, 'Flash of Light')
        for _, line in ipairs(t.lines) do
            assert(not line.leftText:find('Key bound') and line.leftText ~= 'Not bound')
        end
    "#,
    )
    .unwrap();
}

#[test]
fn unit_parent_survives_color_collection() {
    let env = WowLuaEnv::new().unwrap();
    let name = env.state().borrow().player.name.clone();
    collect_in_colors(&env);
    env.exec(&format!("local t = C_TooltipInfo.GetUnit('player'); CheckTip(t, {name:?}); assert(#t.lines == 4 and type(t.guid) == 'string')")).unwrap();
}

#[test]
fn currency_parent_survives_color_collection() {
    let env = WowLuaEnv::new().unwrap();
    collect_in_colors(&env);
    // WM97 DQ6: composition carries quantity as data; the name line keeps the
    // allocated color parent that must survive collection.
    env.exec("local t = C_TooltipInfo.GetCurrencyByID(2245, 99); assert(t.id == 2245 and t.quantity == 99); assert(#t.lines == 1 and t.lines[1].leftText == 'Valorstones'); assert(t.lines[1].leftColor ~= nil and colorCalls > 0)").unwrap();
}

#[test]
fn toy_parent_survives_color_collection() {
    let env = WowLuaEnv::new().unwrap();
    collect_in_colors(&env);
    env.exec("local t = C_TooltipInfo.GetToyByItemID(166779); CheckTip(t, 'Transmorpher Beacon'); assert(t.id == 166779)").unwrap();
}

#[test]
fn aura_parent_survives_color_collection() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.buffs.clear();
    env.exec("A_Admin.AddBuff(19750, 'Audit Aura', 136243, 60, 1)")
        .unwrap();
    collect_in_colors(&env);
    env.exec("local t = C_TooltipInfo.GetUnitAura('player', 1, 'HELPFUL'); CheckTip(t, 'Audit Aura'); assert(#t.lines == 3)").unwrap();
}

#[test]
fn pet_parent_survives_markup_color_collection() {
    let env = WowLuaEnv::new().unwrap();
    let id = {
        let mut state = env.state().borrow_mut();
        let pet = state.world.pets.first_mut().unwrap();
        pet.name = "|cffff0000Audit Pet|r".into();
        pet.pet_id.clone()
    };
    collect_in_colors(&env);
    env.exec(&format!("local t = C_TooltipInfo.GetCompanionPet({id:?}); CheckTip(t, D56 and '|cffff0000Audit Pet|r' or 'Audit Pet'); assert(t.id == {id:?})")).unwrap();
}

#[test]
fn achievement_parent_survives_markup_color_collection() {
    let env = WowLuaEnv::new().unwrap();
    let id = {
        let mut state = env.state().borrow_mut();
        let (&id, achievement) = state.achievements.iter_mut().next().unwrap();
        achievement.name = "|cffff0000Audit Achievement|r".into();
        id
    };
    collect_in_colors(&env);
    env.exec(&format!("CheckTip(C_TooltipInfo.GetAchievementByID({id}), D56 and '|cffff0000Audit Achievement|r' or 'Audit Achievement')")).unwrap();
}

#[test]
fn mount_parent_survives_markup_color_collection() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        let mount = state
            .world
            .mounts
            .iter_mut()
            .find(|m| m.spell_id == 23338)
            .unwrap();
        mount.spell_id = 4_000_000_001;
        mount.name = "|cffff0000Audit Mount|r".into();
    }
    collect_in_colors(&env);
    env.exec("local t = C_TooltipInfo.GetMountBySpellID(4000000001); CheckTip(t, D56 and '|cffff0000Audit Mount|r' or 'Audit Mount'); assert(t.id == 4000000001 and t.wordWrapMinWidth == 240)").unwrap();
}

#[test]
fn markup_segments_and_shapeshift_parent_survive_color_collection() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().shapeshift_forms[0].name = "Plain |cffff0000Red|r End".into();
    collect_in_colors(&env);
    env.exec(r#"
        local tip = C_TooltipInfo.GetShapeshift(1)
        CheckTip(tip, D56 and 'Plain |cffff0000Red|r End' or 'Plain Red End')
        assert(tip.id == 465)
        local segments = tip.lines[1].leftColorSegments
        if D56 then
            assert(segments == nil, 'D56 copies markup, no common segment array')
            assert(tip.lines[1].leftColor.a == nil)
            assert(colorCalls == 1, 'collect in the supplied side color factory')
        else
            assert(#segments == 3)
            assert(segments[1].text == 'Plain ' and segments[2].text == 'Red' and segments[3].text == ' End')
            local r,g,b = segments[2].color:GetRGB()
            assert(r == 1 and g == 0 and b == 0)
            assert(colorCalls == 3)
        end
    "#).unwrap();
}
