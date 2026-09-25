//! Tests for the SimState-backed crafting surface.
//!
//! Uses recipe 100001 (Khaz Algar Helm) from BLACKSMITHING_RECIPES:
//!   reagents: item_id=210934 qty=12, item_id=210937 qty=2
//!   output_item_id: 211993 (Entombed Seraph's Casque)

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().unwrap()
}

/// Lua helper: count total stack_count for a given item_id across bag 0 (backpack).
/// AddBagItem uses bag 0 in these tests so slots are accessible via C_Container.
const COUNT_ITEM: &str = r#"
local function count_item(item_id)
    local total = 0
    for slot = 1, C_Container.GetContainerNumSlots(0) do
        local info = C_Container.GetContainerItemInfo(0, slot)
        if info and info.itemID == item_id then
            total = total + (info.stackCount or 0)
        end
    end
    return total
end
"#;

#[test]
fn selected_profession_round_trips_through_admin_and_c_trade_skill_ui() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            A_Admin.SetSelectedProfession(164)
            local info = C_TradeSkillUI.GetChildProfessionInfo()
            if info == nil then return "nil_info" end
            if info.professionID ~= 164 then
                return "professionID=" .. tostring(info.professionID)
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "SetSelectedProfession(164) should be visible via GetChildProfessionInfo: {result}"
    );
}

#[test]
fn learning_a_recipe_makes_it_visible_via_is_recipe_learned() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local id = 100001
            if C_TradeSkillUI.IsRecipeLearned(id) ~= false then
                return "initial_not_false"
            end
            A_Admin.LearnRecipe(id)
            if C_TradeSkillUI.IsRecipeLearned(id) ~= true then
                return "after_learn_not_true"
            end
            A_Admin.UnlearnRecipe(id)
            if C_TradeSkillUI.IsRecipeLearned(id) ~= false then
                return "after_unlearn_not_false"
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "LearnRecipe/UnlearnRecipe round-trip: {result}"
    );
}

#[test]
fn is_recipe_craftable_returns_false_when_reagents_missing() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            A_Admin.ClearBags()
            return C_TradeSkillUI.IsRecipeCraftable(100001)
            "#,
        )
        .unwrap();
    assert!(!result, "IsRecipeCraftable should be false with empty bags");
}

#[test]
fn is_recipe_craftable_returns_true_when_all_reagents_present() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            A_Admin.ClearBags()
            -- reagents for recipe 100001: 210934 x12, 210937 x2
            A_Admin.AddBagItem(0, 1, 210934, 12)
            A_Admin.AddBagItem(0, 2, 210937, 2)
            return C_TradeSkillUI.IsRecipeCraftable(100001)
            "#,
        )
        .unwrap();
    assert!(
        result,
        "IsRecipeCraftable should be true when all reagents present"
    );
}

#[test]
fn is_recipe_craftable_handles_count_arg() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            A_Admin.ClearBags()
            -- seed exactly 1 craft worth of reagents
            A_Admin.AddBagItem(0, 1, 210934, 12)
            A_Admin.AddBagItem(0, 2, 210937, 2)
            local one = C_TradeSkillUI.IsRecipeCraftable(100001, 1)
            local two = C_TradeSkillUI.IsRecipeCraftable(100001, 2)
            if one ~= true then return "one=" .. tostring(one) end
            if two ~= false then return "two=" .. tostring(two) end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "IsRecipeCraftable with count arg: {result}");
}

#[test]
fn craft_recipe_consumes_reagents_and_adds_output() {
    let env = env();
    let result: String = env
        .eval(&format!(
            r#"
            {COUNT_ITEM}
            A_Admin.ClearBags()
            A_Admin.AddBagItem(0, 1, 210934, 12)
            A_Admin.AddBagItem(0, 2, 210937, 2)
            local ok = C_TradeSkillUI.CraftRecipe(100001, 1)
            if ok ~= true then return "craft_failed" end

            -- reagent 210934 should be gone
            local r1 = count_item(210934)
            if r1 ~= 0 then return "reagent_210934=" .. tostring(r1) end
            local r2 = count_item(210937)
            if r2 ~= 0 then return "reagent_210937=" .. tostring(r2) end

            -- output 211993 should be in bags (synthetic slot 99)
            local out = count_item(211993)
            if out < 1 then return "output=" .. tostring(out) end
            return "ok"
            "#,
        ))
        .unwrap();
    assert_eq!(
        result, "ok",
        "CraftRecipe should consume reagents and add output: {result}"
    );
}

