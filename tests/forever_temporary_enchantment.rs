//! Forever publication of the existing temporary weapon-enchant snapshot model.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::weapon_enchants::{TEMPORARY_ENCHANT_TYPE, WeaponEnchant};
use wow_ui_sim::lua_api::WowLuaEnv;

fn enchant(enchant_type: u32, id: u32, time_left: f64, charges: u32) -> WeaponEnchant {
    WeaponEnchant {
        enchant_type,
        time_left,
        charges,
        enchant_id: id,
        icon_id: 135913,
    }
}

#[test]
fn forever_temporary_enchantment_selects_shared_snapshots_by_inventory_slot() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        for (index, time_left) in [1250.0, 30_000.0, 45_500.0].into_iter().enumerate() {
            state.weapon_enchants[index] = vec![
                enchant(1, 900, 999.0, 0),
                enchant(TEMPORARY_ENCHANT_TYPE, 40 + index as u32, time_left, 3),
                enchant(TEMPORARY_ENCHANT_TYPE, 800, 1.0, 0),
            ];
        }
    }
    env.exec(
        r#"
        local query = rawget(C_PaperDollInfo, 'GetTemporaryEnchantmentInfo')
        assert(type(query) == 'function')
        for index, time in ipairs({1250, 30000, 45500}) do
            local slot = index + 15
            local info = query(slot)
            assert(type(info) == 'table' and select('#', query(slot)) == 1)
            assert(info.enchantID == 39 + index)
            assert(info.remainingTimeMs == time and info.chargesRemaining == 3)
            assert(info.hasExpirationTime == true)
            assert(type(info.hasExpirationTime) == 'boolean')
            local count = 0
            for _ in pairs(info) do count = count + 1 end
            assert(count == 4, 'TemporaryItemEnchantInfo has exactly four fields')
            local shared = C_Item.GetWeaponEnchantInfo(index - 1)[2]
            assert(shared.enchantID == info.enchantID and shared.timeLeft == info.remainingTimeMs)
        end
        local _, mainTime, mainCharges, mainID, _, offTime, offCharges, offID = GetWeaponEnchantInfo()
        assert(mainTime == 1250 and mainCharges == 3 and mainID == 40)
        assert(offTime == 30000 and offCharges == 3 and offID == 41)
        assert(C_PaperDollInfo.GetTemporaryEnchantmentInfo(16, 'ignored').enchantID == 40)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_temporary_enchantment_empty_and_unsupported_slots_return_zero_values() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().weapon_enchants[0].push(enchant(1, 900, 1000.0, 0));
    env.exec(
        r#"
        local query = C_PaperDollInfo.GetTemporaryEnchantmentInfo
        for _, slot in ipairs({0, 1, 15, 16, 17, 18, 19, -1}) do
            assert(query(slot) == nil)
            assert(select('#', query(slot)) == 0, 'no enchant returns nothing, not one nil')
        end
        assert(not pcall(query))
        assert(not pcall(query, nil))
        assert(not pcall(query, {}))
        "#,
    )
    .unwrap();
}

#[test]
fn forever_temporary_enchantment_updates_are_shared_and_queries_survive_cleanup() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().weapon_enchants[0] =
        vec![enchant(TEMPORARY_ENCHANT_TYPE, 40, 1250.0, 3)];
    env.exec(
        r#"
        local info = C_PaperDollInfo.GetTemporaryEnchantmentInfo(16)
        info.enchantID = 999
        info.remainingTimeMs = 1
        assert(C_PaperDollInfo.GetTemporaryEnchantmentInfo(16).enchantID == 40)
        assert(C_PaperDollInfo.GetTemporaryEnchantmentInfo(16).remainingTimeMs == 1250)
        "#,
    )
    .unwrap();
    let event_count = env.state().borrow().events.pending().len();
    {
        let mut state = env.state().borrow_mut();
        state.weapon_enchants[0][0].time_left = 0.0;
        state.weapon_enchants[0][0].charges = 2;
    }
    env.apply_post_load_workarounds();
    env.exec(
        r#"
        local query = rawget(C_PaperDollInfo, 'GetTemporaryEnchantmentInfo')
        assert(type(query) == 'function')
        local info = query(16)
        assert(info.enchantID == 40 and info.remainingTimeMs == 0)
        assert(info.chargesRemaining == 2 and info.hasExpirationTime == false)
        local _, time, charges = GetWeaponEnchantInfo()
        assert(time == 0 and charges == 2)
        assert(C_Item.GetWeaponEnchantInfo(0)[1].charges == 2)
        "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().events.pending().len(), event_count);
    assert_eq!(env.state().borrow().weapon_enchants[0][0].time_left, 0.0);
    env.state().borrow_mut().weapon_enchants[0].clear();
    env.exec("assert(select('#', C_PaperDollInfo.GetTemporaryEnchantmentInfo(16)) == 0)")
        .unwrap();
}

