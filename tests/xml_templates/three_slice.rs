use super::*;
// ============================================================================
// Three-Slice Button Tests
// ============================================================================

const THREE_SLICE_TEMPLATE_XML: &str = r#"<Ui>
    <Button name="ThreeSliceButtonTemplate" mixin="ThreeSliceButtonMixin" virtual="true">
        <Size x="20" y="20"/>
        <Layers><Layer level="BACKGROUND">
            <Texture parentKey="Left"><Anchors><Anchor point="TOPLEFT"/></Anchors></Texture>
            <Texture parentKey="Right"><Anchors><Anchor point="TOPRIGHT"/></Anchors></Texture>
            <Texture parentKey="Center">
                <Anchors>
                    <Anchor point="TOPLEFT" relativeKey="$parent.Left" relativePoint="TOPRIGHT"/>
                    <Anchor point="BOTTOMRIGHT" relativeKey="$parent.Right" relativePoint="BOTTOMLEFT"/>
                </Anchors>
            </Texture>
        </Layer></Layers>
        <Frames><Frame parentKey="Controller" mixin="ButtonControllerMixin">
            <Scripts><OnLoad method="OnLoad"/></Scripts>
        </Frame></Frames>
    </Button>
    <Button name="BigRedThreeSliceButtonTemplate" inherits="ThreeSliceButtonTemplate" virtual="true">
        <Size x="441" y="128"/>
        <KeyValues><KeyValue key="atlasName" value="128-RedButton" type="string"/></KeyValues>
    </Button>
    <Button name="SharedButtonSmallTemplate" inherits="BigRedThreeSliceButtonTemplate" virtual="true">
        <Size x="138" y="28"/>
    </Button>
</Ui>"#;

/// Set up env with three-slice templates and mixins registered.
fn setup_three_slice_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    register_three_slice_templates();
    install_three_slice_mixins(&env);
    env
}

fn register_three_slice_templates() {
    let ui = parse_xml(THREE_SLICE_TEMPLATE_XML).unwrap();
    for element in &ui.elements {
        register_three_slice_button_template(element);
    }
}

fn register_three_slice_button_template(element: &XmlElement) {
    let XmlElement::Button(frame) = element else {
        return;
    };
    let Some(name) = frame.name.as_deref() else {
        return;
    };
    register_template(name, "Button", frame.clone());
}

fn install_three_slice_mixins(env: &WowLuaEnv) {
    env.exec(THREE_SLICE_MIXIN_LUA).unwrap();
}

const THREE_SLICE_MIXIN_LUA: &str = r#"
    ThreeSliceButtonMixin = {}
    function ThreeSliceButtonMixin:InitButton()
        self.leftAtlasInfo = C_Texture.GetAtlasInfo(self.atlasName .. "-Left")
        self.rightAtlasInfo = C_Texture.GetAtlasInfo(self.atlasName .. "-Right")
        self:SetHighlightAtlas(self.atlasName .. "-Highlight")
    end
    function ThreeSliceButtonMixin:UpdateButton(buttonState)
        buttonState = buttonState or "NORMAL"
        self.Left:SetAtlas(self.atlasName .. "-Left", true)
        self.Center:SetAtlas("_" .. self.atlasName .. "-Center")
        self.Right:SetAtlas(self.atlasName .. "-Right", true)
        self:UpdateScale()
    end
    function ThreeSliceButtonMixin:UpdateScale()
        local scale = self:GetHeight() / self.leftAtlasInfo.height
        self.Left:SetScale(scale)
        self.Right:SetScale(scale)
        self.Left:SetTexCoord(0, 1, 0, 1)
        self.Left:SetWidth(self.leftAtlasInfo.width)
        self.Right:SetTexCoord(0, 1, 0, 1)
        self.Right:SetWidth(self.rightAtlasInfo.width)
    end
    ButtonControllerMixin = {}
    function ButtonControllerMixin:OnLoad()
        self:GetParent():InitButton()
    end
"#;

/// Three-slice InitButton runs via Controller:OnLoad after all templates applied.
#[test]
fn test_three_slice_button_texture_scaling() {
    let env = setup_three_slice_env();
    assert!(
        env.eval::<bool>("return C_Texture.GetAtlasInfo('128-RedButton-Left') ~= nil")
            .unwrap()
    );

    let result: String = env.eval(r#"
        local btn = CreateFrame("Button", "TestThreeSliceBtn", UIParent, "SharedButtonSmallTemplate")
        btn:SetSize(120, 22)
        if not btn.leftAtlasInfo then return "leftAtlasInfo nil" end
        if not btn.rightAtlasInfo then return "rightAtlasInfo nil" end
        return "ok"
    "#).unwrap();
    assert!(
        result.starts_with("ok"),
        "InitButton should have run: {result}"
    );
}

/// The three-slice template should end up with Left/Right/Center atlases set
/// after the real InitButton + UpdateButton lifecycle runs.
#[test]
fn test_three_slice_button_children_get_expected_atlases() {
    let env = setup_three_slice_env();
    let result: String = env
        .eval(
            r#"
        local btn = CreateFrame("Button", "TestThreeSliceAtlases", UIParent, "SharedButtonSmallTemplate")
        btn:SetSize(120, 22)
        btn:Show()
        btn:UpdateButton("NORMAL")

        local leftAtlas = btn.Left and btn.Left:GetAtlas() or ""
        local centerAtlas = btn.Center and btn.Center:GetAtlas() or ""
        local rightAtlas = btn.Right and btn.Right:GetAtlas() or ""
        return table.concat({ leftAtlas, centerAtlas, rightAtlas }, "|")
    "#,
        )
        .unwrap();

    assert_eq!(
        result, "128-RedButton-Left|_128-RedButton-Center|128-RedButton-Right",
        "Three-slice button should assign the expected Left/Center/Right atlases"
    );
}

/// Center texture gets non-zero width via cross-frame anchors to Left/Right siblings.
#[test]
fn test_three_slice_center_texture_layout() {
    let env = setup_three_slice_env();
    let result: String = env
        .eval(
            r#"
        local parent = CreateFrame("Frame", "TestThreeSliceParent", UIParent)
        parent:SetSize(600, 550)
        parent:SetPoint("CENTER", UIParent, "CENTER")

        local btn = CreateFrame("Button", "TestThreeSlice2", parent, "SharedButtonSmallTemplate")
        btn:SetSize(120, 22)
        btn:SetPoint("BOTTOMLEFT", parent, "BOTTOMLEFT", 4, 4)
        if not btn.Center then return "Center child missing" end
        if btn.Center:GetNumPoints() ~= 2 then
            return "Center has " .. btn.Center:GetNumPoints() .. " anchors, expected 2"
        end
        btn:UpdateButton()
        local leftW = btn.Left:GetWidth()
        local rightW = btn.Right:GetWidth()
        if leftW == 0 then return "Left width 0" end
        if rightW == 0 then return "Right width 0" end
        local centerW = btn.Center:GetWidth()
        if centerW == 0 then return "Center width 0 (cross-frame anchors not resolving)" end
        return "ok:" .. string.format("L=%.1f R=%.1f C=%.1f", leftW, rightW, centerW)
    "#,
        )
        .unwrap();
    assert!(
        result.starts_with("ok"),
        "Center texture should have non-zero width: {result}"
    );
}
