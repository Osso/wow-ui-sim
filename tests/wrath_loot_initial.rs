//! Complete unchanged cached consumers; source-staged, runtime unverified.
#![cfg(feature = "client-wrath")]

use sha2::{Digest, Sha256};
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

const ORIGINALS: &[(&str, &str)] = &[
    (
        "GlobalStrings.lua",
        "1d6662e6e77945a1d82d91dbc2fec70a5dffeb31e123c392e24adf7548d17dec",
    ),
    (
        "Constants.lua",
        "367f5024098edda7d8dbcc9f57cf2616c85da4a3a9eb9851d2d4fc0bea8b72fd",
    ),
    (
        "Fonts.xml",
        "cf08457a8fd1e1d1788532bb10c2de7e05280291c11f6a00294709f104d26757",
    ),
    (
        "FontStyles.xml",
        "618ec8ba7234c5725576c31b6d97f30588e8653474292d3783edf602d0d31d8a",
    ),
    (
        "Localization.xml",
        "57b8680f2e2b833143db01bbdb486499addffadd72ffda16eae94e1b95b6e9a6",
    ),
    (
        "Localization.lua",
        "d1e7748fb0774c11baa381c4b264e814b5e29e518882a008bde3783a153fce66",
    ),
    (
        "BasicControls.xml",
        "649b1dec3c05e6a0c4988ae703fc936ae9eca640f78c0eea48001100c0f2fc30",
    ),
    (
        "UIParent.lua",
        "a48687367d05ba9965bdd7a43bdecb92666c8ba74873bd7d9da08b205b34e177",
    ),
    (
        "MoneyFrame.lua",
        "f7392b7c139c8694fc3fe7581231c21342b9b6ed23b19f0b4bb77f3c156a7d90",
    ),
    (
        "MoneyFrame.xml",
        "2d72bc532d440ef7c01db88986be0a0151b9ff644e0b8a5a22def9998c109590",
    ),
    (
        "MoneyInputFrame.lua",
        "47b5cb82c6632450348b51b4f3301209eb37a5677190e4e8cd6cefa6410abee2",
    ),
    (
        "MoneyInputFrame.xml",
        "8c178e530b037adce610eed18cc17822656c9eec548962f5bb912fd8d5b71c5f",
    ),
    (
        "GameTooltip.xml",
        "d76a3add5eda00ed67e61369900241f0ded4b30d19ee828d92849e513857ed1e",
    ),
    (
        "HealthBar.lua",
        "093bbde4271332f1adb610deebec99718d1467986cfc583920049d363d833b4a",
    ),
    (
        "GameTooltip.lua",
        "29d0f65410030173d9041c415e914e86a57b4eb724c6b0e532d075a191a76587",
    ),
    (
        "GameTooltipTemplate.xml",
        "1ee083c247430dd66739acccb7119ad048019cd7b380b68849adad7f23531cbc",
    ),
    (
        "UIMenu.xml",
        "9ff104f1bbf97dc9feccf6692c6447eeccb48ec65f80cd991bab6a9180f0d5a2",
    ),
    (
        "UIMenu.lua",
        "ef9a84f42734253d1e42ab64a74b0baa67b0669bfabe66e1fd2e85e25d75f2a1",
    ),
    (
        "UIDropDownMenu.xml",
        "efe001430531c977c9d50c88d6e05419787f0a11c204f7b13981b9e44d7eaa61",
    ),
    (
        "UIDropDownMenu.lua",
        "d83c1d450884a8319045f671b3b81da442250ca9402dc94e85a542e7f1068a6a",
    ),
    (
        "UIDropDownMenuTemplates.xml",
        "5be5e90b8636c26df3af7976daf3094062c02164394b896588be048daf040619",
    ),
    (
        "UIPanelTemplates.lua",
        "d975f94a6d7deb8895a8ab47a2b6975cca4fd9e45f67cc15c54c8e100b37427e",
    ),
    (
        "UIPanelTemplates.xml",
        "e3cb10077e9296a5067ec32e33655b01c8930bee4019db37196ac292f602b91c",
    ),
    (
        "SecureTemplates.xml",
        "7c366e7784e01843801c8b6f50909645806250539a2c023318d8b4fc050ea426",
    ),
    (
        "SecureTemplates.lua",
        "8582f8a16032d25c50873fc74c50fe58a08c265dc7bc162bfae171e3454ea8e4",
    ),
    (
        "SecureStateDriver.lua",
        "c65dda91a5937c4fafce72d268b82d3bd273e5f139265d654d294ee5911652d3",
    ),
    (
        "SecureHoverDriver.lua",
        "11e5e856f226a45b0447c26ad26fd21100da2d287729cc927878fffa4c5b8dc8",
    ),
    (
        "SecureHandlerTemplates.xml",
        "b101d3c06fc76d456a58ed66ecfefbaabc4bb37ce4fd8efe8040e35a4c30dcda",
    ),
    (
        "RestrictedEnvironment.lua",
        "7bb78e7401ac1738c068c437cb0ee25836768a53635ed36dc084244e27d91a46",
    ),
    (
        "RestrictedFrames.lua",
        "4db2203bfab5d63fc5fb127b571476bf210153e6888b33b4794c80cf32b40059",
    ),
    (
        "RestrictedExecution.lua",
        "f63c5265d4cb23ff85f5b0751a2ee4dd749226871af9f0b2d325ee83b8f21593",
    ),
    (
        "SecureHandlers.lua",
        "dc14c17e7b788570f1d959c8452245aa9a3c82286a6b433151a2bb5fc6445326",
    ),
    (
        "ItemButtonTemplate.xml",
        "a6a8a4971f813a57089a0bd640d98c748367bea3564b86aa1565334b3acdbefd",
    ),
    (
        "ItemButtonTemplate.lua",
        "4e1f95ba20cb7285c8e044b0ddc609fd9cf2001ae2cf8083ee343c28e77cb4a0",
    ),
    (
        "SparkleFrame.xml",
        "818346fa1a5c58a1c38ecad2f00ab6680bb03789d01efdc6135dccecbbe2df6d",
    ),
    (
        "SparkleFrame.lua",
        "10ff8165a5aa51f2732dfecf884803723347109a8c8328487d55b2509b79dfb0",
    ),
    (
        "TextStatusBar.lua",
        "ceeed97ce260c1c505486ebc4a6757007585d450c3964c8b4428916a81f29c4d",
    ),
    (
        "TextStatusBar.xml",
        "a0f3aca312af2c229203ef9ca0bad7a662353e18ed63ceb505e4e95f00006006",
    ),
    (
        "AutoComplete.xml",
        "18d08285a969cea1878ad7f4742105083140bb0a6b68448ac916f8e71eedbb6f",
    ),
    (
        "AutoComplete.lua",
        "30102d7915abcc2125b63d220f560703da954d4512bf59aefcb6899f5ba5741a",
    ),
    (
        "Cooldown.xml",
        "e0a6f7edca59762e1c6a76dd45e53a25dc027dbf52dba192e434f3312f07483c",
    ),
    (
        "Cooldown.lua",
        "010ed45b7713b29d831d7a0e9afbb9a253dca4a28a2f95de910d4ec8caa32f97",
    ),
    (
        "ActionButtonTemplate.xml",
        "f1d376a849fca3867c2e1b3cb6dfeb4cd21db7838914482a16e48d5ce95c3ffa",
    ),
    (
        "BuffFrame.xml",
        "c4a1a1e6449c7ed866712f28c60e57080b69a8ade2b00e766bc58d3c0aa068ff",
    ),
    (
        "BuffFrame.lua",
        "c95749ebe1dbac513d3523e98c56231390486e48ae80e0e51072aa24368393bf",
    ),
    (
        "CombatFeedback.xml",
        "e3231c7fd2a01fee69d8bdd07b6eb0390f7f74c38ef007c9f92fb93736b3d168",
    ),
    (
        "CombatFeedback.lua",
        "8b40a5feda3b856d763228c9aaa40a4a062cee0e45fb90c036a72b0dd59732fa",
    ),
    (
        "CastingBarFrame.xml",
        "79252d78a74ccc765a37daf86726d841dfc0d0c6f93ee186cd8072f97dc94201",
    ),
    (
        "CastingBarFrame.lua",
        "14901aa0ad3a5fc07ba0ccec1ec7109b944a2a2b988d730cc2d0a8056201ef70",
    ),
    (
        "UnitPopup.xml",
        "fe0ae17443c88fbe98a154b4d99822aeaf7fda1deccc1cda42a59a7fd366ba16",
    ),
    (
        "UnitPopup.lua",
        "ed3f60d1152d7054fb846afce1febf3249494c89ebff42fe88e5c09b6be40209",
    ),
    (
        "UnitFrame.xml",
        "4ea79736e0fee81e4bae333eecbed526fb3f0325a54b1874c020d8378fb60d53",
    ),
    (
        "UnitFrame.lua",
        "4b44e519d2a1f5edbbb0c130aa93b408dea2139c8a2be7a64946dda13ce9ddde",
    ),
    (
        "TargetFrame.lua",
        "dd2e492a097ac0da7731379a746012cd4031b3c4ac68ff204c7414872e99d3d9",
    ),
    (
        "SpellBookFrame.xml",
        "2512241621c7f2dcaff860def125088bb6e13035ecf43c222b777a0f06c9399a",
    ),
    (
        "SpellBookFrame.lua",
        "af2d753f865f5c04d1332eb4c667b22b7f3dd828696a7d5c4587d3c530c89c32",
    ),
    (
        "MailFrame.xml",
        "c31065aaaf2190613ac7c2cbdab743af3deedcc0680921a48c696f7d2d8cb2f4",
    ),
    (
        "MailFrame.lua",
        "d97121320cc2cc7ee64ecc52708b968a199126382a0f20b6263d93f50c03e167",
    ),
    (
        "PartyFrame.xml",
        "0d4fc268b2e4bd17f882f499345e893e0f6599d0447653771fccfce192977421",
    ),
    (
        "PartyMemberFrame.lua",
        "6b33ac81c75f15466d981d24b672465f89639b389e51ccea1a68bb58a3da98f1",
    ),
    (
        "PartyFrameTemplates.xml",
        "9d49b0808eba7a46ee6c43d5ee249446d667b05430a5aaac6d692a86c6db80f1",
    ),
    (
        "RaidFrame.lua",
        "1c2231c864f0d6e0c1bf02bd780ed1d12793ba3457323d86ebcf1d33079b6a1e",
    ),
    (
        "VoiceChat.lua",
        "7154de538cf9d0f0cf8bfaad8e113585c68deb145d7331f068601744e1c3d67c",
    ),
    (
        "VoiceChat.xml",
        "b3a62a6a670fe2cc28f002ef8fbdd8a6ccee1fbbd896548a2fb24ea09748c8fc",
    ),
];

