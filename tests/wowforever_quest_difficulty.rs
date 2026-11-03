//! Forever's supplied quest difficulty and unchanged title consumers.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_quest_difficulty_raw_publication_reads_seed_and_live_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        local get = rawget(C_QuestLog, "GetQuestDifficultyLevel")
        assert(type(get) == "function", "raw quest difficulty publication")
        for _, id in ipairs({80000, 80001, 90001, 90101}) do
            assert(select("#", get(id)) == 1)
            assert(get(id) == 80)
        end
        "##,
    )
    .unwrap();
    {
        let mut state = env.state().borrow_mut();
        let entry = state
            .quest_log_entries
            .entries
            .iter_mut()
            .find(|entry| entry.quest_id == 80000)
            .unwrap();
        entry.level = 37;
    }
    let level: i32 = env
        .eval("return C_QuestLog.GetQuestDifficultyLevel(80000)")
        .unwrap();
    assert_eq!(level, 37);
    {
        let mut state = env.state().borrow_mut();
        let entry = state
            .quest_log_entries
            .entries
            .iter_mut()
            .find(|entry| entry.quest_id == 80000)
            .unwrap();
        entry.quest_id = 456789;
        entry.level = 23;
    }
    let level: i32 = env
        .eval("return C_QuestLog.GetQuestDifficultyLevel(456789)")
        .unwrap();
    assert_eq!(level, 23);
    let fresh = WowLuaEnv::new().unwrap();
    fresh
        .exec(
            r#"
        local get = rawget(C_QuestLog, "GetQuestDifficultyLevel")
        assert(type(get) == "function")
        assert(get(80000) == 80, "fresh environment owns independent seed")
        assert(not pcall(get, 456789), "other environment's row must not leak")
        "#,
        )
        .unwrap();
}

#[test]
fn forever_quest_difficulty_rejects_invalid_and_unrepresented_ids_explicitly() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local get = rawget(C_QuestLog, "GetQuestDifficultyLevel")
        assert(type(get) == "function")
        local function rejects(value)
            local ok, err = pcall(get, value)
            assert(not ok, "invalid or missing data must fail")
            assert(type(err) == "string" and err:find("C_QuestLog.GetQuestDifficultyLevel", 1, true))
            assert(err:find("questID", 1, true))
        end
        local ok, err = pcall(get)
        assert(not ok and err:find("questID", 1, true))
        rejects(nil)
        for _, value in ipairs({"80000", false, {}, 80000.5, 0/0, math.huge, -math.huge, 2147483648}) do
            rejects(value)
        end
        for _, value in ipairs({-1, 0, 80002, 999999}) do
            rejects(value)
        end
        local absent, message = pcall(get, 999999)
        assert(not absent and message:find("999999", 1, true))
        "#,
    )
    .unwrap();
    env.state().borrow_mut().quest_log_entries.entries.clear();
    env.exec(
        r#"
        local ok, err = pcall(C_QuestLog.GetQuestDifficultyLevel, 80000)
        assert(not ok and err:find("80000", 1, true), "removed row must not use catalog/default")
        "#,
    )
    .unwrap();
}

fn load_forever_title_consumer(env: &WowLuaEnv) {
    // Explicit test dependencies, not alternate production implementations.
    env.exec(
        r#"
        DifficultyUtil = {}
        QuestDifficultyColors = {difficult={r=0.2, g=0.4, b=0.6}}
        QuestDifficultyHighlightColors = {difficult={r=0.3, g=0.5, b=0.7}}
        CVarCallbackRegistry = {enabled=true}
        function CVarCallbackRegistry:GetCVarValueBool(name)
            assert(name == "showQuestDifficultyColor")
            return self.enabled
        end
        function CreateColor(r, g, b, a)
            assert(r == 0.2 and g == 0.4 and b == 0.6 and a == 1)
            return {WrapTextInColorCode=function(self, text) return "|cff336699" .. text .. "|r" end}
        end
        "#,
    )
    .unwrap();
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    for file in [
        "Blizzard_FrameXMLUtil/Mainline/DifficultyUtil.lua",
        "Blizzard_FrameXMLUtil/Camelot/QuestUtilsOverrides.lua",
    ] {
        let source =
            std::fs::read_to_string(cache.join(file)).expect("synced whole Forever consumer");
        env.exec(&source).unwrap();
    }
}

#[test]
fn forever_quest_difficulty_whole_vendor_title_consumer_uses_live_level() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.player.level = 37;
        for entry in &mut state.quest_log_entries.entries {
            if entry.quest_id == 80000 || entry.quest_id == 80001 {
                entry.level = 37;
            }
            if entry.quest_id == 80001 {
                entry.tag_id = Some(1);
            }
        }
    }
    load_forever_title_consumer(&env);
    env.exec(
        r#"
        assert(C_QuestLog.IsEliteQuest(999999) == false)
        assert(C_PlayerInfo.GetContentDifficultyQuestForPlayer(80000) == Enum.RelativeContentDifficulty.Fair)
        assert(SetQuestTitleLevelAndDifficultyColor(80000, "Expedition") == "|cff336699[37] Expedition|r")
        assert(SetQuestTitleLevelAndDifficultyColor(80001, "Gates") == "|cff336699[37+] Gates|r")
        CVarCallbackRegistry.enabled = false
        assert(SetQuestTitleLevelAndDifficultyColor(80000, "Expedition") == "[37] Expedition")
        assert(SetQuestTitleLevelAndDifficultyColor(80001, "Gates") == "[37+] Gates")

        local eliteID = secretwrap(80001)
        assert(C_QuestLog.IsEliteQuest(eliteID))
        assert(C_PlayerInfo.GetContentDifficultyQuestForPlayer(eliteID) == Enum.RelativeContentDifficulty.Fair)

        local function taintedQuery(query)
            debug.setstacktaint("QuestDifficultyProbe")
            return query(eliteID)
        end
        assert(not pcall(taintedQuery, C_QuestLog.IsEliteQuest))
        assert(not pcall(taintedQuery, C_PlayerInfo.GetContentDifficultyQuestForPlayer))
        "#,
    )
    .unwrap();
}
