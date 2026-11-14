#![cfg(all(
    feature = "client-retail",
    feature = "retail-12-1-5",
    feature = "optional-tests"
))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::c_api::recipe_rank_tooltip::{RecipeRankTooltipCache, RecipeRankTooltipRecord};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

const SPELL_A: u32 = 19_750;
const SPELL_B: u32 = 375_576;
const RECIPE_A_BASE: i32 = 990_701;
const RECIPE_A_NEXT: i32 = 990_702;
const RECIPE_B_BASE: i32 = 990_711;
const RECIPE_B_NEXT: i32 = 990_712;
const RECIPE_RANK_BASE: i32 = 990_721;
const RECIPE_RANK_NEXT: i32 = 990_722;

const RANK_TEST_HELPERS: &str = r#"
function RecipeRankTestQuery(recipeID, rank)
    GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
    GameTooltip:SetRecipeRankInfo(recipeID, rank)
    return GameTooltip:GetPrimaryTooltipData()
end

function RecipeRankTestDescription(spellID)
    local spell = C_TooltipInfo.GetSpellByID(spellID)
    local description = spell.lines[#spell.lines].leftText
    assert(description and description ~= '', 'spell fixture has no description')
    return description
end

function RecipeRankTestFindText(data, text)
    for index, line in ipairs(data.lines) do
        if line.leftText == text then return line, index end
    end
end

function RecipeRankTestColorsDiffer(left, right)
    return math.abs(left.r - right.r) > 0.000001
        or math.abs(left.g - right.g) > 0.000001
        or math.abs(left.b - right.b) > 0.000001
end

function RecipeRankTestAssertPackedColor(line, color)
    local red, green, blue = color:GetRGB()
    local function packed(value)
        return math.floor(value * 255 + 0.5) * 0.003921568859368563
    end
    assert(math.abs(line.leftColor.r - packed(red)) < 0.000001, 'red channel')
    assert(math.abs(line.leftColor.g - packed(green)) < 0.000001, 'green channel')
    assert(math.abs(line.leftColor.b - packed(blue)) < 0.000001, 'blue channel')
end
"#;

fn original_tooltip_handler_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let selected = wow_ui_sim::loader::discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    assert!(
        selected
            .iter()
            .any(|(name, _)| name == "Blizzard_SharedXMLGame"),
        "Retail game-screen TOC selection must include Blizzard_SharedXMLGame"
    );

    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_SharedXMLGame", "Blizzard_GameTooltip"],
        &[],
    );
    assert!(
        loaded.iter().any(|name| name == "Blizzard_SharedXMLGame"),
        "selected SharedXMLGame closure must load: {loaded:?}"
    );
    assert_original_accessor(&env);
    env
}

fn assert_original_accessor(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_SharedXMLGame'))
        local source = debug.getinfo(GameTooltip.SetRecipeRankInfo, 'S').source:gsub('\\', '/')
        assert(source:find('TooltipDataHandler.lua', 1, true), source)
        "#,
    )
    .expect("the selected TooltipDataHandler accessor must be loaded");
}

fn install_rank_test_helpers(env: &WowLuaEnv) {
    env.exec(RANK_TEST_HELPERS)
        .expect("recipe-rank Lua test helpers should load");
}

fn add_recipe(
    cache: &mut RecipeRankTooltipCache,
    recipe_id: i32,
    parent_recipe_id: i32,
    next_recipe_id: i32,
    spell_id: u32,
    label: Option<&str>,
    next_rank_source: Option<&str>,
) {
    cache.insert(
        recipe_id,
        RecipeRankTooltipRecord {
            parent_recipe_id,
            next_recipe_id,
            spell_id,
            label: label.map(str::to_owned),
            next_rank_source: next_rank_source.map(str::to_owned),
        },
    );
}

fn add_recipe_pair(
    env: &WowLuaEnv,
    base_id: i32,
    next_id: i32,
    base_spell_id: u32,
    next_spell_id: u32,
    label: Option<&str>,
    next_rank_source: Option<&str>,
) {
    let mut state = env.state().borrow_mut();
    add_recipe(
        &mut state.recipe_rank_tooltip_cache,
        base_id,
        0,
        next_id,
        base_spell_id,
        label,
        None,
    );
    add_recipe(
        &mut state.recipe_rank_tooltip_cache,
        next_id,
        base_id,
        0,
        next_spell_id,
        None,
        next_rank_source,
    );
}

fn add_recipe_selector_fixture(env: &WowLuaEnv) {
    add_recipe_pair(
        env,
        RECIPE_A_BASE,
        RECIPE_A_NEXT,
        SPELL_A,
        SPELL_B,
        None,
        None,
    );
    add_recipe_pair(
        env,
        RECIPE_B_BASE,
        RECIPE_B_NEXT,
        SPELL_B,
        SPELL_A,
        None,
        None,
    );

    add_recipe(
        &mut env.state().borrow_mut().recipe_rank_tooltip_cache,
        RECIPE_A_BASE,
        0,
        RECIPE_A_NEXT,
        SPELL_B,
        None,
        None,
    );
}

