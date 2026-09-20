use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_ghost_frame_hides_when_graveyard_port_is_unavailable() {
    let env = WowLuaEnv::new().unwrap();
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let source = std::fs::read_to_string(root.join("Blizzard_FrameXML/GhostFrame.lua")).unwrap();
    env.exec(&source).unwrap();
    env.exec(
        r#"
        assert(CanPortGraveyard() == false)
        assert(select('#', CanPortGraveyard()) == 1)
        GhostFrame = CreateFrame('Frame')
        GhostFrame:Show()
        GhostFrameMixin.OnLoad(GhostFrame)
        assert(not GhostFrame:IsShown())
        assert(GhostFrame:IsEventRegistered('ADDON_LOADED'))
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_graveyard_availability_changes_without_crossing_environments() {
    let env = WowLuaEnv::new().unwrap();
    let other = WowLuaEnv::new().unwrap();
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let source = std::fs::read_to_string(root.join("Blizzard_FrameXML/GhostFrame.lua")).unwrap();
    env.exec(&source).unwrap();
    env.state().borrow_mut().player.can_port_graveyard = true;
    env.exec(
        r#"
        assert(CanPortGraveyard() == true)
        GhostFrame = CreateFrame('Frame')
        GhostFrame:Show()
        GhostFrameMixin.OnLoad(GhostFrame)
        assert(GhostFrame:IsShown())
        GhostFrame:Hide()
        GhostFrameMixin.OnLoad(GhostFrame)
        assert(not GhostFrame:IsShown())
    "#,
    )
    .unwrap();
    assert!(!other.eval::<bool>("return CanPortGraveyard()").unwrap());
    env.state().borrow_mut().player.can_port_graveyard = false;
    env.exec(
        r#"
        GhostFrame:Show()
        GhostFrameMixin.OnLoad(GhostFrame)
        assert(not GhostFrame:IsShown())
        assert(CanPortGraveyard() == false)
    "#,
    )
    .unwrap();
}

#[test]
#[cfg(not(feature = "client-wowforever"))]
fn graveyard_port_publication_does_not_change_other_profiles() {
    let env = WowLuaEnv::new().unwrap();
    assert!(env.eval::<bool>("return CanPortGraveyard == nil").unwrap());
}