fn assert_no_errors(env: &WowLuaEnv) {
    env.exec("assert(#BoundaryErrors == 0, table.concat(BoundaryErrors, '\\n'))")
        .unwrap();
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "{:?}",
        env.state().borrow().lua_errors
    );
}

fn exec(env: &WowLuaEnv, code: &str) {
    let result = env.exec(code);
    assert_no_errors(env);
    result.unwrap_or_else(|error| panic!("original consumer and assertions:\n{code}\n{error}"));
}

fn load(env: &WowLuaEnv, files: &[&str]) {
    let toc = wow_ui_sim::client_profile::blizzard_ui_framexml_toc()
        .expect("SETUP FAILURE: Wrath cache required");
    let cache = toc.parent().unwrap();
    // Check all transitive inputs, including XML scripts/includes, before loading.
    for (name, digest) in ORIGINALS {
        let path = cache.join(name);
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("SETUP FAILURE: {}: {e}", path.display()));
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            *digest,
            "SETUP FAILURE: {}",
            path.display()
        );
    }
    let dir = tempfile::tempdir().unwrap();
    let mut source = String::from("## Interface: 30300\n## Title: WrathBoundary\n");
    for file in files {
        let original = cache.join(file);
        let common = dir
            .path()
            .ancestors()
            .find(|base| original.starts_with(base))
            .unwrap();
        let mut relative = std::path::PathBuf::new();
        for _ in dir.path().strip_prefix(common).unwrap().components() {
            relative.push("..");
        }
        relative.push(original.strip_prefix(common).unwrap());
        source.push_str(relative.to_str().unwrap());
        source.push('\n');
    }
    let path = dir.path().join("WrathBoundary.toc");
    std::fs::write(&path, source).unwrap();
    let result = load_addon(&env.loader_env(), &path);
    assert_no_errors(env);
    let loaded = result.expect("complete unchanged original files");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
}

