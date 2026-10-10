//! Retail 12.0.5 trusted addon instruction accounting and shutdown exemptions.
//! Quotas and reset periods are host policy, not native elapsed-time parity.

use super::methods::borrow_state;
use rilua::vm::state::{InstructionBudget, LuaState};
use rilua::{LuaResult, Val};

const DEFAULT_INSTRUCTION_LIMIT: u64 = 10_000_000;

pub(crate) fn with_addon_budget<T>(
    state: &mut LuaState,
    owner: &str,
    operation: impl FnOnce(&mut LuaState) -> LuaResult<T>,
) -> LuaResult<T> {
    if state.instruction_budget(owner).is_none() {
        state.set_instruction_budget(owner, Some(DEFAULT_INSTRUCTION_LIMIT));
    }
    state.with_instruction_owner(owner, operation)
}

pub(crate) fn with_event_exemption<T>(
    state: &mut LuaState,
    event: Option<&str>,
    operation: impl FnOnce(&mut LuaState) -> T,
) -> T {
    if matches!(event, Some("PLAYER_LOGOUT" | "ADDONS_UNLOADING")) {
        state.with_instruction_budget_exemption(operation)
    } else {
        operation(state)
    }
}

pub(crate) fn call_frame_handler(
    state: &mut LuaState,
    frame_id: u64,
    handler: Val,
    args: &[Val],
    event: Option<&str>,
) -> Result<Vec<Val>, String> {
    let owner = frame_addon_owner(state, frame_id).map_err(|error| error.to_string())?;
    with_event_exemption(state, event, |state| {
        let call = |state: &mut LuaState| {
            super::script_helpers::protected_lua_pcall_state(state, handler, args)
                .map_err(rilua::runtime_error)
        };
        let result = match owner {
            Some(owner) => with_addon_budget(state, &owner, |state| {
                let logging = super::handler_timing::is_enabled();
                let before = logging.then(|| state.instruction_budget(&owner)).flatten();
                let result = call(state);
                if logging && result.is_err() {
                    eprintln!(
                        "{}",
                        format_budget_error(
                            &owner,
                            frame_id,
                            event,
                            before,
                            state.instruction_budget(&owner),
                        )
                    );
                }
                result
            }),
            None => call(state),
        };
        result.map_err(|error| error.to_string())
    })
}

fn format_budget_error(
    owner: &str,
    frame_id: u64,
    event: Option<&str>,
    before: Option<InstructionBudget>,
    after: Option<InstructionBudget>,
) -> String {
    let mut line = format!("[handler-budget-error] owner={owner:?} frame=#{frame_id}");
    if let Some(event) = event {
        line.push_str(&format!(" event={event:?}"));
    }
    line.push_str(&format_budget_counters(before, after));
    line
}

pub(crate) fn format_file_budget_error(
    owner: &str,
    chunk_name: &str,
    before: Option<InstructionBudget>,
    after: Option<InstructionBudget>,
) -> String {
    format!(
        "[file-budget-error] owner={owner:?} file={chunk_name:?}{}",
        format_budget_counters(before, after),
    )
}

pub(crate) fn format_file_budget_success(
    owner: &str,
    chunk_name: &str,
    before: Option<InstructionBudget>,
    after: Option<InstructionBudget>,
) -> String {
    format!(
        "[file-budget-success] owner={owner:?} file={chunk_name:?}{}",
        format_budget_counters(before, after),
    )
}

fn format_budget_counters(
    before: Option<InstructionBudget>,
    after: Option<InstructionBudget>,
) -> String {
    let limit = match before {
        Some(InstructionBudget {
            limit: Some(limit), ..
        }) => limit.to_string(),
        Some(InstructionBudget { limit: None, .. }) => "none".to_owned(),
        None => "unavailable".to_owned(),
    };
    let usage = |budget: Option<InstructionBudget>| {
        budget.map_or_else(
            || "unavailable".to_owned(),
            |budget| budget.used.to_string(),
        )
    };
    format!(
        " limit={limit} used_before={} used_after={}",
        usage(before),
        usage(after),
    )
}

fn frame_addon_owner(state: &LuaState, frame_id: u64) -> LuaResult<Option<String>> {
    let sim = borrow_state(state)?;
    let addon = sim
        .widgets
        .get(frame_id)
        .and_then(|frame| frame.owner_addon)
        .and_then(|index| sim.addons.get(index as usize));
    Ok(addon
        .filter(|addon| {
            addon.folder_name != "__BuiltIn"
                && !crate::blizzard_ui_sync::is_builtin_addon_folder(&addon.folder_name)
        })
        .map(|addon| addon.folder_name.clone()))
}

