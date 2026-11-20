#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::c_api::private_aura_data::PrivateAuraEntry;
use wow_ui_sim::lua_api::state::AuraInfo;
use wow_ui_sim::lua_api::WowLuaEnv;

const CONSUMER_PROBE: &str = r#"
    local privateUI = rawget(__secureenv, 'PrivateAuraAnchorContainerMixin')
    local privateAuras = rawget(__secureenv, 'PrivateAuras')
    local watcherMixin = privateAuras.PrivateAuraUnitWatcher
    local private = rawget(__secureenv, 'C_UnitAurasPrivate')
    local anchorSeen = {}
    local container = {
        watcher = { GetUnit = function() return 'player' end },
        ClearPriorityTables = function() end,
        ProcessAura = function(self, aura) anchorSeen[aura.auraInstanceID] = aura end,
    }
    privateUI.ParseAllAuras(container)
    assert(anchorSeen[301].spellId == 301 and anchorSeen[302].spellId == 302)
    assert(anchorSeen[301].hideOnPartyFrames and anchorSeen[301].canActivePlayerDispel)
    local firstRead = private.GetAllPrivateAuras('player')
    firstRead[1].spellId = 0
    assert(private.GetAllPrivateAuras('player')[1].spellId == 301)

    local returned = privateUI.GetUpdatedAuraByInstance(container, true, 301)
    assert(returned.auraInstanceID == 301 and returned.dispelName == 'Magic')
    returned.spellId = 0
    assert(privateUI.GetUpdatedAuraByInstance(container, true, 301).spellId == 301)

    local parsed
    local watcher = {
        unit = 'player', auras = { Clear = function() end }, anchors = {},
        ClearPriorityTables = function() end,
        ProcessAuras = function(_, full, auras)
            assert(full == true)
            parsed = auras
        end,
        UpdateAuraByIndex = function() end,
    }
    watcherMixin.ParseAllAuras(watcher)
    assert(parsed[1].auraInstanceID == 301 and parsed[2].auraInstanceID == 302)

    watcher.auras[301] = true
    assert(watcherMixin.HandleUpdateInfo(watcher, true, {
        updatedAuraInstanceIDs = { 301 },
    }) == true)
    assert(watcher.auras[301].auraInstanceID == 301 and watcher.auras[301].isPrivate == true)

    local ids = AuraContainerPrivateAuraSourceMixin:GetAllAuraInstanceIDs('player', 'HARMFUL')
    assert(#ids == 2 and ids[1] == 301 and ids[2] == 302)
    assert(AuraContainerPrivateAuraSourceMixin:GetAuraCasterGUID('player', 301) == 'Player-1-301')
"#;

fn load_selected_private_aura_ui() -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_PrivateAurasUI", "Blizzard_AuraContainer"],
        &[],
    );
    env
}

fn entry(id: i32, hide: bool, can_dispel: bool) -> PrivateAuraEntry {
    let mut aura = AuraInfo {
        name: format!("Private {id}"),
        spell_id: id,
        icon: id,
        duration: 30.0,
        expiration_time: 20.0,
        applications: 1,
        source_unit: "player".into(),
        is_helpful: false,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: false,
        is_from_player_or_player_pet: false,
        dispel_type: Some("Magic".into()),
        aura_instance_id: id,
    };
    aura.is_helpful = true;
    PrivateAuraEntry {
        aura,
        can_active_player_dispel: can_dispel,
        hide_on_party_frames: hide,
        source_guid: Some(format!("Player-1-{id}")),
    }
}

fn supply_entries(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    let entries = vec![entry(301, true, true), entry(302, false, false)];
    sim.private_aura_data
        .private_auras_by_unit
        .insert("player".into(), entries.clone());
    sim.private_aura_data.aura_data_by_unit.insert(
        "player".into(),
        entries
            .into_iter()
            .map(|aura| (aura.aura.aura_instance_id, aura))
            .collect(),
    );
}

#[test]
fn private_aura_state_drives_selected_blizzard_consumers() {
    let env = load_selected_private_aura_ui();
    supply_entries(&env);
    env.exec(CONSUMER_PROBE)
        .expect("selected private-aura consumer call sites should observe host entries");
}
