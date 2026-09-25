//! Tests for professions: GetProfessions, GetProfessionInfo, C_TradeSkillUI.

use crate::common;

use crate::professions_api_helpers::{env, env_with_professions_util};

// ============================================================================
// GetProfessions / GetProfessionInfo
// ============================================================================

#[test]
fn test_get_professions_returns_two_indices() {
    let env = env();
    let (p1, p2): (i32, i32) = env
        .eval("local a, b = GetProfessions(); return a, b")
        .unwrap();
    assert_eq!(p1, 1);
    assert_eq!(p2, 2);
}

#[test]
fn test_get_professions_no_secondary() {
    let env = env();
    let is_nil: bool = env
        .eval("local _, _, a, b, c = GetProfessions(); return a == nil and b == nil and c == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_profession_info_blacksmithing() {
    let env = env();
    let (name, skill, max_skill): (String, i32, i32) = env
        .eval("local n, _, s, m = GetProfessionInfo(1); return n, s, m")
        .unwrap();
    assert_eq!(name, "Blacksmithing");
    assert_eq!(skill, 80);
    assert_eq!(max_skill, 100);
}

#[test]
fn trade_skill_profession_inventory_slots_are_available() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local blacksmithingSlots = C_TradeSkillUI.GetProfessionSlots(1)
            if #blacksmithingSlots ~= 3 or blacksmithingSlots[1] ~= 20 or blacksmithingSlots[3] ~= 22 then
                return "blacksmithing_slots"
            end

            local miningSlots = C_TradeSkillUI.GetProfessionSlots(6)
            if #miningSlots ~= 3 or miningSlots[1] ~= 23 or miningSlots[3] ~= 25 then
                return "mining_slots"
            end

            local allSlots = C_TradeSkillUI.GetProfessionInventorySlots()
            if #allSlots ~= 9 or allSlots[1] ~= 20 or allSlots[9] ~= 28 then
                return "all_slots"
            end

            if C_TradeSkillUI.GetProfessionByInventorySlot(20) ~= 1 then
                return "slot_20_profession"
            end
            if C_TradeSkillUI.GetProfessionByInventorySlot(24) ~= 6 then
                return "slot_24_profession"
            end
            if C_TradeSkillUI.GetProfessionByInventorySlot(99) ~= nil then
                return "unknown_slot"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn trade_skill_concentration_currency_is_disabled_when_unsupported() {
    let env = env();
    let concentration_currency_id: i32 = env
        .eval("return C_TradeSkillUI.GetConcentrationCurrencyID(164)")
        .unwrap();

    assert_eq!(concentration_currency_id, 0);
}

#[test]
fn trade_skill_set_recipe_tracked_accepts_missing_recrafting_flag() {
    let env = env();
    env.eval::<()>("C_TradeSkillUI.SetRecipeTracked(1001, true)")
        .unwrap();
}

#[test]
fn test_profession_info_mining() {
    let env = env();
    let (name, skill): (String, i32) = env
        .eval("local n, _, s = GetProfessionInfo(2); return n, s")
        .unwrap();
    assert_eq!(name, "Mining");
    assert_eq!(skill, 90);
}

#[test]
fn get_profession_info_exposes_skill_line_name_for_addon_lookup() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local skillLinesLUT = {}
            for _, skillLine in ipairs(C_TradeSkillUI.GetAllProfessionTradeSkillLines()) do
                skillLinesLUT[C_TradeSkillUI.GetTradeSkillDisplayName(skillLine)] = skillLine
            end

            local professions = { GetProfessions() }
            local skills = {}
            for i = 1, 2 do
                local profidx = professions[i]
                if profidx then
                    local profname = select(11, GetProfessionInfo(profidx))
                    skills[skillLinesLUT[profname]] = true
                end
            end

            if skills[164] ~= true then return "missing_blacksmithing" end
            if skills[186] ~= true then return "missing_mining" end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn test_get_profession_skill_line_id_maps_known_profession_enums() {
    let env = env();
    let (blacksmithing, mining): (i32, i32) = env
        .eval(
            r#"
            return C_TradeSkillUI.GetProfessionSkillLineID(Enum.Profession.Blacksmithing),
                C_TradeSkillUI.GetProfessionSkillLineID(Enum.Profession.Mining)
            "#,
        )
        .unwrap();
    assert_eq!(blacksmithing, 164);
    assert_eq!(mining, 186);
}

#[test]
fn test_trade_skill_display_name_maps_known_skill_lines() {
    let env = env();
    let (blacksmithing, mining, unknown): (String, String, String) = env
        .eval(
            r#"
            return C_TradeSkillUI.GetTradeSkillDisplayName(164),
                   C_TradeSkillUI.GetTradeSkillDisplayName(186),
                   C_TradeSkillUI.GetTradeSkillDisplayName(999999)
            "#,
        )
        .unwrap();
    assert_eq!(blacksmithing, "Blacksmithing");
    assert_eq!(mining, "Mining");
    assert_eq!(unknown, "");
}

#[test]
fn test_profession_info_exposes_spell_slots_for_professions_book_buttons() {
    let env = env();
    let (num_spells_1, num_spells_2, bs_spell_id, mining_spell_id): (i32, i32, i32, i32) = env
        .eval(
            r#"
            local _, _, _, _, numSpells1, spellOffset1 = GetProfessionInfo(1)
            local _, _, _, _, numSpells2, spellOffset2 = GetProfessionInfo(2)

            local bsInfo = C_SpellBook.GetSpellBookItemInfo(spellOffset1 + 1, Enum.SpellBookSpellBank.Player)
            local miningInfo = C_SpellBook.GetSpellBookItemInfo(spellOffset2 + 1, Enum.SpellBookSpellBank.Player)
            return numSpells1, numSpells2, bsInfo and bsInfo.spellID or 0, miningInfo and miningInfo.spellID or 0
            "#,
        )
        .unwrap();

    assert!(num_spells_1 > 0);
    assert!(num_spells_2 > 0);
    assert_eq!(bs_spell_id, 2018);
    assert_eq!(mining_spell_id, 2575);
}

#[test]
fn test_prof_specs_exposes_default_skill_line_and_tabs() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_ProfSpecs.ShouldShowSpecTab() ~= true then
                return "show_spec=" .. tostring(C_ProfSpecs.ShouldShowSpecTab())
            end

            local skillLineID = C_ProfSpecs.GetDefaultSpecSkillLine()
            if skillLineID ~= 164 then
                return "skill_line=" .. tostring(skillLineID)
            end

            local configID = C_ProfSpecs.GetConfigIDForSkillLine(skillLineID)
            if configID ~= 1 then
                return "config_id=" .. tostring(configID)
            end

            local tabIDs = C_ProfSpecs.GetSpecTabIDsForSkillLine(skillLineID)
            if #tabIDs == 0 then
                return "tab_count=0"
            end

            local tabInfo = C_ProfSpecs.GetTabInfo(tabIDs[1])
            if not tabInfo then
                return "missing_tab_info"
            end
            if not tabInfo.rootNodeID then
                return "missing_root_node"
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "C_ProfSpecs should expose a default skill line, config, and at least one tab: {result}"
    );
}