fn add_inferred_rank_presentation_fixture(env: &WowLuaEnv) {
    add_recipe_pair(
        env,
        RECIPE_RANK_BASE,
        RECIPE_RANK_NEXT,
        SPELL_A,
        SPELL_B,
        Some("Host recipe label"),
        Some("Host supplied next-rank source"),
    );
    install_rank_test_helpers(env);
}

#[test]
fn recipe_id_selects_distinct_chains_through_original_tooltip_accessor() {
    let env = original_tooltip_handler_env();
    add_recipe_selector_fixture(&env);
    install_rank_test_helpers(&env);

    env.exec(&format!(
        r#"
        local descriptionA = RecipeRankTestDescription({SPELL_A})
        local descriptionB = RecipeRankTestDescription({SPELL_B})
        local first = RecipeRankTestQuery({RECIPE_A_NEXT}, 2)
        assert(first, 'RED: recipeID must publish its supplied recipe chain')
        assert(first.type == Enum.TooltipDataType.RecipeRankInfo)
        assert(first.id == {RECIPE_A_NEXT})
        local firstA, firstAIndex = RecipeRankTestFindText(first, descriptionA)
        local firstB, firstBIndex = RecipeRankTestFindText(first, descriptionB)
        assert(firstA and firstB and firstAIndex < firstBIndex,
            'first recipe chain or retained duplicate differs')

        local second = RecipeRankTestQuery({RECIPE_B_NEXT}, 2)
        assert(second and second.type == Enum.TooltipDataType.RecipeRankInfo)
        assert(second.id == {RECIPE_B_NEXT})
        local secondA, secondAIndex = RecipeRankTestFindText(second, descriptionA)
        local secondB, secondBIndex = RecipeRankTestFindText(second, descriptionB)
        assert(secondA and secondB and secondBIndex < secondAIndex,
            'second recipe ID must select its distinct chain')
        "#
    ))
    .expect("the original accessor must select the requested recipe chain");
}

/// VERIFIED: the supplied rank qualitatively selects normal or disabled description color.
/// INFERRED coverage: this test uses ranks 1 and 2 as its concrete comparison boundary.
#[test]
fn rank_threshold_changes_description_colors_through_original_accessor() {
    let env = original_tooltip_handler_env();
    add_recipe_pair(
        &env,
        RECIPE_RANK_BASE,
        RECIPE_RANK_NEXT,
        SPELL_A,
        SPELL_B,
        None,
        None,
    );
    install_rank_test_helpers(&env);

    env.exec(&format!(
        r#"
        local descriptionA = RecipeRankTestDescription({SPELL_A})
        local descriptionB = RecipeRankTestDescription({SPELL_B})
        local rankOne = RecipeRankTestQuery({RECIPE_RANK_BASE}, 1)
        local rankTwo = RecipeRankTestQuery({RECIPE_RANK_BASE}, 2)
        assert(rankOne and rankTwo, 'RED: rank must change supplied description colors')
        assert(rankOne.type == Enum.TooltipDataType.RecipeRankInfo)
        assert(rankTwo.type == Enum.TooltipDataType.RecipeRankInfo)
        assert(rankOne.id == {RECIPE_RANK_BASE} and rankTwo.id == {RECIPE_RANK_BASE})
        local rankOneA = RecipeRankTestFindText(rankOne, descriptionA)
        local rankOneB = RecipeRankTestFindText(rankOne, descriptionB)
        local rankTwoA = RecipeRankTestFindText(rankTwo, descriptionA)
        local rankTwoB = RecipeRankTestFindText(rankTwo, descriptionB)
        assert(rankOneA and rankOneB and rankTwoA and rankTwoB,
            'rank threshold must retain both supplied descriptions')
        assert(not RecipeRankTestColorsDiffer(rankOneA.leftColor, rankTwoA.leftColor),
            'first description remains on the same side of the threshold')
        assert(RecipeRankTestColorsDiffer(rankOneB.leftColor, rankTwoB.leftColor),
            'second description changes color across the threshold')
        "#
    ))
    .expect("the original accessor must apply the rank display threshold");
}

