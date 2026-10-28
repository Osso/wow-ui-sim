//! PTR preferred-datacenter dropdown consumer.
#![cfg(feature = "client-ptr")]

#[path = "common/keybinding_panel_fixtures.rs"]
mod keybinding_panel_fixtures;

#[test]
fn user_defaults_batch2_datacenter_dropdown() {
    let env = keybinding_panel_fixtures::setup_env();
    env.exec(
        r#"
        local init = CommunitiesGuildPreferredPlaySettingsFrameMixin.InitializeDatacenterDropdown
        assert(debug.getinfo(init, 'S').source:find('GuildPreferredPlaySettings.lua', 1, true))
        local menu
        local dropdown = { SetupMenu = function(_, callback) menu = callback end }
        init({ DatacenterDropdown = dropdown })
        local entries = 0
        menu(dropdown, { CreateButton = function() entries = entries + 1 end })
        local ids, names = C_AccountServices.GetPreferredDatacenterLocalities()
        assert(type(ids) == 'table' and type(names) == 'table')
        assert(#ids == 0 and #names == 0 and entries == 0)
    "#,
    )
    .expect("original dropdown consumes the approved empty catalogs");
}
