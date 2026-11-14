//! Retail tooltip party-progress tests use the selected ObjectiveTracker and
//! the original TooltipDataHandler accessor. All progress values are supplied.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use wow_ui_sim::c_api::quest_party_progress::{
    QuestPartyProgressHeading, QuestPartyProgressParticipant, QuestPartyProgressRow,
    QuestPartyProgressSnapshot, QuestPartyProgressText,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const RED: (f64, f64, f64) = (1.0, 0.0, 0.0);
const GREEN: (f64, f64, f64) = (0.0, 1.0, 0.0);
const QUEST_A: i32 = 990_701;
const QUEST_B: i32 = 990_702;

fn selected_objective_tracker() -> WowLuaEnv {
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .expect("Retail Blizzard cache is required");
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &cache,
        &["Blizzard_ObjectiveTracker"],
        &[],
    );
    for addon in ["Blizzard_SharedXMLGame", "Blizzard_ObjectiveTracker"] {
        assert!(
            loaded.iter().any(|name| name == addon),
            "selected dependency closure must load {addon}: {loaded:?}"
        );
    }
    env.exec("A_Admin.SetPartySize(1)")
        .expect("supply group membership for the original header consumer");
    env
}

fn snapshot(title: &str, player: &str, guid: &str, objective: &str) -> QuestPartyProgressSnapshot {
    QuestPartyProgressSnapshot {
        title: Some(QuestPartyProgressText {
            text: title.to_string(),
            color: RED,
        }),
        participants: vec![participant(player, guid, objective, GREEN, false)],
    }
}

fn participant(
    name: &str,
    guid: &str,
    objective: &str,
    color: (f64, f64, f64),
    exclude_when_ignoring_active_player: bool,
) -> QuestPartyProgressParticipant {
    QuestPartyProgressParticipant {
        heading: Some(QuestPartyProgressHeading {
            name: name.to_string(),
            guid: guid.to_string(),
            color,
        }),
        exclude_when_ignoring_active_player,
        rows: vec![QuestPartyProgressRow {
            text: objective.to_string(),
            color,
            completed: false,
            num_required: Some(3),
            num_fulfilled: Some(1),
            progress_bar_percent: Some(33.0),
        }],
    }
}

fn supply_snapshot(env: &WowLuaEnv, quest_id: i32, value: QuestPartyProgressSnapshot) {
    env.state()
        .borrow_mut()
        .quest_log_entries
        .party_progress
        .insert(quest_id, value);
}

#[test]
fn selected_objective_tracker_publishes_party_progress_for_the_requested_quest() {
    let env = selected_objective_tracker();
    supply_snapshot(
        &env,
        QUEST_A,
        snapshot(
            "First supplied title",
            "First member",
            "Player-1-0001",
            "First objective",
        ),
    );
    supply_snapshot(
        &env,
        QUEST_B,
        snapshot(
            "Second supplied title",
            "Second member",
            "Player-1-0002",
            "Second objective",
        ),
    );
    env.exec(&format!(
        r#"
        local function requireSource(fn, suffix)
            local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(source:find(suffix, 1, true), source)
        end
        requireSource(QuestObjectiveTrackerMixin.OnBlockHeaderEnter,
            'Blizzard_QuestObjectiveTracker.lua')
        requireSource(GameTooltip.SetQuestPartyProgress, 'TooltipDataHandler.lua')
        local function hover(questID)
            local block = CreateFrame('Frame')
            block.id = questID
            QuestObjectiveTrackerMixin:OnBlockHeaderEnter(block)
            return GameTooltip:GetPrimaryTooltipData()
        end
        local first = hover({QUEST_A})
        assert(first, 'RED: supplied quest progress must reach the original hover consumer')
        assert(first.type == 24 and first.id == {QUEST_A})
        assert(first.lines[1].type == 17 and first.lines[1].id == {QUEST_A})
        assert(first.lines[1].leftText == 'First supplied title')
        assert(first.lines[2].type == 18 and first.lines[2].guid == 'Player-1-0001')
        assert(first.lines[3].type == 8 and first.lines[3].completed == false)
        assert(first.lines[3].numRequired == 3 and first.lines[3].numFulfilled == 1)
        assert(first.lines[3].progressBarPercent == 33)

        local second = hover({QUEST_B})
        assert(second and second.id == {QUEST_B})
        assert(second.lines[1].leftText == 'Second supplied title')
        assert(second.lines[2].guid == 'Player-1-0002')
        assert(second.lines[3].leftText == 'Second objective')
        "#
    ))
    .expect("selected ObjectiveTracker hover must publish the supplied quest snapshot");
}

