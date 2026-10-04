//! Independent native callback and sort proofs. Additional-template failure is archived separately.
#![cfg(feature = "retail-12-1-0")]
use wow_ui_sim::lua_api::{WowLuaEnv, state::AuraInfo};

#[test]
fn patch_12_1_0_aura_group_options_apply_additional_template_size_and_alpha() {
    let env = load_container_env();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("AuditAuraExternalTemplates.toc");
    std::fs::write(&toc, "Templates.xml\n").unwrap();
    std::fs::write(root.path().join("Templates.xml"), r#"<Ui>
        <AuraButton name="AuditAuraSize" virtual="true"><Size x="37" y="29"/></AuraButton>
        <AuraButton name="AuditAuraAlpha" virtual="true" alpha="0.5"/>
    </Ui>"#).unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(r#"
        local container = CreateFrame('ManagedAuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
        local calls = 0
        container:AddAuraGroup('external-templates', 'HELPFUL', {
            templateNames={'AuditAuraSize', 'AuditAuraAlpha'},
            initializeFrame=function(frame)
                assert(frame:GetWidth() == 37 and frame:GetHeight() == 29,
                    'additional size template applied before callback')
                assert(frame:GetAlpha() == 0.5, 'additional alpha template applied before callback')
                local icon = frame:CreateTexture(nil, 'BACKGROUND')
                frame:SetIcon(icon)
                assert(frame:GetIcon() == icon, 'base CustomAuraButtonTemplate retained')
                calls = calls + 1
            end,
        })
        assert(calls > 0 and calls == container:GetAuraGroupFrameCount('external-templates'))
    "#).unwrap();
}

fn load_container_env() -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui, &["Blizzard_AuraContainer"], &[],
    ).0
}

#[test]
fn patch_12_1_0_aura_group_options_initialize_each_native_button_without_extra_templates() {
    let env = load_container_env();
    env.exec(r#"
        local container = CreateFrame('ManagedAuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
        local callbacks, seen = 0, {}
        local function initialize(frame)
            assert(debug.getstacktaint() == 'AuditInitAddon', 'callback retains addon taint')
            assert(frame:GetObjectType() == 'AuraButton' and frame:GetParent() == container)
            assert(not seen[frame], 'callback exactly once per created frame')
            seen[frame] = true
            callbacks = callbacks + 1
            frame:SetSize(23, 31)
            local icon = frame:CreateTexture(nil, 'BACKGROUND')
            icon:SetAllPoints(frame)
            frame:SetIcon(icon)
            assert(frame:GetIcon() == icon, 'base CustomAuraButtonTemplate methods retained')
        end
        debug.setobjecttaint(initialize, 'AuditInitAddon')
        container:AddAuraGroup('callback:group', 'HELPFUL', {initializeFrame=initialize})
        local count = container:GetAuraGroupFrameCount('callback:group')
        assert(count > 0 and callbacks == count, 'every created button initialized')
        for index=1,count do
            local frame = container:GetAuraGroupFrame('callback:group', index)
            assert(seen[frame] and frame:GetWidth() == 23 and frame:GetHeight() == 31)
            assert(frame:GetIcon():GetNumPoints() > 0)
        end
        assert(issecure())
    "#).unwrap();
}

#[test]
fn patch_12_1_0_aura_group_options_sort_direction_changes_actual_icon_positions() {
    let env = load_container_env();
    env.state().borrow_mut().player.buffs = [33, 11, 22].into_iter().map(|id| AuraInfo {
        name: format!("Audit {id}"), spell_id: 1000 + id, icon: id,
        duration: 30.0, expiration_time: 30.0, applications: 1,
        source_unit: "player".into(), is_helpful: true, is_raid: true,
        is_nameplate_only: false, is_stealable: false, can_apply_aura: true,
        is_from_player_or_player_pet: true, dispel_type: None, aura_instance_id: id,
    }).collect();
    env.exec(r#"
        AuditSort = CreateFrame('ManagedAuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
        AuditSort:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 100, -100)
        AuditSort:SetUnit('player')
        AuditSort:AddAuraGroup('sorted', 'HELPFUL', {
            sortMethod=AuraContainerSortMethod.AuraInstanceIDOnly,
            sortDirection=AuraContainerSortDirection.Normal,
            initializeFrame=function(frame)
                frame:SetSize(20, 20)
                local icon = frame:CreateTexture(nil, 'BACKGROUND')
                icon:SetAllPoints(frame)
                frame:SetIcon(icon)
            end,
        })
        function AuditIconOrder()
            local positions = {}
            for index=1,AuditSort:GetAuraGroupFrameCount('sorted') do
                local frame = AuditSort:GetAuraGroupFrame('sorted', index)
                local shown = frame:IsShown()
                if issecretvalue(shown) then shown = secretunwrap(shown) end
                if shown then
                    local icon = frame:GetIcon():GetTexture()
                    if issecretvalue(icon) then icon = secretunwrap(icon) end
                    local x = frame:GetLeft()
                    if issecretvalue(x) then x = secretunwrap(x) end
                    assert(type(x) == 'number' and frame:GetNumPoints() > 0)
                    positions[#positions+1] = {x=x, icon=icon}
                end
            end
            assert(#positions == 3, 'three actual shown aura icons required')
            table.sort(positions, function(a,b) return a.x < b.x end)
            assert(positions[1].x < positions[2].x and positions[2].x < positions[3].x)
            return tostring(positions[1].icon)..','..tostring(positions[2].icon)..','..tostring(positions[3].icon)
        end
    "#).unwrap();
    env.fire_on_update(0.016).unwrap();
    assert_eq!(env.eval::<String>("return AuditIconOrder()").unwrap(), "11,22,33");
    env.exec("AuditSort:SetAuraGroupSortMethod('sorted', AuraContainerSortMethod.AuraInstanceIDOnly, AuraContainerSortDirection.Reverse)").unwrap();
    env.fire_on_update(0.016).unwrap();
    assert_eq!(env.eval::<String>("return AuditIconOrder()").unwrap(), "33,22,11");
    assert!(env.state().borrow().lua_errors.is_empty(), "{:?}", env.state().borrow().lua_errors);
}
