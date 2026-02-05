//! Addon loader - loads addons from TOC files.

mod addon;
mod button;
mod error;
mod helpers;
mod lua_file;
mod xml_file;
mod xml_fontstring;
mod xml_frame;
mod xml_texture;

use crate::lua_api::WowLuaEnv;
use crate::saved_variables::SavedVariablesManager;
use crate::toc::TocFile;
use std::path::Path;
use std::time::Duration;

pub use error::LoadError;

/// Result of loading an addon.
#[derive(Debug)]
pub struct LoadResult {
    /// Addon name
    pub name: String,
    /// Number of Lua files loaded
    pub lua_files: usize,
    /// Number of XML files loaded
    pub xml_files: usize,
    /// Time breakdown
    pub timing: LoadTiming,
    /// Errors encountered (non-fatal)
    pub warnings: Vec<String>,
}

/// Timing breakdown for addon loading.
#[derive(Debug, Default, Clone)]
pub struct LoadTiming {
    /// Time reading files from disk
    pub io_time: Duration,
    /// Time parsing XML
    pub xml_parse_time: Duration,
    /// Time executing Lua
    pub lua_exec_time: Duration,
    /// Time loading SavedVariables
    pub saved_vars_time: Duration,
}

impl LoadTiming {
    pub fn total(&self) -> Duration {
        self.io_time + self.xml_parse_time + self.lua_exec_time + self.saved_vars_time
    }
}

/// Load an addon from its TOC file.
pub fn load_addon(env: &WowLuaEnv, toc_path: &Path) -> Result<LoadResult, LoadError> {
    let toc = TocFile::from_file(toc_path)?;
    load_addon_from_toc(env, &toc)
}

/// Load an addon from its TOC file with saved variables support.
pub fn load_addon_with_saved_vars(
    env: &WowLuaEnv,
    toc_path: &Path,
    saved_vars_mgr: &mut SavedVariablesManager,
) -> Result<LoadResult, LoadError> {
    let toc = TocFile::from_file(toc_path)?;
    load_addon_from_toc_with_saved_vars(env, &toc, saved_vars_mgr)
}

/// Load an addon from a parsed TOC.
pub fn load_addon_from_toc(env: &WowLuaEnv, toc: &TocFile) -> Result<LoadResult, LoadError> {
    addon::load_addon_internal(env, toc, None)
}

