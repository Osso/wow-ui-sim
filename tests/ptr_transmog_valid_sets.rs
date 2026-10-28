#![cfg(feature = "client-ptr")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::c_api::c_transmog_sets::TransmogSetInfo;

#[test]
fn wardrobe_tabs_follow_valid_set_state() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_Transmog"], &[]);
    env.exec(
        r#"
        local info = debug.getinfo(TransmogWardrobeMixin.UpdateTabs, 'S')
        assert(info.source:gsub('\\', '/'):match('Blizzard_Transmog/Blizzard_Transmog.lua$'))
        probe = { itemsTabID=1, setsTabID=2, customSetsTabID=3, situationsTabID=4,
            TabHeaders = { SetTabShown = function(self, id, shown) self[id] = shown end } }
        TransmogWardrobeMixin.UpdateTabs(probe)
        assert(probe.TabHeaders[2] == false)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .transmog_sets
        .entries
        .push(TransmogSetInfo {
            set_id: 71,
            valid_for_character: true,
            ..Default::default()
        });
    env.exec(
        r#"
        TransmogWardrobeMixin.UpdateTabs(probe)
        assert(probe.TabHeaders[2] == true)
        assert(probe.TabHeaders[1] and probe.TabHeaders[3] and probe.TabHeaders[4])
    "#,
    )
    .unwrap();
}