fn create_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec("BoundaryErrors = {}; seterrorhandler(function(e) BoundaryErrors[#BoundaryErrors+1] = tostring(e) end)").unwrap();
    load(
        &env,
        &[
            "GlobalStrings.lua",
            "Constants.lua",
            "Fonts.xml",
            "FontStyles.xml",
            "Localization.xml",
            "BasicControls.xml",
            "UIParent.lua",
            "VoiceChat.lua",
            "MoneyFrame.lua",
            "MoneyFrame.xml",
            "MoneyInputFrame.lua",
            "MoneyInputFrame.xml",
            "GameTooltip.xml",
            "UIMenu.xml",
            "UIDropDownMenu.xml",
            "UIPanelTemplates.lua",
            "UIPanelTemplates.xml",
            "SecureTemplates.xml",
            "SecureHandlerTemplates.xml",
            "ItemButtonTemplate.xml",
            "SparkleFrame.xml",
            "TextStatusBar.lua",
            "TextStatusBar.xml",
            "AutoComplete.xml",
            "Cooldown.xml",
            "ActionButtonTemplate.xml",
            "BuffFrame.xml",
            "CombatFeedback.xml",
            "CastingBarFrame.xml",
            "UnitPopup.xml",
            "UnitFrame.xml",
        ],
    );
    register_original_voice_template();
    env
}

