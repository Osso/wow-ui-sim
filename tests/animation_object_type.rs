//! Animation ancestry comes from registered animation state, not Lua methods.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn animation_object_type_recognizes_registered_subtypes_and_preserves_exact_names() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local owner = CreateFrame('Frame')
        local group = owner:CreateAnimationGroup()
        for _, kind in ipairs({'Animation', 'Alpha', 'Translation', 'Scale', 'Rotation',
            'LineTranslation', 'LineScale', 'Path', 'FlipBook', 'VertexColor', 'RadialProgress'}) do
            local animation = group:CreateAnimation(kind)
            assert(animation:GetObjectType() == kind)
            assert(animation:IsObjectType(kind))
            assert(animation:IsObjectType('Animation'), kind .. ' must inherit Animation')
            assert(animation:IsObjectType('aNiMaTiOn'))
            assert(select('#', animation:IsObjectType('Animation')) == 1)
            assert(not animation:IsObjectType('AnimationGroup'))
            assert(not animation:IsObjectType('Button'))
            assert(animation:IsObjectType('Region'), 'preserve existing backing classification')
        end
        assert(group:GetObjectType() == 'AnimationGroup')
        assert(group:IsObjectType('AnimationGroup'))
        assert(not group:IsObjectType('Animation'))
        local point = group:CreateAnimation('Path'):CreateControlPoint()
        assert(point:GetObjectType() == 'ControlPoint')
        assert(not point:IsObjectType('Animation'))
        local lookalike = CreateFrame('Frame')
        lookalike.GetObjectType = function() return 'Translation' end
        lookalike.GetDuration = function() return 1 end
        assert(not lookalike:IsObjectType('Animation'), 'methods do not create animation state')
        assert(not owner:IsObjectType('Animation'))
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(feature = "client-wowforever")]
fn animation_object_type_selected_aura_container_adopts_translation_group() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_AuraContainer"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_AuraContainer"));
    env.exec(
        r#"
        local util = assert(__secureenv.AuraContainerUtil, 'selected AuraContainerUtil is missing')
        local source = debug.getinfo(util.InitializeInboundAnimationGroup, 'S').source
        assert(source:find('Blizzard_AuraContainerUtil.lua', 1, true), source)
        local owner = CreateFrame('AuraButton', nil, UIParent, 'CustomAuraButtonTemplate')
        assert(rawget(owner, '__wowUseForbiddenObjectTable') == true,
            'the selected native owner uses forbidden-object references')
        local target = owner:CreateTexture()
        local group = target:CreateAnimationGroup()
        local translation = group:CreateAnimation('Translation')
        translation:SetTarget(target)
        local alpha = group:CreateAnimation('Alpha')
        alpha:SetTarget(target)
        local inboundGroup = GetForbiddenObjectTable(group)
        local inboundOwner = GetForbiddenObjectTable(owner)
        local requireAnimation = util.RequireObjectType('Animation')
        local valid, validationError = pcall(requireAnimation, GetForbiddenObjectTable(translation))
        assert(valid, 'Translation validation failed: ' .. tostring(validationError))
        assert(not pcall(requireAnimation, inboundOwner), 'reject constant true')
        assert(not pcall(requireAnimation, inboundGroup), 'group is not an Animation')
        local adopted = util.InitializeInboundAnimationGroup(inboundGroup, inboundOwner)
        assert(GetForbiddenObjectTable(adopted) == inboundGroup,
            'adoption must return the supplied animation group across the environment boundary')
        assert(translation:GetObjectType() == 'Translation', 'Translation exact type changed')
        assert(alpha:GetObjectType() == 'Alpha', 'Alpha exact type changed')
        "#,
    )
    .expect("selected original inbound validator must accept genuine animation subclasses");
    assert!(env.state().borrow().lua_errors.is_empty());
}
