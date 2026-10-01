//! Cached Blizzard template inheritance; input delivery and native parity are unverified.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use wow_ui_sim::loader::load_addon_from_toc;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::toc::TocFile;

fn load_insecure_propagator_templates() -> WowLuaEnv {
    // New environments clear the thread-local template registry before loading.
    let env = WowLuaEnv::new().expect("create isolated template environment");
    let xml = default_blizzard_ui_addons_path()
        .expect("active-profile Blizzard UI cache must be synced")
        .join("Blizzard_FrameXML/SecureTemplatesBase.xml");
    assert!(xml.is_file(), "missing cached fixture: {}", xml.display());
    let toc = TocFile::parse(
        xml.parent().expect("cached XML has an addon directory"),
        "## Title: Insecure propagator probe\nSecureTemplatesBase.xml\n",
    );
    let loaded =
        load_addon_from_toc(&env.loader_env(), &toc).expect("load cached SecureTemplatesBase.xml");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env
}

#[test]
fn insecure_propagator_templates_inherit_separate_mouse_masks() {
    let env = load_insecure_propagator_templates();
    env.exec(
        r#"
        local control = CreateFrame("Frame")
        assert(control:CanPropagateMouseMotion() == false)
        assert(control:CanPropagateMouseClicks() == false)
        local motion = CreateFrame("Frame", nil, nil, "InsecureMouseMotionPropagatorTemplate")
        assert(motion:CanPropagateMouseMotion() == true)
        assert(motion:CanPropagateMouseClicks() == false)
        local clicks = CreateFrame("Frame", nil, nil, "InsecureMouseClicksPropagatorTemplate")
        assert(clicks:CanPropagateMouseClicks() == true)
        assert(clicks:CanPropagateMouseMotion() == false)
        assert(control:CanPropagateMouseMotion() == false)
        assert(control:CanPropagateMouseClicks() == false)
        "#,
    )
    .expect("cached mouse templates inherit only their intended propagation mask");
}

#[test]
fn insecure_propagator_templates_inherit_keyboard_input_flag() {
    let env = load_insecure_propagator_templates();
    env.exec(
        r#"
        local control = CreateFrame("Frame")
        assert(control:GetPropagateKeyboardInput() == false)
        local keyboard = CreateFrame("Frame", nil, nil, "InsecureKeyboardInputPropagatorTemplate")
        assert(keyboard:GetPropagateKeyboardInput() == true)
        assert(control:GetPropagateKeyboardInput() == false)
        "#,
    )
    .expect("cached keyboard template inherits keyboard propagation");
}

#[test]
fn insecure_propagator_templates_inherit_hyperlink_parent_flag() {
    let env = load_insecure_propagator_templates();
    env.exec(
        r#"
        local control = CreateFrame("Frame")
        assert(control:DoesHyperlinkPropagateToParent() == false)
        local hyperlink = CreateFrame("Frame", nil, nil, "InsecureHyperlinkPropagatorTemplate")
        assert(hyperlink:DoesHyperlinkPropagateToParent() == true)
        assert(control:DoesHyperlinkPropagateToParent() == false)
        "#,
    )
    .expect("cached hyperlink template inherits propagation to parent");
}
