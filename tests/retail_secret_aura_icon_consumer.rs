#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common::aura_container_harness::{assert_no_lua_errors, aura, load_env, settle};

#[test]
fn retail_tainted_nameplate_container_sets_secret_icon_through_original_button() {
    let env = load_env();
    let mut nameplate_aura = aura(35_395);
    nameplate_aura.is_helpful = false;
    nameplate_aura.is_raid = false;
    nameplate_aura.is_nameplate_only = true;
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![nameplate_aura];
        state.restricted_actions[0] = 2;
    }

    env.exec(
        r#"
        ApplyIconHookCalls = 0
        SecretIconHookCalls = 0
        SecretIconAtApplyIcon = false

        local function createNameplateContainer()
            assert(not issecure(), 'AuraKit setup must run as tainted code')
            hooksecurefunc(CustomAuraButtonPrivateMixin, 'ApplyIcon', function(_, _, auraData)
                ApplyIconHookCalls = ApplyIconHookCalls + 1
                if auraData and issecretvalue(auraData.icon) then
                    SecretIconHookCalls = SecretIconHookCalls + 1
                    SecretIconAtApplyIcon = true
                end
            end)

            local container = CreateFrame('AuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
            container:SetPoint('CENTER', UIParent, 'CENTER')
            container:SetSize(1, 1)
            local function initializeFrame(button)
                local icon = button:CreateTexture(nil, 'ARTWORK')
                icon:SetAllPoints(button)
                button:SetIcon(icon)
            end
            debug.setobjecttaint(initializeFrame, 'EllesmereUINameplates')
            container:AddAuraGroup('np:debuffs',
                'HARMFUL PLAYER INCLUDE_NAME_PLATE_ONLY !CROWD_CONTROL', {
                    maxFrameCount = 1,
                    initializeFrame = initializeFrame,
                })
            container:SetUnit('player')
            container:UpdateAllAuras()
            container:Show()
        end
        debug.setobjecttaint(createNameplateContainer, 'EllesmereUINameplates')
        createNameplateContainer()
        "#,
    )
    .expect("tainted nameplate setup must create the original aura-container consumer");
    settle(&env);

    let observation = env
        .eval::<String>(
            "return table.concat({tostring(ApplyIconHookCalls), tostring(SecretIconHookCalls), tostring(SecretIconAtApplyIcon)}, ',')",
        )
        .expect("the original ApplyIcon hook must report its observations");
    println!("ApplyIcon hook calls, secret-icon calls, secret icon observed: {observation}");
    let mut fields = observation.split(',');
    let apply_icon_calls = fields.next().unwrap().parse::<usize>().unwrap();
    let secret_icon_calls = fields.next().unwrap().parse::<usize>().unwrap();
    let secret_icon_observed = fields.next().unwrap() == "true";

    assert_no_lua_errors(&env);
    assert!(apply_icon_calls > 0, "the original ApplyIcon hook must run");
    assert!(
        secret_icon_calls > 0 && secret_icon_observed,
        "the original ApplyIcon must receive a secret aura icon"
    );
}
