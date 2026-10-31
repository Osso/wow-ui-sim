//! Pinned rarity rows through original GroupFinder and direct declarations.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]
use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::c_api::c_lfg_list_search::DungeonScore;

#[test]
fn original_lfg_result_tooltip_uses_supplied_scores() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_GroupFinder"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_GroupFinder"));
    {
        let shared = env.state();
        let mut state = shared.borrow_mut();
        let activity = state.world.premade_listings[0].activity_id;
        state
            .lfg_activities
            .iter_mut()
            .find(|row| row.activity_id == activity)
            .unwrap()
            .group_id = 295;
        let listing = &mut state.world.premade_listings[0];
        listing.metadata.overall_dungeon_score = Some(2750.0);
        listing.metadata.dungeon_scores = vec![DungeonScore {
            map_score: 275.0,
            map_name: "Fixture Dungeon".into(),
            best_run_level: 12,
            finished_success: true,
            best_run_duration_ms: 900000.0,
            best_level_increment: 1,
        }];
        listing.metadata.best_dungeon_score = Some(DungeonScore {
            map_score: 225.0,
            map_name: "Best Dungeon".into(),
            best_run_level: 10,
            finished_success: true,
            best_run_duration_ms: 850000.0,
            best_level_increment: 2,
        });
    }
    env.exec(
        r#"
        C_LFGList.Search(2)
        local source = debug.getinfo(LFGListUtil_SetSearchEntryTooltip, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_GroupFinder/', 1, true), source)
        GameTooltip:ClearLines()
        LFGListUtil_SetSearchEntryTooltip(GameTooltip, 1)
        assert(C_LFGList.GetSearchResultEncounterInfo(1) == nil)
        local texts = ''
        for i=1,GameTooltip:NumLines() do
            for _, side in ipairs({'Left', 'Right'}) do
                local line = _G['GameTooltipText' .. side .. i]
                if line then texts = texts .. (line:GetText() or '') .. '\n' end
            end
        end
        assert(texts:find('|cffff80002750|r', 1, true), texts)
        assert(texts:find('|cffffffffFixture Dungeon|r', 1, true), texts)
        assert(texts:find('|cffffffffBest Dungeon|r', 1, true), texts)
    "#,
    )
    .unwrap();
    {
        let shared = env.state();
        let mut state = shared.borrow_mut();
        state.world.premade_listings[0]
            .metadata
            .overall_dungeon_score = Some(1000.0);
        state.world.premade_listings[0].metadata.dungeon_scores[0].best_run_level = 7;
        state.world.premade_listings[0].metadata.defeated_encounters = vec!["Fixture Boss".into()];
    }
    env.exec(
        r#"
        C_LFGList.Search(2)
        GameTooltip:ClearLines()
        LFGListUtil_SetSearchEntryTooltip(GameTooltip, 1)
        local texts = ''
        for i=1,GameTooltip:NumLines() do
            for _, side in ipairs({'Left', 'Right'}) do
                local line = _G['GameTooltipText' .. side .. i]
                if line then texts = texts .. (line:GetText() or '') .. '\n' end
            end
        end
        assert(texts:find('Fixture Boss', 1, true), texts)
        assert(C_LFGList.GetSearchResultEncounterInfo(1)[1] == 'Fixture Boss')
        assert(texts:find('|cff1eff001000|r', 1, true), texts)
        assert(texts:find('|cffffffffFixture Dungeon|r', 1, true), texts)
        assert(C_LFGList.GetSearchResultEncounterInfo(2) == nil)
        GameTooltip:ClearLines()
        LFGListUtil_SetSearchEntryTooltip(GameTooltip, 2)
        for i=1,GameTooltip:NumLines() do
            local line = _G['GameTooltipTextLeft' .. i]
            assert(not line or not (line:GetText() or ''):find('Fixture Boss', 1, true))
        end
    "#,
    )
    .unwrap();
}

#[test]
fn missing_cache_reports_generation_command() {
    if std::env::var_os("WOW_SIM_TEST_MISSING_RARITY_CACHE").is_some() {
        let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
        let error = env
            .exec("C_ChallengeMode.GetDungeonScoreRarityColor(1250)")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("wow-cli casc sync-dungeon-score-rarity"),
            "{error}"
        );
        return;
    }
    let cache = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "dungeon_score_rarity::missing_cache_reports_generation_command",
            "--exact",
            "--nocapture",
        ])
        .env("WOW_SIM_TEST_MISSING_RARITY_CACHE", "1")
        .env("XDG_CACHE_HOME", cache.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rarity_boundaries_follow_pinned_rows() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(r#"
        local function rgb(c) return math.floor(c.r*255+0.5), math.floor(c.g*255+0.5), math.floor(c.b*255+0.5) end
        local function check(f, score, r, g, b)
            local x,y,z = rgb(f(score)); assert(x==r and y==g and z==b, tostring(score))
        end
        local f = C_ChallengeMode.GetDungeonScoreRarityColor
        assert(f(-1) == nil); check(f,999,255,255,255); check(f,1000,30,255,0)
        check(f,2199,163,53,238); check(f,2200,255,128,0); check(f,99999,255,128,0)
        check(C_ChallengeMode.GetSpecificDungeonOverallScoreRarityColor,124,255,255,255)
        check(C_ChallengeMode.GetSpecificDungeonOverallScoreRarityColor,125,30,255,0)
        check(C_ChallengeMode.GetSpecificDungeonScoreRarityColor,62,255,255,255)
        check(C_ChallengeMode.GetSpecificDungeonScoreRarityColor,63,30,255,0)
        check(C_ChallengeMode.GetKeystoneLevelRarityColor,3,255,255,255)
        check(C_ChallengeMode.GetKeystoneLevelRarityColor,4,30,255,0)
    "#).unwrap();
    assert!(
        env.exec("C_ChallengeMode.GetDungeonScoreRarityColor()")
            .is_err()
    );
    assert!(
        env.exec("C_ChallengeMode.GetDungeonScoreRarityColor('bad')")
            .is_err()
    );
}
