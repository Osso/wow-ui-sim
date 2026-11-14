//! Shared composition for `GetRecipeRankInfo`.

use std::collections::HashSet;

use crate::c_api::recipe_rank_tooltip::{RecipeRankTooltipCache, RecipeRankTooltipRecord};
use crate::lua_api::methods::{borrow_state, table_get};
use crate::lua_api::state::SimState;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// VERIFIED (R88n.4): 17 is the top-level TooltipData type.
const TOOLTIP_TYPE_RECIPE_RANK_INFO: f64 = 17.0;
// INFERRED: the shared text-row builder receives line type 0.
const TOOLTIP_LINE_TYPE_TEXT: f64 = 0.0;
// INFERRED: the native evidence does not establish the recipe-label delimiter.
const RECIPE_LABEL_SEPARATOR: &str = ": ";

struct PreparedRank {
    number: i32,
    description: Option<String>,
    next_rank_source: Option<String>,
}

struct PreparedRecipeRankInfo {
    title: String,
    ranks: Vec<PreparedRank>,
}

struct RecipeRankTooltipLine {
    text: String,
    color: Option<(f64, f64, f64)>,
    wrap: bool,
}

struct RankDisplayContext<'a> {
    requested_rank: i32,
    first_next_rank: Option<i32>,
    next_rank_header: Option<&'a str>,
    normal_color: (f64, f64, f64),
    disabled_color: (f64, f64, f64),
}

impl RankDisplayContext<'_> {
    fn description_color(&self, rank_number: i32) -> (f64, f64, f64) {
        // INFERRED: the inclusive comparison and one-based numbering are simulator policy.
        if rank_number <= self.requested_rank {
            self.normal_color
        } else {
            self.disabled_color
        }
    }
}

pub(crate) fn build_recipe_rank_info(
    state: &mut LuaState,
    recipe_id: i32,
    rank: i32,
) -> LuaResult<Option<Val>> {
    let prepared = {
        let sim = borrow_state(state)?;
        prepare_recipe_rank_info(&sim, recipe_id)
    };
    let Some(prepared) = prepared else {
        return Ok(None);
    };
    // INFERRED: missing color globals make this simulator producer miss.
    let Some((normal_color, disabled_color)) = description_colors(state) else {
        return Ok(None);
    };
    let lines = prepare_tooltip_lines(state, prepared, rank, normal_color, disabled_color)?;
    let tooltip = super::compose_tooltip(
        state,
        TOOLTIP_TYPE_RECIPE_RANK_INFO,
        Some(f64::from(recipe_id)),
    );
    publish_tooltip_lines(state, tooltip, lines);
    Ok(Some(tooltip))
}

fn description_colors(state: &mut LuaState) -> Option<((f64, f64, f64), (f64, f64, f64))> {
    // INFERRED: color globals and packed RGB are not recipe-specific native proof.
    let normal = super::named_color(state, "NORMAL_FONT_COLOR")?;
    let disabled = super::named_color(state, "DISABLED_FONT_COLOR")?;
    Some((normal, disabled))
}

fn publish_tooltip_lines(state: &mut LuaState, tooltip: Val, lines: Vec<RecipeRankTooltipLine>) {
    let saved_top = state.top;
    state.push(tooltip);
    let lines_table = table_get(state, tooltip, "lines");
    for (index, tooltip_line) in lines.into_iter().enumerate() {
        super::builders::push_tooltip_line(
            state,
            lines_table,
            index as i64 + 1,
            TOOLTIP_LINE_TYPE_TEXT,
            &tooltip_line.text,
            tooltip_line.color,
            tooltip_line.wrap,
        );
    }
    state.top = saved_top;
}

fn prepare_recipe_rank_info(sim: &SimState, recipe_id: i32) -> Option<PreparedRecipeRankInfo> {
    // INFERRED: this host model treats a missing parent record or base spell as producer failure.
    let (base_id, base_record) = find_base_recipe(&sim.recipe_rank_tooltip_cache, recipe_id)?;
    let base_spell = crate::spells::get_spell(base_record.spell_id)?;
    let title = match base_record.label.as_deref() {
        Some(label) => format!("{label}{RECIPE_LABEL_SEPARATOR}{}", base_spell.name),
        None => base_spell.name.to_owned(),
    };
    let ranks = collect_rank_rows(sim, &sim.recipe_rank_tooltip_cache, base_id)?;
    Some(PreparedRecipeRankInfo { title, ranks })
}