#[test]
fn craft_recipe_respects_modeled_backpack_capacity_without_invalid_overflow_slots() {
    use wow_ui_sim::c_api::bag_info::BagInfo;

    let env = env();
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_info.insert(0, BagInfo {
            num_slots: 0, family: 0, name: Some("Backpack".to_string()),
            inventory_slot: Some(20), item_id: None, hyperlink: None,
        });
    }
    let result: String = env.eval(
        r#"
        A_Admin.ClearBags()
        A_Admin.AddBagItem(0, 1, 210934, 12)
        A_Admin.AddBagItem(0, 2, 210937, 2)
        return tostring(C_TradeSkillUI.CraftRecipe(100001, 1))
        "#,
    ).unwrap();
    assert_eq!(result, "false");
    let sim = env.state().borrow();
    assert_eq!(sim.bag_items.get(&(0, 1)).map(|item| item.stack_count), Some(12));
    assert!(!sim.bag_items.keys().any(|(_, slot)| *slot < 1));
}

#[test]
fn craft_recipe_starts_player_cast_with_output_item_name_for_cast_bar() {
    let env = env();
    env.exec(
        r#"
        A_Admin.SetSelectedProfession(164)
        A_Admin.LearnRecipe(100001)
        A_Admin.SeedReagentsForRecipe(100001, 1)
        "#,
    )
    .unwrap();

    let before: bool = env.eval("return UnitCastingInfo('player') ~= nil").unwrap();
    assert!(!before, "player should not already be casting");

    let result: String = env
        .eval(
            r#"
            local events = {}
            local listener = CreateFrame("Frame")
            listener:RegisterEvent("UNIT_SPELLCAST_START")
            listener:SetScript("OnEvent", function(_, event, unit)
                table.insert(events, event .. ":" .. tostring(unit))
            end)

            C_TradeSkillUI.CraftRecipe(100001, 1)
            local name, _, _, startTime, endTime, _, _, _, spellID = UnitCastingInfo("player")
            if not name then return "not_casting" end
            if events[1] ~= "UNIT_SPELLCAST_START:player" then
                return "event=" .. tostring(events[1])
            end
            if endTime <= startTime then return "bad_times" end
            return name .. ":" .. tostring(spellID)
            "#,
        )
        .unwrap();

    assert_eq!(result, "Entombed Seraph's Casque:100001");
}

#[test]
fn craft_recipe_uses_profession_cast_duration() {
    let env = env();
    env.exec(
        r#"
        A_Admin.SetSelectedProfession(164)
        A_Admin.LearnRecipe(100001)
        A_Admin.SeedReagentsForRecipe(100001, 1)
        "#,
    )
    .unwrap();

    let duration_ms: f64 = env
        .eval(
            r#"
            C_TradeSkillUI.CraftRecipe(100001, 1)
            local _, _, _, startTime, endTime = UnitCastingInfo("player")
            return endTime - startTime
            "#,
        )
        .unwrap();

    assert!(
        (duration_ms - 2000.0).abs() < 10.0,
        "profession craft cast duration should be ~2000ms, got {duration_ms}"
    );
}

#[test]
fn craft_recipe_fires_bag_update_events() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            A_Admin.ClearBags()
            A_Admin.AddBagItem(0, 1, 210934, 12)
            A_Admin.AddBagItem(0, 2, 210937, 2)

            local listener = CreateFrame("Frame")
            listener:RegisterEvent("BAG_UPDATE")
            listener:RegisterEvent("BAG_UPDATE_DELAYED")
            local bag_update_count = 0
            local bag_delayed_count = 0
            listener:SetScript("OnEvent", function(_, event, bagID)
                if event == "BAG_UPDATE" then
                    bag_update_count = bag_update_count + 1
                elseif event == "BAG_UPDATE_DELAYED" then
                    bag_delayed_count = bag_delayed_count + 1
                end
            end)

            if not C_TradeSkillUI.CraftRecipe(100001, 1) then return "craft_failed" end
            if bag_update_count < 1 then return "bag_update=" .. bag_update_count end
            if bag_delayed_count ~= 1 then return "bag_delayed=" .. bag_delayed_count end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "CraftRecipe should fire BAG_UPDATE and BAG_UPDATE_DELAYED so the bag UI refreshes: {result}"
    );
}

#[test]
fn get_remaining_recasts_returns_zero_for_normal_recipes() {
    // The Blizzard ProfessionsCrafting.lua ValidateControls path does
    // `C_TradeSkillUI.GetRemainingRecasts() + 1`, so a missing stub crashes the
    // entire post-craft validation, leaving the Create button greyed and the
    // UI desynced from bag state.
    let env = env();
    let result: f64 = env
        .eval("return C_TradeSkillUI.GetRemainingRecasts()")
        .unwrap();
    assert_eq!(result, 0.0);
}