// ============================================================================
// C_TradeSkillUI
// ============================================================================

#[test]
fn test_base_profession_info() {
    let env = env();
    let (id, profession, name): (i32, i32, String) = env
        .eval(
            "local i = C_TradeSkillUI.GetBaseProfessionInfo(); \
             return i.professionID, i.profession, i.professionName",
        )
        .unwrap();
    assert_eq!(id, 164);
    assert_eq!(profession, 1);
    assert_eq!(name, "Blacksmithing");
}

#[test]
fn test_child_profession_info() {
    let env = env();
    let id: i32 = env
        .eval("return C_TradeSkillUI.GetChildProfessionInfo().professionID")
        .unwrap();
    assert_eq!(id, 164);
}

#[test]
fn test_is_trade_skill_ready() {
    let env = env();
    let ready: bool = env
        .eval("return C_TradeSkillUI.IsTradeSkillReady()")
        .unwrap();
    assert!(ready);
}

#[test]
fn test_all_recipe_ids_count() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TradeSkillUI.GetAllRecipeIDs()")
        .unwrap();
    assert_eq!(count, 34);
}

#[test]
fn blacksmithing_recipes_include_wago_db2_examples_for_each_expansion() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local expected = {
                {2660, "Rough Sharpening Stone", "Classic Blacksmithing"},
                {29545, "Fel Iron Plate Gloves", "Outland Blacksmithing"},
                {52567, "Cobalt Legplates", "Northrend Blacksmithing"},
                {76178, "Folded Obsidium", "Cataclysm Blacksmithing"},
                {122568, "Spiritguard Helm", "Pandaria Blacksmithing"},
                {171690, "Truesteel Ingot", "Draenor Blacksmithing"},
                {182928, "Leystone Armguards", "Legion Blacksmithing"},
                {253110, "Monel-Hardened Hoofplates", "Kul Tiran Blacksmithing"},
                {307611, "Shadowghast Ingot", "Shadowlands Blacksmithing"},
                {365729, "Primal Molten Warglaive", "Dragon Isles Blacksmithing"},
                {438914, "Algari Competitor's Plate Breastplate", "Khaz Algar Blacksmithing"},
                {1229598, "Sun-Blessed Blacksmith's Hammer", "Midnight Blacksmithing"},
            }

            for _, item in ipairs(expected) do
                local recipe = C_TradeSkillUI.GetRecipeInfo(item[1])
                if not recipe then
                    return "missing_recipe=" .. item[1]
                end
                if recipe.name ~= item[2] then
                    return "name=" .. item[1] .. ":" .. tostring(recipe.name)
                end

                local category = C_TradeSkillUI.GetCategoryInfo(recipe.categoryID)
                if not category or category.name ~= item[3] then
                    return "category=" .. item[1] .. ":" .. tostring(category and category.name)
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn trade_skill_recipes_are_visible_in_blacksmithing_skill_line() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local categories = { C_TradeSkillUI.GetCategories() }
            if #categories == 0 then
                return "categories=0"
            end

            local sawClassic = false
            for _, categoryID in ipairs(categories) do
                local category = C_TradeSkillUI.GetCategoryInfo(categoryID)
                if category and category.name == "Classic Blacksmithing" then
                    sawClassic = true
                    break
                end
            end
            if not sawClassic then
                return "missing_classic_category"
            end

            if not C_TradeSkillUI.IsRecipeInSkillLine(2660, 164) then
                return "classic_recipe_hidden"
            end
            if C_TradeSkillUI.IsRecipeInSkillLine(2660, 186) then
                return "classic_recipe_in_mining"
            end
            if C_TradeSkillUI.GetRecipeInfo(2660).maxTrivialLevel == nil then
                return "missing_max_trivial"
            end
            if #C_TradeSkillUI.GetRecipeRequirements(2660) ~= 0 then
                return "requirements"
            end
            if C_TradeSkillUI.GetCraftableCount(2660) ~= 0 then
                return "craftable_count"
            end
            if C_TradeSkillUI.GetRecipeSchematic(2660).outputItemID ~= 2862 then
                return "output_item"
            end
            local outputItemData = C_TradeSkillUI.GetRecipeOutputItemData(2660)
            if type(outputItemData) ~= "table" then
                return "missing_output_data"
            end
            if outputItemData.itemID ~= 2862 then
                return "output_item_data"
            end
            if type(C_TradeSkillUI.GetRecipeDescription(2660)) ~= "string" then
                return "missing_description"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn wago_blacksmithing_recipes_have_output_icons_and_reagents() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local expected = {
                {2660, 2862, 2835, 1},
                {29545, 23482, 23445, 4},
                {52567, 39086, 36916, 5},
                {365729, 190508, 189541, 17},
                {438914, 217143, 222426, 6},
                {1229598, 238018, 238528, 1},
            }

            for _, row in ipairs(expected) do
                local recipeID, outputItemID, reagentItemID, reagentCount = unpack(row)
                local output = C_TradeSkillUI.GetRecipeOutputItemData(recipeID)
                if output.itemID ~= outputItemID then
                    return "output=" .. recipeID .. ":" .. tostring(output.itemID)
                end
                if type(output.icon) ~= "number" or output.icon <= 0 then
                    return "icon=" .. recipeID .. ":" .. tostring(output.icon)
                end
                if C_TradeSkillUI.GetRecipeSchematic(recipeID).outputItemID ~= outputItemID then
                    return "schematic_output=" .. recipeID
                end
                if C_TradeSkillUI.GetRecipeNumReagents(recipeID) == 0 then
                    return "reagents=0:" .. recipeID
                end

                local reagent = C_TradeSkillUI.GetRecipeReagentInfo(recipeID, 1)
                if reagent.itemID ~= reagentItemID then
                    return "reagent=" .. recipeID .. ":" .. tostring(reagent.itemID)
                end
                if reagent.numRequired ~= reagentCount then
                    return "reagent_count=" .. recipeID .. ":" .. tostring(reagent.numRequired)
                end
                if reagent.name == nil or reagent.name == "Unknown" then
                    return "reagent_name=" .. recipeID .. ":" .. tostring(reagent.name)
                end

                local link = C_TradeSkillUI.GetRecipeReagentItemLink(recipeID, 1)
                if not link or not link:find("Hitem:" .. reagentItemID) then
                    return "reagent_link=" .. recipeID .. ":" .. tostring(link)
                end
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

