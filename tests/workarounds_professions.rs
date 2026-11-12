//! Integration tests for the professions panel open flow.
//!
//! Loads every Blizzard addon like the real client, then verifies that
//! casting a profession opener spell (Blacksmithing) fires the chain
//! UIParent expects and that `ProfessionsFrame` actually becomes visible.

use std::path::PathBuf;
use std::time::Duration;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{fire_one_on_update_tick, fire_startup_events, process_pending_timers};

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn full_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1600.0, 1200.0);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    load_all_blizzard_addons(&env);
    load_professions_bootstrap_publishers(&env);
    settle_env(&env);

    env
}

fn load_all_blizzard_addons(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    for (name, toc_path) in &discover_blizzard_addons(&ui) {
        if let Err(err) = load_addon(&env.loader_env(), toc_path) {
            panic!("[load {name}] FAILED: {err}");
        }
    }
}

fn load_professions_bootstrap_publishers(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    for (addon_name, bootstrap_file) in [
        ("Blizzard_Professions", "Blizzard_Professions_Bootstrap.lua"),
        (
            "Blizzard_ProfessionsBook",
            "Blizzard_ProfessionsBook_Bootstrap.lua",
        ),
    ] {
        crate::common::load_blizzard_addon_bootstrap(env, &ui, addon_name, bootstrap_file);
    }
}

fn settle_env(env: &WowLuaEnv) {
    env.apply_post_load_workarounds();
    fire_startup_events(env);
    env.apply_post_event_workarounds();
    env.state().borrow_mut().widgets.rebuild_anchor_index();
    process_pending_timers(env);
    fire_one_on_update_tick(env);
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(env.lua());

    std::thread::sleep(Duration::from_secs(2));
    for _ in 0..3 {
        env.state().borrow_mut().ensure_layout_rects();
        fire_one_on_update_tick(env);
        process_pending_timers(env);
    }
}

#[test]
fn profession_source_filter_updates_are_deferred_and_coalesced() {
    let env = full_game_env();
    let toc = blizzard_ui_dir()
        .join("Blizzard_ProfessionsTemplates")
        .join("Blizzard_ProfessionsTemplates.toc");
    load_addon(&env.loader_env(), &toc).expect("load Blizzard Professions templates");
    env.exec(
        r#"
        professionFilterUpdates = 0
        professionFilterListener = CreateFrame('Frame')
        professionFilterListener:RegisterEvent('TRADE_SKILL_LIST_UPDATE')
        professionFilterListener:SetScript('OnEvent', function()
            professionFilterUpdates = professionFilterUpdates + 1
        end)
        -- ProfessionsFrame:SetProfessionInfo uses this Blizzard helper when
        -- opening Blacksmithing. Its no-op clear must not reenter the frame.
        Professions.SetAllSourcesFiltered(false)
        assert(professionFilterUpdates == 0, 'unchanged clear dispatched synchronously')
        C_TradeSkillUI.SetRecipeSourceTypeFilter(1, true)
        C_TradeSkillUI.SetOnlyShowMakeableRecipes(true)
        Professions.SetAllSourcesFiltered(false)
        assert(professionFilterUpdates == 0, 'changed filters dispatched synchronously')
        "#,
    )
    .unwrap();
    fire_one_on_update_tick(&env);
    env.exec(
        r#"
        assert(professionFilterUpdates == 1, 'changed filters must coalesce into one update')
        Professions.SetAllSourcesFiltered(false)
        assert(professionFilterUpdates == 1, 'unchanged clear must not dispatch')
        "#,
    )
    .unwrap();
    fire_one_on_update_tick(&env);
    env.exec("assert(professionFilterUpdates == 1, 'unchanged clear must not queue an update')")
        .unwrap();
}

