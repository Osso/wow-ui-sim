use std::io::Write;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::xml::clear_templates;

fn create_test_addon(xml: &str, addon_name: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let toc_path = dir.path().join(format!("{addon_name}.toc"));
    let xml_path = dir.path().join(format!("{addon_name}.xml"));
    let mut toc = std::fs::File::create(&toc_path).unwrap();
    writeln!(toc, "## Title: {addon_name}").unwrap();
    writeln!(toc, "{}.xml", addon_name).unwrap();
    std::fs::write(xml_path, xml).unwrap();
    dir
}

#[test]
fn inherited_chat_glow_keeps_animation_on_its_texture() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let addon = create_test_addon(
        r#"<Ui>
        <AnimationGroup name="ChatFlashBase" virtual="true" looping="REPEAT">
            <Alpha order="1" duration="1" fromAlpha="0" toAlpha="1"/>
        </AnimationGroup>
        <Frame name="ChatGlowBase" virtual="true">
            <Layers><Layer level="BORDER">
                <Texture name="$parentGlow" parentKey="glow" hidden="true">
                    <Animations><AnimationGroup parentKey="FlashAnim" inherits="ChatFlashBase"/></Animations>
                </Texture>
            </Layer></Layers>
        </Frame>
        <Frame name="ChatGlowDerived" inherits="ChatGlowBase" virtual="true"/>
        <Frame name="ChatGlowInstance" inherits="ChatGlowDerived" parent="UIParent"/>
        </Ui>"#,
        "ChatGlowIdentity",
    );
    load_addon(&env.loader_env(), &addon.path().join("ChatGlowIdentity.toc")).unwrap();
    env.exec(r#"
        assert(ChatGlowInstance.glow == ChatGlowInstanceGlow)
        assert(ChatGlowInstance.glow.FlashAnim, "inherited glow lost FlashAnim")
        assert(ChatGlowInstance.glow.FlashAnim:GetParent() == ChatGlowInstance.glow)
        ChatGlowInstance.glow.FlashAnim:Play()
        assert(ChatGlowInstance.glow.FlashAnim:IsPlaying())
        ChatGlowInstance.glow.FlashAnim:Stop()
        assert(not ChatGlowInstance.glow.FlashAnim:IsPlaying())
    "#).unwrap();
}

#[test]
#[cfg(feature = "client-wowforever")]
fn forever_chat_xml_attaches_flash_groups_to_named_glows() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let xml = std::fs::read_to_string(cache.join("Blizzard_ChatFrameBase/Mainline/FloatingChatFrame.xml")).unwrap();
    // Keep the real virtual definitions, excluding unrelated concrete dock frames.
    let templates = xml.split("<!-- Main dock manager -->").next().unwrap();
    let addon = create_test_addon(&format!("{templates}</Ui>"), "ActualChatGlow");
    let animation_xml = std::fs::read_to_string(cache.join("Blizzard_SharedXML/AnimationTemplates.xml")).unwrap();
    std::fs::write(addon.path().join("AnimationTemplates.xml"), animation_xml).unwrap();
    std::fs::write(addon.path().join("ActualChatGlow.toc"), "AnimationTemplates.xml\nActualChatGlow.xml\n").unwrap();
    load_addon(&env.loader_env(), &addon.path().join("ActualChatGlow.toc")).unwrap();
    let compat = std::fs::read_to_string(cache.join("Blizzard_SharedXMLBase/Compat.lua")).unwrap();
    env.exec(&compat).unwrap();
    for file in ["Shared/ChatFrameConstants.lua", "Shared/ChatFrameUtil.lua"] {
        let source = std::fs::read_to_string(cache.join("Blizzard_ChatFrameBase").join(file)).unwrap();
        env.exec(&source).unwrap();
    }
    env.exec(r#"
        local tab = CreateFrame("Button", "ActualChatGlowTab", UIParent, "ChatTabArtTemplate")
        assert(tab.glow == ActualChatGlowTabGlow)
        assert(tab.glow.FlashAnim, "actual tab glow lost FlashAnim")
        local minimized = CreateFrame("Button", "ActualChatGlowMin", UIParent, "FloatingChatFrameMinimizedTemplate")
        assert(minimized.glow == ActualChatGlowMinGlow)
        assert(minimized.glow.FlashAnim, "actual minimized glow lost FlashAnim")
        for _, region in ipairs({tab.glow, minimized.glow}) do
            ChatFrameUtil.StartFlash(region, region.FlashAnim)
            assert(region.FlashAnim:IsPlaying())
            ChatFrameUtil.StopFlash(region, region.FlashAnim, false)
            assert(not region.FlashAnim:IsPlaying())
            assert(not region:IsShown())
        end
    "#).unwrap();
}

#[test]
fn xml_animation_group_onload_hides_target_textures() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        XmlAnimTargetMixin = {}
        function XmlAnimTargetMixin:Show()
            self:SetTargetsShown(true, self:GetAnimations())
        end
        function XmlAnimTargetMixin:Hide()
            self:SetTargetsShown(false, self:GetAnimations())
        end
        function XmlAnimTargetMixin:SetTargetsShown(shown, ...)
            for i = 1, select("#", ...) do
                local anim = select(i, ...)
                local target = anim and anim:GetTarget()
                if target and target.SetShown then
                    target:SetShown(shown)
                end
            end
        end
    "##,
    )
    .unwrap();

    let addon = create_test_addon(
        r#"<Ui>
        <AnimationGroup name="XmlAnimTargetTemplate" mixin="XmlAnimTargetMixin" virtual="true">
            <Scripts><OnLoad method="Hide"/></Scripts>
        </AnimationGroup>
        <Frame name="XmlAnimTargetFrame" parent="UIParent">
            <Layers>
                <Layer level="ARTWORK">
                    <Texture parentKey="Pulse" file="Interface\Icons\INV_Misc_QuestionMark" setAllPoints="true"/>
                </Layer>
            </Layers>
            <Animations>
                <AnimationGroup parentKey="PulseAnim" inherits="XmlAnimTargetTemplate">
                    <Alpha childKey="Pulse" order="1" fromAlpha="0" toAlpha="1" duration="1"/>
                </AnimationGroup>
            </Animations>
        </Frame>
    </Ui>"#,
        "XmlAnimTargetOnLoad",
    );

    load_addon(
        &env.loader_env(),
        &addon.path().join("XmlAnimTargetOnLoad.toc"),
    )
    .unwrap();

    let hidden: bool = env
        .eval("return XmlAnimTargetFrame.Pulse:IsShown() == false")
        .unwrap();
    assert!(
        hidden,
        "XML animation-group OnLoad should hide child targets before play"
    );
}
