#![cfg(all(feature = "retail-12-0-5", any(feature = "profile-retail", feature = "client-ptr")))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn stored_unit(env: &WowLuaEnv) -> Option<String> {
    let sim = env.state().borrow();
    let id = sim.widgets.get_id_by_name("IdentityFollowupModel").unwrap();
    sim.widgets
        .get(id)
        .unwrap()
        .model_state()
        .player_model_state
        .last_unit
        .clone()
}

fn create_bound_model(widget_type: &str) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("model identity environment");
    env.state().borrow_mut().party_group_active = true;
    env.exec(&format!(
        "IdentityFollowupModel = CreateFrame('{widget_type}', 'IdentityFollowupModel')"
    ))
    .expect("create actual model widget through CreateFrame");
    {
        let mut sim = env.state().borrow_mut();
        assert!(sim.party_members.len() >= 2);
        let id = sim.widgets.get_id_by_name("IdentityFollowupModel").unwrap();
        sim.widgets
            .get_mut_visual(id)
            .unwrap()
            .model_state_mut()
            .player_model_state
            .last_unit = Some("party2".to_owned());
    }
    env
}

fn assert_set_unit_identity_contract(widget_type: &str) {
    let env = create_bound_model(widget_type);
    let guid: String = env.eval("return UnitGUID('party1')").unwrap();
    env.state()
        .borrow_mut()
        .identity_secret_guids
        .insert(guid.clone());
    env.exec(
        r#"
        assert(not issecretvalue('party1'))
        local function check(...)
            assert(select('#', ...) == 2, 'pcall status plus exactly one denial result')
            local ok, result = ...
            assert(ok == true and result == nil, 'secret identity denied without error')
            assert(not issecretvalue(result), 'denial nil is public')
        end
        check(pcall(function() return IdentityFollowupModel:SetUnit('party1') end))
        "#,
    )
    .expect("secret identity returns exactly one public nil");
    assert_eq!(stored_unit(&env).as_deref(), Some("party2"));

    env.exec(
        r#"
        local function check(...)
            assert(select('#', ...) == 1, 'missing identity returns exactly one result')
            local result = ...
            assert(result == false and not issecretvalue(result))
        end
        check(IdentityFollowupModel:SetUnit('missing-identity'))
        "#,
    )
    .expect("missing identity returns one public false");
    assert_eq!(stored_unit(&env).as_deref(), Some("party2"));

    env.state().borrow_mut().identity_secret_guids.remove(&guid);
    env.exec(
        r#"
        local function check(...)
            assert(select('#', ...) == 1, 'public assignment returns exactly one result')
            local result = ...
            assert(result == true and not issecretvalue(result))
        end
        check(IdentityFollowupModel:SetUnit('party1'))
        "#,
    )
    .expect("public existing identity assigns after classification clears");
    assert_eq!(stored_unit(&env).as_deref(), Some("party1"));
}

#[test]
fn player_model_set_unit_obeys_identity_contract() {
    assert_set_unit_identity_contract("PlayerModel");
}

#[test]
fn dress_up_model_set_unit_obeys_identity_contract() {
    assert_set_unit_identity_contract("DressUpModel");
}

#[test]
fn cinematic_model_set_unit_obeys_identity_contract() {
    assert_set_unit_identity_contract("CinematicModel");
}

#[test]
fn tabard_model_set_unit_obeys_identity_contract() {
    assert_set_unit_identity_contract("TabardModel");
}

#[test]
fn model_scene_set_unit_obeys_identity_contract() {
    assert_set_unit_identity_contract("ModelScene");
}

#[test]
fn tooltip_set_unit_keeps_content_binding_and_callback() {
    let env = WowLuaEnv::new().expect("tooltip environment");
    env.exec(
        r#"
        TooltipSetUnitCalls = 0
        GameTooltip:SetScript('OnTooltipSetUnit', function(self)
            TooltipSetUnitCalls = TooltipSetUnitCalls + 1
            local name, unit = self:GetUnit()
            assert(name == UnitName('player') and unit == 'player')
        end)
        assert(GameTooltip:SetUnit('player') == true)
        assert(GameTooltip:NumLines() >= 2 and GameTooltip:IsVisible())
        local name, unit = GameTooltip:GetUnit()
        assert(name == UnitName('player') and unit == 'player')
        assert(TooltipSetUnitCalls == 1)
        "#,
    )
    .expect("tooltip dispatcher populates content and fires callback once");
    let sim = env.state().borrow();
    let id = sim.widgets.get_id_by_name("GameTooltip").unwrap();
    let tooltip = sim.tooltips.get(&id).expect("stored tooltip content");
    assert_eq!(tooltip.lines[0].left_text, sim.player.name);
    assert!(tooltip.lines[1].left_text.contains("Level"));
    assert_eq!(
        sim.widgets
            .get(id)
            .unwrap()
            .model_state()
            .player_model_state
            .last_unit,
        None,
        "tooltip must not take the model binding path"
    );
}
