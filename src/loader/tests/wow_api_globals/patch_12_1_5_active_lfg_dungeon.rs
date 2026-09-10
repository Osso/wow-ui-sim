//! Instance-ID lookup policy; native inactive/error behavior remains unverified.
use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
fn active_name(env: &WowLuaEnv) -> String {
    env.eval("return C_LFGInfo.GetActiveLFGDungeonName()")
        .unwrap()
}

#[cfg(feature = "client-ptr")]
#[test]
fn active_lfg_dungeon_name_tracks_instance_and_catalog_changes() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.world.instance_lfg_dungeon_id = Some(1201);
        state.world.instance_name = "Not the catalog name".into();
    }
    assert_eq!(active_name(&env), "Ara-Kara, City of Echoes");
    {
        let mut state = env.state().borrow_mut();
        let dungeon = state
            .lfd_dungeons
            .iter_mut()
            .find(|dungeon| dungeon.dungeon_id == 1201)
            .unwrap();
        dungeon.name = "Updated Ara-Kara".into();
    }
    assert_eq!(active_name(&env), "Updated Ara-Kara");
    env.state().borrow_mut().world.instance_lfg_dungeon_id = Some(1202);
    assert_eq!(active_name(&env), "City of Threads");
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_eq!(active_name(&env), "City of Threads");
    env.state().borrow_mut().world.instance_lfg_dungeon_id = None;
    assert_eq!(active_name(&env), "");
}

#[cfg(feature = "client-ptr")]
#[test]
fn active_lfg_dungeon_name_ignores_proposal_without_instance_id() {
    let env = WowLuaEnv::new().unwrap();
    assert_eq!(active_name(&env), "");
    env.state().borrow_mut().lfg_active_proposal = Some(crate::lua_api::state::LfgProposalState {
        category: 2,
        dungeon_id: 1202,
    });
    assert_eq!(active_name(&env), "");
    env.exec("assert(select('#', C_LFGInfo.GetActiveLFGDungeonName()) == 1)")
        .unwrap();
    env.state().borrow_mut().world.instance_lfg_dungeon_id = Some(1201);
    assert_eq!(active_name(&env), "Ara-Kara, City of Echoes");
}

#[cfg(feature = "client-ptr")]
#[test]
fn active_lfg_dungeon_name_reports_unknown_instance_id() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().world.instance_lfg_dungeon_id = Some(987654);
    let (ok, error): (bool, String) = env
        .eval("return pcall(C_LFGInfo.GetActiveLFGDungeonName)")
        .unwrap();
    assert!(!ok);
    assert!(
        error.contains("C_LFGInfo.GetActiveLFGDungeonName"),
        "{error}"
    );
    assert!(error.contains("987654"), "{error}");
    assert!(error.contains("lfd_dungeons"), "{error}");
    assert_eq!(
        env.state().borrow().world.instance_lfg_dungeon_id,
        Some(987654)
    );
    env.state().borrow_mut().world.instance_lfg_dungeon_id = Some(1202);
    assert_eq!(active_name(&env), "City of Threads");
}

#[cfg(feature = "client-retail")]
#[test]
fn active_lfg_dungeon_name_remains_absent_on_retail() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec(
            r#"
            assert(rawget(C_LFGInfo, "GetActiveLFGDungeonName") == nil)
            assert(C_LFGInfo.GetActiveLFGDungeonName == nil)
            assert(rawget(C_LFGInfo, "GetActiveLFGDungeonName") == nil)
            assert(C_LFGInfo.CanPlayerUseLFD())
            assert(C_LFGInfo.IsLFGFollowerDungeon(1202))
            "#,
        )
        .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