#[test]
fn craft_recipe_with_count_consumes_proportionally() {
    let env = env();
    let result: String = env
        .eval(&format!(
            r#"
            {COUNT_ITEM}
            A_Admin.ClearBags()
            -- 3× reagents
            A_Admin.AddBagItem(0, 1, 210934, 36)
            A_Admin.AddBagItem(0, 2, 210937, 6)
            local ok = C_TradeSkillUI.CraftRecipe(100001, 3)
            if ok ~= true then return "craft_failed" end

            local r1 = count_item(210934)
            if r1 ~= 0 then return "reagent_210934=" .. tostring(r1) end
            local r2 = count_item(210937)
            if r2 ~= 0 then return "reagent_210937=" .. tostring(r2) end

            local out = count_item(211993)
            if out ~= 3 then return "output=" .. tostring(out) end
            return "ok"
            "#,
        ))
        .unwrap();
    assert_eq!(
        result, "ok",
        "CraftRecipe(id, 3) should produce 3 outputs and zero reagents: {result}"
    );
}

#[test]
fn craft_recipe_returns_false_and_no_op_when_reagents_missing() {
    let env = env();
    let result: String = env
        .eval(&format!(
            r#"
            {COUNT_ITEM}
            A_Admin.ClearBags()
            local ok = C_TradeSkillUI.CraftRecipe(100001, 1)
            if ok ~= false then return "expected_false got=" .. tostring(ok) end
            -- bags should still be empty
            local r1 = count_item(210934)
            local r2 = count_item(210937)
            local out = count_item(211993)
            if r1 ~= 0 or r2 ~= 0 or out ~= 0 then
                return "bags_changed r1=" .. r1 .. " r2=" .. r2 .. " out=" .. out
            end
            return "ok"
            "#,
        ))
        .unwrap();
    assert_eq!(
        result, "ok",
        "CraftRecipe with missing reagents should be a no-op: {result}"
    );
}

#[test]
fn set_reagent_count_creates_then_replaces_then_clears() {
    let env = env();
    let result: String = env
        .eval(&format!(
            r#"
            {COUNT_ITEM}
            A_Admin.ClearBags()

            -- Create a fresh slot.
            A_Admin.SetReagentCount(210934, 5)
            local first = count_item(210934)
            if first ~= 5 then return "create=" .. first end

            -- Replace, not append, when the item is already present.
            A_Admin.SetReagentCount(210934, 12)
            local replaced = count_item(210934)
            if replaced ~= 12 then return "replace=" .. replaced end

            -- qty=0 clears every slot for that item.
            A_Admin.SetReagentCount(210934, 0)
            local cleared = count_item(210934)
            if cleared ~= 0 then return "clear=" .. cleared end

            return "ok"
            "#
        ))
        .unwrap();
    assert_eq!(
        result, "ok",
        "SetReagentCount should create / replace / clear: {result}"
    );
}

#[test]
fn set_reagent_count_collapses_duplicate_slots_into_one() {
    let env = env();
    let result: String = env
        .eval(&format!(
            r#"
            {COUNT_ITEM}
            A_Admin.ClearBags()
            -- Manually plant two stacks of the same item across two slots,
            -- then SetReagentCount should collapse them into a single
            -- slot with the requested total (not 2 stacks adding up).
            A_Admin.AddBagItem(0, 1, 210934, 4)
            A_Admin.AddBagItem(0, 2, 210934, 7)
            local before = count_item(210934)
            if before ~= 11 then return "before=" .. before end

            A_Admin.SetReagentCount(210934, 20)
            local after = count_item(210934)
            if after ~= 20 then return "after=" .. after end

            -- Confirm only one slot now holds the item.
            local slot_count = 0
            for slot = 1, C_Container.GetContainerNumSlots(0) do
                local info = C_Container.GetContainerItemInfo(0, slot)
                if info and info.itemID == 210934 then
                    slot_count = slot_count + 1
                end
            end
            if slot_count ~= 1 then return "slots=" .. slot_count end

            return "ok"
            "#
        ))
        .unwrap();
    assert_eq!(
        result, "ok",
        "SetReagentCount should collapse duplicate slots into one: {result}"
    );
}

#[test]
fn seed_reagents_for_recipe_makes_it_craftable_at_count_one() {
    let env = env();
    let result: String = env
        .eval(&format!(
            r#"
            {COUNT_ITEM}
            A_Admin.ClearBags()
            local ok = A_Admin.SeedReagentsForRecipe(100001, 1)
            if ok ~= true then return "seed_failed" end
            if not C_TradeSkillUI.IsRecipeCraftable(100001) then
                return "not_craftable"
            end
            -- Recipe 100001 needs 12 of 210934 + 2 of 210937.
            if count_item(210934) ~= 12 then return "r1=" .. count_item(210934) end
            if count_item(210937) ~= 2 then return "r2=" .. count_item(210937) end
            return "ok"
            "#
        ))
        .unwrap();
    assert_eq!(
        result, "ok",
        "SeedReagentsForRecipe(id, 1) should populate exactly one craft worth: {result}"
    );
}

