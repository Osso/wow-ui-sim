use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_api::state_types::{LfgActivityInfo, LfgAdvancedFilter, LfgCategoryInfo};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

fn find_activity(state: &LuaState, activity_id: u32) -> Option<LfgActivityInfo> {
    borrow_state(state)
        .ok()?
        .lfg_activities
        .iter()
        .find(|a| a.activity_id == activity_id)
        .cloned()
}

pub(super) fn get_activity_info_table(state: &mut LuaState) -> LuaResult<u32> {
    let activity_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as u32;
    let Some(act) = find_activity(state, activity_id) else {
        return Ok(0);
    };
    let info = create_table(state);
    set_activity_identity_fields(state, info, &act);
    set_activity_group_fields(state, info, &act);
    set_activity_filter_fields(state, info, &act);
    state.push(info);
    Ok(1)
}

fn set_activity_identity_fields(state: &mut LuaState, info: Val, act: &LfgActivityInfo) {
    let full_name = create_string(state, &act.full_name);
    let short_name = create_string(state, &act.short_name);
    table_set(state, info, "activityID", Val::Num(act.activity_id as f64));
    table_set(state, info, "fullName", full_name);
    table_set(state, info, "shortName", short_name);
}

fn set_activity_group_fields(state: &mut LuaState, info: Val, act: &LfgActivityInfo) {
    table_set(state, info, "categoryID", Val::Num(act.category_id as f64));
    table_set(
        state,
        info,
        "groupFinderActivityGroupID",
        Val::Num(act.group_id as f64),
    );
    table_set(state, info, "maxPlayers", Val::Num(act.max_players as f64));
}

fn set_activity_filter_fields(state: &mut LuaState, info: Val, act: &LfgActivityInfo) {
    table_set(
        state,
        info,
        "displayType",
        Val::Num(act.display_type as f64),
    );
    table_set(state, info, "filters", Val::Num(act.filters as f64));
    let is_mplus = act.group_id == 295;
    table_set(state, info, "isMythicPlusActivity", Val::Bool(is_mplus));
    table_set(
        state,
        info,
        "allowCrossFaction",
        Val::Bool(act.allow_cross_faction),
    );
    table_set(
        state,
        info,
        "ilvlSuggestion",
        Val::Num(act.item_level as f64),
    );
    table_set(state, info, "useHonorLevel", Val::Bool(act.use_honor_level));
}

/// `GetAvailableCategories(filters?)` -> array of category IDs ordered by
/// `LfgCategoryInfo.order`. If filters is 0/nil every category is returned.
pub(super) fn get_available_categories(state: &mut LuaState) -> LuaResult<u32> {
    let _filters = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as u32;
    let mut cats = {
        let sim = borrow_state(state)?;
        sim.lfg_category_info
            .iter()
            .map(|(id, info)| (*id, info.order))
            .collect::<Vec<_>>()
    };
    cats.sort_by_key(|(_, order)| *order);
    let result = create_table(state);
    if let Val::Table(table_ref) = result {
        for (index, (cat_id, _)) in cats.iter().enumerate() {
            if let Some(table) = state.gc.tables.get_mut(table_ref) {
                let _ = table.raw_set(
                    Val::Num(index as f64 + 1.0),
                    Val::Num(*cat_id as f64),
                    &state.gc.string_arena,
                );
            }
        }
        state.gc.barrier_back(table_ref);
    }
    state.push(result);
    Ok(1)
}

/// `GetLfgCategoryInfo(categoryID)` -> table or nil.
pub(super) fn get_lfg_category_info(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let info_opt = borrow_state(state)?
        .lfg_category_info
        .get(&category_id)
        .cloned();
    let Some(info) = info_opt else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = create_table(state);
    set_lfg_category_info_fields(state, table, &info);
    state.push(table);
    Ok(1)
}

fn set_lfg_category_info_fields(state: &mut LuaState, table: Val, info: &LfgCategoryInfo) {
    let name = create_string(state, &info.name);
    table_set(state, table, "name", name);
    set_lfg_category_info_flags(state, table, info);
    table_set(state, table, "searchPromptOverride", Val::Nil);
}