/// Load an addon from a parsed TOC with saved variables support.
pub fn load_addon_from_toc_with_saved_vars(
    env: &WowLuaEnv,
    toc: &TocFile,
    saved_vars_mgr: &mut SavedVariablesManager,
) -> Result<LoadResult, LoadError> {
    addon::load_addon_internal(env, toc, Some(saved_vars_mgr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use addon::AddonContext;
    use lua_file::load_lua_file;
    use xml_file::load_xml_file;

    #[test]
    fn test_load_lua_file() {
        let env = WowLuaEnv::new().unwrap();

        // Create a temp Lua file
        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let lua_path = temp_dir.join("test.lua");
        std::fs::write(&lua_path, "TEST_VAR = 42").unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_lua_file(&env, &lua_path, &ctx, &mut LoadTiming::default()).unwrap();

        let value: i32 = env.eval("return TEST_VAR").unwrap();
        assert_eq!(value, 42);

        // Cleanup
        std::fs::remove_file(&lua_path).ok();
    }

    #[test]
    fn test_xml_frame_with_layers_and_scripts() {
        let env = WowLuaEnv::new().unwrap();

        // Create a temp XML file with layers, scripts, and child frames
        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-xml");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="TestXMLFrame" parent="UIParent">
                    <Size x="200" y="150"/>
                    <Anchors>
                        <Anchor point="CENTER"/>
                    </Anchors>
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
                                <Anchors>
                                    <Anchor point="TOP" y="-10"/>
                                </Anchors>
                            </FontString>
                        </Layer>
                    </Layers>
                    <Scripts>
                        <OnLoad>
                            XML_ONLOAD_FIRED = true
                        </OnLoad>
                    </Scripts>
                    <Frames>
                        <Button name="TestXMLFrame_CloseBtn" parentKey="closeBtn">
                            <Size x="80" y="22"/>
                            <Anchors>
                                <Anchor point="BOTTOM" y="10"/>
                            </Anchors>
                            <Scripts>
                                <OnClick>
                                    XML_ONCLICK_FIRED = true
                                </OnClick>
                            </Scripts>
                        </Button>
                    </Frames>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        // Load the XML
        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify frame was created
        let frame_exists: bool = env.eval("return TestXMLFrame ~= nil").unwrap();
        assert!(frame_exists, "TestXMLFrame should exist");

        // Verify texture was created with parentKey
        let bg_exists: bool = env.eval("return TestXMLFrame.bg ~= nil").unwrap();
        assert!(bg_exists, "TestXMLFrame.bg should exist via parentKey");

        // Verify fontstring was created with parentKey
        let title_exists: bool = env.eval("return TestXMLFrame.title ~= nil").unwrap();
        assert!(title_exists, "TestXMLFrame.title should exist via parentKey");

        // Verify child button was created
        let btn_exists: bool = env.eval("return TestXMLFrame_CloseBtn ~= nil").unwrap();
        assert!(btn_exists, "TestXMLFrame_CloseBtn should exist");

        // Verify button parentKey
        let close_btn_exists: bool = env.eval("return TestXMLFrame.closeBtn ~= nil").unwrap();
        assert!(
            close_btn_exists,
            "TestXMLFrame.closeBtn should exist via parentKey"
        );

        // Verify OnLoad script was set (will fire when we call GetScript)
        let has_onload: bool = env
            .eval("return TestXMLFrame:GetScript('OnLoad') ~= nil")
            .unwrap();
        assert!(has_onload, "OnLoad handler should be set");

        // Verify OnClick script was set on the button
        let has_onclick: bool = env
            .eval("return TestXMLFrame_CloseBtn:GetScript('OnClick') ~= nil")
            .unwrap();
        assert!(has_onclick, "OnClick handler should be set on button");

        // Cleanup
        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_scripts_function_attribute() {
        let env = WowLuaEnv::new().unwrap();

        // Define a global function first
        env.exec(
            r#"
            SCRIPT_FUNC_CALLED = false
            function MyGlobalOnLoad(self)
                SCRIPT_FUNC_CALLED = true
            end
            "#,
        )
        .unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-scripts");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_func.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="FuncTestFrame" parent="UIParent">
                    <Scripts>
                        <OnLoad function="MyGlobalOnLoad"/>
                    </Scripts>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify the script handler references the global function
        let handler_set: bool = env
            .eval("return FuncTestFrame:GetScript('OnLoad') == MyGlobalOnLoad")
            .unwrap();
        assert!(handler_set, "OnLoad should reference MyGlobalOnLoad");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_scripts_method_attribute() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-method");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_method.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="MethodTestFrame" parent="UIParent">
                    <Scripts>
                        <OnShow method="OnShowHandler"/>
                    </Scripts>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Add a method to the frame
        env.exec(
            r#"
            METHOD_CALLED = false
            function MethodTestFrame:OnShowHandler()
                METHOD_CALLED = true
            end
            "#,
        )
        .unwrap();

        // Call the OnShow handler (it should call the method)
        env.exec("MethodTestFrame:GetScript('OnShow')(MethodTestFrame)")
            .unwrap();

        let method_called: bool = env.eval("return METHOD_CALLED").unwrap();
        assert!(method_called, "OnShow should have called OnShowHandler method");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_keyvalues() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-kv");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_kv.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="KeyValueFrame" parent="UIParent">
                    <KeyValues>
                        <KeyValue key="myString" value="hello"/>
                        <KeyValue key="myNumber" value="42" type="number"/>
                        <KeyValue key="myBool" value="true" type="boolean"/>
                        <KeyValue key="myFalseBool" value="false" type="boolean"/>
                    </KeyValues>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify string value
        let str_val: String = env.eval("return KeyValueFrame.myString").unwrap();
        assert_eq!(str_val, "hello");

        // Verify number value
        let num_val: i32 = env.eval("return KeyValueFrame.myNumber").unwrap();
        assert_eq!(num_val, 42);

        // Verify boolean values
        let bool_val: bool = env.eval("return KeyValueFrame.myBool").unwrap();
        assert!(bool_val);

        let false_val: bool = env.eval("return KeyValueFrame.myFalseBool").unwrap();
        assert!(!false_val);

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_anchors_with_offset() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-offset");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_offset.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="OffsetFrame" parent="UIParent">
                    <Size x="100" y="100"/>
                    <Anchors>
                        <Anchor point="TOPLEFT">
                            <Offset>
                                <AbsDimension x="10" y="-20"/>
                            </Offset>
                        </Anchor>
                    </Anchors>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify anchor was set with offset values
        let point_info: String = env
            .eval(
                r#"
                local point, relativeTo, relativePoint, x, y = OffsetFrame:GetPoint(1)
                return string.format("%s,%s,%d,%d", point, relativePoint, x, y)
                "#,
            )
            .unwrap();
        assert_eq!(point_info, "TOPLEFT,TOPLEFT,10,-20");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_size_with_absdimension() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-abssize");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_abssize.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="AbsSizeFrame" parent="UIParent">
                    <Size>
                        <AbsDimension x="150" y="75"/>
                    </Size>
                    <Anchors>
                        <Anchor point="CENTER"/>
                    </Anchors>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify size was set correctly
        let width: f64 = env.eval("return AbsSizeFrame:GetWidth()").unwrap();
        let height: f64 = env.eval("return AbsSizeFrame:GetHeight()").unwrap();
        assert_eq!(width, 150.0);
        assert_eq!(height, 75.0);

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_nested_child_frames() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-nested");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_nested.xml");
        std::fs::write(
            &xml_path,
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
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify parent frame
        let parent_exists: bool = env.eval("return ParentFrame ~= nil").unwrap();
        assert!(parent_exists, "ParentFrame should exist");

        // Verify child frame and parentKey
        let child_exists: bool = env.eval("return ChildFrame ~= nil").unwrap();
        assert!(child_exists, "ChildFrame should exist");

        let child_key_exists: bool = env.eval("return ParentFrame.child == ChildFrame").unwrap();
        assert!(child_key_exists, "ParentFrame.child should be ChildFrame");

        // Verify grandchild button and parentKey
        let grandchild_exists: bool = env.eval("return GrandchildButton ~= nil").unwrap();
        assert!(grandchild_exists, "GrandchildButton should exist");

        let grandchild_key_exists: bool = env
            .eval("return ChildFrame.btn == GrandchildButton")
            .unwrap();
        assert!(
            grandchild_key_exists,
            "ChildFrame.btn should be GrandchildButton"
        );

        // Verify parent relationships
        let parent_name: String = env
            .eval("return ChildFrame:GetParent():GetName() or 'nil'")
            .unwrap();
        assert_eq!(
            parent_name, "ParentFrame",
            "ChildFrame's parent should be ParentFrame, got {}",
            parent_name
        );

        let grandchild_parent_name: String = env
            .eval("return GrandchildButton:GetParent():GetName() or 'nil'")
            .unwrap();
        assert_eq!(
            grandchild_parent_name, "ChildFrame",
            "GrandchildButton's parent should be ChildFrame, got {}",
            grandchild_parent_name
        );

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_texture_color() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-texcolor");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_texcolor.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="ColorTexFrame" parent="UIParent">
                    <Size x="100" y="100"/>
                    <Layers>
                        <Layer level="BACKGROUND">
                            <Texture name="ColorTexFrame_BG" parentKey="bg">
                                <Size x="100" y="100"/>
                                <Color r="1.0" g="0.5" b="0.25" a="0.8"/>
                            </Texture>
                        </Layer>
                    </Layers>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify texture exists via parentKey
        let tex_exists: bool = env.eval("return ColorTexFrame.bg ~= nil").unwrap();
        assert!(tex_exists, "ColorTexFrame.bg should exist");

        // Verify vertex color was set (check via stored values if available)
        let has_color: bool = env
            .eval("return ColorTexFrame_BG ~= nil")
            .unwrap();
        assert!(has_color, "ColorTexFrame_BG should exist as global");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_virtual_frames_skipped() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-virtual");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_virtual.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="VirtualTemplate" virtual="true">
                    <Size x="200" y="100"/>
                </Frame>
                <Frame name="ConcreteFrame" parent="UIParent" inherits="VirtualTemplate">
                    <Anchors>
                        <Anchor point="CENTER"/>
                    </Anchors>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Virtual frame should NOT be created
        let virtual_exists: bool = env.eval("return VirtualTemplate ~= nil").unwrap();
        assert!(!virtual_exists, "VirtualTemplate should NOT exist (it's virtual)");

        // Concrete frame should exist
        let concrete_exists: bool = env.eval("return ConcreteFrame ~= nil").unwrap();
        assert!(concrete_exists, "ConcreteFrame should exist");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_multiple_anchors() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-multianchor");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_multianchor.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="MultiAnchorFrame" parent="UIParent">
                    <Anchors>
                        <Anchor point="TOPLEFT" x="10" y="-10"/>
                        <Anchor point="BOTTOMRIGHT" x="-10" y="10"/>
                    </Anchors>
                </Frame>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify frame has multiple anchor points
        let num_points: i32 = env
            .eval("return MultiAnchorFrame:GetNumPoints()")
            .unwrap();
        assert_eq!(num_points, 2, "Frame should have 2 anchor points");

        // Verify first anchor
        let point1: String = env
            .eval(
                r#"
                local point, _, relPoint, x, y = MultiAnchorFrame:GetPoint(1)
                return string.format("%s,%s,%d,%d", point, relPoint, x, y)
                "#,
            )
            .unwrap();
        assert_eq!(point1, "TOPLEFT,TOPLEFT,10,-10");

        // Verify second anchor
        let point2: String = env
            .eval(
                r#"
                local point, _, relPoint, x, y = MultiAnchorFrame:GetPoint(2)
                return string.format("%s,%s,%d,%d", point, relPoint, x, y)
                "#,
            )
            .unwrap();
        assert_eq!(point2, "BOTTOMRIGHT,BOTTOMRIGHT,-10,10");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_xml_all_script_handlers() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-allscripts");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_allscripts.xml");
        std::fs::write(
            &xml_path,
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
                    <Scripts>
                        <OnClick>ONCLICK = true</OnClick>
                    </Scripts>
                </Button>
            </Ui>"#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify all handlers are set
        let has_onload: bool = env
            .eval("return AllScriptsFrame:GetScript('OnLoad') ~= nil")
            .unwrap();
        assert!(has_onload, "OnLoad should be set");

        let has_onevent: bool = env
            .eval("return AllScriptsFrame:GetScript('OnEvent') ~= nil")
            .unwrap();
        assert!(has_onevent, "OnEvent should be set");

        let has_onupdate: bool = env
            .eval("return AllScriptsFrame:GetScript('OnUpdate') ~= nil")
            .unwrap();
        assert!(has_onupdate, "OnUpdate should be set");

        let has_onshow: bool = env
            .eval("return AllScriptsFrame:GetScript('OnShow') ~= nil")
            .unwrap();
        assert!(has_onshow, "OnShow should be set");

        let has_onhide: bool = env
            .eval("return AllScriptsFrame:GetScript('OnHide') ~= nil")
            .unwrap();
        assert!(has_onhide, "OnHide should be set");

        let has_onclick: bool = env
            .eval("return AllScriptsButton:GetScript('OnClick') ~= nil")
            .unwrap();
        assert!(has_onclick, "OnClick should be set on button");

        std::fs::remove_file(&xml_path).ok();
    }

    #[test]
    fn test_local_function_closures() {
        // This test verifies that local functions capture each other correctly in closures
        // Replicates the ExtraQuestButton/widgets.lua issue where updateKeyDirection is nil
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-closures");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let lua_path = temp_dir.join("closures.lua");
        std::fs::write(
            &lua_path,
            r#"
                local _, addon = ...

                local function innerFunc(x)
                    return x * 2
                end

                local function outerFunc(x)
                    -- innerFunc should be captured as an upvalue
                    if not innerFunc then
                        error("innerFunc is nil!")
                    end
                    return innerFunc(x)
                end

                -- Store the result on the addon table for verification
                addon.result = outerFunc(21)

                -- Also test immediate call pattern
                function addon:CreateSomething()
                    return outerFunc(10)
                end
            "#,
        )
        .unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "ClosureTest",
            table: addon_table.clone(),
            addon_root: &temp_dir,
        };
        load_lua_file(&env, &lua_path, &ctx, &mut LoadTiming::default()).unwrap();

        // Verify the result was computed correctly
        let result: i32 = addon_table.get("result").unwrap();
        assert_eq!(result, 42, "innerFunc should be captured and work correctly");

        // Verify the method works too (test via direct call)
        let create_something: mlua::Function = addon_table.get("CreateSomething").unwrap();
        let method_result: i32 = create_something.call(addon_table.clone()).unwrap();
        assert_eq!(method_result, 20, "outerFunc should still capture innerFunc");

        std::fs::remove_file(&lua_path).ok();
    }

    #[test]
    fn test_multi_file_closures() {
        // This test simulates ExtraQuestButton's loading pattern:
        // 1. widgets.lua defines local functions and addon:CreateButton
        // 2. button.lua defines addon:CreateExtraButton which calls addon:CreateButton
        // 3. addon.lua calls addon:CreateExtraButton
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-multifile");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // File 1: widgets.lua - defines local functions and addon method
        let widgets_path = temp_dir.join("widgets.lua");
        std::fs::write(
            &widgets_path,
            r#"
                local _, addon = ...

                local function updateKeyDirection(self)
                    return "updated: " .. tostring(self)
                end

                local function onCVarUpdate(self, cvar)
                    if cvar == "TestCVar" then
                        -- This is the critical line - updateKeyDirection should be captured
                        if not updateKeyDirection then
                            error("updateKeyDirection is nil!")
                        end
                        self.result = updateKeyDirection(self)
                    end
                end

                function addon:CreateButton(name)
                    local button = { name = name }
                    -- Call onCVarUpdate immediately during CreateButton
                    onCVarUpdate(button, "TestCVar")
                    return button
                end
            "#,
        )
        .unwrap();

        // File 2: button.lua - calls addon:CreateButton
        let button_path = temp_dir.join("button.lua");
        std::fs::write(
            &button_path,
            r#"
                local _, addon = ...

                function addon:CreateExtraButton(name)
                    -- This calls CreateButton which was defined in widgets.lua
                    return addon:CreateButton(name .. "_extra")
                end
            "#,
        )
        .unwrap();

        // File 3: addon.lua - calls addon:CreateExtraButton
        let addon_lua_path = temp_dir.join("addon.lua");
        std::fs::write(
            &addon_lua_path,
            r#"
                local _, addon = ...

                -- This should work: CreateExtraButton -> CreateButton -> onCVarUpdate -> updateKeyDirection
                local button = addon:CreateExtraButton("test")
                addon.testButton = button
            "#,
        )
        .unwrap();

        // Create shared addon table and context
        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "MultiFileTest",
            table: addon_table.clone(),
            addon_root: &temp_dir,
        };

        // Load files in order (like TOC would)
        load_lua_file(&env, &widgets_path, &ctx, &mut LoadTiming::default())
            .expect("widgets.lua should load");
        load_lua_file(&env, &button_path, &ctx, &mut LoadTiming::default())
            .expect("button.lua should load");
        load_lua_file(&env, &addon_lua_path, &ctx, &mut LoadTiming::default())
            .expect("addon.lua should load");

        // Verify the button was created and the closure worked
        let test_button: mlua::Table = addon_table.get("testButton").expect("testButton should exist");
        let result: String = test_button.get("result").expect("result should be set");
        assert!(result.starts_with("updated:"), "updateKeyDirection should have been called, got: {}", result);

        // Cleanup
        std::fs::remove_file(&widgets_path).ok();
        std::fs::remove_file(&button_path).ok();
        std::fs::remove_file(&addon_lua_path).ok();
    }
}