#[test]
fn seed_reagents_for_recipe_supports_count_arg() {
    let env = env();
    let result: String = env
        .eval(&format!(
            r#"
            {COUNT_ITEM}
            A_Admin.ClearBags()
            A_Admin.SeedReagentsForRecipe(100001, 4)
            -- 4 crafts of 12+2 reagents.
            if count_item(210934) ~= 48 then return "r1=" .. count_item(210934) end
            if count_item(210937) ~= 8 then return "r2=" .. count_item(210937) end
            -- And the recipe is craftable 4 times but not 5.
            if not C_TradeSkillUI.IsRecipeCraftable(100001, 4) then return "not_4" end
            if C_TradeSkillUI.IsRecipeCraftable(100001, 5) then return "is_5" end
            return "ok"
            "#
        ))
        .unwrap();
    assert_eq!(
        result, "ok",
        "SeedReagentsForRecipe should scale by count: {result}"
    );
}

#[test]
fn get_dependent_reagents_returns_empty_table_for_unknown_reagent() {
    // Regression: GetDependentReagents used to be unregistered, so callers
    // got nil and `ipairs(nil)` crashed in
    // ProfessionsRecipeTransactionMixin:AreDependentReagentsAllocated.
    let env = env();
    let result: String = env
        .eval(
            r#"
            local r = C_TradeSkillUI.GetDependentReagents({ itemID = 210934 })
            if type(r) ~= "table" then return "type=" .. type(r) end
            -- Iterating with ipairs must not error; current data has no deps.
            local count = 0
            for _ in ipairs(r) do count = count + 1 end
            return "count=" .. count
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "count=0",
        "GetDependentReagents must return an empty table (not nil) for reagents with no declared deps: {result}"
    );
}

#[test]
fn get_dependent_reagents_handles_nil_or_malformed_input() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local cases = {
                C_TradeSkillUI.GetDependentReagents(nil),
                C_TradeSkillUI.GetDependentReagents({}),
                C_TradeSkillUI.GetDependentReagents({ currencyID = 3008 }),
            }
            for i, r in ipairs(cases) do
                if type(r) ~= "table" then return "case " .. i .. " type=" .. type(r) end
                for _ in ipairs(r) do return "case " .. i .. " unexpected entries" end
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok", "{result}");
}

#[test]
fn seed_reagents_for_unknown_recipe_returns_false() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            A_Admin.ClearBags()
            return tostring(A_Admin.SeedReagentsForRecipe(99999, 1))
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "false",
        "SeedReagentsForRecipe with unknown id should return false"
    );
}

#[test]
fn abandon_skill_removes_profession_from_get_professions() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            -- Before: both primaries present.
            local p1, p2 = GetProfessions()
            if p1 == nil or p2 == nil then return "before_missing p1=" .. tostring(p1) .. " p2=" .. tostring(p2) end

            -- Abandon Blacksmithing (skill_line_id 164, PROFESSIONS index 1).
            AbandonSkill(164)

            local a1, a2 = GetProfessions()
            -- Mining survives at slot 1 (index 2 in PROFESSIONS array).
            if a1 ~= 2 then return "slot1=" .. tostring(a1) end
            if a2 ~= nil then return "slot2_not_nil=" .. tostring(a2) end

            -- GetProfessionInfo for the abandoned index returns nil.
            local info = GetProfessionInfo(1)
            if info ~= nil then return "info_not_nil=" .. tostring(info) end

            -- C_TradeSkillUI.GetProfessions no longer contains 164.
            local profs = C_TradeSkillUI.GetProfessions()
            for _, v in ipairs(profs) do
                if v == 164 then return "164_still_present" end
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "AbandonSkill(164) should remove Blacksmithing from all profession queries: {result}"
    );
}

#[test]
fn relearn_profession_round_trip() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            A_Admin.UnlearnProfession(164)
            local a1, a2 = GetProfessions()
            if a1 ~= 2 then return "after_unlearn slot1=" .. tostring(a1) end
            if a2 ~= nil then return "after_unlearn slot2=" .. tostring(a2) end

            A_Admin.RelearnProfession(164)
            local b1, b2 = GetProfessions()
            if b1 == nil or b2 == nil then return "after_relearn missing b1=" .. tostring(b1) .. " b2=" .. tostring(b2) end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "UnlearnProfession/RelearnProfession round trip: {result}"
    );
}

#[test]
fn abandon_skill_clears_selection_when_selected_unlearned() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            A_Admin.SetSelectedProfession(164)
            AbandonSkill(164)

            -- GetChildProfessionInfo should not return Blacksmithing.
            local info = C_TradeSkillUI.GetChildProfessionInfo()
            if info ~= nil and info.skillLineID == 164 then
                return "still_selected skill_line=" .. tostring(info.skillLineID)
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "AbandonSkill should clear selection when the abandoned profession was selected: {result}"
    );
}