#[test]
fn inferred_recipe_rank_presentation_uses_supplied_labels_and_source() {
    // INFERRED coverage: exact title, RGB packing, wrapping, localization, and source rows.
    let env = original_tooltip_handler_env();
    add_inferred_rank_presentation_fixture(&env);

    env.exec(&format!(
        r#"
        local baseName = C_TooltipInfo.GetSpellByID({SPELL_A}).lines[1].leftText
        local descriptionA = RecipeRankTestDescription({SPELL_A})
        local descriptionB = RecipeRankTestDescription({SPELL_B})
        local rankOne = RecipeRankTestQuery({RECIPE_RANK_BASE}, 1)
        local title = 'Host recipe label: ' .. baseName
        assert(rankOne and rankOne.lines[1].leftText == title)
        local descriptionOne = RecipeRankTestFindText(rankOne, descriptionA)
        local descriptionTwo = RecipeRankTestFindText(rankOne, descriptionB)
        assert(descriptionOne and descriptionOne.wrapText == true)
        assert(descriptionTwo and descriptionTwo.wrapText == true)
        RecipeRankTestAssertPackedColor(descriptionOne, NORMAL_FONT_COLOR)
        RecipeRankTestAssertPackedColor(descriptionTwo, DISABLED_FONT_COLOR)
        assert(RecipeRankTestFindText(rankOne, TRADESKILL_NEXT_RANK_HEADER))
        assert(RecipeRankTestFindText(rankOne, 'Host supplied next-rank source'))

        local rankTwo = RecipeRankTestQuery({RECIPE_RANK_BASE}, 2)
        local descriptionAtRankTwo = RecipeRankTestFindText(rankTwo, descriptionB)
        assert(descriptionAtRankTwo)
        RecipeRankTestAssertPackedColor(descriptionAtRankTwo, NORMAL_FONT_COLOR)
        assert(not RecipeRankTestFindText(rankTwo, TRADESKILL_NEXT_RANK_HEADER))
        assert(not RecipeRankTestFindText(rankTwo, 'Host supplied next-rank source'))
        "#
    ))
    .expect("the simulator's inferred recipe-rank presentation should remain stable");
}

#[test]
fn preserve_recipe_rank_zero_values_for_failed_producer() {
    // PRESERVATION: R88n.1 verifies zero values after a failed producer, not these selectors.
    let env = WowLuaEnv::new().expect("Retail Lua environment should initialize");
    env.exec("assert(select('#', C_TooltipInfo.GetRecipeRankInfo(999701, 1)) == 0)")
        .expect("an absent recipe must preserve the zero-value result");

    // INFERRED producer predicates: missing-parent and missing-base-spell paths are not specified.
    add_recipe(
        &mut env.state().borrow_mut().recipe_rank_tooltip_cache,
        999_702,
        999_703,
        0,
        SPELL_A,
        None,
        None,
    );
    add_recipe(
        &mut env.state().borrow_mut().recipe_rank_tooltip_cache,
        999_704,
        0,
        0,
        0,
        None,
        None,
    );
    env.exec(
        r#"
        assert(select('#', C_TooltipInfo.GetRecipeRankInfo(999702, 1)) == 0,
            'inferred missing-parent path should preserve zero values')
        assert(select('#', C_TooltipInfo.GetRecipeRankInfo(999704, 1)) == 0,
            'inferred missing-base-spell path should preserve zero values')
        "#,
    )
    .expect("the simulator's inferred missing-input predicates should remain stable");
}

#[test]
fn inferred_simulator_route_open_trade_skill_clears_recipe_rank_cache() {
    // INFERRED simulator-route coverage: the native direct caller does not prove this API trigger.
    let env = WowLuaEnv::new().expect("Retail Lua environment should initialize");
    add_recipe(
        &mut env.state().borrow_mut().recipe_rank_tooltip_cache,
        991_731,
        0,
        0,
        SPELL_A,
        None,
        None,
    );
    env.exec(
        "assert(select('#', C_TooltipInfo.GetRecipeRankInfo(991731, 1)) == 1, 'fixture must be visible before initialization')",
    )
    .expect("host recipe record should populate the tooltip before initialization");

    env.exec("assert(C_TradeSkillUI.OpenTradeSkill(164))")
        .expect("the simulator OpenTradeSkill route should run");
    env.exec(
        "assert(select('#', C_TooltipInfo.GetRecipeRankInfo(991731, 1)) == 0, 'modeled initialization route must clear the recipe cache')",
    )
    .expect("the simulator initialization route should clear the cache");
}

#[test]
fn inferred_simulator_route_dirty_filter_update_clears_recipe_rank_cache() {
    // INFERRED simulator-route coverage: the filter/tick correspondence is not native-proved.
    let env = WowLuaEnv::new().expect("Retail Lua environment should initialize");
    add_recipe(
        &mut env.state().borrow_mut().recipe_rank_tooltip_cache,
        991_741,
        0,
        0,
        SPELL_A,
        None,
        None,
    );
    env.exec(
        "assert(select('#', C_TooltipInfo.GetRecipeRankInfo(991741, 1)) == 1, 'fixture must be visible before update')",
    )
    .expect("host recipe record should populate the tooltip before update");

    env.exec("C_TradeSkillUI.SetRecipeItemNameFilter('sharpening')")
        .expect("the changed filter should mark the simulator update pending");
    wow_ui_sim::startup::fire_one_on_update_tick(&env);
    env.exec(
        "assert(select('#', C_TooltipInfo.GetRecipeRankInfo(991741, 1)) == 0, 'modeled filter-update route must clear the recipe cache')",
    )
    .expect("the simulator's deferred filter notification route should clear the cache");
}
