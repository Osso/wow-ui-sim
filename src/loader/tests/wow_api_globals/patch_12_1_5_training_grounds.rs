//! Synthetic catalog classifications; no native Training Grounds IDs are asserted.
use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
fn seed_training_catalog(env: &WowLuaEnv) {
    use crate::c_api::c_pvp::TrainingGroundKind;
    let mut state = env.state().borrow_mut();
    let template = state.lfd_dungeons[0].clone();
    for (id, kind) in [
        (900001, Some(TrainingGroundKind::Arena)),
        (900002, Some(TrainingGroundKind::Battleground)),
        (900003, None),
    ] {
        let mut entry = template.clone();
        entry.dungeon_id = id;
        entry.name = format!("Simulator training fixture {id}");
        entry.training_ground_kind = kind;
        state.lfd_dungeons.push(entry);
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn training_grounds_queries_catalog_classification_and_updates() {
    use crate::c_api::c_pvp::TrainingGroundKind;
    let env = WowLuaEnv::new().unwrap();
    let independent = WowLuaEnv::new().unwrap();
    seed_training_catalog(&env);
    for _ in 0..2 {
        env.exec(
            r#"
            for _, case in ipairs({{900001, true, false}, {900002, false, true},
                                   {900003, false, false}, {900004, false, false}}) do
                local id, arena, bg = unpack(case)
                assert(C_PvP.IsTrainingGroundsArena(id) == arena)
                assert(C_PvP.IsTrainingGroundsBG(id) == bg)
            end
            "#,
        )
        .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
    for kind in [Some(TrainingGroundKind::Battleground), None] {
        env.state()
            .borrow_mut()
            .lfd_dungeons
            .iter_mut()
            .find(|entry| entry.dungeon_id == 900001)
            .unwrap()
            .training_ground_kind = kind;
        let actual: (bool, bool) = env
            .eval("return C_PvP.IsTrainingGroundsArena(900001), C_PvP.IsTrainingGroundsBG(900001)")
            .unwrap();
        assert_eq!(actual, (false, kind.is_some()));
    }
    independent.exec(
        "assert(C_PvP.IsTrainingGroundsArena(900001) == false); assert(C_PvP.IsTrainingGroundsBG(900002) == false)"
    ).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn training_grounds_validates_ids_and_leaves_default_catalog_unclassified() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({"IsTrainingGroundsArena", "IsTrainingGroundsBG"}) do
            local query = rawget(C_PvP, name)
            assert(type(query) == "function", name .. " missing")
            assert(query(1201) == false)
            assert(query(987654) == false)
            assert(query(0) == false)
            assert(query(-1) == false)
            assert(select('#', query(1201)) == 1)
            for _, invalid in ipairs({true, {}, "1201", 1.5, math.huge, -math.huge,
                                      2147483648, -2147483649}) do
                assert(not pcall(query, invalid), name .. " accepted invalid ID")
            end
            assert(not pcall(query, 0 / 0))
            assert(not pcall(query))
        end
        assert(C_LFGInfo.IsLFGFollowerDungeon(1202))
        assert(C_PvP.CanSurrenderArena() == false)
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn training_grounds_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec(
            r#"
            for _, name in ipairs({"IsTrainingGroundsArena", "IsTrainingGroundsBG"}) do
                assert(rawget(C_PvP, name) == nil)
                assert(C_PvP[name] == nil)
                assert(rawget(C_PvP, name) == nil)
            end
            assert(C_PvP.CanSurrenderArena() == false)
            assert(C_LFGInfo.IsLFGFollowerDungeon(1202))
            "#,
        )
        .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