fn find_base_recipe(
    cache: &RecipeRankTooltipCache,
    recipe_id: i32,
) -> Option<(i32, &RecipeRankTooltipRecord)> {
    let mut current_id = recipe_id;
    let mut visited = HashSet::new();
    // INFERRED: cycles return no result; nonpositive parent IDs are this simulator's base sentinel.
    loop {
        if !visited.insert(current_id) {
            return None;
        }
        let record = cache.get(current_id)?;
        if record.parent_recipe_id <= 0 {
            return Some((current_id, record));
        }
        current_id = record.parent_recipe_id;
    }
}

fn collect_rank_rows(
    sim: &SimState,
    cache: &RecipeRankTooltipCache,
    base_id: i32,
) -> Option<Vec<PreparedRank>> {
    // INFERRED: cycles/missing successors stop traversal; ranks start at one, empty descriptions
    // are omitted, and an empty chain is a miss.
    let mut ranks = Vec::new();
    let mut current_id = base_id;
    let mut visited = HashSet::new();
    loop {
        if !visited.insert(current_id) {
            break;
        }
        let Some(record) = cache.get(current_id) else {
            break;
        };
        if crate::spells::get_spell(record.spell_id).is_none() {
            break;
        }
        let description = crate::spell_description_resolver::resolve_spell_description_or_empty(
            sim,
            record.spell_id,
        );
        let number = i32::try_from(ranks.len() + 1).ok()?;
        ranks.push(PreparedRank {
            number,
            description: (!description.is_empty()).then_some(description),
            next_rank_source: record.next_rank_source.clone(),
        });
        if record.next_recipe_id <= 0 {
            break;
        }
        current_id = record.next_recipe_id;
    }
    (!ranks.is_empty()).then_some(ranks)
}

fn prepare_tooltip_lines(
    state: &mut LuaState,
    prepared: PreparedRecipeRankInfo,
    rank: i32,
    normal_color: (f64, f64, f64),
    disabled_color: (f64, f64, f64),
) -> LuaResult<Vec<RecipeRankTooltipLine>> {
    // INFERRED: the strict threshold and `TRADESKILL_NEXT_RANK_HEADER` are simulator policy.
    let first_next_rank = prepared
        .ranks
        .iter()
        .find(|entry| entry.number > rank)
        .map(|entry| entry.number);
    let next_rank_header =
        first_next_rank.and_then(|_| super::global_text(state, "TRADESKILL_NEXT_RANK_HEADER"));
    let display = RankDisplayContext {
        requested_rank: rank,
        first_next_rank,
        next_rank_header: next_rank_header.as_deref(),
        normal_color,
        disabled_color,
    };
    let PreparedRecipeRankInfo { title, ranks } = prepared;
    let mut lines = vec![plain_line(title)];
    for entry in &ranks {
        append_rank_lines(state, &mut lines, entry, &display)?;
    }
    Ok(lines)
}

fn append_rank_lines(
    state: &mut LuaState,
    lines: &mut Vec<RecipeRankTooltipLine>,
    entry: &PreparedRank,
    display: &RankDisplayContext<'_>,
) -> LuaResult<()> {
    // INFERRED: blank separators and the rank-header localization key are simulator policy.
    if entry.number > 1 {
        lines.push(plain_line(String::new()));
    }
    append_next_rank_lines(lines, entry, display);
    let header =
        super::format_global_numeric(state, "TRADESKILL_RANK_HEADER", f64::from(entry.number))?;
    lines.push(plain_line(header));
    if let Some(description) = &entry.description {
        lines.push(RecipeRankTooltipLine {
            text: description.clone(),
            color: Some(display.description_color(entry.number)),
            // INFERRED: recipe description rows set wrapText=true in this simulator.
            wrap: true,
        });
    }
    Ok(())
}

fn append_next_rank_lines(
    lines: &mut Vec<RecipeRankTooltipLine>,
    entry: &PreparedRank,
    display: &RankDisplayContext<'_>,
) {
    // INFERRED: the header/source placement and header-dependent source are not native-proved.
    if display.first_next_rank != Some(entry.number) {
        return;
    }
    let Some(header) = display.next_rank_header else {
        return;
    };
    lines.push(plain_line(header.to_owned()));
    append_next_rank_source(lines, entry.next_rank_source.as_deref());
}

fn append_next_rank_source(lines: &mut Vec<RecipeRankTooltipLine>, source: Option<&str>) {
    if let Some(source) = source {
        lines.push(plain_line(source.to_owned()));
    }
}

fn plain_line(text: String) -> RecipeRankTooltipLine {
    // INFERRED: title/header/source rows use wrapText=false.
    RecipeRankTooltipLine {
        text,
        color: None,
        wrap: false,
    }
}
