//! R88k.1/.2 VERIFIED conditional contracts with explicit host input seams.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]
use wow_ui_sim::c_api::container_purchase::PurchaseRecord;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::real::inventory_candidates::{
    CandidateLocation as Location, InventoryCandidate,
};

fn candidate(location: Location, link: Option<&str>) -> InventoryCandidate {
    InventoryCandidate {
        location,
        link: link.map(str::to_owned),
    }
}

#[test]
fn inventory_slot_arguments_and_table_identity() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.present = true;
    env.state()
        .borrow_mut()
        .inventory_candidates
        .slot_names
        .insert("HeadSlot".into(), 1);
    env.exec(
        r#"
        local usage = 'Usage: GetInventoryItemsForSlot(slot, returnTable, ["transmogrify"])'
        for _, f in ipairs({function() GetInventoryItemsForSlot() end,
            function() GetInventoryItemsForSlot(false,{}) end,
            function() GetInventoryItemsForSlot('unknown',{}) end,
            function() GetInventoryItemsForSlot(1) end}) do
            local ok, err = pcall(f); assert(not ok and tostring(err):find(usage,1,true))
        end
        assert(select('#',GetInventoryItemsForSlot(0,false))==0)
        assert(select('#',GetInventoryItemsForSlot(31,false))==0)
        local t={sentinel=42}
        for _, slot in ipairs({1,'1',1.9,4294967297,'hEaDsLoT'}) do
            assert(select('#',GetInventoryItemsForSlot(slot,t))==1)
            assert(GetInventoryItemsForSlot(slot,t,nil,'ignored')==t and t.sentinel==42)
        end
    "#,
    )
    .unwrap();
    env.state().borrow_mut().player.present = false;
    env.exec("assert(select('#',GetInventoryItemsForSlot(1,{}))==0)")
        .unwrap();
}

#[test]
fn inventory_packed_keys_replace_preserve_and_remove() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.player.present = true;
        let i = &mut sim.inventory_candidates;
        i.carried_enabled = true;
        i.carried_count = 2;
        i.accepted.insert(
            (0, false),
            vec![
                candidate(Location::Equipment(18), Some("equipment text")),
                candidate(Location::Carried(36), Some("carried text")),
                candidate(Location::Container { id: 0, slot: 2 }, Some("bag text")),
                candidate(Location::Container { id: 10, slot: 3 }, Some("bank text")),
                candidate(Location::Equipment(0), None),
            ],
        );
    }
    env.exec(
        r#"
        t={[1048577]='remove',[1048595]='replace',sentinel='keep'}
        assert(GetInventoryItemsForSlot(1,t)==t)
        assert(t[1048577]==nil and t[1048595]=='equipment text')
        assert(t[3145730]=='carried text' and t[3145987]=='bag text')
        assert(t[6292996]==nil and t.sentinel=='keep')
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .inventory_candidates
        .interaction_guid_nonzero = true;
    env.exec("assert(GetInventoryItemsForSlot(1,t)==t and t[6292996]=='bank text')")
        .unwrap();
    env.state()
        .borrow_mut()
        .inventory_candidates
        .accepted
        .clear();
    env.exec("assert(GetInventoryItemsForSlot(1,t)==t and t[6292996]=='bank text')")
        .unwrap();
}

#[test]
fn inventory_transmog_and_frozen_write_guard() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.present = true;
    env.exec("assert(select('#',GetInventoryItemsForSlot(1,{},'TRANSMOGRIFY'))==0)")
        .unwrap();
    {
        let mut sim = env.state().borrow_mut();
        let i = &mut sim.inventory_candidates;
        i.transmog_ready_slots.insert(0);
        i.accepted.insert(
            (0, true),
            vec![candidate(
                Location::Container { id: 5, slot: 0 },
                Some("supplied"),
            )],
        );
    }
    env.exec(r#"
        local t={}; assert(GetInventoryItemsForSlot(1,t,'TRANSMOGRIFY\0tail')==t)
        assert(t[6291713]=='supplied')
        local frozen={}; table.freeze(frozen)
        assert(GetInventoryItemsForSlot(1,frozen)==frozen) -- empty scan never writes
        local ok,err=pcall(GetInventoryItemsForSlot,1,frozen,'transmogrify')
        assert(not ok and tostring(err):find('attempted to perform indexed assignment on a frozen table',1,true))
    "#).unwrap();
}