fn set_lfg_category_info_flags(state: &mut LuaState, table: Val, info: &LfgCategoryInfo) {
    table_set(
        state,
        table,
        "separateRecommended",
        Val::Bool(info.separate_recommended),
    );
    table_set(
        state,
        table,
        "preferCurrentArea",
        Val::Bool(info.prefer_current_area),
    );
    table_set(
        state,
        table,
        "allowCrossFaction",
        Val::Bool(info.allow_cross_faction),
    );
    table_set(
        state,
        table,
        "autoChooseActivity",
        Val::Bool(info.auto_choose_activity),
    );
    table_set(
        state,
        table,
        "showPlaystyleDropdown",
        Val::Bool(info.show_playstyle_dropdown),
    );
}

/// `GetAvailableActivityGroups(categoryID, filters?)` -> array of groupIDs.
pub(super) fn get_available_activity_groups(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let filters = Option::<f64>::from_stack(state, 2)?.unwrap_or(0.0) as u32;
    let mut groups = {
        let sim = borrow_state(state)?;
        sim.lfg_activity_groups
            .iter()
            .filter(|g| g.category_id == category_id && (filters == 0 || g.filters & filters != 0))
            .map(|g| (g.group_id, g.order_index))
            .collect::<Vec<_>>()
    };
    groups.sort_by_key(|(_, order)| *order);
    let result = create_table(state);
    if let Val::Table(table_ref) = result {
        for (index, (group_id, _)) in groups.iter().enumerate() {
            if let Some(table) = state.gc.tables.get_mut(table_ref) {
                let _ = table.raw_set(
                    Val::Num(index as f64 + 1.0),
                    Val::Num(*group_id as f64),
                    &state.gc.string_arena,
                );
            }
        }
        state.gc.barrier_back(table_ref);
    }
    state.push(result);
    Ok(1)
}

/// `GetAvailableActivities(categoryID?, groupID?, filters?, searchTerm?)`
/// -> array of activityIDs.
pub(super) fn get_available_activities(state: &mut LuaState) -> LuaResult<u32> {
    let criteria = AvailableActivityCriteria::from_stack(state)?;
    let mut activities = {
        let sim = borrow_state(state)?;
        sim.lfg_activities
            .iter()
            .filter(|activity| criteria.matches(activity))
            .map(|a| (a.activity_id, a.order_index))
            .collect::<Vec<_>>()
    };
    activities.sort_by_key(|(_, order)| *order);
    let result = ordered_activity_id_table(state, &activities);
    state.push(result);
    Ok(1)
}

struct AvailableActivityCriteria {
    category_id: Option<i32>,
    group_id: Option<u32>,
    filters: u32,
    search_term: Option<String>,
}

impl AvailableActivityCriteria {
    fn from_stack(state: &mut LuaState) -> LuaResult<Self> {
        let category_id = Option::<f64>::from_stack(state, 1)?.map(|v| v as i32);
        let group_id = Option::<f64>::from_stack(state, 2)?.map(|v| v as u32);
        let filters = Option::<f64>::from_stack(state, 3)?.unwrap_or(0.0) as u32;
        let search_term = Option::<String>::from_stack(state, 4)?.map(|s| s.to_lowercase());
        Ok(Self {
            category_id,
            group_id,
            filters,
            search_term,
        })
    }

    fn matches(&self, activity: &LfgActivityInfo) -> bool {
        self.matches_category(activity)
            && self.matches_group(activity)
            && self.matches_filters(activity)
            && self.matches_search_term(activity)
    }

    fn matches_category(&self, activity: &LfgActivityInfo) -> bool {
        self.category_id
            .is_none_or(|category_id| activity.category_id == category_id)
    }

    fn matches_group(&self, activity: &LfgActivityInfo) -> bool {
        self.group_id
            .is_none_or(|group_id| activity.group_id == group_id)
    }

    fn matches_filters(&self, activity: &LfgActivityInfo) -> bool {
        self.filters == 0 || activity.filters & self.filters != 0
    }