fn register_original_voice_template() {
    // Register the unchanged virtual speaker template, without instantiating
    // unrelated minimap/voice-session roots from the same original XML file.
    let toc = wow_ui_sim::client_profile::blizzard_ui_framexml_toc().unwrap();
    let source = std::fs::read_to_string(toc.parent().unwrap().join("VoiceChat.xml")).unwrap();
    let ui = wow_ui_sim::xml::parse_xml(&source).unwrap();
    let (frame, kind) = ui
        .elements
        .iter()
        .filter_map(|element| element.as_frame_data())
        .find(|(frame, _)| frame.name.as_deref() == Some("VoiceChatSpeakerTemplate"))
        .unwrap();
    wow_ui_sim::xml::register_template("VoiceChatSpeakerTemplate", kind, frame.clone());
}

fn seed_party(env: &WowLuaEnv, linked: bool) {
    use wow_ui_sim::lua_api::state::PartyMember;
    {
        let mut sim = env.state().borrow_mut();
        sim.party_group_active = true;
        sim.party_members = vec![PartyMember {
            map_position: None,
            faction_group: None,
            #[cfg(feature = "client-retail")]
            pvp: None,
            name: "LootFriend".into(),
            name_cached: true,
            connected: true,
            class_index: 2,
            level: 60,
            health: 100,
            health_max: 100,
            power: 100,
            power_max: 100,
            power_type: 0,
            power_type_name: "MANA".into(),
            is_leader: false,
            dead_since: None,
            buffs: vec![],
            debuffs: vec![],
        }];
    }
    if linked {
        let guid: String = env.eval("return UnitGUID('party1')").unwrap();
        assert!(!guid.is_empty());
        env.state()
            .borrow_mut()
            .refer_a_friend_linked_guids
            .insert(guid);
    }
}

