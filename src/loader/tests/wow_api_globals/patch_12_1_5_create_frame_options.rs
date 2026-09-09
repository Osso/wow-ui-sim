//! Options-constructor behavior; lifecycle/validation choices are simulator assumptions.
use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
fn options_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        OptionsTrace = {}
        OptionsFirst = { marker = "first" }
        OptionsSecond = { marker = "second" }
        function OptionsLoaded(self)
            table.insert(OptionsTrace, "load:" .. tostring(self:IsShown()) .. ":" .. tostring(self:IsForbidden()))
        end
        function OptionsShown(self)
            table.insert(OptionsTrace, "show:" .. self.marker)
        end
    "#).unwrap();
    let ui = crate::xml::parse_xml(
        r#"<Ui>
        <Frame name="OptionsFirstTemplate" virtual="true" mixin="OptionsFirst">
            <Scripts><OnLoad function="OptionsLoaded"/><OnShow function="OptionsShown"/></Scripts>
        </Frame>
        <Frame name="OptionsSecondTemplate" virtual="true" mixin="OptionsSecond"/>
        <Frame name="OptionsHiddenTemplate" virtual="true" hidden="true"/>
    </Ui>"#,
    )
    .unwrap();
    for element in ui.elements {
        if let crate::xml::XmlElement::Frame(frame) = element {
            let name = frame.name.clone().unwrap();
            crate::xml::register_template(&name, "Frame", frame);
        }
    }
    env
}

#[cfg(feature = "client-ptr")]
#[test]
fn create_frame_options_identity_and_types() {
    let env = options_env();
    env.exec(r#"
        assert(type(CreateFrameWithOptions) == "function")
        assert(CreateFrameOptions == nil)
        for _, kind in ipairs({"Frame", "Button", "CheckButton", "EditBox", "Slider", "Cooldown"}) do
            local f = CreateFrameWithOptions({frameType=kind})
            assert(f:GetObjectType() == kind)
            assert(f:GetParent() == nil)
            assert(f:GetName() == nil)
            assert(f:IsShown() and not f:IsForbidden())
        end
        local parent = CreateFrame("Frame", "OptionsParent")
        local child = CreateFrameWithOptions({frameType="Frame", name="$parentChild", parent=parent, id=37})
        assert(child:GetParent() == parent)
        assert(child:GetName() == "OptionsParentChild")
        assert(OptionsParentChild == child and child:GetID() == 37)
        local positional = CreateFrame("Button", "OptionsLegacy", parent, nil, 19)
        assert(positional:GetParent() == parent and positional:GetID() == 19)
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn create_frame_options_template_order_and_lifecycle() {
    let env = options_env();
    env.exec(
        r#"
        local f = CreateFrameWithOptions({frameType="Frame", parent=UIParent,
            inherits={"OptionsFirstTemplate", "OptionsSecondTemplate"}})
        assert(f.marker == "second")
        assert(table.concat(OptionsTrace, ",") == "load:true:false,show:second")
        OptionsTrace = {}
        local reversed = CreateFrameWithOptions({frameType="Frame", parent=UIParent,
            inherits={"OptionsSecondTemplate", "OptionsFirstTemplate"}})
        assert(reversed.marker == "first")
        assert(table.concat(OptionsTrace, ",") == "load:true:false,show:first")
        OptionsTrace = {}
        local hidden = CreateFrameWithOptions({frameType="Frame", parent=UIParent,
            inherits={"OptionsFirstTemplate"}, hidden=true, forbidden=true})
        assert(not hidden:IsShown() and hidden:IsForbidden())
        assert(table.concat(OptionsTrace, ",") == "load:false:true")
        hidden:Show()
        assert(table.concat(OptionsTrace, ",") == "load:false:true,show:first")
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn create_frame_options_explicit_flags_override_template_visibility() {
    let env = options_env();
    env.exec(
        r#"
        local f = CreateFrameWithOptions({frameType="Frame", hidden=false, forbidden=false,
            inherits={"OptionsHiddenTemplate"}})
        assert(f:IsShown() and not f:IsForbidden())
        local parent = CreateFrame("Frame")
        parent:Hide()
        OptionsTrace = {}
        local child = CreateFrameWithOptions({frameType="Frame", parent=parent,
            inherits={"OptionsFirstTemplate"}})
        assert(child:IsShown() and not child:IsVisible())
        assert(table.concat(OptionsTrace, ",") == "load:true:false")
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn create_frame_options_rejects_malformed_fields_before_creation() {
    let env = options_env();
    env.exec(r#"
        local cases = {
            false, {}, {frameType=12}, {frameType="UnknownWidget"},
            {frameType="Frame", name=7}, {frameType="Frame", parent={}},
            {frameType="Frame", inherits="OptionsFirstTemplate"},
            {frameType="Frame", inherits={"OptionsFirstTemplate", false}},
            {frameType="Frame", inherits={[2]="OptionsFirstTemplate"}},
            {frameType="Frame", inherits={"OptionsFirstTemplate,OptionsSecondTemplate"}},
            {frameType="Frame", hidden="yes"}, {frameType="Frame", forbidden=1},
            {frameType="Frame", id=1.5}, {frameType="Frame", id=0/0},
        }
        for _, options in ipairs(cases) do
            if type(options) == "table" and options.name == nil then options.name = "OptionsInvalid" end
            local ok, err = pcall(CreateFrameWithOptions, options)
            assert(not ok and type(err) == "string")
            assert(OptionsInvalid == nil)
        end
        assert(not pcall(CreateFrameWithOptions))
        local valid = CreateFrameWithOptions({frameType="Frame", name="OptionsAfterInvalid"})
        assert(valid:IsShown() and not valid:IsForbidden())
    "#).unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn create_frame_options_preserves_retail_absence_and_positional_api() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec(
            r#"
            assert(CreateFrameWithOptions == nil and CreateFrameOptions == nil)
            local parent = CreateFrame("Frame")
            local f = CreateFrame("Button", nil, parent, nil, 27)
            assert(f:GetObjectType() == "Button" and f:GetParent() == parent)
            assert(f:GetID() == 27 and f:IsShown() and not f:IsForbidden())
        "#,
        )
        .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