#[test]
fn test_filtered_recipe_ids_nonempty() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TradeSkillUI.GetFilteredRecipeIDs()")
        .unwrap();
    assert!(count > 0);
}

#[test]
fn test_trade_skill_lists_and_counts_are_seeded() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local skillLines = C_TradeSkillUI.GetAllProfessionTradeSkillLines()
            local professions = C_TradeSkillUI.GetProfessions()
            if #skillLines ~= 2 then return "skillLines=" .. #skillLines end
            if skillLines[1] ~= 164 or skillLines[2] ~= 186 then
                return "skillLineIDs=" .. tostring(skillLines[1]) .. "," .. tostring(skillLines[2])
            end
            if #professions ~= 2 then return "professions=" .. #professions end
            if C_TradeSkillUI.GetCraftingOrderCount() ~= 0 then
                return "orderCount=" .. tostring(C_TradeSkillUI.GetCraftingOrderCount())
            end
            if C_TradeSkillUI.GetNumRecipes() ~= 34 then
                return "numRecipes=" .. tostring(C_TradeSkillUI.GetNumRecipes())
            end
            if C_TradeSkillUI.GetNumTradeSkills() ~= 34 then
                return "numTradeSkills=" .. tostring(C_TradeSkillUI.GetNumTradeSkills())
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_recipe_info_valid() {
    let env = env();
    let (id, name, learned, craftable): (i32, String, bool, bool) = env
        .eval(
            "local r = C_TradeSkillUI.GetRecipeInfo(100001); \
             return r.recipeID, r.name, r.learned, r.craftable",
        )
        .unwrap();
    assert_eq!(id, 100001);
    assert_eq!(name, "Khaz Algar Helm");
    assert!(learned);
    assert!(craftable);
}

