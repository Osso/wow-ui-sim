//! Source-grounded original Wrath callbacks; not full startup or native parity.
#![cfg(feature = "client-wrath")]

use sha2::{Digest, Sha256};
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::{WowLuaEnv, state::PartyMember};
use wow_ui_sim::render::font::WowFontSystem;

fn read_original(name: &str) -> PathBuf {
    let toc = wow_ui_sim::client_profile::blizzard_ui_framexml_toc()
        .expect("SETUP FAILURE: pinned Wrath cache required");
    let root = toc.parent().unwrap().parent().unwrap();
    let path = root.join(name);
    let bytes = std::fs::read(&path).expect("SETUP FAILURE: cached consumer");
    let pins: serde_json::Value =
        serde_json::from_str(include_str!("wrath_residual_a_consumers.json")).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        pins[name].as_str().unwrap(),
        "SETUP FAILURE: changed original {}",
        path.display()
    );
    path
}

fn consumer_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(1600.0, 1200.0);
    env.set_font_system(Rc::new(RefCell::new(WowFontSystem::new_without_casc())));
    env.exec("ResidualErrors = {}; seterrorhandler(function(e) ResidualErrors[#ResidualErrors+1] = tostring(e) end)").unwrap();
    env
}

fn assert_no_errors(env: &WowLuaEnv) {
    env.exec("assert(#ResidualErrors == 0, table.concat(ResidualErrors, '\\n'))")
        .unwrap();
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "{:?}",
        env.state().borrow().lua_errors
    );
}

fn exec_and_audit(env: &WowLuaEnv, code: &str) {
    let result = env.exec(code);
    assert_no_errors(env);
    result.expect("original consumer / behavioral assertion");
}

fn load_lua(env: &WowLuaEnv, name: &str) {
    let source = std::fs::read_to_string(read_original(name)).unwrap();
    let result = env.loader_env().exec(&source);
    assert_no_errors(env);
    result.expect("SETUP FAILURE: complete unchanged Lua chunk");
}

fn load_xml_files(env: &WowLuaEnv, names: &[&str]) {
    let dir = tempfile::tempdir().unwrap();
    let mut toc = String::from("## Interface: 30300\n## Title: WrathResidualA\n");
    for name in names {
        let original = read_original(name);
        let common = dir
            .path()
            .ancestors()
            .find(|base| original.starts_with(base))
            .unwrap();
        let mut relative = PathBuf::new();
        for _ in dir.path().strip_prefix(common).unwrap().components() {
            relative.push("..");
        }
        relative.push(original.strip_prefix(common).unwrap());
        toc.push_str(relative.to_str().unwrap());
        toc.push('\n');
    }
    let path = dir.path().join("WrathResidualA.toc");
    std::fs::write(&path, toc).unwrap();
    let result = load_addon(&env.loader_env(), &path);
    assert_no_errors(env);
    let loaded = result.expect("SETUP FAILURE: original XML loading");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
}