    fn matches_search_term(&self, activity: &LfgActivityInfo) -> bool {
        self.search_term
            .as_ref()
            .is_none_or(|term| activity.full_name.to_lowercase().contains(term.as_str()))
    }
}

fn ordered_activity_id_table(state: &mut LuaState, activities: &[(u32, i32)]) -> Val {
    let result = create_table(state);
    if let Val::Table(table_ref) = result {
        for (index, (activity_id, _)) in activities.iter().enumerate() {
            if let Some(table) = state.gc.tables.get_mut(table_ref) {
                let _ = table.raw_set(
                    Val::Num(index as f64 + 1.0),
                    Val::Num(*activity_id as f64),
                    &state.gc.string_arena,
                );
            }
        }
        state.gc.barrier_back(table_ref);
    }
    result
}

/// `GetActivityGroupInfo(groupID)` -> (name, orderIndex) or nothing.
pub(super) fn get_activity_group_info(state: &mut LuaState) -> LuaResult<u32> {
    let group_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as u32;
    let found = borrow_state(state)?
        .lfg_activity_groups
        .iter()
        .find(|g| g.group_id == group_id)
        .map(|g| (g.name.clone(), g.order_index));
    let Some((name, order_index)) = found else {
        return Ok(0);
    };
    let name_val = create_string(state, &name);
    state.push(name_val);
    state.push(Val::Num(order_index as f64));
    Ok(2)
}

/// `GetActivityFullName(activityID, questID?, showWarmode?)` -> string.
pub(super) fn get_activity_full_name(state: &mut LuaState) -> LuaResult<u32> {
    let activity_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as u32;
    let name = borrow_state(state)?
        .lfg_activities
        .iter()
        .find(|a| a.activity_id == activity_id)
        .map(|a| a.full_name.clone())
        .unwrap_or_default();
    let val = create_string(state, &name);
    state.push(val);
    Ok(1)
}

/// `GetPlaystyleString(playstyle, generalPlaystyle, activityInfo)` -> string.
///
/// Retail always returns a string. The current sim seeds modern
/// `generalPlaystyle` values, so map those first and fall back to the legacy
/// `playstyle` enum only when no general playstyle is provided.
pub(super) fn get_playstyle_string(state: &mut LuaState) -> LuaResult<u32> {
    let playstyle = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let general_playstyle = Option::<f64>::from_stack(state, 2)?.unwrap_or(0.0) as i32;
    let label = match general_playstyle {
        1 => "Learning",
        2 => "Relaxed",
        3 => "Focused",
        4 => "Expert",
        _ => match playstyle {
            1 => "Standard",
            2 => "Casual",
            3 => "Hardcore",
            _ => "",
        },
    };
    let val = create_string(state, label);
    state.push(val);
    Ok(1)
}

/// `HasActivityList()` -> bool. True when the activity catalog is seeded.
pub(super) fn has_activity_list(state: &mut LuaState) -> LuaResult<u32> {
    let has = !borrow_state(state)?.lfg_activities.is_empty();
    state.push(Val::Bool(has));
    Ok(1)
}

/// `GetAvailableRoles()` -> (tank, healer, dps). Always true.
pub(super) fn get_available_roles(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    state.push(Val::Bool(true));
    state.push(Val::Bool(true));
    Ok(3)
}

/// `GetAvailableLanguageSearchFilter()` -> array of language strings.
pub(super) fn get_available_language_search_filter(state: &mut LuaState) -> LuaResult<u32> {
    let result = create_table(state);
    if let Val::Table(table_ref) = result {
        let lang = state.gc.intern_string_static(b"enUS");
        if let Some(table) = state.gc.tables.get_mut(table_ref) {
            let _ = table.raw_set(Val::Num(1.0), Val::Str(lang), &state.gc.string_arena);
        }
        state.gc.barrier_back(table_ref);
    }
    state.push(result);
    Ok(1)
}