#[test]
fn test_recipe_info_unknown() {
    let env = env();
    let id: i32 = env
        .eval("return C_TradeSkillUI.GetRecipeInfo(999999).recipeID")
        .unwrap();
    assert_eq!(id, 0);
}

#[test]
fn test_recipe_schematic_has_reagents() {
    let env = env();
    let (id, name, output, min, max, count, slot_index, item, quantity): (
        i32,
        String,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
        i32,
    ) = env
        .eval(
            r#"
            local s = C_TradeSkillUI.GetRecipeSchematic(100001)
            local slot = s.reagentSlotSchematics[1]
            return s.recipeID, s.name, s.outputItemID, s.quantityMin, s.quantityMax,
                #s.reagentSlotSchematics, slot.slotIndex, slot.reagents[1].itemID,
                slot.quantityRequired
            "#,
        )
        .unwrap();
    assert_eq!(id, 100001);
    assert_eq!(name, "Khaz Algar Helm");
    assert_eq!(output, 211993);
    assert_eq!((min, max), (1, 1));
    assert_eq!(count, 2);
    assert_eq!(slot_index, 1);
    assert_eq!(item, 210934);
    assert_eq!(quantity, 12);
}

#[test]
fn test_recipe_profession_and_item_links_exist() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local info = C_TradeSkillUI.GetProfessionInfoByRecipeID(100005)
            if info.professionID ~= 164 then return "professionID=" .. tostring(info.professionID) end
            if info.professionName ~= "Blacksmithing" then return "professionName=" .. tostring(info.professionName) end

            local recipeLink = C_TradeSkillUI.GetRecipeItemLink(100001)
            if not recipeLink or not recipeLink:find("Hitem:211993") then return "recipeLink=" .. tostring(recipeLink) end

            local tradeLink = C_TradeSkillUI.GetTradeSkillListLink()
            if not tradeLink or not tradeLink:find("Htrade:164:80:100") then return "tradeLink=" .. tostring(tradeLink) end
            if not tradeLink:find("%[Blacksmithing%]") then return "tradeLinkName=" .. tostring(tradeLink) end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_recipe_schematic_unknown_short_circuits_before_reagent_construction() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local function restore(recipeID)
                local schematicInfo = C_TradeSkillUI.GetRecipeSchematic(recipeID, false)
                if not schematicInfo then
                    return "skipped"
                end
                for _ in pairs(schematicInfo.reagentSlotSchematics) do end
                return "constructed"
            end
            return restore(1228977)
            "#,
        )
        .unwrap();
    assert_eq!(result, "skipped");
}