#[test]
fn original_wrath_achievement_addon_event_reads_empty_and_changed_tracking() {
    let env = consumer_env();
    load_lua(&env, "FrameXML/UIParent.lua");
    load_lua(
        &env,
        "AddOns/Blizzard_AchievementUI/Blizzard_AchievementUI.lua",
    );
    // Original :633-639 consumes varargs; :49-55 would reject a nil table key.
    exec_and_audit(
        &env,
        r#"
        AchievementMicroButton = CreateFrame('Button', 'ResidualAchievementMicroButton')
        AchievementMicroButton:Show()
        local f = CreateFrame('Frame')
        AchievementFrameAchievements_OnEvent(f, 'ADDON_LOADED', 'Blizzard_AchievementUI')
        assert(f:IsEventRegistered('TRACKED_ACHIEVEMENT_UPDATE'))
        assert(select('#', GetTrackedAchievements()) == 0)
        SetTrackedAchievement(503, true)
        SetTrackedAchievement(6, true)
        SetTrackedAchievement(503, true)
        AchievementFrameAchievements_OnEvent(f, 'TRACKED_ACHIEVEMENT_UPDATE')
        local ids = {GetTrackedAchievements()}
        table.sort(ids)
        assert(#ids == 2 and ids[1] == 6 and ids[2] == 503)
        UntrackAchievement(6)
        AchievementFrameAchievements_OnEvent(f, 'TRACKED_ACHIEVEMENT_UPDATE')
        assert(select('#', GetTrackedAchievements()) == 1 and GetTrackedAchievements() == 503)
        SetTrackedAchievement(503, false)
        AchievementFrameAchievements_OnEvent(f, 'TRACKED_ACHIEVEMENT_UPDATE')
        assert(select('#', GetTrackedAchievements()) == 0)
    "#,
    );
}

fn seed_aura(env: &WowLuaEnv, helpful: bool) {
    let mut state = env.state().borrow_mut();
    let aura = wow_ui_sim::lua_api::state::AuraInfo {
        name: "Residual aura".into(),
        spell_id: 465,
        icon: 135940,
        duration: 30.0,
        expiration_time: state.start_time.elapsed().as_secs_f64() + 12.0,
        applications: 3,
        source_unit: "player".into(),
        is_helpful: helpful,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: Some("Magic".into()),
        aura_instance_id: 901,
    };
    state.player.buffs.clear();
    state.player.buffs.push(aura);
}

#[test]
fn original_wrath_aura_button_consumes_rank_slot_and_numeric_deadline() {
    let env = consumer_env();
    load_lua(&env, "FrameXML/UIParent.lua");
    exec_and_audit(
        &env,
        "PlayerFrame = CreateFrame('Frame'); PlayerFrame.unit = 'player'; SHOW_BUFF_DURATIONS = '1'; CONSOLIDATE_BUFFS = '0'",
    );
    // These XML templates create the icon/count/duration fields used by :123-228.
    read_original("FrameXML/BuffFrame.lua");
    load_xml_files(
        &env,
        &[
            "FrameXML/Fonts.xml",
            "FrameXML/FontStyles.xml",
            "FrameXML/BuffFrame.xml",
        ],
    );
    for helpful in [true, false] {
        seed_aura(&env, helpful);
        let (prefix, filter, api) = if helpful {
            ("BuffButton", "HELPFUL", "UnitBuff")
        } else {
            ("DebuffButton", "HARMFUL", "UnitDebuff")
        };
        exec_and_audit(
            &env,
            &format!(
                r#"
            local now = GetTime()
            assert(AuraButton_Update('{prefix}', 1, '{filter}') == 1)
            local b = _G['{prefix}1']
            assert(b:IsShown() and b.duration:IsShown() and b.count:IsShown())
            assert(tonumber(b.count:GetText()) == 3)
            assert(b.timeLeft > 10 and b.timeLeft <= 12)
            local name, rank, icon, count, dispel, duration, expiration, source, stealable, consolidated = {api}('player', 1)
            -- Pinned BuffFrame.lua:125 destructures `texture` and passes it to
            -- SetTexture, so the Wrath tuple carries an icon path and a rank
            -- string (empty when the spell has none; see
            -- wrath_casting_aura_contracts). The numeric fileID is the
            -- C_UnitAuras field. Consolidation has no modeled input yet, so the
            -- absent flag stays nil (wrath_startup_defaults.rs:57-59).
            assert(name == 'Residual aura' and type(rank) == 'string' and type(icon) == 'string' and count == 3)
            assert(dispel == 'Magic' and duration == 30 and type(expiration) == 'number')
            assert(expiration > now + 10 and source == 'player' and consolidated == nil)
            assert(_G['{prefix}1Icon']:GetTexture() == icon)
            local data = C_UnitAuras.GetAuraDataByIndex('player', 1, '{filter}')
            assert(data.name == name and data.icon == 135940 and data.applications == count)
            assert(data.duration == duration and data.expirationTime == expiration and data.sourceUnit == source)
            AuraButton_OnUpdate(b, 0.25)
            assert(b.timeLeft > 9 and b.timeLeft <= 11.75)
        "#
            ),
        );
    }
    env.state().borrow_mut().player.buffs.clear();
    exec_and_audit(
        &env,
        r#"
        assert(AuraButton_Update('BuffButton', 1, 'HELPFUL') == nil)
        assert(AuraButton_Update('DebuffButton', 1, 'HARMFUL') == nil)
        assert(not BuffButton1:IsShown() and not BuffButton1.duration:IsShown())
        assert(not DebuffButton1:IsShown() and not DebuffButton1.duration:IsShown())
    "#,
    );
}

#[test]
fn original_wrath_durability_selects_offhand_weapon_or_shield() {
    let env = consumer_env();
    load_lua(&env, "FrameXML/DurabilityFrame.lua");
    // Passive named geometry for the unchanged direct consumer. Full XML/event
    // layout and vehicle-event recovery remain separate parent acceptance work.
    exec_and_audit(
        &env,
        r#"
        DurabilityFrame = CreateFrame('Frame')
        VehicleSeatIndicator = CreateFrame('Frame'); VehicleSeatIndicator:Hide()
        for _, row in pairs(INVENTORY_ALERT_STATUS_SLOTS) do
            DurabilityFrame:CreateTexture('Durability'..row.slot)
        end
        DurabilityFrame:CreateTexture('DurabilityOffWeapon')
    "#,
    );
    for (item, weapon) in [(Some(221165), true), (Some(133503), false), (None, false)] {
        {
            let mut state = env.state().borrow_mut();
            let mut equipment = state.player.equipped_items[&16].clone();
            state.player.equipped_items.remove(&17);
            if let Some(id) = item {
                equipment.item_id = id;
                state.player.equipped_items.insert(17, equipment);
            }
        }
        // Alert status remains the existing empty/zero producer. Observe which texture
        // the original :21-50 selected and reset, not a fabricated positive alert.
        let (selected, other) = if weapon {
            ("DurabilityOffWeapon", "DurabilityShield")
        } else {
            ("DurabilityShield", "DurabilityOffWeapon")
        };
        exec_and_audit(
            &env,
            &format!(
                r#"
            for _, name in ipairs({{'DurabilityShield', 'DurabilityOffWeapon'}}) do
                _G[name]:Show(); _G[name]:SetVertexColor(0.2, 0.3, 0.4, 1)
            end
            DurabilityFrame_SetAlerts()
            assert(OffhandHasWeapon() == {weapon})
            local r,g,b,a = {selected}:GetVertexColor()
            assert(r == 1 and g == 1 and b == 1 and a == 0.5)
            local _,_,_,otherAlpha = {other}:GetVertexColor()
            assert(otherAlpha == 1 and not {other}:IsShown())
        "#
            ),
        );
    }
}

#[test]
fn original_wrath_trainer_dropdown_roundtrips_numeric_filter_flags() {
    let env = consumer_env();
    load_lua(&env, "FrameXML/UIParent.lua");
    exec_and_audit(&env, "StaticPopupDialogs = {}");
    // Explicit supplied filter policy, not asserted native defaults.
    env.state().borrow_mut().wrath_trainer_filters = [true, false, false];
    read_original("FrameXML/UIDropDownMenu.lua");
    read_original("FrameXML/UIDropDownMenuTemplates.xml");
    load_xml_files(
        &env,
        &[
            "FrameXML/Fonts.xml",
            "FrameXML/FontStyles.xml",
            "FrameXML/UIDropDownMenu.xml",
        ],
    );
    load_lua(&env, "AddOns/Blizzard_TrainerUI/Blizzard_TrainerUI.lua");
    exec_and_audit(
        &env,
        r#"
        ClassTrainerListScrollFrameScrollBar = CreateFrame('Slider')
        ClassTrainerListScrollFrameScrollBar:SetMinMaxValues(0, 100)
        local dropdown = CreateFrame('Frame', 'ResidualTrainerFilter', UIParent, 'UIDropDownMenuTemplate')
        ClassTrainerFrameFilterDropDown_OnLoad(dropdown)
        assert(DropDownList1.numButtons == 3)
        local a, u, used = DropDownList1Button1, DropDownList1Button2, DropDownList1Button3
        assert(a.value == 'available' and u.value == 'unavailable' and used.value == 'used')
        assert(UIDropDownMenuButton_GetChecked(a) and not UIDropDownMenuButton_GetChecked(u) and not UIDropDownMenuButton_GetChecked(used))
        -- Actual click callback reads the post-toggle check region, as the menu click path does.
        DropDownList1Button1Check:Hide()
        ClassTrainerListScrollFrameScrollBar:SetValue(20)
        assert(ClassTrainerListScrollFrameScrollBar:GetValue() == 20)
        ClassTrainerFrameFilterDropDown_OnClick(a)
        assert(ClassTrainerListScrollFrameScrollBar:GetValue() == 0)
        ClassTrainerFrameFilterDropDown_OnLoad(dropdown)
        assert(not UIDropDownMenuButton_GetChecked(a) and not GetTrainerServiceTypeFilter('available'))
        DropDownList1Button3Check:Show()
        ClassTrainerFrameFilterDropDown_OnClick(used)
        ClassTrainerFrameFilterDropDown_OnLoad(dropdown)
        assert(UIDropDownMenuButton_GetChecked(used) and GetTrainerServiceTypeFilter('used'))
        DropDownList1Button3Check:Hide()
        ClassTrainerFrameFilterDropDown_OnClick(used)
        ClassTrainerFrameFilterDropDown_OnLoad(dropdown)
        assert(not UIDropDownMenuButton_GetChecked(used) and not GetTrainerServiceTypeFilter('used'))
    "#,
    );
    assert_eq!(
        env.state().borrow().wrath_trainer_filters,
        [false, false, false]
    );
}

#[test]
fn original_wrath_watchframe_empty_timers_emit_no_lines() {
    let env = consumer_env();
    load_lua(&env, "FrameXML/UIParent.lua");
    load_lua(&env, "FrameXML/WatchFrame.lua");
    // Passive pool fixture permits the old one-nil path to reach original SecondsToTime,
    // rather than falsely treating an absent frame cache as the requested RED.
    exec_and_audit(
        &env,
        r#"
        WatchFrame = CreateFrame('Frame')
        WatchFrame.lineCache = { GetFrame = function()
            local f = CreateFrame('Frame', nil, WatchFrame); f:SetHeight(16)
            f.text = f:CreateFontString(nil, 'OVERLAY'); f.text:SetFont('Fonts\\FRIZQT__.TTF', 12)
            f.Reset = function(self) self.text:SetText('') end
            return f
        end }
        local h,w,reserved = WatchFrame_HandleDisplayQuestTimers(WatchFrame, 0, 300, 200)
        assert(h == 0 and w == 0 and reserved == 0)
        assert(WATCHFRAME_NUM_TIMERS == 0 and #WATCHFRAME_TIMERLINES == 0)
        assert(select('#', GetQuestTimers()) == 0)
    "#,
    );
}

fn seed_group(env: &WowLuaEnv, count: usize, local_leader: bool) {
    let mut state = env.state().borrow_mut();
    state.party_group_active = count > 0;
    state.party_leader_index = if local_leader { None } else { Some(0) };
    state.party_members = (0..count)
        .map(|index| PartyMember {
            map_position: None,
            name: format!("Member{index}"),
            name_cached: true,
            connected: true,
            class_index: 2,
            level: 80,
            health: 100,
            health_max: 100,
            power: 100,
            power_max: 100,
            power_type: 0,
            power_type_name: "MANA".into(),
            is_leader: !local_leader && index == 0,
            dead_since: None,
            buffs: vec![],
            debuffs: vec![],
        })
        .collect();
}

#[test]
fn original_wrath_lfg_buttons_follow_raid_and_party_leader_state() {
    let env = consumer_env();
    load_lua(&env, "FrameXML/UIParent.lua");
    load_lua(&env, "FrameXML/LFGFrame.lua");
    load_lua(&env, "FrameXML/LFDFrame.lua");
    load_lua(&env, "FrameXML/LFRFrame.lua");
    exec_and_audit(
        &env,
        r#"
        for _, name in ipairs({'LFDQueueFrameFindGroupButton','LFRQueueFrameFindGroupButton',
            'LFRQueueFrameNoLFRWhileLFDLeaveQueueButton','LFDQueueFramePartyBackfillBackfillButton',
            'LFRQueueFrameAcceptCommentButton','LFDQueueFrameNoLFDWhileLFRLeaveQueueButton'}) do
            CreateFrame('Button', name)
        end
        LFDQueueFramePartyBackfill = CreateFrame('Frame'); LFDQueueFramePartyBackfill:Hide()
        LFRRaidList = nil
        assert(not HasLFGRestrictions())
        assert(GetLFGMode() ~= 'proposal' and GetLFGMode() ~= 'queued' and GetLFGMode() ~= 'listed')
    "#,
    );
    for (count, leader, empowered) in [
        (6, true, true),
        (6, false, false),
        (1, true, true),
        (1, false, false),
        (0, false, true),
    ] {
        seed_group(&env, count, leader);
        let raid_leader = count >= 6 && leader;
        let party_leader = count > 0 && count < 6 && leader;
        exec_and_audit(
            &env,
            &format!(
                r#"
            assert(LFD_IsEmpowered() == {empowered} and LFR_IsEmpowered() == {empowered})
            LFG_UpdateFindGroupButtons()
            assert(LFDQueueFrameFindGroupButton:IsEnabled() == {empowered})
            assert(LFRQueueFrameFindGroupButton:IsEnabled() == {empowered})
            assert(IsRaidLeader() == {raid_leader} and IsPartyLeader() == {party_leader})
        "#
            ),
        );
    }
}