pub(crate) fn reset_frame_budgets(state: &mut LuaState) -> LuaResult<()> {
    let owners: Vec<String> = borrow_state(state)?
        .addons
        .iter()
        .map(|addon| addon.folder_name.clone())
        .collect();
    for owner in owners {
        if state.instruction_budget(&owner).is_some() {
            state.reset_instruction_usage(&owner)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_budget_success_log_reports_cumulative_counters_and_escaped_identity() {
        let before = InstructionBudget {
            limit: Some(1_000),
            used: 27,
        };
        let after = InstructionBudget { used: 53, ..before };
        assert_eq!(
            format_file_budget_success(
                "Example\"\n",
                "@Interface/AddOns/Example/Group\"Tools\n.lua",
                Some(before),
                Some(after),
            ),
            "[file-budget-success] owner=\"Example\\\"\\n\" file=\"@Interface/AddOns/Example/Group\\\"Tools\\n.lua\" limit=1000 used_before=27 used_after=53"
        );
    }

    #[test]
    fn file_budget_success_log_labels_missing_snapshots() {
        assert_eq!(
            format_file_budget_success("Example", "@Interface/AddOns/Example/Main.lua", None, None),
            "[file-budget-success] owner=\"Example\" file=\"@Interface/AddOns/Example/Main.lua\" limit=unavailable used_before=unavailable used_after=unavailable"
        );
    }

    #[test]
    fn file_budget_success_log_reports_unlimited_meter() {
        let before = InstructionBudget {
            limit: None,
            used: 23,
        };
        let after = InstructionBudget { used: 41, ..before };
        assert_eq!(
            format_file_budget_success(
                "Example",
                "@Interface/AddOns/Example/Main.lua",
                Some(before),
                Some(after),
            ),
            "[file-budget-success] owner=\"Example\" file=\"@Interface/AddOns/Example/Main.lua\" limit=none used_before=23 used_after=41"
        );
    }

    #[test]
    fn file_budget_error_log_distinguishes_exhausted_entry_from_file_consumption() {
        let exhausted = InstructionBudget {
            limit: Some(100),
            used: 100,
        };
        let chunk = "@Interface/AddOns/Example/Settings/GroupTools.lua";
        assert_eq!(
            format_file_budget_error("Example", chunk, Some(exhausted), Some(exhausted)),
            "[file-budget-error] owner=\"Example\" file=\"@Interface/AddOns/Example/Settings/GroupTools.lua\" limit=100 used_before=100 used_after=100"
        );
        assert_eq!(
            format_file_budget_error(
                "Example",
                chunk,
                Some(InstructionBudget {
                    used: 27,
                    ..exhausted
                }),
                Some(exhausted),
            ),
            "[file-budget-error] owner=\"Example\" file=\"@Interface/AddOns/Example/Settings/GroupTools.lua\" limit=100 used_before=27 used_after=100"
        );
    }

    #[test]
    fn file_budget_error_log_labels_missing_snapshots() {
        assert_eq!(
            format_file_budget_error("Example", "@Interface/AddOns/Example/Main.lua", None, None),
            "[file-budget-error] owner=\"Example\" file=\"@Interface/AddOns/Example/Main.lua\" limit=unavailable used_before=unavailable used_after=unavailable"
        );
    }

    #[test]
    fn file_budget_error_log_escapes_metadata_and_reports_unlimited_meter() {
        let budget = InstructionBudget {
            limit: None,
            used: 23,
        };
        assert_eq!(
            format_file_budget_error(
                "Example\"\n",
                "@Interface/AddOns/Example/Settings/Group\"Tools\n.lua",
                Some(budget),
                Some(budget),
            ),
            "[file-budget-error] owner=\"Example\\\"\\n\" file=\"@Interface/AddOns/Example/Settings/Group\\\"Tools\\n.lua\" limit=none used_before=23 used_after=23"
        );
    }

    #[test]
    fn budget_error_log_distinguishes_exhausted_entry_from_callback_consumption() {
        let before = InstructionBudget {
            limit: Some(10),
            used: 10,
        };
        let after = before;
        assert_eq!(
            format_budget_error(
                "DynamicAnchors",
                42,
                Some("PLAYER_LOGIN"),
                Some(before),
                Some(after)
            ),
            "[handler-budget-error] owner=\"DynamicAnchors\" frame=#42 event=\"PLAYER_LOGIN\" limit=10 used_before=10 used_after=10"
        );
        assert_eq!(
            format_budget_error(
                "DynamicAnchors",
                42,
                Some("PLAYER_LOGIN"),
                Some(InstructionBudget { used: 3, ..before }),
                Some(after)
            ),
            "[handler-budget-error] owner=\"DynamicAnchors\" frame=#42 event=\"PLAYER_LOGIN\" limit=10 used_before=3 used_after=10"
        );
    }

    #[test]
    fn budget_error_log_labels_unavailable_snapshots_without_inventing_counts() {
        assert_eq!(
            format_budget_error("Example", 7, None, None, None),
            "[handler-budget-error] owner=\"Example\" frame=#7 limit=unavailable used_before=unavailable used_after=unavailable"
        );
    }

    #[test]
    fn budget_error_log_reports_unlimited_meter_and_escapes_event_metadata() {
        let budget = InstructionBudget {
            limit: None,
            used: 23,
        };
        assert_eq!(
            format_budget_error(
                "Example",
                7,
                Some("CUSTOM\nEVENT"),
                Some(budget),
                Some(budget)
            ),
            "[handler-budget-error] owner=\"Example\" frame=#7 event=\"CUSTOM\\nEVENT\" limit=none used_before=23 used_after=23"
        );
    }
}