#[test]
fn recipe_filter_setters_queue_only_changed_values() {
    let env = WowLuaEnv::new().expect("environment");
    env.exec(
        r#"
        filterUpdates = 0
        filterListener = CreateFrame('Frame')
        filterListener:RegisterEvent('TRADE_SKILL_LIST_UPDATE')
        filterListener:SetScript('OnEvent', function() filterUpdates = filterUpdates + 1 end)
        "#,
    )
    .unwrap();
    for (index, setter) in [
        "C_TradeSkillUI.SetShowLearned(false)",
        "C_TradeSkillUI.SetShowUnlearned(true)",
        "C_TradeSkillUI.SetOnlyShowMakeableRecipes(true)",
        "C_TradeSkillUI.SetOnlyShowSkillUpRecipes(true)",
        "C_TradeSkillUI.SetOnlyShowFirstCraftRecipes(true)",
        "C_TradeSkillUI.SetRecipeSourceTypeFilter(1, true)",
        "C_TradeSkillUI.ClearRecipeSourceTypeFilter()",
        "C_TradeSkillUI.SetSourceTypeFilter(2)",
        "C_TradeSkillUI.ClearRecipeSourceTypeFilter()",
        "C_TradeSkillUI.SetRecipeItemLevelFilter(590, 610)",
        "C_TradeSkillUI.SetRecipeItemNameFilter('sharpening')",
    ]
    .into_iter()
    .enumerate()
    {
        env.exec(setter).unwrap();
        env.exec(setter).unwrap();
        assert_eq!(
            env.eval::<i32>("return filterUpdates").unwrap(),
            index as i32
        );
        fire_one_on_update_tick(&env);
        assert_eq!(
            env.eval::<i32>("return filterUpdates").unwrap(),
            index as i32 + 1
        );
        env.exec(setter).unwrap();
        fire_one_on_update_tick(&env);
        assert_eq!(
            env.eval::<i32>("return filterUpdates").unwrap(),
            index as i32 + 1
        );
    }
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn profession_favorites_feed_the_original_recipe_group() {
    let env = full_game_env();
    env.exec(
        r#"
        C_TradeSkillUI.SetProfessionChildSkillLineID(164)
        assert(C_AddOns.LoadAddOn('Blizzard_Professions'))
        function countFavoriteRecipe(recipeID)
            local provider = Professions.GenerateCraftingDataProvider(164, false, false, nil)
            local count = 0
            for _, node in provider:EnumerateEntireRange() do
                local info = node:GetData().recipeInfo
                if info and info.recipeID == recipeID and info.favoritesInstance then
                    count = count + 1
                end
            end
            return count
        end
        assert(countFavoriteRecipe(2662) == 0, 'emulated initial favorites are empty')
        C_TradeSkillUI.SetRecipeFavorite(2662, true)
        "#,
    )
    .unwrap();
    fire_one_on_update_tick(&env);
    env.exec(
        r#"
        assert(C_TradeSkillUI.IsRecipeFavorite(2662) == true)
        assert(C_TradeSkillUI.GetRecipeInfo(2662).favorite == true)
        assert(countFavoriteRecipe(2662) == 1, 'Blizzard favorites group contains recipe')
        C_TradeSkillUI.SetRecipeFavorite(2662, false)
        "#,
    )
    .unwrap();
    fire_one_on_update_tick(&env);
    env.exec("assert(countFavoriteRecipe(2662) == 0, 'unfavorite removes the group entry')")
        .unwrap();
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn recipe_favorites_validate_state_and_defer_distinct_events() {
    let env = WowLuaEnv::new().expect("environment");
    env.exec(
        r#"
        local api = C_TradeSkillUI
        assert(select('#', api.IsRecipeFavorite(99999999)) == 0)
        assert(api.IsRecipeFavorite(2662) == false, 'emulated initial state')
        local ok, err = pcall(api.IsRecipeFavorite, {})
        assert(not ok and err:find('Usage: IsRecipeFavorite%(recipeID%)'))
        ok, err = pcall(api.SetRecipeFavorite, nil, true)
        assert(not ok and err:find('Usage: SetRecipeFavorite%(recipeID, favorite%)'))
        ok, err = pcall(api.IsRecipeFavorite, 2147483648)
        assert(not ok and err:find('integer overflow attempting to store'))
        favoriteEvents = {}
        favoriteListener = CreateFrame('Frame')
        favoriteListener:RegisterEvent('TRADE_SKILL_FAVORITES_CHANGED')
        favoriteListener:RegisterEvent('TRADE_SKILL_LIST_UPDATE')
        favoriteListener:SetScript('OnEvent', function(_, event, favorite, recipeID)
            if event == 'TRADE_SKILL_FAVORITES_CHANGED' then
                table.insert(favoriteEvents, tostring(favorite) .. ':' .. tostring(recipeID))
            else
                table.insert(favoriteEvents, 'list')
            end
        end)
        assert(select('#', api.SetRecipeFavorite(2662, true)) == 0)
        assert(api.IsRecipeFavorite(2662) == false, 'no active context means no write')
        api.SetProfessionChildSkillLineID(186)
        api.SetRecipeFavorite(2662, true)
        assert(api.IsRecipeFavorite(2662) == false, 'different profession means no write')
        api.SetProfessionChildSkillLineID(164)
        api.SetRecipeFavorite(99999999, true)
        assert(select('#', api.SetRecipeFavorite(' 2662.9 ', 0)) == 0)
        assert(api.IsRecipeFavorite('0xa66') == true)
        assert(api.IsRecipeFavorite(2660) == false, 'favorites are per recipe')
        api.SetRecipeFavorite(2662, true)
        api.SetRecipeFavorite(2662, false)
        api.SetRecipeFavorite(2662, true)
        api.SetOnlyShowMakeableRecipes(true)
        assert(#favoriteEvents == 0, 'favorite and list notifications are deferred')
        "#,
    )
    .unwrap();
    fire_one_on_update_tick(&env);
    env.exec(
        r#"
        assert(#favoriteEvents == 3, 'identical pending pairs coalesce')
        assert(favoriteEvents[1] == 'true:2662')
        assert(favoriteEvents[2] == 'false:2662')
        assert(favoriteEvents[3] == 'list', 'favorites drain before the list marker')
        assert(C_TradeSkillUI.IsRecipeFavorite(2662) == true)
        assert(C_TradeSkillUI.GetRecipeInfo(2662).favorite == true)
        C_TradeSkillUI.SetRecipeFavorite(2662, true)
        "#,
    )
    .unwrap();
    fire_one_on_update_tick(&env);
    env.exec(
        r#"
        assert(#favoriteEvents == 3, 'unchanged writes do not queue events')
        C_TradeSkillUI.SetRecipeFavorite(2662, false)
        assert(#favoriteEvents == 3, 'favorite-only notification is deferred')
        "#,
    )
    .unwrap();
    fire_one_on_update_tick(&env);
    env.exec(
        "assert(#favoriteEvents == 4 and favoriteEvents[4] == 'false:2662', 'favorite setter does not queue a list update')",
    )
    .unwrap();
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn recipe_favorites_are_not_persisted_across_ui_reload() {
    let mut env = WowLuaEnv::new().expect("environment");
    env.exec(
        r#"
        C_TradeSkillUI.SetProfessionChildSkillLineID(164)
        C_TradeSkillUI.SetRecipeFavorite(2662, true)
        assert(C_TradeSkillUI.IsRecipeFavorite(2662))
        ReloadUI()
        "#,
    )
    .unwrap();
    assert!(env.service_reload(&mut None).unwrap());
    env.exec("assert(C_TradeSkillUI.IsRecipeFavorite(2662) == false)")
        .unwrap();
}

/// WM87: with no favorite rows the original order-page button is disabled.
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn favorite_order_recipes_drive_the_original_orders_button() {
    let env = full_game_env();
    env.exec(
        r#"
        C_TradeSkillUI.SetProfessionChildSkillLineID(164)
        assert(C_AddOns.LoadAddOn('Blizzard_Professions'))
        local api = C_TradeSkillUI.HasFavoriteOrderRecipes
        assert(select('#', api()) == 1 and api('ignored', 7) == false)
        local page = ProfessionsFrame.OrdersPage
        local button = page.BrowseFrame.FavoritesSearchButton
        button:SetEnabled(true)
        button.Icon:SetDesaturated(false)
        page:UpdateFavoritesButton()
        assert(not button:IsEnabled(), 'no favorite order rows disables search')
        assert(button.Icon:IsDesaturated(), 'no favorite order rows desaturates icon')
        "#,
    )
    .unwrap();
}

/// WM87 original consumer: metadata-backed favorites enable search; other favorites do not.
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn favorite_order_recipes_metadata_drives_original_orders_button() {
    let env = full_game_env();
    env.exec(
        r#"
        C_TradeSkillUI.SetProfessionChildSkillLineID(164)
        assert(C_AddOns.LoadAddOn('Blizzard_Professions'))
        local page = ProfessionsFrame.OrdersPage
        local button = page.BrowseFrame.FavoritesSearchButton
        C_TradeSkillUI.SetRecipeFavorite(2662, true)
        page:UpdateFavoritesButton()
        assert(not button:IsEnabled(), 'favorite without CraftingOrder metadata stays disabled')
        assert(button.Icon:IsDesaturated(), 'ineligible favorite keeps icon desaturated')
        C_TradeSkillUI.SetRecipeFavorite(365729, true)
        page:UpdateFavoritesButton()
        assert(button:IsEnabled(), 'favorite with CraftingOrder metadata enables search')
        assert(not button.Icon:IsDesaturated(), 'eligible favorite restores icon color')
        "#,
    )
    .unwrap();
}

/// API-level WM87 table: no favorites, only ineligible favorites, eligible favorite.
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn favorite_order_recipes_api_metadata_rows() {
    let env = WowLuaEnv::new().expect("environment");
    env.exec(
        r#"
        C_TradeSkillUI.SetProfessionChildSkillLineID(164)
        local api = C_TradeSkillUI.HasFavoriteOrderRecipes
        assert(select('#', api()) == 1 and api() == false, 'no favorite rows')
        C_TradeSkillUI.SetRecipeFavorite(2662, true)
        assert(select('#', api()) == 1 and api() == false, 'favorite is not order eligible')
        C_TradeSkillUI.SetRecipeFavorite(365729, true)
        assert(select('#', api()) == 1 and api('ignored', {}) == true, 'eligible favorite row')
        C_TradeSkillUI.SetRecipeFavorite(365729, false)
        assert(api() == false, 'only the ineligible favorite remains')
        C_TradeSkillUI.SetRecipeFavorite(438914, true)
        assert(api() == true, 'metadata row qualifies even with Field_10_0_2_46479_002 zero')
        "#,
    )
    .unwrap();
}

/// WM88: one Boolean from the ability relation; GetRecipeInfo shares the predicate.
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn recipe_first_craft_resolves_ability_and_validates_arguments() {
    let env = WowLuaEnv::new().expect("environment");
    env.exec(
        r#"
        local api = C_TradeSkillUI.IsRecipeFirstCraft
        local usage = 'Usage: local result = C_TradeSkillUI.IsRecipeFirstCraft(recipeID)'
        local range = 'outside of expected range -2147483648 to 2147483647 - ' .. usage
        assert(select('#', api(99999999)) == 1, 'unknown recipe publishes one value')
        assert(api(99999999) == false, 'no ability relation is false')
        assert(api(-2147483648) == false, 'negative IDs are not rejected')
        assert(select('#', api(2662)) == 1)
        assert(api(' 2662.9 ', 'ignored') == true, 'unavailable completion state reads clear')
        assert(C_TradeSkillUI.GetRecipeInfo(2662).firstCraft == api(2662), 'recipe info shares the predicate')
        local ok, err = pcall(api)
        assert(not ok and err:find(usage, 1, true), tostring(err))
        ok, err = pcall(api, nil)
        assert(not ok and err:find(usage, 1, true), tostring(err))
        for _, bad in ipairs({2147483648, -2147483649, 'abc', {}, 0/0, math.huge}) do
            ok, err = pcall(api, bad)
            assert(not ok and err:find(range, 1, true), tostring(err))
        end
        local secretRecipe = secretwrap(2662)
        assert(api(secretRecipe) == true, 'untainted callers may pass secrets')
        local function denied() return pcall(api, secretRecipe) end
        debug.setobjecttaint(denied, 'FirstCraftProbe')
        ok, err = denied()
        local secret = usage .. '. Secret values are only allowed during untainted execution for this argument.'
        assert(not ok and err:find(secret, 1, true), tostring(err))
        "#,
    )
    .unwrap();
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn professions_consumer_uses_supplied_recipe_cooldown_and_charges() {
    use wow_ui_sim::c_api::charge_state::SpellChargeState;
    use wow_ui_sim::lua_api::state::SpellCooldownState;

    let env = full_game_env();
    let toc = blizzard_ui_dir()
        .join("Blizzard_ProfessionsTemplates")
        .join("Blizzard_ProfessionsTemplates.toc");
    load_addon(&env.loader_env(), &toc).expect("load Blizzard Professions templates");
    env.exec(
        r#"
        A_Admin.SetManualClock(100)
        assert(not Professions.IsRecipeOnCooldown(2662), 'uncooldowned recipe')
        "#,
    )
    .unwrap();
    env.state().borrow_mut().spell_cooldowns.insert(
        2662,
        SpellCooldownState {
            start: 90.0,
            duration: 30.0,
        },
    );
    env.exec("assert(Professions.IsRecipeOnCooldown(2662), 'active cooldown')")
        .unwrap();
    env.state().borrow_mut().spell_charges.insert(
        2662,
        SpellChargeState {
            current_charges: 2,
            max_charges: 3,
            recharge_start: 96.0,
            recharge_duration: 10.0,
            charge_mod_rate: 1.0,
        },
    );
    env.exec("assert(not Professions.IsRecipeOnCooldown(2662), 'available charges permit use')")
        .unwrap();
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn recipe_cooldown_projects_supplied_spell_state_and_validates_arguments() {
    use wow_ui_sim::c_api::charge_state::SpellChargeState;
    use wow_ui_sim::lua_api::state::SpellCooldownState;

    let env = WowLuaEnv::new().expect("environment");
    env.exec(
        r#"
        A_Admin.SetManualClock(100)
        local api = C_TradeSkillUI.GetRecipeCooldown
        assert(select('#', api(99999999)) == 0, 'missing cached recipe')
        assert(select('#', api(2662)) == 4, 'cached recipe return count')
        local remaining, day, charges, maximum = api(' 2662.9 ', 'ignored')
        assert(remaining == nil and day == false and charges == 0 and maximum == 0)
        local ok, err = pcall(api, nil)
        assert(not ok and err:find('Usage: GetRecipeCooldown%(recipeID%)'))
        ok, err = pcall(api, {})
        assert(not ok and err:find('Usage: GetRecipeCooldown%(recipeID%)'))
        ok, err = pcall(api, 2147483648)
        assert(not ok and err:find('integer overflow attempting to store'))
        local secretNumber = secretwrap(2662)
        local function denied() return pcall(api, secretNumber) end
        debug.setobjecttaint(denied, 'RecipeCooldownProbe')
        ok, err = denied()
        assert(not ok and err:find('Usage: GetRecipeCooldown%(recipeID%)'))
        ok, err = pcall(api, secretwrap('2662'))
        assert(not ok and err:find('Usage: GetRecipeCooldown%(recipeID%)'))
        "#,
    )
    .unwrap();
    env.state().borrow_mut().spell_cooldowns.insert(
        2662,
        SpellCooldownState {
            start: 90.0,
            duration: 30.0,
        },
    );
    env.exec(
        r#"
        local remaining, day, charges, maximum = C_TradeSkillUI.GetRecipeCooldown(2662)
        assert(math.abs(remaining - 20) < 0.001 and day == false and charges == 0 and maximum == 0)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().spell_charges.insert(
        2662,
        SpellChargeState {
            current_charges: 2,
            max_charges: 3,
            recharge_start: 96.0,
            recharge_duration: 10.0,
            charge_mod_rate: 1.0,
        },
    );
    env.exec(
        r#"
        local remaining, day, charges, maximum = C_TradeSkillUI.GetRecipeCooldown(2662)
        assert(math.abs(remaining - 6) < 0.001 and day == false and charges == 2 and maximum == 3)
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .spell_charges
        .get_mut(&2662)
        .unwrap()
        .current_charges = 3;
    env.exec(
        r#"
        local remaining, _, charges, maximum = C_TradeSkillUI.GetRecipeCooldown(2662)
        assert(math.abs(remaining - 20) < 0.001 and charges == 3 and maximum == 3)
        "#,
    )
    .unwrap();
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn profession_day_cooldown_displays_midnight_for_truesteel() {
    use wow_ui_sim::lua_api::state::SpellCooldownState;

    let env = full_game_env();
    env.exec("A_Admin.SetManualClock(100)").unwrap();
    for recipe_id in [171690, 2662] {
        env.state().borrow_mut().spell_cooldowns.insert(
            recipe_id,
            SpellCooldownState {
                start: 90.0,
                duration: 3600.0,
            },
        );
    }
    env.exec(
        r#"
        CastSpellByID(2018)
        local form = ProfessionsFrame.CraftingPage.SchematicForm
        local recipe = C_TradeSkillUI.GetRecipeInfo(171690)
        assert(recipe and recipe.recipeID == 171690, 'Truesteel recipe is loaded')
        form:Init(recipe)
        assert(form.Cooldown:GetText() == COOLDOWN_EXPIRES_AT_MIDNIGHT,
            'Truesteel cooldown should display midnight: ' .. tostring(form.Cooldown:GetText()))
        local ordinary = C_TradeSkillUI.GetRecipeInfo(2662)
        form:Init(ordinary)
        local text = form.Cooldown:GetText()
        assert(text and text ~= '' and text ~= COOLDOWN_EXPIRES_AT_MIDNIGHT,
            'ordinary recipe should display remaining time: ' .. tostring(text))
        "#,
    )
    .unwrap();
}

#[test]
fn casting_blacksmithing_opens_professions_frame() {
    let env = full_game_env();
    env.state().borrow_mut().lua_errors.clear();

    let result: String = env
        .eval(
            r#"
            C_AddOns.LoadAddOn("Blizzard_Professions")
            local original = ProfessionsRecipeSchematicFormMixin.Init
            local form = ProfessionsFrame.CraftingPage.SchematicForm
            local originalInit = form.Init
            form.Init = function(self, ...)
                print("DIAG Init entry", ...)
                local result = originalInit(self, ...)
                print("DIAG Init exit", self.currentRecipeInfo, self.transaction)
                return result
            end
            CastSpellByID(2018)
            print("DIAG transaction", form.currentRecipeInfo, form.transaction)
            if not ProfessionsFrame or not ProfessionsFrame:IsShown() then
                return "frame_not_shown"
            end
            if not ProfessionsFrame.CraftingPage or not ProfessionsFrame.CraftingPage.LinkButton then
                return "crafting_page_link_button_missing"
            end
            if ProfessionsFrame:GetWidth() < 900 then
                return "width=" .. tostring(ProfessionsFrame:GetWidth())
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "ProfessionsFrame should be visible and expanded after casting Blacksmithing (spell 2018): {result}"
    );
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "Casting Blacksmithing should not report Lua errors: {:?}",
        env.state().borrow().lua_errors
    );
}

#[test]
fn clicking_blacksmithing_button_in_professions_book_opens_panel() {
    let env = full_game_env();
    env.state().borrow_mut().lua_errors.clear();

    let shown: bool = env
        .eval(
            r#"
            ToggleProfessionsBook()

            local candidates
            if WOW_PROJECT_ID == WOW_PROJECT_MAINLINE and ProfessionsBookFrame.ProfessionsContentFrame then
                -- PTR 70493 Mainline/Blizzard_ProfessionsBookTemplates.xml:58,63,166.
                local content = ProfessionsBookFrame.ProfessionsContentFrame
                candidates = {
                    content.PrimaryProfession1.SpellButton1,
                    content.PrimaryProfession1.SpellButton2,
                    content.PrimaryProfession2.SpellButton1,
                    content.PrimaryProfession2.SpellButton2,
                }
            else
                candidates = {
                    PrimaryProfession1SpellButtonTop,
                    PrimaryProfession1SpellButtonBottom,
                    PrimaryProfession2SpellButtonTop,
                    PrimaryProfession2SpellButtonBottom,
                }
            end
            local button
            for _, candidate in ipairs(candidates) do
                if candidate then
                    local slot = ProfessionsBook_GetSpellBookItemSlot(candidate)
                    if slot then
                        local name = C_SpellBook.GetSpellBookItemName(slot, 0)
                        if name == "Blacksmithing" then
                            button = candidate
                            break
                        end
                    end
                end
            end

            assert(button, "no ProfessionsBook button is labelled 'Blacksmithing'")
            button:Click("LeftButton")

            return ProfessionsFrame ~= nil and ProfessionsFrame:IsShown() or false
            "#,
        )
        .unwrap();

    assert!(
        shown,
        "Clicking the Blacksmithing spell button should open ProfessionsFrame"
    );
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "Clicking Blacksmithing should not report Lua errors: {:?}",
        env.state().borrow().lua_errors
    );
}
