#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::c_api::c_transmog_outfit_info::{OutfitEntry, PendingTransmogCost};
use wow_ui_sim::lua_api::WowLuaEnv;

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().transmog_outfit_catalog.entries = vec![
        OutfitEntry {
            outfit_id: 91,
            name: "Raid".into(),
            situation_categories: vec![],
            icon: 135_771,
            is_event_outfit: false,
            is_disabled: false,
            player_facing_outfit_index: 1,
        },
        OutfitEntry {
            outfit_id: 305,
            name: "Travel".into(),
            situation_categories: vec![],
            icon: 132_489,
            is_event_outfit: false,
            is_disabled: false,
            player_facing_outfit_index: 7,
        },
    ];
    env
}

#[test]
fn viewed_outfit_reads_host_state_not_writable_lua_storage_and_is_isolated() {
    let first = seeded_env();
    let second = seeded_env();
    first
        .exec(
            r#"
        local function inspect(...)
            assert(select('#', ...) == 1)
            local id = ...
            assert(type(id) == 'number' and id == 0 and not issecretvalue(id))
        end
        inspect(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID())
        C_TransmogOutfitInfo.__currentlyViewedOutfitID = 999
        assert(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() == 0)
        "#,
        )
        .unwrap();
    first.state().borrow_mut().viewed_transmog_outfit_id = Some(305);
    assert_eq!(
        first
            .eval::<f64>("return C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID()")
            .unwrap(),
        305.0
    );
    assert_eq!(
        second
            .eval::<f64>("return C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID()")
            .unwrap(),
        0.0
    );
    first
        .exec("C_TransmogOutfitInfo.ChangeViewedOutfit(91)")
        .unwrap();
    assert_eq!(first.state().borrow().viewed_transmog_outfit_id, Some(91));
    assert_eq!(second.state().borrow().viewed_transmog_outfit_id, None);
}

#[test]
fn viewed_outfit_id_transition_is_synchronous_and_preserves_active_and_pending() {
    let env = seeded_env();
    env.state().borrow_mut().pending_transmog_cost = Some(PendingTransmogCost {
        cost: 123_450,
        modifier_flags: 14,
    });
    env.exec(
        r#"
        local api = C_TransmogOutfitInfo
        api.ChangeToOutfit(7, false)
        api.SetPendingTransmogSheatheCategory(16, 2, 2)
        local pending = rawget(api, '__pendingSheatheCategories')
        local events = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('VIEWED_TRANSMOG_OUTFIT_CHANGED')
        listener:RegisterEvent('TRANSMOG_DISPLAYED_OUTFIT_CHANGED')
        listener:RegisterEvent('TRANSMOG_OUTFITS_CHANGED')
        listener:SetScript('OnEvent', function(_, event, ...)
            assert(event == 'VIEWED_TRANSMOG_OUTFIT_CHANGED')
            assert(select('#', ...) == 0)
            events[#events + 1] = api.GetCurrentlyViewedOutfitID()
        end)
        assert(select('#', api.ChangeViewedOutfit(91)) == 0)
        assert(api.GetCurrentlyViewedOutfitID() == 91)
        assert(#events == 1 and events[1] == 91)
        assert(api.GetActiveOutfitID() == 305)
        assert(rawget(api, '__pendingSheatheCategories') == pending and pending['16:2'] == 2)
        assert(select('#', api.ChangeViewedOutfit(305)) == 0)
        assert(#events == 2 and events[2] == 305)
        -- INFERRED: valid repeated requests refresh, including initial UI selection.
        api.ChangeViewedOutfit(305)
        assert(#events == 3 and events[3] == 305)
        for _, id in ipairs({1, 7, 999, -1, 0, 91.5, math.huge, 0/0}) do
            assert(select('#', api.ChangeViewedOutfit(id)) == 0)
            assert(api.GetCurrentlyViewedOutfitID() == 305 and #events == 3)
        end
        for _, value in ipairs({'91', false, {}}) do
            assert(not pcall(api.ChangeViewedOutfit, value))
            assert(api.GetCurrentlyViewedOutfitID() == 305 and #events == 3)
        end
        assert(not pcall(api.ChangeViewedOutfit))
        api.ClearOutfit()
        assert(api.GetActiveOutfitID() == 0 and api.GetCurrentlyViewedOutfitID() == 305)
        assert(rawget(api, '__pendingSheatheCategories') == pending)
        assert(#events == 3)
        "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().viewed_transmog_outfit_id, Some(305));
    assert_eq!(
        env.state().borrow().pending_transmog_cost,
        Some(PendingTransmogCost {
            cost: 123_450,
            modifier_flags: 14,
        })
    );
    let empty = WowLuaEnv::new().unwrap();
    empty.exec("C_TransmogOutfitInfo.ChangeViewedOutfit(91); assert(C_TransmogOutfitInfo.GetCurrentlyViewedOutfitID() == 0)").unwrap();
}

#[test]
fn viewed_outfit_authenticates_secrets_before_mutation_and_preserves_caller_taint() {
    let env = seeded_env();
    env.exec(
        r#"
        local api = C_TransmogOutfitInfo
        local id = secretwrap(91)
        collectgarbage('collect')
        api.ChangeViewedOutfit(id)
        assert(api.GetCurrentlyViewedOutfitID() == 91)
        local count = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('VIEWED_TRANSMOG_OUTFIT_CHANGED')
        listener:SetScript('OnEvent', function() count = count + 1 end)
        local deniedID = secretwrap(305)
        local function denied()
            api.ChangeViewedOutfit(deniedID)
        end
        debug.setobjecttaint(denied, 'ViewedOutfitProbe')
        assert(not pcall(denied))
        assert(api.GetCurrentlyViewedOutfitID() == 91 and count == 0)
        assert(not pcall(api.ChangeViewedOutfit, secretwrap('305')))
        assert(api.GetCurrentlyViewedOutfitID() == 91 and count == 0)
        local function ordinary()
            assert(debug.getstacktaint() == 'ViewedOutfitProbe' and not issecure())
            api.ChangeViewedOutfit(305)
            assert(debug.getstacktaint() == 'ViewedOutfitProbe' and not issecure())
        end
        debug.setobjecttaint(ordinary, 'ViewedOutfitProbe')
        ordinary()
        assert(api.GetCurrentlyViewedOutfitID() == 305 and count == 1)
        assert(issecure() and debug.getstacktaint() == nil)
        "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().viewed_transmog_outfit_id, Some(305));
}