#[test]
fn forever_temporary_enchantment_selected_original_consumer_uses_ms_updates_and_removal() {
    use std::path::Path;
    use wow_ui_sim::screen::ScreenKind;
    use wow_ui_sim::toc::TocFile;

    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let toc_path = wow_ui_sim::loader::find_toc_file(&ui.join("Blizzard_AuraContainer")).unwrap();
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game) && !toc.is_game_type_restricted());
    for name in [
        "Blizzard_AuraContainerUtil.lua",
        "Blizzard_AuraContainerEnchantments.lua",
    ] {
        let index = toc
            .files
            .iter()
            .position(|path| path == Path::new(name))
            .unwrap();
        assert!(toc.file_allows_screen(index, ScreenKind::Game));
        assert!(
            toc.file_use_secure_env(index)
                .unwrap_or_else(|| toc.is_secure_env())
        );
        assert!(toc.file_allows_environment(index, true));
    }
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_AuraContainer"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_AuraContainer"));
    env.state().borrow_mut().weapon_enchants[0] =
        vec![enchant(TEMPORARY_ENCHANT_TYPE, 40, 1250.0, 3)];
    env.exec(
        r#"
        local util = assert(__secureenv.AuraContainerUtil)
        local slots = assert(__secureenv.AuraContainerItemEnchantmentSlot)
        TemporaryEnchantOwner = {
            OnItemEnchantmentsChanged = function() end,
            InitializeItemEnchantmentFrame = function(self, _, _, unit, data)
                self.initialized = (self.initialized or 0) + 1
                self.unit, self.data = unit, data
            end,
            UpdateItemEnchantmentFrame = function(self, _, _, unit, data)
                self.updated = (self.updated or 0) + 1
                self.unit, self.data = unit, data
            end,
            ClearItemEnchantmentFrame = function(self) self.cleared = (self.cleared or 0) + 1 end,
        }
        TemporaryEnchantManager = util.CreateItemEnchantmentManager(TemporaryEnchantOwner)
        TemporaryEnchantConsumer = TemporaryEnchantManager:RegisterItemEnchantment(slots.MainHand,
            {auraFrame = CreateFrame('Frame')})
        assert(TemporaryEnchantManager:RefreshItemEnchantment(TemporaryEnchantConsumer))
        assert(TemporaryEnchantConsumer:IsActive())
        local data = TemporaryEnchantOwner.data
        assert(TemporaryEnchantOwner.initialized == 1 and TemporaryEnchantOwner.unit == 'player')
        assert(data.itemEnchantmentID == 40 and data.inventorySlot == 16 and data.applications == 3)
        assert(data.duration == 1.25, 'original consumer divides milliseconds by 1000')
        assert(math.abs(data.expirationTime - GetTime() - 1.25) < 0.1)
        "#,
    )
    .expect("original enchantment manager must populate real table values and duration arithmetic");
    {
        let mut state = env.state().borrow_mut();
        state.weapon_enchants[0][0].time_left = 500.0;
        state.weapon_enchants[0][0].charges = 2;
    }
    env.exec(
        r#"
        assert(TemporaryEnchantManager:RefreshItemEnchantment(TemporaryEnchantConsumer))
        assert(TemporaryEnchantOwner.updated == 1 and TemporaryEnchantOwner.initialized == 1)
        local data = TemporaryEnchantOwner.data
        assert(data.applications == 2 and data.duration == 1.25)
        assert(TemporaryEnchantConsumer:GetRemainingTimeMs() == 500)
        assert(math.abs(data.expirationTime - GetTime() - 0.5) < 0.1)
        "#,
    )
    .expect("original retained update must use the latest snapshot without resetting duration");
    env.state().borrow_mut().weapon_enchants[0].clear();
    env.exec(
        r#"
        assert(TemporaryEnchantManager:RefreshItemEnchantment(TemporaryEnchantConsumer))
        assert(not TemporaryEnchantConsumer:IsActive() and TemporaryEnchantConsumer:GetAuraData() == nil)
        assert(TemporaryEnchantOwner.cleared == 1)
        "#,
    )
    .expect("original consumer must clear the display after the modeled enchant is removed");
}
