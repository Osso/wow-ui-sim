use wow_ui_sim::lua_api::WowLuaEnv;

const SOURCE: &str = include_str!("../docs/addons/ServerSnapshot/ServerSnapshot.lua");
const BAG_API: &str = r#"
ServerSnapshotDB = nil
NUM_TOTAL_EQUIPPED_BAG_SLOTS = 5
BACKPACK_CONTAINER = 0
C_Container = {
    GetContainerNumSlots = function(bag) return ({[0]=16, [1]=2, [2]=0, [3]=0, [4]=0, [5]=1})[bag] end,
    GetContainerNumFreeSlots = function(bag) return 0, bag == 1 and 8 or 0 end,
    GetContainerItemInfo = function(bag, slot)
        if bag == 0 and slot == 1 then return {itemID=100, stackCount=4, hyperlink="item:100"} end
        if bag == 5 and slot == 1 then return {itemID=200, stackCount=1} end
    end,
    ContainerIDToInventoryID = function(bag) return 19 + bag end,
    GetBagName = function(bag) if bag == 1 then return "Herb Bag" end end,
}
GetInventoryItemID = function(unit, slot) if slot == 20 then return 500 end end
GetInventoryItemLink = function(unit, slot) if slot == 20 then return "item:500" end end
snapshotFrame = nil
CreateFrame = function()
    local f = {registered = {}}
    function f:RegisterEvent(event) self.registered[event] = true end
    function f:SetScript(_, callback) self.callback = callback end
    if not snapshotFrame then snapshotFrame = f end
    return f
end
"#;

fn load_with_bag_api() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua env");
    env.exec(BAG_API).expect("seed modern container API");
    env.exec(SOURCE).expect("load actual producer");
    env
}

#[test]
fn captures_all_carried_containers_and_occupied_slots() {
    let env = load_with_bag_api();
    let valid: bool = env.eval(r#"
        local bags = ServerSnapshot:Snapshot("test").bags
        local b0, b1, b2, b5 = bags.containers[0], bags.containers[1], bags.containers[2], bags.containers[5]
        return bags.maxBagID == 5
            and b0.numSlots == 16 and b0.family == 0 and b0.inventorySlot == nil
            and b0.items[1].itemID == 100 and b0.items[1].stackCount == 4
            and b0.items[1].hyperlink == "item:100" and b0.items[2] == nil
            and b1.numSlots == 2 and b1.family == 8 and b1.name == "Herb Bag"
            and b1.inventorySlot == 20 and b1.itemID == 500 and b1.hyperlink == "item:500"
            and b1.items[1] == nil and b2.numSlots == 0 and b2.items[1] == nil
            and b5.numSlots == 1 and b5.items[1].itemID == 200
            and b5.items[1].stackCount == 1 and b5.items[1].hyperlink == nil
    "#).expect("inspect captured containers");
    assert!(
        valid,
        "full carried range, empty bags and slots, and optional metadata"
    );
}

#[test]
fn missing_required_api_or_unready_backpack_omits_domain_without_reusing_prior_capture() {
    let env = load_with_bag_api();
    let valid: bool = env
        .eval(
            r#"
        local previous = ServerSnapshot:Snapshot("ready")
        C_Container.GetContainerItemInfo = nil
        local unavailable = ServerSnapshot:Snapshot("missing API")
        C_Container.GetContainerItemInfo = function() end
        C_Container.GetContainerNumSlots = function() return 0 end
        local unready = ServerSnapshot:Snapshot("unready")
        return previous.bags ~= nil and unavailable.bags == nil and unready.bags == nil
            and ServerSnapshotDB.characters[previous.characterKey].bags == nil
    "#,
        )
        .expect("inspect unavailable snapshots");
    assert!(
        valid,
        "unavailable capture must not encode an empty authoritative bag set"
    );
}

#[test]
fn optional_bag_metadata_can_be_missing_without_losing_contents() {
    let env = load_with_bag_api();
    let valid: bool = env
        .eval(
            r#"
        C_Container.GetBagName = nil
        GetInventoryItemID = nil
        GetInventoryItemLink = nil
        local bags = ServerSnapshot:Snapshot("optional metadata").bags
        return bags ~= nil and bags.containers[1].inventorySlot == 20
            and bags.containers[1].name == nil and bags.containers[1].itemID == nil
            and bags.containers[1].hyperlink == nil
            and bags.containers[0].items[1].itemID == 100
    "#,
        )
        .expect("inspect optional metadata");
    assert!(
        valid,
        "unavailable bag names and inventory item metadata are optional"
    );
}

#[test]
fn missing_modern_container_mapping_or_failed_slot_read_omits_domain() {
    let env = load_with_bag_api();
    let valid: bool = env
        .eval(
            r#"
        local mapping = C_Container.ContainerIDToInventoryID
        C_Container.ContainerIDToInventoryID = nil
        local noMapping = ServerSnapshot:Snapshot("missing mapping")
        C_Container.ContainerIDToInventoryID = mapping
        C_Container.GetContainerItemInfo = function() error("not ready") end
        local failedRead = ServerSnapshot:Snapshot("failed item read")
        C_Container = nil
        local noModernAPI = ServerSnapshot:Snapshot("missing container API")
        return noMapping.bags == nil and failedRead.bags == nil and noModernAPI.bags == nil
    "#,
        )
        .expect("inspect incomplete capture");
    assert!(
        valid,
        "missing modern APIs and failed reads must not produce partial bags"
    );
}

#[test]
fn delayed_bag_events_refresh_snapshot_without_per_item_updates() {
    let env = load_with_bag_api();
    let valid: bool = env
        .eval(
            r#"
        local eventFrame = snapshotFrame
        ServerSnapshot:Snapshot("intermediate")
        local events = eventFrame.registered
        eventFrame.callback(eventFrame, "BAG_UPDATE_DELAYED")
        local delayed = ServerSnapshotDB.characters[ServerSnapshotDB.lastCharacterKey]
        eventFrame.callback(eventFrame, "BAG_CONTAINER_UPDATE")
        local changed = ServerSnapshotDB.characters[ServerSnapshotDB.lastCharacterKey]
        return events.BAG_UPDATE_DELAYED and events.BAG_CONTAINER_UPDATE
            and events.PLAYER_ENTERING_WORLD and not events.BAG_UPDATE
            and delayed.reason == "BAG_UPDATE_DELAYED"
            and changed.reason == "BAG_CONTAINER_UPDATE" and changed.bags ~= nil
    "#,
        )
        .expect("inspect bag refresh events");
    assert!(
        valid,
        "coalesced bag and container events must refresh snapshot"
    );
}
