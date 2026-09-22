#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn classic_expansion_comparison_preserves_distinct_current_level() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(GetClassicExpansionLevel() == 10)
        assert(GetExpansionLevel() == 0)
        assert(LE_EXPANSION_LEVEL_CURRENT == 0)
        for _, level in ipairs({4, 8, 9, 9.5}) do
            assert(ClassicExpansionAtMost(level) == false)
            assert(ClassicExpansionAtLeast(level) == true)
        end
        for _, level in ipairs({10, 10.5, 11}) do
            assert(ClassicExpansionAtMost(level) == true)
        end
        assert(ClassicExpansionAtLeast(11) == false)
        assert(select('#', ClassicExpansionAtMost(8)) == 1)
        assert(type(ClassicExpansionAtMost(8)) == 'boolean')
    "#,
    )
    .unwrap();
}

#[test]
fn classic_expansion_predicate_matches_bigwigs_forever_branch() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local isSpecBased = ClassicExpansionAtLeast(LE_EXPANSION_MISTS_OF_PANDARIA)
        assert(isSpecBased == true)
        local version = select(4, GetBuildInfo())
        if version > 16000 and version < 20000 then
            isSpecBased = false
        end
        assert(isSpecBased == false)
        assert(ClassicExpansionAtMost(LE_EXPANSION_SHADOWLANDS) == false)
        assert(ClassicExpansionAtMost(LE_EXPANSION_CATACLYSM) == false)
        local branch = ClassicExpansionAtMost(LE_EXPANSION_CATACLYSM) and 'pre-cata' or 'forever'
        assert(branch == 'forever')
    "#,
    )
    .unwrap();
}

#[test]
fn classic_expansion_at_most_retains_existing_bootstrap_and_rejects_bad_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(not pcall(ClassicExpansionAtMost))
        assert(not pcall(ClassicExpansionAtMost, nil))
        for _, value in ipairs({'8', true, {}, function() end}) do
            assert(not pcall(ClassicExpansionAtMost, value))
        end
        assert(ClassicExpansionAtMost(secretwrap(10.5)) == true)
        assert(not pcall(ClassicExpansionAtMost, secretwrap('8')))
        local function taintedCaller()
            local ok, err = pcall(ClassicExpansionAtMost, secretwrap(8))
            assert(not ok and err:find('untainted caller', 1, true))
            assert(ClassicExpansionAtMost(10) == true)
        end
        debug.setobjecttaint(taintedCaller, 'ClassicExpansionProbe')
        taintedCaller()
    "#,
    )
    .unwrap();
    env.loader_env().restore_post_cleanup_globals().unwrap();
    env.exec(
        "assert(GetClassicExpansionLevel() == 10 and ClassicExpansionAtLeast(8) and ClassicExpansionAtMost(10))",
    )
    .unwrap();
}