#[test]
fn party_progress_omit_title_selects_the_title_row() {
    let env = selected_objective_tracker();
    supply_snapshot(
        &env,
        QUEST_A,
        snapshot(
            "Optional supplied title",
            "Member",
            "Player-1-0011",
            "Objective",
        ),
    );
    env.exec(&format!(
        r#"
        local source = debug.getinfo(GameTooltip.SetQuestPartyProgress, 'S').source:gsub('\\', '/')
        assert(source:find('TooltipDataHandler.lua', 1, true), source)
        local function query(omitTitle)
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetQuestPartyProgress({QUEST_A}, omitTitle, false)
            return GameTooltip:GetPrimaryTooltipData()
        end
        local included = query(false)
        assert(included, 'RED: omitTitle=false must publish supplied data')
        assert(included.lines[1].type == 17 and included.lines[1].id == {QUEST_A})
        assert(included.lines[1].leftText == 'Optional supplied title')
        local omitted = query(true)
        assert(omitted and omitted.id == {QUEST_A})
        assert(#omitted.lines == #included.lines - 1)
        assert(omitted.lines[1].type == 18 and omitted.lines[1].guid == 'Player-1-0011')
        "#
    ))
    .expect("the original tooltip accessor must honor the verified omitTitle selector");
}

#[test]
fn party_progress_ignore_active_player_selects_supplied_participants() {
    let env = selected_objective_tracker();
    supply_snapshot(
        &env,
        QUEST_A,
        QuestPartyProgressSnapshot {
            title: Some(QuestPartyProgressText {
                text: "Party title".to_string(),
                color: RED,
            }),
            participants: vec![
                participant(
                    "Filtered participant",
                    "Player-1-0021",
                    "Filtered objective",
                    GREEN,
                    true,
                ),
                participant(
                    "Retained participant",
                    "Player-1-0022",
                    "Retained objective",
                    GREEN,
                    false,
                ),
            ],
        },
    );
    env.exec(&format!(
        r#"
        local source = debug.getinfo(GameTooltip.SetQuestPartyProgress, 'S').source:gsub('\\', '/')
        assert(source:find('TooltipDataHandler.lua', 1, true), source)
        local function query(ignoreActivePlayer)
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetQuestPartyProgress({QUEST_A}, false, ignoreActivePlayer)
            return GameTooltip:GetPrimaryTooltipData()
        end
        local all = query(false)
        assert(all, 'RED: ignoreActivePlayer=false must publish supplied data')
        assert(all.lines[3].leftText == 'Filtered objective')
        assert(all.lines[5].leftText == 'Retained objective')
        local filtered = query(true)
        assert(filtered and filtered.id == {QUEST_A})
        assert(filtered.lines[3].leftText == 'Retained objective')
        assert(#filtered.lines == #all.lines - 2)
        "#
    ))
    .expect("the original tooltip accessor must honor the verified participant filter");
}

#[test]
fn selected_quest_item_hover_preserves_the_unmodeled_item_relation() {
    let env = selected_objective_tracker();
    env.exec(
        r#"
        local function requireSource(fn, suffix)
            local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(source:find(suffix, 1, true), source)
        end
        requireSource(QuestObjectiveItemButtonMixin.OnEnter,
            'Blizzard_ObjectiveTrackerShared.lua')
        requireSource(GameTooltip.SetQuestLogSpecialItem, 'TooltipDataHandler.lua')
        local button = CreateFrame('Button')
        button:SetAttribute('questLogIndex', 1)
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        QuestObjectiveItemButtonMixin.OnEnter(button)
        assert(GameTooltip:GetPrimaryTooltipData() == nil,
            'no special-item relation is supplied in this simulator state')
        "#,
    )
    .expect("the selected original hover remains callable without fabricated item state");
}
