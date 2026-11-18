#![cfg(feature = "client-mists")]

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "common/blizzard_addon_closure.rs"]
mod closure;

const MANAGER_SOURCE: &str = "Blizzard_CombatAudioAlerts/Blizzard_CombatAudioAlertManager.lua";

/// Player and target health-percent CVars, then whether the original
/// `RegisterForUnitHealth` table call leaves `UNIT_HEALTH` registered with an
/// empty unit filter (a later zero-unit call returns false) or unregistered.
const HEALTH_CASES: [(&str, &str, bool); 4] = [
    ("2", "0", true),
    ("0", "2", true),
    ("2", "2", true),
    ("0", "0", false),
];

fn load_combat_audio_manager(player_percent: &str, target_percent: &str) -> WowLuaEnv {
    assert_eq!(ACTIVE, ClientProfile::Mists);
    let ui = closure::blizzard_ui_dir();
    let env = closure::new_blizzard_addon_env(&ui);
    {
        let state = env.state();
        let sim = state.borrow();
        sim.cvars.set_imported("CAAEnabled", "1");
        sim.cvars.set_imported("CAAPartyHealthPercent", "0");
        sim.cvars
            .set_imported("CAAPlayerHealthPercent", player_percent);
        sim.cvars
            .set_imported("CAATargetHealthPercent", target_percent);
    }
    let addons = ["Blizzard_CombatAudioAlerts"];
    let loaded = closure::load_blizzard_addon_closure_into_env(&env, &ui, &addons, &[]);
    assert!(loaded.iter().any(|name| name == addons[0]));
    env
}

fn unit_health_registration(env: &WowLuaEnv) -> String {
    env.eval(&format!(
        r#"
        local source = debug.getinfo(CombatAudioAlertManagerMixin.RegisterForUnitHealth, 'S').source
        assert(source:sub(-#'{MANAGER_SOURCE}') == '{MANAGER_SOURCE}', source)
        local frame = CombatAudioAlertManager
        frame.initDone = true
        local ok, err = pcall(function() frame:RefreshEvents(false) end)
        local registered = frame:IsEventRegistered('UNIT_HEALTH')
        local changed = frame:RegisterUnitEvent('UNIT_HEALTH', {{}})
        return string.format('ok=%s UNIT_HEALTH=%s zeroUnitCallChanged=%s err=%s',
            tostring(ok), tostring(registered), tostring(changed), tostring(err))
        "#
    ))
    .expect("original combat-audio manager refresh should be observable")
}

const UNIT_SCAN_BOUNDARIES: &str = r#"
    local calls = {}
    local frame = CreateFrame("Frame")
    frame:SetScript("OnEvent", function(_, _, unit)
        calls[#calls + 1] = unit
    end)

    frame:RegisterUnitEvent("UNIT_HEALTH", "player", "bogus", "target")
    FireEvent("UNIT_HEALTH", "target")
    assert(table.concat(calls, ",") == "target", "scan continues after unknown unit")

    calls = {}
    frame:RegisterUnitEvent("UNIT_HEALTH", "player", "target", "focus", "pet", "boss1")
    FireEvent("UNIT_HEALTH", "boss1")
    assert(#calls == 0, "fifth unit position is ignored")

    frame:RegisterUnitEvent("UNIT_HEALTH", "player", false, "target")
    FireEvent("UNIT_HEALTH", "target")
    assert(#calls == 0, "non-string stops the scan")
    FireEvent("UNIT_HEALTH", "player")
    assert(table.concat(calls, ",") == "player", "valid units before non-string remain")

    local tableFrame = CreateFrame("Frame")
    local changed = tableFrame:RegisterUnitEvent("UNIT_HEALTH", {"player"}, "target")
    local registered, unit = tableFrame:IsEventRegistered("UNIT_HEALTH")
    assert(changed and registered and unit == nil, "table first registers with an empty filter")

    local unknownFrame = CreateFrame("Frame")
    unknownFrame:SetScript("OnEvent", function(_, _, unit)
        calls[#calls + 1] = unit
    end)
    unknownFrame:RegisterUnitEvent("UNIT_HEALTH", "player", "boss6")
    FireEvent("UNIT_HEALTH", "boss6")
    assert(#calls == 1, "unknown Mists token is skipped")
"#;

#[test]
fn register_unit_event_scans_native_unit_argument_boundaries() {
    let env = WowLuaEnv::new().expect("create Mists unit-event environment");
    env.exec(UNIT_SCAN_BOUNDARIES)
        .expect("Mists unit-event scan follows the verified argument rules");
}

const CAST_SEQUENCE_CONSUMER: &str = r#"
    SecureCmdOptionParse = function(message) return message end
    SecureCmdItemParse = function() return nil end
    CastSpellByName = function() end
    C_Item.GetItemInfo = function() return nil end
    C_Item.GetItemSpell = function() return nil, 123 end
    C_Item.IsEquippableItem = function() return false end
    C_Spell.GetSpellIDForSpellIdentifier = function() return 123 end
    C_SpellBook.FindSpellOverrideByID = function(spellID) return spellID end
    C_SpellBook.FindBaseSpellByID = function(spellID) return spellID end

    local handler
    for index = 1, 20 do
        local key, value = debug.getupvalue(SlashCommandUtil.CheckAddSecureSlashCommand, index)
        if not key then break end
        if key == "addonTbl" then handler = value.SecureCmdList["CASTSEQUENCE"] end
    end
    assert(handler, "original cast-sequence command should be registered")
    local source = debug.getinfo(handler, "S").source
    assert(source:find("CastSequenceManager.lua", 1, true), source)
    handler("Alpha,Beta,Gamma,Delta")
    local function current_index()
        return select(1, QueryCastSequence("Alpha,Beta,Gamma,Delta"))
    end
    local function cast(unit, castID)
        FireEvent("UNIT_SPELLCAST_SENT", unit, "", castID, 123)
        FireEvent("UNIT_SPELLCAST_SUCCEEDED", unit, castID, 123)
    end

    cast("target", 1)
    assert(current_index() == 1, "target is not registered")
    cast("player", 2)
    assert(current_index() == 2, "player event advances cast sequence")
    cast("pet", 3)
    assert(current_index() == 3, "pet event advances cast sequence")
    cast("focus", 4)
    assert(current_index() == 3, "focus is not registered")
"#;

#[test]
fn cast_sequence_consumer_receives_only_registered_units() {
    let ui = closure::blizzard_ui_dir();
    let env = closure::new_blizzard_addon_env(&ui);
    let loaded =
        closure::load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_ChatFrameBase"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_ChatFrameBase"));
    env.exec(CAST_SEQUENCE_CONSUMER)
        .expect("original cast-sequence consumer should filter player and pet events");
}

#[test]
fn combat_audio_manager_table_registers_unit_health_with_empty_filter() {
    let mut observed = Vec::new();
    let mut expected = Vec::new();
    for (player, target, registered) in HEALTH_CASES {
        let env = load_combat_audio_manager(player, target);
        observed.push(unit_health_registration(&env));
        expected.push(format!(
            "ok=true UNIT_HEALTH={registered} zeroUnitCallChanged={} err=nil",
            !registered
        ));
    }
    assert_eq!(observed, expected);
}
