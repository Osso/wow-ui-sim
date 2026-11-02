//! API-level coverage: no selected Retail FrameXML consumer was found.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn subzone_pvp_poi_reads_current_area_record_flags() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.pvp_area_inputs.area_flags.insert(101, 32768 | 4);
        sim.pvp_area_inputs.area_flags.insert(202, 4);
        sim.pvp_area_inputs.current_area_id = Some(101);
    }
    env.exec("assert(select('#', C_PvP.IsSubZonePVPPOI()) == 1); assert(C_PvP.IsSubZonePVPPOI())")
        .unwrap();
    env.state().borrow_mut().pvp_area_inputs.current_area_id = Some(202);
    env.exec("assert(not C_PvP.IsSubZonePVPPOI())").unwrap();
    env.state().borrow_mut().pvp_area_inputs.current_area_id = Some(303);
    env.exec("assert(C_PvP.IsSubZonePVPPOI() == false)")
        .unwrap();
    env.state().borrow_mut().pvp_area_inputs.current_area_id = None;
    env.exec("assert(C_PvP.IsSubZonePVPPOI() == false)")
        .unwrap();
}