/// `GetLanguageSearchFilter()` -> `{ enUS = true, ... }`.
pub(super) fn get_language_search_filter(state: &mut LuaState) -> LuaResult<u32> {
    let entries = {
        let sim = borrow_state(state)?;
        sim.lfg_language_filter
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect::<Vec<_>>()
    };
    let result = create_table(state);
    if let Val::Table(table_ref) = result {
        for (lang, enabled) in &entries {
            let key = create_string(state, lang);
            if let Val::Str(s) = key
                && let Some(t) = state.gc.tables.get_mut(table_ref)
            {
                let _ = t.raw_set(Val::Str(s), Val::Bool(*enabled), &state.gc.string_arena);
            }
        }
        state.gc.barrier_back(table_ref);
    }
    state.push(result);
    Ok(1)
}

/// `GetDefaultLanguageSearchFilter()` -> current default language map.
pub(super) fn get_default_language_search_filter(state: &mut LuaState) -> LuaResult<u32> {
    get_language_search_filter(state)
}

/// `GetAdvancedFilter()` -> table mirroring `LfgAdvancedFilter`.
pub(super) fn get_advanced_filter(state: &mut LuaState) -> LuaResult<u32> {
    let filter = borrow_state(state)?.lfg_advanced_filter.clone();
    let info = create_table(state);
    set_advanced_filter_role_fields(state, info, &filter);
    table_set(
        state,
        info,
        "minimumRating",
        Val::Num(filter.minimum_rating as f64),
    );
    set_advanced_filter_activities(state, info, &filter.activities);
    set_advanced_filter_difficulty_fields(state, info, &filter);
    set_advanced_filter_playstyle_fields(state, info, &filter);
    state.push(info);
    Ok(1)
}

fn set_advanced_filter_role_fields(state: &mut LuaState, info: Val, filter: &LfgAdvancedFilter) {
    table_set(state, info, "needsTank", Val::Bool(filter.needs_tank));
    table_set(state, info, "needsHealer", Val::Bool(filter.needs_healer));
    table_set(state, info, "needsDamage", Val::Bool(filter.needs_damage));
    table_set(
        state,
        info,
        "needsMyClass",
        Val::Bool(filter.needs_my_class),
    );
    table_set(state, info, "hasTank", Val::Bool(filter.has_tank));
    table_set(state, info, "hasHealer", Val::Bool(filter.has_healer));
}

fn set_advanced_filter_activities(state: &mut LuaState, info: Val, activities: &[u32]) {
    let activities_table = u32_array_table(state, activities);
    table_set(state, info, "activities", activities_table);
}

pub(crate) fn u32_array_table(state: &mut LuaState, values: &[u32]) -> Val {
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return table;
    };
    for (index, value) in values.iter().enumerate() {
        if let Some(table) = state.gc.tables.get_mut(table_ref) {
            let _ = table.raw_set(
                Val::Num(index as f64 + 1.0),
                Val::Num(*value as f64),
                &state.gc.string_arena,
            );
        }
    }
    state.gc.barrier_back(table_ref);
    table
}

fn set_advanced_filter_difficulty_fields(
    state: &mut LuaState,
    info: Val,
    filter: &LfgAdvancedFilter,
) {
    table_set(
        state,
        info,
        "difficultyNormal",
        Val::Bool(filter.difficulty_normal),
    );
    table_set(
        state,
        info,
        "difficultyHeroic",
        Val::Bool(filter.difficulty_heroic),
    );
    table_set(
        state,
        info,
        "difficultyMythic",
        Val::Bool(filter.difficulty_mythic),
    );
    table_set(
        state,
        info,
        "difficultyMythicPlus",
        Val::Bool(filter.difficulty_mythic_plus),
    );
}

fn set_advanced_filter_playstyle_fields(
    state: &mut LuaState,
    info: Val,
    filter: &LfgAdvancedFilter,
) {
    table_set(
        state,
        info,
        "generalPlaystyle1",
        Val::Bool(filter.general_playstyle1),
    );
    table_set(
        state,
        info,
        "generalPlaystyle2",
        Val::Bool(filter.general_playstyle2),
    );
    table_set(
        state,
        info,
        "generalPlaystyle3",
        Val::Bool(filter.general_playstyle3),
    );
    table_set(
        state,
        info,
        "generalPlaystyle4",
        Val::Bool(filter.general_playstyle4),
    );
}
