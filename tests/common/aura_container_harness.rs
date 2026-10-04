//! Managed AuraContainer harness for 12.1.0 proofs.
//!
//! Loads the real cached `Blizzard_AuraContainer` closure, seeds the native
//! player aura model (`SimState.player.buffs`), and delivers changes through
//! `UNIT_AURA` dispatch. Containers are created by addon-tainted Lua through
//! the cached `CustomAuraContainerTemplate`; nothing calls Blizzard internals.

use wow_ui_sim::lua_api::{WowLuaEnv, state::AuraInfo};

/// Lua helpers shared by every harness test.
///
/// - `AuditAddon(fn, ...)` runs `fn` as code owned by the `AuditAuraAddon` addon.
/// - `AuditContainer(groups)` creates a player-unit ManagedAuraContainer from
///   addon code; `groups` is a list of `{key, filter, options}` passed to
///   `AddAuraGroup`, each frame receiving a 20x20 icon unless overridden.
/// - `AuditShown(container, key)` lists shown frames of a group as
///   `{icon=, x=, y=, frame=}` ordered by group index.
/// - `AuditIcons(container, key)` joins the shown icons in screen order.
const PRELUDE: &str = r#"
    AUDIT_ADDON = 'AuditAuraAddon'
    function AuditPlain(value)
        if issecretvalue(value) then return secretunwrap(value) end
        return value
    end
    function AuditAddon(fn, ...)
        debug.setobjecttaint(fn, AUDIT_ADDON)
        return fn(...)
    end
    function AuditInitIcon(frame)
        frame:SetSize(20, 20)
        local icon = frame:CreateTexture(nil, 'BACKGROUND')
        icon:SetAllPoints(frame)
        frame:SetIcon(icon)
    end
    function AuditContainer(groups, configure)
        return AuditAddon(function()
            local container = CreateFrame('ManagedAuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
            container:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 100, -100)
            container:SetUnit('player')
            if configure then configure(container) end
            for _, group in ipairs(groups or {}) do
                local options = group[3] or {}
                options.initializeFrame = options.initializeFrame or AuditInitIcon
                debug.setobjecttaint(options.initializeFrame, AUDIT_ADDON)
                container:AddAuraGroup(group[1], group[2], options)
            end
            return container
        end)
    end
    function AuditFrameIcon(frame)
        local icon = frame:GetIcon()
        return icon and AuditPlain(icon:GetTexture())
    end
    function AuditIsShown(frame)
        return AuditPlain(frame:IsShown()) == true
    end
    function AuditShown(container, key)
        local shown = {}
        for index = 1, container:GetAuraGroupFrameCount(key) do
            local frame = container:GetAuraGroupFrame(key, index)
            if AuditIsShown(frame) then
                shown[#shown + 1] = {
                    frame = frame,
                    icon = AuditFrameIcon(frame),
                    x = AuditPlain(frame:GetLeft()),
                    y = AuditPlain(frame:GetTop()),
                }
            end
        end
        return shown
    end
    function AuditIcons(container, key)
        local shown = AuditShown(container, key)
        table.sort(shown, function(a, b)
            if a.y ~= b.y then return a.y > b.y end
            return a.x < b.x
        end)
        local icons = {}
        for index, entry in ipairs(shown) do icons[index] = tostring(entry.icon) end
        return table.concat(icons, ',')
    end
"#;

/// Real cached `Blizzard_AuraContainer` closure with harness helpers.
pub fn load_env() -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = super::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_AuraContainer"],
        &[],
    );
    env.exec(PRELUDE).unwrap();
    env
}

/// A helpful 30-second player-cast aura whose spell ID, icon, and instance ID
/// all equal `id`, so shown icons identify the aura.
pub fn aura(id: i32) -> AuraInfo {
    AuraInfo {
        name: format!("Audit Aura {id}"),
        spell_id: id,
        icon: id,
        duration: 30.0,
        expiration_time: 30.0,
        applications: 1,
        source_unit: "player".into(),
        is_helpful: true,
        is_raid: true,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: id,
    }
}

/// A harmful aura on the player cast by `target`.
pub fn debuff(id: i32, dispel_type: Option<&str>) -> AuraInfo {
    AuraInfo {
        is_helpful: false,
        is_raid: false,
        can_apply_aura: false,
        is_from_player_or_player_pet: false,
        source_unit: "target".into(),
        dispel_type: dispel_type.map(str::to_string),
        ..aura(id)
    }
}

/// Replace the player's auras and notify listeners with a full `UNIT_AURA`.
pub fn set_player_auras(env: &WowLuaEnv, auras: Vec<AuraInfo>) {
    env.state().borrow_mut().player.buffs = auras;
    env.fire_unit_aura_full_update("player").unwrap();
}

/// Run one frame so dirty containers process aura and layout phases.
pub fn settle(env: &WowLuaEnv) {
    env.fire_on_update(0.016).unwrap();
}

pub fn icons(env: &WowLuaEnv, container: &str, key: &str) -> String {
    env.eval::<String>(&format!("return AuditIcons({container}, '{key}')"))
        .unwrap()
}

/// Native widget id of the frame a Lua expression evaluates to.
pub fn frame_id(env: &WowLuaEnv, expression: &str) -> u64 {
    let debug_name = env
        .eval::<String>(&format!("return ({expression}):GetDebugName()"))
        .unwrap();
    debug_name
        .rsplit(':')
        .next()
        .and_then(|id| id.parse().ok())
        .unwrap_or_else(|| panic!("{expression} has no unnamed debug id: {debug_name}"))
}

pub fn assert_no_lua_errors(env: &WowLuaEnv) {
    let errors = env.state().borrow().lua_errors.clone();
    assert!(errors.is_empty(), "{errors:?}");
}
