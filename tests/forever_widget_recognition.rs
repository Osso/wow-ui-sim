//! Forever publication of the existing Region and slot-zero recognition policy.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_widget_recognition_returns_one_boolean_for_regions() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local query = rawget(C_Widget, 'IsWidget')
        assert(type(query) == 'function')
        local frame = CreateFrame('Frame')
        local texture = frame:CreateTexture()
        local text = frame:CreateFontString()
        for _, region in ipairs({frame, texture, text}) do
            assert(query(region) == true)
            assert(type(query(region)) == 'boolean')
            assert(select('#', query(region)) == 1)
            assert(query(region, false, {}) == true, 'extra arguments are ignored')
        end
        assert(query() == false and select('#', query()) == 1)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_widget_recognition_rejects_primitives_lookalikes_and_animation_objects() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local query = C_Widget.IsWidget
        local frame = CreateFrame('Frame')
        local group = frame:CreateAnimationGroup()
        local animation = group:CreateAnimation('Alpha')
        local path = group:CreateAnimation('Path')
        local point = path:CreateControlPoint()
        local lookalike = {
            GetObjectType = function() return 'Frame' end,
            IsObjectType = function() return true end,
            SetPoint = function() end,
            GetName = function() return 'PretendFrame' end,
        }
        for _, value in ipairs({false, true, 0, 42, '', 'Frame', function() end,
            {}, lookalike, group, animation, path, point}) do
            assert(query(value) == false)
            assert(type(query(value)) == 'boolean')
            assert(select('#', query(value)) == 1)
        end
        assert(query(nil) == false)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_widget_recognition_requires_slot_zero_and_recovers_after_restoration() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local query = C_Widget.IsWidget
        local frame = CreateFrame('Frame')
        local identity = rawget(frame, 0)
        assert(identity ~= nil and query(frame) == true)
        rawset(frame, 0, nil)
        assert(query(frame) == false, 'missing slot zero must not recover native backing')
        assert(C_Widget.IsFrameWidget(frame) == true, 'native backing remains present')
        for _, invalid in ipairs({false, 17, 'invalid', {}}) do
            rawset(frame, 0, invalid)
            assert(query(frame) == false, 'invalid slot zero must not recover native backing')
        end
        rawset(frame, 0, identity)
        assert(query(frame) == true, 'restored identity is recognized')
        "#,
    )
    .unwrap();
}

#[test]
fn forever_widget_recognition_survives_post_load_cleanup() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("WidgetRecognitionRegion = CreateFrame('Frame')")
        .unwrap();
    let probe = r#"
        local query = rawget(C_Widget, 'IsWidget')
        assert(type(query) == 'function')
        assert(query(WidgetRecognitionRegion) == true)
        assert(query({}) == false)
        assert(select('#', query(WidgetRecognitionRegion)) == 1)
    "#;
    env.exec(probe).unwrap();
    env.apply_post_load_workarounds();
    env.exec(probe)
        .expect("cleanup must preserve the modeled widget recognition surface");
}

#[test]
fn forever_widget_recognition_selected_moveany_consumer_distinguishes_real_regions() {
    use std::path::{Path, PathBuf};
    use wow_ui_sim::screen::ScreenKind;
    use wow_ui_sim::toc::TocFile;

    let Some(addon_dir) = std::env::var_os("MOVEANY_UI_PATH").map(PathBuf::from) else {
        eprintln!(
            "skipping installed MoveAny consumer: set MOVEANY_UI_PATH to the MoveAny addon directory"
        );
        return;
    };
    let toc_path = wow_ui_sim::loader::find_toc_file(&addon_dir)
        .expect("installed MoveAny must have a profile-selected TOC");
    let toc = TocFile::from_file(&toc_path).expect("parse installed MoveAny TOC");
    assert_eq!(
        toc_path.file_name().and_then(|name| name.to_str()),
        Some("MoveAny_Camelot.toc")
    );
    assert_eq!(
        toc.metadata.get("Interface").map(String::as_str),
        Some("16001")
    );
    assert!(toc.allows_screen(ScreenKind::Game));
    let settings_index = toc
        .files
        .iter()
        .position(|path| path == Path::new("settings.lua"))
        .expect("selected MoveAny TOC must include settings.lua");
    assert!(
        toc.file_screen_allows[settings_index]
            .as_deref()
            .is_none_or(|screen| screen.eq_ignore_ascii_case("game"))
    );

    let source = std::fs::read_to_string(addon_dir.join("settings.lua"))
        .expect("read installed selected settings.lua");
    let start = source
        .find("function MoveAny:IsValidFrame(frame)")
        .expect("selected settings.lua must define IsValidFrame");
    let end = start
        + source[start..]
            .find("\nend")
            .expect("original IsValidFrame method must have its top-level terminator")
        + 4;
    let method = &source[start..end];
    assert!(method.contains("C_Widget.IsWidget(frame)"));
    // Preserve the exact original method. Load only this independent definition,
    // with the addon-local table binding used by settings.lua; no API is replaced.
    let chunk = format!("local _, MoveAny = ...\n{method}");
    let mut separator = String::from("=");
    while chunk.contains(&format!("]{separator}]")) {
        separator.push('=');
    }
    let env = WowLuaEnv::new().unwrap();
    env.exec("MoveAnyWidgetConsumer = {};").unwrap();
    env.exec(&format!(
        "assert(loadstring([{separator}[{chunk}]{separator}], '@MoveAny/settings.lua'))('MoveAny', MoveAnyWidgetConsumer)"
    ))
    .expect("load unchanged IsValidFrame from the selected installed consumer");
    env.exec(
        r#"
        local consumer = MoveAnyWidgetConsumer
        local frame = CreateFrame('Frame')
        local fake = {GetObjectType = function() return 'Frame' end,
            IsObjectType = function() return true end, SetPoint = function() end}
        assert(consumer:IsValidFrame(frame) == true, 'reject constant false')
        assert(consumer:IsValidFrame(frame:CreateTexture()) == true)
        assert(consumer:IsValidFrame(frame:CreateFontString()) == true)
        assert(consumer:IsValidFrame(fake) == false, 'reject constant true')
        assert(consumer:IsValidFrame(frame:CreateAnimationGroup()) == false)
        assert(consumer:IsValidFrame(nil) == false)
        local identity = rawget(frame, 0)
        rawset(frame, 0, nil)
        assert(consumer:IsValidFrame(frame) == false)
        rawset(frame, 0, identity)
        assert(consumer:IsValidFrame(frame) == true)
        "#,
    )
    .expect("unchanged MoveAny consumer must discriminate real Regions from lookalikes");
}
