//! Original selected Retail consumers. Explicit host inputs do not establish
//! unknown candidate predicates, link payload construction or record writers.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]
use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::c_api::container_purchase::PurchaseRecord;
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::real::inventory_candidates::{
    CandidateLocation, InventoryCandidate,
};
use wow_ui_sim::lua_api::state::BagItem;
use wow_ui_sim::screen::ScreenKind;

fn startup() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    let addons = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    let selected = addons
        .iter()
        .find(|a| a.name == "Blizzard_UIPanels_Game")
        .unwrap();
    assert_eq!(
        selected.toc_path.file_name().unwrap().to_str(),
        Some("Blizzard_UIPanels_Game_Mainline.toc")
    );
    for addon in addons {
        if !env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|a| a.folder_name == addon.name && a.loaded)
        {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None).unwrap();
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.sync_string_metatable_to_global_string();
    env
}

#[test]
fn original_paper_doll_flyout_mutates_same_table_across_inputs() {
    let env = startup();
    env.state().borrow_mut().player.present = true;
    env.exec(r#"
        assert(debug.getinfo(PaperDollFrameItemFlyout_GetItems,'S').source:find('Blizzard_UIPanels_Game/Mainline/PaperDollFrame.lua',1,true))
        InventoryConsumerTable={sentinel='keep'}; PreviousInventoryConsumerLink=nil
    "#).unwrap();
    for text in ["host-supplied-link-one", "host-supplied-link-two"] {
        env.state()
            .borrow_mut()
            .inventory_candidates
            .accepted
            .insert(
                (10, false),
                vec![InventoryCandidate {
                    location: CandidateLocation::Equipment(10),
                    link: Some(text.into()),
                }],
            );
        env.exec(&format!(
            r#"
            PaperDollFrameItemFlyout_GetItems(11,InventoryConsumerTable)
            local current=InventoryConsumerTable[1048587]
            assert(current=='{text}' and current~=PreviousInventoryConsumerLink)
            assert(InventoryConsumerTable.sentinel=='keep')
            PreviousInventoryConsumerLink=current
        "#
        ))
        .unwrap();
    }
    env.state()
        .borrow_mut()
        .inventory_candidates
        .accepted
        .get_mut(&(10, false))
        .unwrap()[0]
        .link = None;
    env.exec("PaperDollFrameItemFlyout_GetItems(11,InventoryConsumerTable); assert(InventoryConsumerTable[1048587]==nil and InventoryConsumerTable.sentinel=='keep')").unwrap();
}

fn prepare_refund_consumer(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.player.present = true;
        sim.bag_items.insert(
            (0, 1),
            BagItem {
                owned_identity: Some(100),
                item_id: 6948,
                stack_count: 1,
                hyperlink: None,
                durability: None,
                applied_transmog: None,
            },
        );
        let input = &mut sim.purchase_inputs;
        input.inventory_count = 40;
        input.carried_count = 1;
        input.inventory.insert(35, 100);
    }
    env.exec(r#"
        assert(debug.getinfo(ContainerFrame_GetExtendedPriceString,'S').source:find('Blizzard_UIPanels_Game/Mainline/ContainerFrame.lua',1,true))
        RefundConsumerButton=ContainerFrame1:AcquireNewItemButton()
        RefundConsumerButton:SetID(1); RefundConsumerButton:SetBagID(0)
        RefundConsumerObserved=nil; PreviousRefundConsumerText=nil
        -- Observe only the terminal popup publication; original behavior remains.
        hooksecurefunc('StaticPopup_Show',function(id,text)
            if id=='CONFIRM_REFUND_TOKEN_ITEM' then RefundConsumerObserved=text end
        end)
        assert(ContainerFrame_GetExtendedPriceString(RefundConsumerButton,false)==false)
        assert(RefundConsumerObserved==nil)
    "#).unwrap();
}

#[test]
fn original_container_refund_consumer_tracks_amount_and_expiry() {
    let env = startup();
    prepare_refund_consumer(&env);
    for money in [12345, 67890] {
        env.state().borrow_mut().purchase_inputs.records.insert(
            100,
            PurchaseRecord {
                money,
                expiration: 1,
                ..Default::default()
            },
        );
        env.exec(&format!(
            r#"
            RefundConsumerObserved=nil
            assert(ContainerFrame_GetExtendedPriceString(RefundConsumerButton,false)==true)
            local text=RefundConsumerObserved
            assert(text=='|W'..GetMoneyString({money})..'|w')
            assert(text~=PreviousRefundConsumerText, 'constant refund producer')
            PreviousRefundConsumerText=text
            StaticPopup_Hide('CONFIRM_REFUND_TOKEN_ITEM')
        "#
        ))
        .unwrap();
    }
    env.state().borrow_mut().purchase_inputs.refund_clock = 7201;
    env.exec(
        r#"
        RefundConsumerObserved=nil
        assert(ContainerFrame_GetExtendedPriceString(RefundConsumerButton,false)==false)
        assert(RefundConsumerObserved==nil)
    "#,
    )
    .unwrap();
}
