//! Original-consumer filter-state RED; not native event-matching proof.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

#[test]
fn original_processor_replaces_and_clears_filter_records() {
    use crate::common::blizzard_addon_harness::blizzard_ui_dir;
    use crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env;

    let (env, loaded) =
        build_blizzard_addon_closure_env(&blizzard_ui_dir(), &["Blizzard_CombatLogProcessor"], &[]);
    assert!(
        loaded
            .iter()
            .any(|name| name == "Blizzard_CombatLogProcessor")
    );
    let errors_before = env.state().borrow().lua_errors.len();
    env.exec(r#"
        local original = assert(rawget(__secureenv, "CombatLogProcessor"), "selected secure consumer")
        local info = debug.getinfo(original.ApplyFilterSettings, "S")
        local suffix = "Blizzard_CombatLogProcessor/Blizzard_CombatLogProcessor.lua"
        local source = info.source:gsub("\\", "/")
        assert(info.what == "Lua" and source:sub(-#suffix) == suffix, "original consumer provenance")
        print("combat-filter original", source, info.linedefined, info.lastlinedefined)
        local processor = CreateFromMixins(original)
        local secure = assert(getfenv(original.ApplyFilterSettings).C_CombatLogSecure)
        assert(secure ~= C_CombatLog, "namespace isolation prerequisite")

        -- Reach the original AddEventFilter before querying the new diagnostic.
        processor:SetFilterSettings({settings = {}, filters = {
            {eventList = {SPELL_DAMAGE = true, SPELL_HEAL = false, RANGE_DAMAGE = "partial"},
             sourceFlags = {[1] = true, [2] = true, [4] = false}},
            {eventList = {SPELL_HEAL = true}, destFlags = {["Creature-0-1"] = true}},
        }})
        local snapshot = assert(__wow_get_combat_log_event_filters, "filter-state query")
        local rows = snapshot(secure)
        assert(#rows == 2, "AddEventFilter must retain both original-consumer filters")
        local bySource, byDest
        for _, row in ipairs(rows) do
            assert(row.n == 3 and row[1] == "SPELL_DAMAGE", "first filter selects events for every row")
            if row[2] == 3 then bySource = row end
            if row[3] == "Creature-0-1" then byDest = row end
        end
        assert(bySource and bySource[3] == nil, "combined source flags and nil destination")
        assert(byDest and byDest[2] == nil, "destination GUID with absent source")

        -- Public producer has its own state; clearing it must not clear secure state.
        C_CombatLog.AddEventFilter("SPELL_HEAL", "Player-0-2", nil)
        local publicRows = snapshot(C_CombatLog)
        assert(#publicRows == 1 and publicRows[1].n == 3)
        assert(publicRows[1][1] == "SPELL_HEAL" and publicRows[1][2] == "Player-0-2")
        assert(publicRows[1][3] == nil)
        C_CombatLog.ClearEventFilters()
        assert(#snapshot(C_CombatLog) == 0 and #snapshot(secure) == 2)

        processor:SetFilterSettings({settings = {}, filters = {
            {eventList = {SPELL_DAMAGE = false, SPELL_HEAL = true},
             sourceFlags = {["Player-0-3"] = true}, destFlags = {}},
        }})
        rows = snapshot(secure)
        assert(#rows == 1, "ClearEventFilters must discard earlier records on replacement")
        assert(rows[1].n == 3 and rows[1][1] == "SPELL_HEAL")
        assert(rows[1][2] == "Player-0-3" and rows[1][3] == nil, "GUID clears zero counterpart")
        rows[1][1] = "snapshot mutation"
        collectgarbage("collect")
        assert(snapshot(secure)[1][1] == "SPELL_HEAL", "snapshot must not mutate retained arguments")

        processor:SetFilterSettings({settings = {}, filters = {
            {eventList = {SPELL_DAMAGE = true}, sourceFlags = {}},
        }})
        rows = snapshot(secure)
        assert(#rows == 1 and rows[1].n == 3 and rows[1][1] == "")
        assert(rows[1][2] == getfenv(original.ApplyFilterSettings).COMBATLOG_FILTER_MINE)
        assert(rows[1][3] == nil, "retain original consumer's empty-flags branch without interpreting it")
        processor:SetFilterSettings({settings = {}, filters = {}})
        assert(#snapshot(secure) == 0, "empty original settings must clear the remaining filter")
        processor:SetFilterSettings({settings = {}, filters = {}})
        assert(#snapshot(secure) == 0, "repeated clear remains empty")
    "#).expect("original consumer must add, replace and clear retained filter arguments");
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}
