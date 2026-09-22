use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn duplicate_explicit_frames_keep_parent_children_and_visibility() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        DuplicateHost = CreateFrame('Frame', nil, UIParent)
        DuplicateFirst = CreateFrame('Frame', 'DuplicateExplicit', DuplicateHost)
        DuplicateChild = CreateFrame('Frame', 'DuplicateExplicitChild', DuplicateFirst)
        DuplicateFirst.Child = DuplicateChild
        DuplicateFirst:SetSize(45, 23)
    "#,
    )
    .unwrap();
    let first_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("DuplicateExplicit")
        .unwrap();
    env.exec(
        r#"
        DuplicateSecond = CreateFrame('Frame', 'DuplicateExplicit', DuplicateHost)
        assert(DuplicateFirst ~= DuplicateSecond)
        assert(DuplicateExplicit == DuplicateSecond)
        assert(DuplicateFirst:GetParent() == DuplicateHost, 'first parent lost')
        assert(DuplicateChild:GetParent() == DuplicateFirst, 'first child migrated')
        assert(DuplicateFirst:IsShown() and DuplicateFirst:IsVisible(), 'first hidden')
        assert(DuplicateFirst:GetWidth() == 45)
        assert(DuplicateFirst.Child == DuplicateChild and DuplicateSecond.Child == nil)
        assert(DuplicateHost:GetNumChildren() == 2)
    "#,
    )
    .unwrap();
    let state = env.state().borrow();
    let second_id = state.widgets.get_id_by_name("DuplicateExplicit").unwrap();
    let child_id = state
        .widgets
        .get_id_by_name("DuplicateExplicitChild")
        .unwrap();
    let first = state.widgets.get(first_id).unwrap();
    let second = state.widgets.get(second_id).unwrap();
    assert_ne!(first_id, second_id);
    assert_eq!(first.parent_id, second.parent_id);
    assert!(first.visible);
    assert_eq!(first.children, vec![child_id]);
    assert_eq!(
        state.widgets.get(child_id).unwrap().parent_id,
        Some(first_id)
    );
    assert!(second.children.is_empty());
    let host = state.widgets.get(first.parent_id.unwrap()).unwrap();
    assert!(host.children.contains(&first_id) && host.children.contains(&second_id));
}

#[test]
fn duplicate_explicit_frames_under_different_parents_keep_hidden_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local parent1 = CreateFrame('Frame', nil, UIParent)
        local parent2 = CreateFrame('Frame', nil, UIParent)
        local first = CreateFrame('Frame', 'DuplicateDifferentParents', parent1)
        local child = CreateFrame('Frame', nil, first)
        first:Hide()
        local second = CreateFrame('Frame', 'DuplicateDifferentParents', parent2)
        assert(first:GetParent() == parent1 and second:GetParent() == parent2)
        assert(child:GetParent() == first)
        assert(not first:IsShown() and second:IsShown())
        assert(parent1:GetNumChildren() == 1 and parent2:GetNumChildren() == 1)
        first:Show()
        assert(first:IsVisible() and child:IsVisible())
        assert(DuplicateDifferentParents == second)
    "#,
    )
    .unwrap();
}

#[test]
fn bootstrap_placeholders_migrate_children_only_on_first_definition() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        -- GameTooltip is built in Rust; HelpFrame is a Lua bootstrap placeholder.
        for _, name in ipairs({'GameTooltip', 'HelpFrame'}) do
            local placeholder = assert(_G[name])
            placeholder:Show()
            local child = CreateFrame('Frame', nil, placeholder)
            local kind = placeholder:GetObjectType()
            local definition = CreateFrame(kind, name, UIParent)
            assert(not placeholder:IsShown() and placeholder:GetParent() == nil)
            assert(child:GetParent() == definition, name .. ' placeholder child stranded')
            assert(definition ~= placeholder and _G[name] == definition)
            local duplicate = CreateFrame(kind, name, UIParent)
            assert(definition:IsShown() and definition:GetParent() == UIParent)
            assert(child:GetParent() == definition, name .. ' explicit child migrated')
            assert(_G[name] == duplicate)
        end
    "#,
    )
    .unwrap();
}

#[test]
fn duplicate_engine_root_names_preserve_original_global_and_children() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({'UIParent', 'WorldFrame'}) do
            local root = _G[name]
            local child = CreateFrame('Frame', nil, root)
            local duplicate = CreateFrame('Frame', name)
            assert(root ~= duplicate and _G[name] == root)
            assert(child:GetParent() == root and root:IsShown())
        end
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn duplicate_native_aura_containers_keep_player_and_target_slots() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &[
            "Blizzard_RestrictedAddOnEnvironment",
            "Blizzard_AuraContainer",
        ],
        &[],
    );
    env.exec(r#"
        local button = CreateFrame('Frame', nil, UIParent)
        local containers, slots = {}, {}
        for _, unit in ipairs({'player', 'target'}) do
            local container = CreateFrame('AuraContainer', 'DuplicateABAContainer', button,
                'CustomAuraContainerTemplate')
            container:SetUnit(unit)
            local slot = container:AddAuraSlot('ABA', unit == 'player' and 'HELPFUL|PLAYER' or 'HARMFUL|PLAYER', {
                templateNames = {'CustomAuraButtonTemplate'},
            })
            containers[unit], slots[unit] = container, slot
        end
        assert(containers.player ~= containers.target)
        assert(DuplicateABAContainer == containers.target)
        assert(button:GetNumChildren() == 2, 'duplicate container retired')
        for _, unit in ipairs({'player', 'target'}) do
            local container = containers[unit]
            assert(container:GetParent() == button and container:IsShown())
            assert(container:GetUnit() == unit)
            assert(container:GetAuraSlotFrame('ABA') == slots[unit])
            assert(slots[unit]:GetParent() == container, 'aura slot migrated to other unit')
        end
    "#).unwrap();
}
