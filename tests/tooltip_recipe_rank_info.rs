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
const SPELL_B: u32 = 467_911;
const RECIPE_A_BASE: i32 = 990_701;
const RECIPE_A_NEXT: i32 = 990_702;
const RECIPE_B_BASE: i32 = 990_711;
const RECIPE_B_NEXT: i32 = 990_712;
const RECIPE_RANK_BASE: i32 = 990_721;
const RECIPE_RANK_NEXT: i32 = 990_722;

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

#[test]
fn recipe_id_selects_distinct_chains_through_original_tooltip_accessor() {
    let env = original_tooltip_handler_env();
    add_recipe_pair(
        &env,
        RECIPE_A_BASE,
        RECIPE_A_NEXT,
        SPELL_A,
        SPELL_B,
        Some("Host recipe label"),
        None,
    );
    add_recipe_pair(
        &env,
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

    env.exec(&format!(
        r#"
        local function query(recipeID)
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetRecipeRankInfo(recipeID, 2)
            return GameTooltip:GetPrimaryTooltipData()
        end
        local first = query({RECIPE_A_NEXT})
        assert(first, 'RED: recipeID must publish its supplied recipe chain')
        assert(first.type == Enum.TooltipDataType.RecipeRankInfo)
        assert(first.id == {RECIPE_A_NEXT})
        local firstName = C_TooltipInfo.GetSpellByID({SPELL_A}).lines[1].leftText
        assert(first.lines[1].leftText == 'Host recipe label: ' .. firstName)

        local second = query({RECIPE_B_NEXT})
        assert(second and second.type == Enum.TooltipDataType.RecipeRankInfo)
        assert(second.id == {RECIPE_B_NEXT})
        local secondName = C_TooltipInfo.GetSpellByID({SPELL_B}).lines[1].leftText
        assert(second.lines[1].leftText == secondName)
        assert(first.lines[1].leftText ~= second.lines[1].leftText)
        "#
    ))
    .expect("the original tooltip accessor must select the requested recipe chain");
}

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
        Some("Host supplied next-rank source"),
    );

    env.exec(&format!(
        r#"
        local function query(rank)
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetRecipeRankInfo({RECIPE_RANK_BASE}, rank)
            return GameTooltip:GetPrimaryTooltipData()
        end
        local function findText(data, text)
            for _, line in ipairs(data.lines) do
                if line.leftText == text then return line end
            end
        end
        local firstSpell = C_TooltipInfo.GetSpellByID({SPELL_A})
        local secondSpell = C_TooltipInfo.GetSpellByID({SPELL_B})
        local firstDescription = firstSpell.lines[#firstSpell.lines].leftText
        local secondDescription = secondSpell.lines[#secondSpell.lines].leftText
        assert(firstDescription and firstDescription ~= '')
        assert(secondDescription and secondDescription ~= '')
        local normalR, normalG, normalB = NORMAL_FONT_COLOR:GetRGB()
        local disabledR, disabledG, disabledB = DISABLED_FONT_COLOR:GetRGB()

        local learned = query(1)
        assert(learned, 'RED: rank must control the supplied descriptions')
        assert(learned.type == Enum.TooltipDataType.RecipeRankInfo)
        assert(learned.id == {RECIPE_RANK_BASE})
        local learnedFirst = findText(learned, firstDescription)
        local learnedSecond = findText(learned, secondDescription)
        assert(learnedFirst and learnedFirst.wrapText == true)
        assert(learnedSecond and learnedSecond.wrapText == true)
        assert(math.abs(learnedFirst.leftColor.r - normalR) < 0.001)
        assert(math.abs(learnedFirst.leftColor.g - normalG) < 0.001)
        assert(math.abs(learnedFirst.leftColor.b - normalB) < 0.001)
        assert(math.abs(learnedSecond.leftColor.r - disabledR) < 0.001)
        assert(math.abs(learnedSecond.leftColor.g - disabledG) < 0.001)
        assert(math.abs(learnedSecond.leftColor.b - disabledB) < 0.001)
        assert(findText(learned, TRADESKILL_NEXT_RANK_HEADER))
        assert(findText(learned, 'Host supplied next-rank source'))

        local allRanks = query(2)
        assert(allRanks and allRanks.id == {RECIPE_RANK_BASE})
        local allRanksSecond = findText(allRanks, secondDescription)
        assert(allRanksSecond)
        assert(math.abs(allRanksSecond.leftColor.r - normalR) < 0.001)
        assert(math.abs(allRanksSecond.leftColor.g - normalG) < 0.001)
        assert(math.abs(allRanksSecond.leftColor.b - normalB) < 0.001)
        assert(not findText(allRanks, TRADESKILL_NEXT_RANK_HEADER))
        assert(not findText(allRanks, 'Host supplied next-rank source'))
        "#
    ))
    .expect("the original tooltip accessor must apply the rank display threshold");
}

#[test]
fn recipe_rank_misses_return_zero_lua_values() {
    let env = WowLuaEnv::new().expect("Retail Lua environment should initialize");
    env.exec("assert(select('#', C_TooltipInfo.GetRecipeRankInfo(999701, 1)) == 0)")
        .expect("an absent recipe must return zero values");

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
            'a missing required parent must return zero values')
        assert(select('#', C_TooltipInfo.GetRecipeRankInfo(999704, 1)) == 0,
            'a missing initial spell name must return zero values')
        "#,
    )
    .expect("R88n.1 recipe misses must return exactly zero values");
}