fn purchase_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.player.present = true;
        sim.purchase_inputs.inventory_count = 110;
        sim.purchase_inputs.carried_count = 2;
        sim.purchase_inputs.inventory.insert(0, 100);
        sim.purchase_inputs.inventory.insert(29, 100);
        sim.purchase_inputs.records.insert(
            100,
            PurchaseRecord {
                money: 12345,
                expiration: 100,
                item_quantities: [7, 0, -2, 9, 0],
                currency_quantities: [0, 3, 0, -1, 0],
                modifications: [(4, false), (0, true), (5, true), (0, false)],
            },
        );
        sim.purchase_inputs.refund_clock = 200;
    }
    env
}

#[test]
fn purchase_required_arguments_and_ranges() {
    let env = purchase_env();
    env.exec(r#"
        local f=C_Container.GetContainerItemPurchaseInfo
        local usage='Usage: local info = C_Container.GetContainerItemPurchaseInfo(containerIndex, slotIndex, isEquipped)'
        for n,call in ipairs({function() f() end,function() f(0) end,function() f(0,1) end}) do
            local ok,e=pcall(call); assert(not ok and tostring(e):find('bad argument #'..n,1,true) and tostring(e):find(usage,1,true))
        end
        for _,v in ipairs({true,{},'bad',2147483648,-2147483649,math.huge}) do
            local ok,e=pcall(f,v,1,true); assert(not ok and tostring(e):find('outside of expected range -2147483648 to 2147483647',1,true))
        end
        for _,v in ipairs({true,{},'bad',-1,4294967296,math.huge}) do
            local ok,e=pcall(f,0,v,false); assert(not ok and tostring(e):find('outside of expected range 0 to 4294967295',1,true))
        end
        for _,truthy in ipairs({true,0,'',{}}) do assert(f('0','1',truthy,'extra').money==12345) end
        assert(f(0,0.5,true).money==12345 and f(0,30,true).money==12345)
        assert(select('#',f(0,0,true))==0 and select('#',f(0,31,true))==0)
    "#).unwrap();
}

#[test]
fn purchase_fields_counts_modifications_and_expiry() {
    let env = purchase_env();
    env.exec(
        r#"
        local f=C_Container.GetContainerItemPurchaseInfo
        assert(select('#',f(0,1,true))==1)
        local t=f(0,1,true); local n=0; for _ in pairs(t) do n=n+1 end
        assert(n==5 and t.money==12345 and t.itemCount==2 and t.currencyCount==1)
        assert(t.refundSeconds==7100 and t.hasEnchants==true)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .purchase_inputs
        .records
        .get_mut(&100)
        .unwrap()
        .modifications = [(0, false); 4];
    env.exec("assert(C_Container.GetContainerItemPurchaseInfo(0,1,true).hasEnchants==false)")
        .unwrap();
    env.state().borrow_mut().purchase_inputs.refund_clock = 7300;
    env.exec("assert(select('#',C_Container.GetContainerItemPurchaseInfo(0,1,true))==0)")
        .unwrap();
    env.state().borrow_mut().purchase_inputs.records.clear();
    env.exec("assert(select('#',C_Container.GetContainerItemPurchaseInfo(0,1,true))==0)")
        .unwrap();
}

#[test]
fn purchase_native_container_selection_and_exclusion() {
    let env = purchase_env();
    {
        let mut sim = env.state().borrow_mut();
        let i = &mut sim.purchase_inputs;
        for slot in [35, 63, 100] {
            i.inventory.insert(slot, 100);
        }
        i.container_counts.insert(0, 1);
        i.containers.insert((0, 0), 100);
        i.container_counts.insert(15, 1);
        i.containers.insert((15, 0), 100);
    }
    env.exec(
        r#"
        local f=C_Container.GetContainerItemPurchaseInfo
        for _,bag in ipairs({1,16,0,-2,-3,-4}) do assert(f(bag,1,false).money==12345) end
        for _,bag in ipairs({-1,17,999}) do assert(select('#',f(bag,1,false))==0) end
        assert(select('#',f(0,3,false))==0)
        assert(select('#',f(-2,7,false))==0 and select('#',f(-3,6,false))==0)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().purchase_inputs.excluded_identity = Some(100);
    env.exec("assert(select('#',C_Container.GetContainerItemPurchaseInfo(0,1,true))==0)")
        .unwrap();
    env.state().borrow_mut().purchase_inputs.excluded_identity = None;
    env.state().borrow_mut().player.present = false;
    env.exec("assert(select('#',C_Container.GetContainerItemPurchaseInfo(0,1,true))==0)")
        .unwrap();
}