#[test]
fn test_recipe_reagent_info_and_links_are_seeded() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if C_TradeSkillUI.GetRecipeNumReagents(100005) ~= 3 then
                return "count=" .. tostring(C_TradeSkillUI.GetRecipeNumReagents(100005))
            end

            local reagent = C_TradeSkillUI.GetRecipeReagentInfo(100005, 2)
            if reagent.itemID ~= 210937 then return "itemID=" .. tostring(reagent.itemID) end
            if reagent.numRequired ~= 4 then return "numRequired=" .. tostring(reagent.numRequired) end
            if reagent.name == nil or reagent.name == "Unknown" then return "name=" .. tostring(reagent.name) end

            local link = C_TradeSkillUI.GetRecipeReagentItemLink(100005, 2)
            if not link or not link:find("Hitem:210937") then return "link=" .. tostring(link) end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn test_category_info() {
    let env = env();
    let (id, name): (i32, String) = env
        .eval(
            "local c = C_TradeSkillUI.GetCategoryInfo(1); \
             return c.categoryID, c.name",
        )
        .unwrap();
    assert_eq!(id, 1);
    assert_eq!(name, "Armor");
}

#[test]
fn test_category_info_unknown() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_TradeSkillUI.GetCategoryInfo(9999) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_all_recipes_retrievable() {
    let env = env();
    let all_valid: bool = env
        .eval(
            "for _, id in ipairs(C_TradeSkillUI.GetAllRecipeIDs()) do \
                 local r = C_TradeSkillUI.GetRecipeInfo(id); \
                 if not r or not r.name or not r.learned then return false end \
             end; \
             return true",
        )
        .unwrap();
    assert!(all_valid);
}

#[test]
fn test_greatsword_recipe_has_three_reagents() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TradeSkillUI.GetRecipeSchematic(100005).reagentSlotSchematics")
        .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_professions_util_resolves_basic_reagents_from_recipe_schematic() {
    test_timeout! {
        let env = env_with_professions_util();
        let (count, reagent_ids): (i32, String) = env
            .eval(
                r#"
                local reagents = ProfessionsUtil.CreateRecipeReagentsForAllBasicReagents(100005)
                local ids = {}
                for index, reagent in ipairs(reagents) do
                    ids[index] = tostring(reagent.itemID)
                end
                return #reagents, table.concat(ids, ",")
                "#,
            )
            .unwrap();

        assert_eq!(count, 3);
        assert_eq!(reagent_ids, "210934,210937,210935");
    }
}

