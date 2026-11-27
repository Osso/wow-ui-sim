#![cfg(feature = "client-retail")]
//! Legacy taxi slot globals read the routes supplied with `A_Admin.SetTaxiMapNodes`.
//! The original consumer is `FlightMap_FlightPathDataProviderMixin:ShowBackgroundRoutesFromCurrent`
//! (FM_FlightPathDataProvider.lua:131-160), reached through `TAXIMAP_OPENED`.

use wow_ui_sim::lua_api::WowLuaEnv;

const NODE_TEMPLATE: &str = r#"
local function node(slot, name, state, x, y, cost, routes)
    return { mapID=14, nodeID=900 + slot, name=name, atlasName="TaxiNode", textureKit="",
        faction=0, x=x, y=y, state=state, slotIndex=slot, isUndiscovered=false,
        useSpecialIcon=false, isMapLayerTransition=false, cost=cost, routes=routes }
end
local function hop(source, destination)
    return { sourceSlot=source, destinationSlot=destination }
end
"#;

const LINKED_PINS: &str = r#"
local names = {}
for pin in FlightMapFrame:EnumeratePinsByTemplate("FlightMap_FlightPointPinTemplate") do
    local linked = {}
    for other in pairs(pin.linkedPins) do
        linked[#linked + 1] = other.taxiNodeData.name
    end
    table.sort(linked)
    names[#names + 1] = pin.taxiNodeData.name .. ":" .. table.concat(linked, ",")
end
table.sort(names)
return table.concat(names, "|")
"#;

const SLOT_QUERIES: &str = r#"
return NumTaxiNodes(), TaxiNodeGetType(1), TaxiNodeGetType(3), TaxiNodeGetType(4),
    TaxiNodeName(3), TaxiNodeCost(3), GetNumRoutes(3), tostring(TaxiIsDirectFlight(3)),
    TaxiGetSrcX(3, 1), TaxiGetDestY(3, 1)
"#;

fn nodes_lua(third_routes: &str, third_cost: u32) -> String {
    format!(
        "{NODE_TEMPLATE}
        A_Admin.SetTaxiMapNodes({{
            node(1, 'Alpha', Enum.FlightPathState.Current, 0.2, 0.3, nil, nil),
            node(2, 'Beta', Enum.FlightPathState.Reachable, 0.4, 0.5, 15, {{ hop(1, 2) }}),
            node(3, 'Gamma', Enum.FlightPathState.Reachable, 0.6, 0.7, {third_cost}, {third_routes}),
        }})"
    )
}

fn open_flight_map(env: &WowLuaEnv, nodes: &str) -> String {
    env.exec(nodes).expect("taxi nodes should be accepted");
    env.exec("A_Admin.SetTaxiMapID(14); A_Admin.OpenTaxiMap(Enum.UIMapSystem.World)")
        .expect("TAXIMAP_OPENED should open the original FlightMapFrame");
    let linked = env.eval(LINKED_PINS).expect("FlightMap pins should be readable");
    env.exec("CloseTaxiMap()").expect("CloseTaxiMap should close the flight map");
    linked
}

type SlotQueries = (i32, String, String, String, String, i32, i32, String, f64, f64);

prefork_full_ui_case! {
fn flight_map_background_routes_follow_supplied_taxi_hops(env: &WowLuaEnv) {
    let direct = open_flight_map(env, &nodes_lua("{ hop(1, 3) }", 40));
    assert_eq!(direct, "Alpha:Beta,Gamma|Beta:Alpha|Gamma:Alpha");
    let direct_slots: SlotQueries = env.eval(SLOT_QUERIES).expect("slot queries");
    assert_eq!(
        direct_slots,
        (3, "CURRENT".into(), "REACHABLE".into(), "NONE".into(), "Gamma".into(), 40, 1,
         "true".into(), 0.2, 0.7)
    );

    let chained = open_flight_map(env, &nodes_lua("{ hop(1, 2), hop(2, 3) }", 75));
    assert_eq!(chained, "Alpha:Beta|Beta:Alpha|Gamma:");
    let chained_slots: SlotQueries = env.eval(SLOT_QUERIES).expect("slot queries");
    assert_eq!(
        chained_slots,
        (3, "CURRENT".into(), "REACHABLE".into(), "NONE".into(), "Gamma".into(), 75, 2,
         "false".into(), 0.2, 0.5)
    );
}
}
