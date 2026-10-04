//! 12.1.0 managed AuraContainer OnSizeChanged restriction: once an aura group is
//! added, layout-driven size changes of the container (and of frames anchored to
//! it) no longer run addon OnSizeChanged handlers, so addons cannot count auras.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::{
    assert_no_lua_errors, aura, load_env, set_player_auras, settle,
};

const SIZE_HELPERS: &str = r#"
    SizeLog = {}
    function RecordSize(tag)
        return function(_, w, h)
            SizeLog[#SizeLog + 1] = string.format('%s=%dx%d', tag, w, h)
        end
    end
    function TakeSizeLog()
        local log = table.concat(SizeLog, ',')
        SizeLog = {}
        return log
    end
"#;

fn take_log(env: &wow_ui_sim::lua_api::WowLuaEnv) -> String {
    env.eval::<String>("return TakeSizeLog()").unwrap()
}

#[test]
fn aura_group_containers_hide_layout_size_changes_from_addon_handlers() {
    let env = load_env();
    env.exec(SIZE_HELPERS).unwrap();
    set_player_auras(&env, vec![aura(101), aura(102)]);
    env.exec(
        r#"
        SizeContainer = AuditContainer({})
        AuditAddon(function()
            SizeContainer:SetScript('OnSizeChanged', RecordSize('container:addon'))
        end)
        SizeContainer:HookScript('OnSizeChanged', RecordSize('container:secure'))
        "#,
    )
    .unwrap();
    settle(&env);
    assert_eq!(
        take_log(&env),
        "container:addon=1x1,container:secure=1x1",
        "before any aura group, addon OnSizeChanged sees the empty container's managed size"
    );

    env.exec(
        r#"
        AuditAddon(function()
            SizeContainer:AddAuraGroup('buffs', 'HELPFUL', {initializeFrame = AuditInitIcon})
        end)
        "#,
    )
    .unwrap();
    settle(&env);
    assert_eq!(
        take_log(&env),
        "container:secure=40x20",
        "the group resizes the container to its auras; only secure handlers observe it"
    );

    env.exec(
        r#"
        -- Assigned by the secure test chunk: reading an addon-written global would
        -- taint the secure hook installed below.
        SizeAnchored = AuditAddon(function()
            local anchored = CreateFrame('Frame', nil, UIParent, 'DisableUntrustedLayoutScriptsTemplate')
            anchored:SetPoint('TOPLEFT', SizeContainer, 'BOTTOMLEFT')
            anchored:SetPoint('TOPRIGHT', SizeContainer, 'BOTTOMRIGHT')
            anchored:SetHeight(5)
            anchored:SetScript('OnSizeChanged', RecordSize('anchored:addon'))
            return anchored
        end)
        "#,
    )
    .unwrap();
    env.exec("SizeAnchored:HookScript('OnSizeChanged', RecordSize('anchored:secure'))")
        .unwrap();
    env.exec(
        r#"
        SizeAnchorRejected = AuditAddon(function()
            local plain = CreateFrame('Frame', nil, UIParent)
            return not pcall(plain.SetPoint, plain, 'TOPLEFT', SizeContainer, 'BOTTOMLEFT')
        end)
        "#,
    )
    .unwrap();
    assert!(
        env.eval::<bool>("return SizeAnchorRejected").unwrap(),
        "frames without the layout aspect cannot anchor to a grouped container"
    );
    settle(&env);
    assert_eq!(take_log(&env), "anchored:secure=40x5");

    // Each aura gained or lost resizes the container and its anchored frame,
    // but the addon handlers that could count auras never run.
    set_player_auras(&env, vec![aura(101), aura(102), aura(103)]);
    settle(&env);
    assert_eq!(
        take_log(&env),
        "container:secure=60x20,anchored:secure=60x5"
    );
    set_player_auras(&env, vec![aura(101)]);
    settle(&env);
    assert_eq!(
        take_log(&env),
        "container:secure=20x20,anchored:secure=20x5"
    );
    assert_no_lua_errors(&env);
}
