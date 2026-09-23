//! MovieFrame subtitle preference remains instance-local without movie playback.

use std::fs;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn movie_subtitles_follow_boolean_setter_without_starting_playback() {
    let directory = tempfile::tempdir().unwrap();
    let addon = directory.path().join("MovieSubtitleProbe");
    fs::create_dir(&addon).unwrap();
    fs::write(
        addon.join("MovieSubtitleProbe.xml"),
        r#"<Ui>
            <MovieFrame name="MovieSubtitleFirst" parent="UIParent" hidden="true"/>
            <MovieFrame name="MovieSubtitleSecond" parent="UIParent" hidden="true"/>
        </Ui>"#,
    )
    .unwrap();
    let toc = addon.join("MovieSubtitleProbe.toc");
    fs::write(
        &toc,
        "## Title: Movie subtitle probe\nMovieSubtitleProbe.xml\n",
    )
    .unwrap();

    let env = WowLuaEnv::new().unwrap();
    let loaded = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let ids = {
        let sim = env.state();
        let sim = sim.borrow();
        let first = sim.widgets.get_id_by_name("MovieSubtitleFirst").unwrap();
        let second = sim.widgets.get_id_by_name("MovieSubtitleSecond").unwrap();
        assert_eq!(
            sim.widgets.get(first).unwrap().object_type_name.as_deref(),
            Some("MovieFrame")
        );
        assert_eq!(
            sim.widgets.get(second).unwrap().object_type_name.as_deref(),
            Some("MovieFrame")
        );
        assert!(!sim.widgets.get(first).unwrap().movie_subtitles_enabled);
        assert!(!sim.widgets.get(second).unwrap().movie_subtitles_enabled);
        (first, second)
    };

    env.exec(
        r#"
        assert(MovieSubtitleFirst:EnableSubtitles(true) == nil)
        assert(not MovieSubtitleFirst:IsShown() and not MovieSubtitleSecond:IsShown())
        "#,
    )
    .unwrap();
    {
        let sim = env.state();
        let sim = sim.borrow();
        assert!(sim.widgets.get(ids.0).unwrap().movie_subtitles_enabled);
        assert!(!sim.widgets.get(ids.1).unwrap().movie_subtitles_enabled);
    }

    env.exec(
        r#"
        assert(MovieSubtitleSecond:EnableSubtitles(true) == nil)
        assert(MovieSubtitleFirst:EnableSubtitles(false) == nil)
        assert(not MovieSubtitleFirst:IsShown() and not MovieSubtitleSecond:IsShown())
        "#,
    )
    .unwrap();
    {
        let sim = env.state();
        let sim = sim.borrow();
        assert!(!sim.widgets.get(ids.0).unwrap().movie_subtitles_enabled);
        assert!(sim.widgets.get(ids.1).unwrap().movie_subtitles_enabled);
    }

    env.exec(
        r#"
        local bad = CreateFrame('Frame', 'MovieSubtitleOrdinary', UIParent)
        assert(not pcall(function() bad:EnableSubtitles(true) end))
        for _, invalid in ipairs({'true', 1, {}}) do
            assert(not pcall(function() MovieSubtitleFirst:EnableSubtitles(invalid) end))
        end
        assert(not pcall(function() MovieSubtitleFirst:EnableSubtitles(nil) end))
        assert(not pcall(function() MovieSubtitleFirst:EnableSubtitles() end))
        "#,
    )
    .unwrap();
    let sim = env.state();
    let sim = sim.borrow();
    assert!(!sim.widgets.get(ids.0).unwrap().movie_subtitles_enabled);
    assert!(sim.widgets.get(ids.1).unwrap().movie_subtitles_enabled);
    assert!(sim.lua_errors.is_empty(), "{:?}", sim.lua_errors);
}