fn assert_party_flow(env: &WowLuaEnv, label: &str, raf_visible: i32) {
    // Production loader traverses complete unchanged PartyFrame.xml roots in order;
    // no placeholder siblings, replacement OnLoads, or swallowed script errors.
    load(env, &["PartyFrame.xml", "RaidFrame.lua"]);
    exec(
        env,
        &format!(
            r#"
        for i = 1, 4 do
            local frame = _G['PartyMemberFrame'..i]
            local dropdown = _G['PartyMemberFrame'..i..'DropDown']
            assert(frame and frame:GetID() == i)
            assert(dropdown and dropdown:GetParent() == frame)
            assert(dropdown.selectedLootMethod == UnitLootMethod['{label}'].text)
            assert(ITEM_QUALITY2_DESC == 'Uncommon')
            assert(dropdown.selectedLootThreshold == ITEM_QUALITY2_DESC)
            assert(not _G['PartyMemberFrame'..i..'LeaderIcon']:IsShown())
        end
        UIDropDownMenu_Initialize(PartyMemberFrame1DropDown, PartyFrameDropDown_Initialize, 'MENU')
        assert(UnitPopupButtons.LOOT_THRESHOLD.text == ITEM_QUALITY2_DESC)
        assert(GetLootThreshold() == 2)
        assert(GetPartyLeaderIndex() == 0)
        local start, duration = GetSummonFriendCooldown()
        assert(start == 0 and duration == 0)
        assert(UnitPopupButtons.RAF_SUMMON.text == RAF_SUMMON)
        assert(GetOptOutOfLoot() == false)
        assert(GetVoiceStatus('party1', 'party') == false)
        assert(GetMuteStatus('party1', 'party') == false)
        assert(not PartyMemberFrame1Speaker:IsShown())
        assert(not PartyMemberFrame1SpeakerMuted:IsShown())
        assert(UnitIsTalking('LootFriend') == false)
        assert(not PartyMemberFrame1SpeakerFrame:IsShown())
        for _, name in ipairs({{'', 'Unknown speaker', 'Jöhn', string.char(255, 0, 128)}}) do
            assert(UnitIsTalking(name) == false)
        end
        assert(UnitPopupButtons.OPT_OUT_LOOT_TITLE.text ==
            format(OPT_OUT_LOOT_TITLE, UnitPopupButtons.OPT_OUT_LOOT_DISABLE.text))
        local found = 0
        for index, name in ipairs(UnitPopupMenus.PARTY) do
            if name == 'RAF_SUMMON' or name == 'RAF_GRANT_LEVEL' then
                assert(UnitPopupShown[1][index] == {raf_visible}, name)
                found = found + 1
            end
        end
        assert(found == 2)
        HIDE_PARTY_INTERFACE = '0'
        RaidOptionsFrame_UpdatePartyFrames()
        assert(PartyMemberFrame1:IsShown())
        for i = 2, 4 do assert(not _G['PartyMemberFrame'..i]:IsShown()) end
    "#
        ),
    );
}

#[test]
fn original_wrath_party_roots_use_fresh_freeforall_with_empty_raf_links() {
    let env = create_env();
    // No loot-state override: this exercises the approved simulator default.
    seed_party(&env, false);
    exec(&env, "assert(not IsReferAFriendLinked('party1'))");
    assert_party_flow(&env, "freeforall", 0);
    exec(&env, "assert(GetLootMethod() == 'freeforall')");
}