#[test]
fn test_set_recipe_tracked_updates_state_and_fires_event() {
    let env = env();

    env.eval::<()>(
        r#"
        __recipe_events = {}
        local f = CreateFrame("Frame")
        f:RegisterEvent("TRACKED_RECIPE_UPDATE")
        f:SetScript("OnEvent", function(_, _event, recipeID, tracked)
            __recipe_events[#__recipe_events + 1] = { id = recipeID, tracked = tracked }
        end)

        C_TradeSkillUI.SetRecipeTracked(100005, true, false)
        C_TradeSkillUI.SetRecipeTracked(100005, true, false)
        C_TradeSkillUI.SetRecipeTracked(100005, true, true)
        "#,
    )
    .unwrap();

    let state = env.state().borrow();
    assert_eq!(state.tracked_recipes.list(false), &[100005]);
    assert_eq!(state.tracked_recipes.list(true), &[100005]);
    drop(state);

    let count: i64 = env.eval("return #__recipe_events").unwrap();
    assert_eq!(count, 2, "only real state changes should fire");

    let (id1, tracked1): (i64, bool) = env
        .eval("return __recipe_events[1].id, __recipe_events[1].tracked")
        .unwrap();
    assert_eq!(id1, 100005);
    assert!(tracked1);

    let (id2, tracked2): (i64, bool) = env
        .eval("return __recipe_events[2].id, __recipe_events[2].tracked")
        .unwrap();
    assert_eq!(id2, 100005);
    assert!(tracked2);
}

#[test]
fn test_get_recipes_tracked_returns_lua_lists_per_bucket() {
    let env = env();

    let (normal_len, normal_ids, recraft_len, recraft_ids): (i32, String, i32, String) = env
        .eval(
            r#"
            C_TradeSkillUI.SetRecipeTracked(100005, true, false)
            C_TradeSkillUI.SetRecipeTracked(100006, true, false)
            C_TradeSkillUI.SetRecipeTracked(200001, true, true)

            local normal = C_TradeSkillUI.GetRecipesTracked(false)
            local recraft = C_TradeSkillUI.GetRecipesTracked(true)

            local function join_ids(list)
                local out = {}
                for _, recipeID in ipairs(list) do
                    out[#out + 1] = tostring(recipeID)
                end
                return table.concat(out, ",")
            end

            return #normal, join_ids(normal), #recraft, join_ids(recraft)
            "#,
        )
        .unwrap();

    assert_eq!(normal_len, 2);
    assert_eq!(normal_ids, "100005,100006");
    assert_eq!(recraft_len, 1);
    assert_eq!(recraft_ids, "200001");
}

#[test]
fn test_is_recipe_tracked_returns_membership_per_bucket() {
    let env = env();

    let (before_normal, before_recraft, after_normal, after_recraft, other_recipe): (
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local beforeNormal = C_TradeSkillUI.IsRecipeTracked(100005, false)
            local beforeRecraft = C_TradeSkillUI.IsRecipeTracked(100005, true)

            C_TradeSkillUI.SetRecipeTracked(100005, true, true)

            local afterNormal = C_TradeSkillUI.IsRecipeTracked(100005, false)
            local afterRecraft = C_TradeSkillUI.IsRecipeTracked(100005, true)
            local otherRecipe = C_TradeSkillUI.IsRecipeTracked(200001, true)

            return beforeNormal, beforeRecraft, afterNormal, afterRecraft, otherRecipe
            "#,
        )
        .unwrap();

    assert!(!before_normal);
    assert!(!before_recraft);
    assert!(!after_normal);
    assert!(after_recraft);
    assert!(!other_recipe);
}
