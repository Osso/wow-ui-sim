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
        assert(control:GetPropagateKeyboardInput() == false, "plain Frame keyboard default remains false")
        local keyboard = CreateFrame("Frame", nil, nil, "InsecureKeyboardInputPropagatorTemplate")
        assert(keyboard:GetPropagateKeyboardInput() == true, "cached keyboard template declares propagation true")
        assert(control:GetPropagateKeyboardInput() == false, "plain Frame keyboard default remains false")
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
        assert(control:DoesHyperlinkPropagateToParent() == false, "plain Frame hyperlink default remains false")
        local hyperlink = CreateFrame("Frame", nil, nil, "InsecureHyperlinkPropagatorTemplate")
        assert(hyperlink:DoesHyperlinkPropagateToParent() == true, "cached hyperlink template declares propagation true")
        assert(control:DoesHyperlinkPropagateToParent() == false, "plain Frame hyperlink default remains false")
        "#,
    )
    .expect("cached hyperlink template inherits propagation to parent");
}

fn load_inline_propagator_xml(xml: &str) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create isolated inline XML environment");
    let directory = tempfile::tempdir().expect("create inline XML fixture directory");
    std::fs::write(directory.path().join("Propagation.xml"), xml)
        .expect("write inline propagation fixture");
    let toc = TocFile::parse(
        directory.path(),
        "## Title: Propagation probe\nPropagation.xml\n",
    );
    let loaded = load_addon_from_toc(&env.loader_env(), &toc).expect("load inline propagation XML");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env
}

const PROPAGATION_TEMPLATES: &str = r#"<Ui>
    <Frame name="PropagationTrue" virtual="true"
        propagateKeyboardInput="true" propagateHyperlinksToParent="true"/>
    <Frame name="PropagationFalse" inherits="PropagationTrue" virtual="true"
        propagateKeyboardInput="false" propagateHyperlinksToParent="false"/>
    <Frame name="PropagationChain" inherits="PropagationFalse" virtual="true"/>
    <Frame name="PropagationExplicitFalse" virtual="true"
        propagateKeyboardInput="false" propagateHyperlinksToParent="false"/>
    <Frame name="KeyboardOnly" virtual="true" propagateKeyboardInput="true"/>
    <Frame name="HyperlinkOnly" virtual="true" propagateHyperlinksToParent="true"/>
    <Frame name="PropagationOmitted" virtual="true"/>
</Ui>"#;

#[test]
fn insecure_propagator_templates_ordinary_xml_preserves_explicit_false() {
    let xml = PROPAGATION_TEMPLATES.replace(
        "</Ui>",
        r#"
        <Frame name="InlineTrue" propagateKeyboardInput="true" propagateHyperlinksToParent="true"/>
        <Frame name="InlineFalse" inherits="PropagationTrue"
            propagateKeyboardInput="false" propagateHyperlinksToParent="false"/>
        <Frame name="InlineChain" inherits="PropagationChain"/>
        <Frame name="InlineMultiple" inherits="KeyboardOnly, HyperlinkOnly, PropagationOmitted"/>
        <Frame name="InlineLastFalse" inherits="PropagationTrue, PropagationExplicitFalse"/>
        <Frame name="InlineLastTrue" inherits="PropagationExplicitFalse, PropagationTrue"/>
        <Frame name="InlineDefault"/>
        </Ui>"#,
    );
    let env = load_inline_propagator_xml(&xml);
    env.exec(
        r#"
        local function flags(frame, expected, label)
            assert(frame:GetPropagateKeyboardInput() == expected, label .. ": keyboard")
            assert(frame:DoesHyperlinkPropagateToParent() == expected, label .. ": hyperlink")
        end
        flags(InlineTrue, true, "explicit inline true")
        flags(InlineFalse, false, "instance false overrides inherited true")
        flags(InlineChain, false, "derived false survives omitted chain leaf")
        flags(InlineMultiple, true, "independent flags survive omitted template fields")
        flags(InlineLastFalse, false, "later false template wins")
        flags(InlineLastTrue, true, "later true template wins")
        flags(InlineDefault, false, "ordinary XML defaults")
        "#,
    )
    .expect("ordinary XML propagation getters preserve declarations and inheritance order");
}

#[test]
fn insecure_propagator_templates_runtime_chains_preserve_explicit_false() {
    let env = load_inline_propagator_xml(PROPAGATION_TEMPLATES);
    env.exec(
        r#"
        local function flags(templates, expected, label)
            local frame = CreateFrame("Frame", nil, nil, templates)
            assert(frame:GetPropagateKeyboardInput() == expected, label .. ": keyboard")
            assert(frame:DoesHyperlinkPropagateToParent() == expected, label .. ": hyperlink")
        end
        flags("PropagationTrue", true, "base true")
        flags("PropagationFalse", false, "derived false overrides base true")
        flags("PropagationChain", false, "omitted leaf retains derived false")
        flags("KeyboardOnly, HyperlinkOnly, PropagationOmitted", true, "independent flags and omissions")
        flags("PropagationTrue, PropagationExplicitFalse", false, "later false template wins")
        flags("PropagationExplicitFalse, PropagationTrue", true, "later true template wins")
        flags(nil, false, "runtime defaults")
        "#,
    )
    .expect("runtime CreateFrame propagation getters preserve explicit false and chain order");
}