#[test]
fn original_wrath_party_roots_preserve_master_indices_with_supplied_raf_link() {
    let env = create_env();
    seed_party(&env, true);
    {
        let mut sim = env.state().borrow_mut();
        sim.loot_method.method = "master".into();
        sim.loot_method.party_master_index = 1;
        sim.loot_method.raid_master_index = 3;
    }
    exec(
        &env,
        r#"
        assert(IsReferAFriendLinked('party1'))
        local method, party, raid = GetLootMethod()
        assert(method == 'master' and party == 1 and raid == 3)
    "#,
    );
    assert_party_flow(&env, "master", 1);
    exec(
        &env,
        "local _, p, r = GetLootMethod(); assert(p == 1 and r == 3)",
    );
    // The existing test-state path must drive the legacy reader and original menu.
    env.state().borrow_mut().loot_method.threshold = 4; // Epic
    exec(
        &env,
        r#"
        assert(GetLootThreshold() == 4)
        assert(GetMasterLooterThreshold() == 4)
        UIDropDownMenu_Initialize(PartyMemberFrame1DropDown, PartyFrameDropDown_Initialize, 'MENU')
        assert(ITEM_QUALITY4_DESC == 'Epic')
        assert(PartyMemberFrame1DropDown.selectedLootThreshold == ITEM_QUALITY4_DESC)
        assert(UnitPopupButtons.LOOT_THRESHOLD.text == ITEM_QUALITY4_DESC)
    "#,
    );
    env.state().borrow_mut().loot_method.opt_out = true;
    exec(
        &env,
        r#"
        assert(GetOptOutOfLoot() == true)
        UIDropDownMenu_Initialize(PartyMemberFrame1DropDown, PartyFrameDropDown_Initialize, 'MENU')
        assert(UnitPopupButtons.OPT_OUT_LOOT_TITLE.text ==
            format(OPT_OUT_LOOT_TITLE, UnitPopupButtons.OPT_OUT_LOOT_ENABLE.text))
    "#,
    );
    let now: f64 = env.eval("return GetTime()").unwrap();
    env.state().borrow_mut().summon_friend_cooldown =
        wow_ui_sim::c_api::c_recruit_a_friend::SummonFriendCooldown {
            start_time_seconds: now + 120.0,
            duration_seconds: 600.0,
            enable_cooldown_timer: true,
        };
    exec(
        &env,
        r#"
        UIDropDownMenu_Initialize(PartyMemberFrame1DropDown, PartyFrameDropDown_Initialize, 'MENU')
        local start, duration, enabled = GetSummonFriendCooldown()
        local nsStart, nsDuration, nsEnabled = C_RecruitAFriend.GetSummonFriendCooldown()
        assert(duration == 600 and enabled == true)
        assert(start == nsStart and duration == nsDuration and enabled == nsEnabled)
        local remaining = start + duration - GetTime()
        assert(remaining > 0)
        assert(UnitPopupButtons.RAF_SUMMON.text ==
            format(RAF_SUMMON_WITH_COOLDOWN, SecondsToTime(remaining, true)))
    "#,
    );
    env.state().borrow_mut().legacy_unit_voice.insert(
        ("party1".into(), "party".into()),
        wow_ui_sim::lua_api::globals::real::wrath_voice::LegacyUnitVoice {
            voice_enabled: true,
            muted: true,
        },
    );
    exec(
        &env,
        r#"
        assert(GetVoiceStatus('party1', 'party') == true)
        assert(GetMuteStatus('party1', 'party') == true)
        assert(GetVoiceStatus('party1', 'raid') == false)
        assert(GetMuteStatus('party1', 'raid') == false)
        PartyMemberFrame_UpdateVoiceStatus(PartyMemberFrame1)
        assert(PartyMemberFrame1Speaker:IsShown())
        assert(PartyMemberFrame1SpeakerMuted:IsShown())
    "#,
    );
    env.state()
        .borrow_mut()
        .legacy_unit_voice
        .get_mut(&("party1".into(), "party".into()))
        .unwrap()
        .muted = false;
    exec(
        &env,
        r#"
        PartyMemberFrame_UpdateVoiceStatus(PartyMemberFrame1)
        assert(PartyMemberFrame1Speaker:IsShown())
        assert(not PartyMemberFrame1SpeakerMuted:IsShown())
    "#,
    );
    env.state().borrow_mut().voice_chat.channels = vec![wow_ui_sim::lua_api::state::VoiceChannel {
        channel_id: 7,
        name: "Supplied party voice".into(),
        channel_type: 1,
        members: vec![wow_ui_sim::lua_api::state::VoiceMember {
            member_id: 41,
            name: "LootFriend".into(),
            is_active_speaker: true,
            volume: 1.0,
        }],
    }];
    exec(
        &env,
        r#"
        assert(UnitIsTalking('LootFriend') == true, 'supplied speaker lookup')
        assert(UnitIsTalking('AnotherFriend') == false, 'unmatched speaker lookup')
        PartyMemberFrame_UpdateVoiceStatus(PartyMemberFrame1)
        assert(PartyMemberFrame1SpeakerFrame:IsShown(),
            'speaker frame name='..tostring(UnitName('party1'))..
            ' talking='..tostring(UnitIsTalking(UnitName('party1'))))
    "#,
    );
    env.state().borrow_mut().voice_chat.channels[0].members[0].is_active_speaker = false;
    exec(
        &env,
        r#"
        assert(UnitIsTalking('LootFriend') == false)
        PartyMemberFrame_UpdateVoiceStatus(PartyMemberFrame1)
        assert(not PartyMemberFrame1SpeakerFrame:IsShown())
    "#,
    );
    // A second concrete supplied party member must reach the original shown path.
    {
        let mut sim = env.state().borrow_mut();
        let mut second = sim.party_members[0].clone();
        second.name = "SecondFriend".into();
        sim.party_members.push(second);
    }
    exec(
        &env,
        r#"
        assert(GetPartyMember(1) == true and GetPartyMember(2) == true)
        assert(GetPartyMember(3) == false)
        assert(GetPartyMember(0) == false and GetPartyMember(5) == false)
        PartyMemberFrame_UpdateMember(PartyMemberFrame2)
        assert(PartyMemberFrame2:IsShown())
    "#,
    );
    env.state().borrow_mut().party_leader_index = Some(1); // zero-based party2
    exec(
        &env,
        r#"
        assert(GetPartyLeaderIndex() == 2)
        assert(UnitIsGroupLeader('party2') and not UnitIsGroupLeader('player'))
        assert(not IsPartyLeader())
        assert(not HasLFGRestrictions())
        for i = 1, 4 do
            PartyMemberFrame_UpdateLeader(_G['PartyMemberFrame'..i])
            assert(_G['PartyMemberFrame'..i..'LeaderIcon']:IsShown() == (i == 2))
        end
    "#,
    );
    env.state().borrow_mut().party_leader_index = None;
    exec(
        &env,
        r#"
        assert(GetPartyLeaderIndex() == 0)
        assert(UnitIsGroupLeader('player') and IsPartyLeader())
        PartyMemberFrame_UpdateLeader(PartyMemberFrame2)
        assert(not PartyMemberFrame2LeaderIcon:IsShown())
    "#,
    );
    env.state().borrow_mut().party_group_active = false;
    exec(
        &env,
        r#"
        assert(GetPartyMember(1) == false and GetPartyMember(2) == false)
        assert(GetPartyLeaderIndex() == 0)
        PartyMemberFrame_UpdateMember(PartyMemberFrame2)
        assert(not PartyMemberFrame2:IsShown())
    "#,
    );
    env.state().borrow_mut().refer_a_friend_linked_guids.clear();
    exec(&env, "assert(not IsReferAFriendLinked('party1'))");
}
