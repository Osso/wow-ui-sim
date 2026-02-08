//! Tests for the addon loader.

use super::*;
use super::addon::AddonContext;
use super::lua_file::load_lua_file;
use super::xml_file::load_xml_file;
use crate::lua_api::WowLuaEnv;

/// Test context holding environment and temp directory for cleanup.
struct TestCtx {
    env: WowLuaEnv,
    temp_dir: PathBuf,
}

impl TestCtx {
    /// Assert a Lua expression evaluates to true.
    fn assert_lua_true(&self, expr: &str, msg: &str) {
        let result: bool = self.env.eval(expr).unwrap();
        assert!(result, "{}", msg);
    }

    /// Assert a Lua expression returns the expected string.
    fn assert_lua_str(&self, expr: &str, expected: &str) {
        let result: String = self.env.eval(expr).unwrap();
        assert_eq!(result, expected);
    }

    /// Assert that a script handler is set on a frame.
    fn assert_script_set(&self, frame: &str, handler: &str) {
        let expr = format!("return {}:GetScript('{}') ~= nil", frame, handler);
        let msg = format!("{} should be set on {}", handler, frame);
        self.assert_lua_true(&expr, &msg);
    }
}

impl Drop for TestCtx {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

/// Create a test environment, write XML content, load it, return context.
fn load_test_xml(dir_suffix: &str, xml_content: &str) -> TestCtx {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join(format!("wow-sim-{}", dir_suffix));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let xml_path = temp_dir.join("test.xml");
    std::fs::write(&xml_path, xml_content).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext {
        name: "TestAddon",
        table: addon_table,
        addon_root: &temp_dir,
    };
    load_xml_file(&env.loader_env(), &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

    TestCtx { env, temp_dir }
}

/// Create a test environment, write a Lua file, load it, return context + addon table.
fn load_test_lua(dir_suffix: &str, lua_content: &str) -> (TestCtx, mlua::Table) {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join(format!("wow-sim-{}", dir_suffix));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let lua_path = temp_dir.join("test.lua");
    std::fs::write(&lua_path, lua_content).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext {
        name: "TestAddon",
        table: addon_table.clone(),
        addon_root: &temp_dir,
    };
    load_lua_file(&env.loader_env(), &lua_path, &ctx, &mut LoadTiming::default()).unwrap();

    (TestCtx { env, temp_dir }, addon_table)
}

#[test]
fn test_load_lua_file() {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let lua_path = temp_dir.join("test.lua");
    std::fs::write(&lua_path, "TEST_VAR = 42").unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext {
        name: "TestAddon",
        table: addon_table,
        addon_root: &temp_dir,
    };
    load_lua_file(&env.loader_env(), &lua_path, &ctx, &mut LoadTiming::default()).unwrap();

    let value: i32 = env.eval("return TEST_VAR").unwrap();
    assert_eq!(value, 42);
    std::fs::remove_file(&lua_path).ok();
}

#[test]
fn test_xml_frame_with_layers_and_scripts() {
    let t = load_test_xml(
        "test-xml",
        r#"<Ui>
            <Frame name="TestXMLFrame" parent="UIParent">
                <Size x="200" y="150"/>
                <Anchors><Anchor point="CENTER"/></Anchors>
                <Layers>
                    <Layer level="BACKGROUND">
                        <Texture name="TestXMLFrame_BG" parentKey="bg">
                            <Size x="200" y="150"/>
                            <Color r="0.1" g="0.1" b="0.1" a="0.8"/>
                            <Anchors>
                                <Anchor point="TOPLEFT"/>
                                <Anchor point="BOTTOMRIGHT"/>
                            </Anchors>
                        </Texture>
                    </Layer>
                    <Layer level="ARTWORK">
                        <FontString name="TestXMLFrame_Title" parentKey="title" text="Test Title">
                            <Anchors><Anchor point="TOP" y="-10"/></Anchors>
                        </FontString>
                    </Layer>
                </Layers>
                <Scripts><OnLoad>XML_ONLOAD_FIRED = true</OnLoad></Scripts>
                <Frames>
                    <Button name="TestXMLFrame_CloseBtn" parentKey="closeBtn">
                        <Size x="80" y="22"/>
                        <Anchors><Anchor point="BOTTOM" y="10"/></Anchors>
                        <Scripts><OnClick>XML_ONCLICK_FIRED = true</OnClick></Scripts>
                    </Button>
                </Frames>
            </Frame>
        </Ui>"#,
    );

    assert_layers_and_scripts_frame(&t);
    assert_layers_and_scripts_children(&t);
}

fn assert_layers_and_scripts_frame(t: &TestCtx) {
    t.assert_lua_true("return TestXMLFrame ~= nil", "TestXMLFrame should exist");
    t.assert_lua_true("return TestXMLFrame.bg ~= nil", "bg should exist via parentKey");
    t.assert_lua_true("return TestXMLFrame.title ~= nil", "title should exist via parentKey");
    t.assert_script_set("TestXMLFrame", "OnLoad");
}

fn assert_layers_and_scripts_children(t: &TestCtx) {
    t.assert_lua_true("return TestXMLFrame_CloseBtn ~= nil", "CloseBtn should exist");
    t.assert_lua_true("return TestXMLFrame.closeBtn ~= nil", "closeBtn should exist via parentKey");
    t.assert_script_set("TestXMLFrame_CloseBtn", "OnClick");
}

#[test]
fn test_xml_scripts_function_attribute() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        SCRIPT_FUNC_CALLED = false
        function MyGlobalOnLoad(self) SCRIPT_FUNC_CALLED = true end
    "#).unwrap();

    let temp_dir = std::env::temp_dir().join("wow-sim-test-scripts");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let xml_path = temp_dir.join("test_func.xml");
    std::fs::write(&xml_path, r#"<Ui>
        <Frame name="FuncTestFrame" parent="UIParent">
            <Scripts><OnLoad function="MyGlobalOnLoad"/></Scripts>
        </Frame>
    </Ui>"#).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext { name: "TestAddon", table: addon_table, addon_root: &temp_dir };
    load_xml_file(&env.loader_env(), &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

    let handler_set: bool = env.eval("return FuncTestFrame:GetScript('OnLoad') == MyGlobalOnLoad").unwrap();
    assert!(handler_set, "OnLoad should reference MyGlobalOnLoad");
    std::fs::remove_file(&xml_path).ok();
}

#[test]
fn test_xml_scripts_method_attribute() {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-method");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let xml_path = temp_dir.join("test_method.xml");
    std::fs::write(&xml_path, r#"<Ui>
        <Frame name="MethodTestFrame" parent="UIParent">
            <Scripts><OnShow method="OnShowHandler"/></Scripts>
        </Frame>
    </Ui>"#).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext { name: "TestAddon", table: addon_table, addon_root: &temp_dir };
    load_xml_file(&env.loader_env(), &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

    env.exec(r#"
        METHOD_CALLED = false
        function MethodTestFrame:OnShowHandler() METHOD_CALLED = true end
    "#).unwrap();
    env.exec("MethodTestFrame:GetScript('OnShow')(MethodTestFrame)").unwrap();

    let method_called: bool = env.eval("return METHOD_CALLED").unwrap();
    assert!(method_called, "OnShow should have called OnShowHandler method");
    std::fs::remove_file(&xml_path).ok();
}

#[test]
fn test_xml_keyvalues() {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-kv");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let xml_path = temp_dir.join("test_kv.xml");
    std::fs::write(&xml_path, r#"<Ui>
        <Frame name="KeyValueFrame" parent="UIParent">
            <KeyValues>
                <KeyValue key="myString" value="hello"/>
                <KeyValue key="myNumber" value="42" type="number"/>
                <KeyValue key="myBool" value="true" type="boolean"/>
                <KeyValue key="myFalseBool" value="false" type="boolean"/>
            </KeyValues>
        </Frame>
    </Ui>"#).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext { name: "TestAddon", table: addon_table, addon_root: &temp_dir };
    load_xml_file(&env.loader_env(), &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

    assert_eq!(env.eval::<String>("return KeyValueFrame.myString").unwrap(), "hello");
    assert_eq!(env.eval::<i32>("return KeyValueFrame.myNumber").unwrap(), 42);
    assert!(env.eval::<bool>("return KeyValueFrame.myBool").unwrap());
    assert!(!env.eval::<bool>("return KeyValueFrame.myFalseBool").unwrap());
    std::fs::remove_file(&xml_path).ok();
}

#[test]
fn test_xml_keyvalue_global_type_resolves_global_string() {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-kv-global");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let xml_path = temp_dir.join("test_kv_global.xml");
    std::fs::write(&xml_path, r#"<Ui>
        <Frame name="KeyValueGlobalFrame" parent="UIParent">
            <KeyValues>
                <KeyValue key="instructionText" value="SEARCH" type="global"/>
            </KeyValues>
        </Frame>
    </Ui>"#).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext { name: "TestAddon", table: addon_table, addon_root: &temp_dir };
    load_xml_file(&env.loader_env(), &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

    let val: String = env.eval("return KeyValueGlobalFrame.instructionText").unwrap();
    assert_eq!(val, "Search", "type='global' should resolve via global string lookup");
    std::fs::remove_file(&xml_path).ok();
}

#[test]
fn test_xml_anchors_with_offset() {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-offset");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let xml_path = temp_dir.join("test_offset.xml");
    std::fs::write(&xml_path, r#"<Ui>
        <Frame name="OffsetFrame" parent="UIParent">
            <Size x="100" y="100"/>
            <Anchors>
                <Anchor point="TOPLEFT">
                    <Offset><AbsDimension x="10" y="-20"/></Offset>
                </Anchor>
            </Anchors>
        </Frame>
    </Ui>"#).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext { name: "TestAddon", table: addon_table, addon_root: &temp_dir };
    load_xml_file(&env.loader_env(), &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

    let point_info: String = env.eval(r#"
        local point, relativeTo, relativePoint, x, y = OffsetFrame:GetPoint(1)
        return string.format("%s,%s,%d,%d", point, relativePoint, x, y)
    "#).unwrap();
    assert_eq!(point_info, "TOPLEFT,TOPLEFT,10,-20");
    std::fs::remove_file(&xml_path).ok();
}

#[test]
fn test_xml_size_with_absdimension() {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-abssize");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let xml_path = temp_dir.join("test_abssize.xml");
    std::fs::write(&xml_path, r#"<Ui>
        <Frame name="AbsSizeFrame" parent="UIParent">
            <Size><AbsDimension x="150" y="75"/></Size>
            <Anchors><Anchor point="CENTER"/></Anchors>
        </Frame>
    </Ui>"#).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext { name: "TestAddon", table: addon_table, addon_root: &temp_dir };
    load_xml_file(&env.loader_env(), &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

    assert_eq!(env.eval::<f64>("return AbsSizeFrame:GetWidth()").unwrap(), 150.0);
    assert_eq!(env.eval::<f64>("return AbsSizeFrame:GetHeight()").unwrap(), 75.0);
    std::fs::remove_file(&xml_path).ok();
}

#[test]
fn test_xml_nested_child_frames() {
    let t = load_test_xml(
        "test-nested",
        r#"<Ui>
            <Frame name="ParentFrame" parent="UIParent">
                <Size x="300" y="200"/>
                <Frames>
                    <Frame name="ChildFrame" parentKey="child">
                        <Size x="100" y="50"/>
                        <Frames>
                            <Button name="GrandchildButton" parentKey="btn">
                                <Size x="80" y="22"/>
                            </Button>
                        </Frames>
                    </Frame>
                </Frames>
            </Frame>
        </Ui>"#,
    );

    assert_nested_frames_exist(&t);
    assert_nested_parent_relationships(&t);
}

fn assert_nested_frames_exist(t: &TestCtx) {
    t.assert_lua_true("return ParentFrame ~= nil", "ParentFrame should exist");
    t.assert_lua_true("return ChildFrame ~= nil", "ChildFrame should exist");
    t.assert_lua_true("return ParentFrame.child == ChildFrame", "child should be ChildFrame");
    t.assert_lua_true("return GrandchildButton ~= nil", "GrandchildButton should exist");
    t.assert_lua_true("return ChildFrame.btn == GrandchildButton", "btn should be GrandchildButton");
}

fn assert_nested_parent_relationships(t: &TestCtx) {
    t.assert_lua_str("return ChildFrame:GetParent():GetName() or 'nil'", "ParentFrame");
    t.assert_lua_str("return GrandchildButton:GetParent():GetName() or 'nil'", "ChildFrame");
}

#[test]
fn test_xml_texture_color() {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-texcolor");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let xml_path = temp_dir.join("test_texcolor.xml");
    std::fs::write(&xml_path, r#"<Ui>
        <Frame name="ColorTexFrame" parent="UIParent">
            <Size x="100" y="100"/>
            <Layers><Layer level="BACKGROUND">
                <Texture name="ColorTexFrame_BG" parentKey="bg">
                    <Size x="100" y="100"/>
                    <Color r="1.0" g="0.5" b="0.25" a="0.8"/>
                </Texture>
            </Layer></Layers>
        </Frame>
    </Ui>"#).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext { name: "TestAddon", table: addon_table, addon_root: &temp_dir };
    load_xml_file(&env.loader_env(), &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

    assert!(env.eval::<bool>("return ColorTexFrame.bg ~= nil").unwrap(), "bg should exist");
    assert!(env.eval::<bool>("return ColorTexFrame_BG ~= nil").unwrap(), "BG should exist as global");
    std::fs::remove_file(&xml_path).ok();
}

#[test]
fn test_xml_virtual_frames_skipped() {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join("wow-sim-test-virtual");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let xml_path = temp_dir.join("test_virtual.xml");
    std::fs::write(&xml_path, r#"<Ui>
        <Frame name="VirtualTemplate" virtual="true"><Size x="200" y="100"/></Frame>
        <Frame name="ConcreteFrame" parent="UIParent" inherits="VirtualTemplate">
            <Anchors><Anchor point="CENTER"/></Anchors>
        </Frame>
    </Ui>"#).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext { name: "TestAddon", table: addon_table, addon_root: &temp_dir };
    load_xml_file(&env.loader_env(), &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

    assert!(!env.eval::<bool>("return VirtualTemplate ~= nil").unwrap(), "VirtualTemplate should NOT exist");
    assert!(env.eval::<bool>("return ConcreteFrame ~= nil").unwrap(), "ConcreteFrame should exist");
    std::fs::remove_file(&xml_path).ok();
}

#[test]
fn test_xml_multiple_anchors() {
    let t = load_test_xml(
        "test-multianchor",
        r#"<Ui>
            <Frame name="MultiAnchorFrame" parent="UIParent">
                <Anchors>
                    <Anchor point="TOPLEFT" x="10" y="-10"/>
                    <Anchor point="BOTTOMRIGHT" x="-10" y="10"/>
                </Anchors>
            </Frame>
        </Ui>"#,
    );

    assert_eq!(t.env.eval::<i32>("return MultiAnchorFrame:GetNumPoints()").unwrap(), 2);
    t.assert_lua_str(r#"
        local point, _, relPoint, x, y = MultiAnchorFrame:GetPoint(1)
        return string.format("%s,%s,%d,%d", point, relPoint, x, y)
    "#, "TOPLEFT,TOPLEFT,10,-10");
    t.assert_lua_str(r#"
        local point, _, relPoint, x, y = MultiAnchorFrame:GetPoint(2)
        return string.format("%s,%s,%d,%d", point, relPoint, x, y)
    "#, "BOTTOMRIGHT,BOTTOMRIGHT,-10,10");
}

#[test]
fn test_xml_all_script_handlers() {
    let t = load_test_xml(
        "test-allscripts",
        r#"<Ui>
            <Frame name="AllScriptsFrame" parent="UIParent">
                <Scripts>
                    <OnLoad>ONLOAD = true</OnLoad>
                    <OnEvent>ONEVENT = true</OnEvent>
                    <OnUpdate>ONUPDATE = true</OnUpdate>
                    <OnShow>ONSHOW = true</OnShow>
                    <OnHide>ONHIDE = true</OnHide>
                </Scripts>
            </Frame>
            <Button name="AllScriptsButton" parent="UIParent">
                <Scripts><OnClick>ONCLICK = true</OnClick></Scripts>
            </Button>
        </Ui>"#,
    );

    for handler in &["OnLoad", "OnEvent", "OnUpdate", "OnShow", "OnHide"] {
        t.assert_script_set("AllScriptsFrame", handler);
    }
    t.assert_script_set("AllScriptsButton", "OnClick");
}

#[test]
fn test_local_function_closures() {
    let (_t, addon_table) = load_test_lua(
        "test-closures",
        r#"
            local _, addon = ...
            local function innerFunc(x) return x * 2 end
            local function outerFunc(x)
                if not innerFunc then error("innerFunc is nil!") end
                return innerFunc(x)
            end
            addon.result = outerFunc(21)
            function addon:CreateSomething() return outerFunc(10) end
        "#,
    );

    assert_eq!(addon_table.get::<i32>("result").unwrap(), 42);
    let create_something: mlua::Function = addon_table.get("CreateSomething").unwrap();
    assert_eq!(create_something.call::<i32>(addon_table.clone()).unwrap(), 20);
}

/// Load multiple Lua files in sequence with a shared addon table.
fn load_test_lua_files(
    dir_suffix: &str,
    addon_name: &str,
    files: &[(&str, &str)],
) -> (TestCtx, mlua::Table) {
    let env = WowLuaEnv::new().unwrap();
    let temp_dir = std::env::temp_dir().join(format!("wow-sim-{}", dir_suffix));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let addon_table = env.create_addon_table().unwrap();
    let ctx = AddonContext {
        name: addon_name,
        table: addon_table.clone(),
        addon_root: &temp_dir,
    };

    for (filename, content) in files {
        let path = temp_dir.join(filename);
        std::fs::write(&path, content).unwrap();
        load_lua_file(&env.loader_env(), &path, &ctx, &mut LoadTiming::default())
            .unwrap_or_else(|e| panic!("{} should load: {}", filename, e));
    }

    (TestCtx { env, temp_dir }, addon_table)
}

#[test]
fn test_multi_file_closures() {
    let (_t, addon_table) = load_test_lua_files(
        "test-multifile",
        "MultiFileTest",
        &[
            ("widgets.lua", MULTI_FILE_WIDGETS_LUA),
            ("button.lua", MULTI_FILE_BUTTON_LUA),
            ("addon.lua", MULTI_FILE_ADDON_LUA),
        ],
    );

    let test_button: mlua::Table = addon_table.get("testButton").expect("testButton should exist");
    let result: String = test_button.get("result").expect("result should be set");
    assert!(result.starts_with("updated:"), "updateKeyDirection should have been called, got: {}", result);
}

const MULTI_FILE_WIDGETS_LUA: &str = r#"
    local _, addon = ...
    local function updateKeyDirection(self) return "updated: " .. tostring(self) end
    local function onCVarUpdate(self, cvar)
        if cvar == "TestCVar" then
            if not updateKeyDirection then error("updateKeyDirection is nil!") end
            self.result = updateKeyDirection(self)
        end
    end
    function addon:CreateButton(name)
        local button = { name = name }
        onCVarUpdate(button, "TestCVar")
        return button
    end
"#;

const MULTI_FILE_BUTTON_LUA: &str = r#"
    local _, addon = ...
    function addon:CreateExtraButton(name) return addon:CreateButton(name .. "_extra") end
"#;

const MULTI_FILE_ADDON_LUA: &str = r#"
    local _, addon = ...
    local button = addon:CreateExtraButton("test")
    addon.testButton = button
"#;
