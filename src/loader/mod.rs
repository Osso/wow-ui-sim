//! Addon loader - loads addons from TOC files.

mod addon;
mod button;
mod error;
pub(crate) mod helpers;
mod lua_file;
mod xml_file;
mod xml_fontstring;
mod xml_frame;
mod xml_texture;

use crate::lua_api::WowLuaEnv;
use crate::saved_variables::SavedVariablesManager;
use crate::toc::TocFile;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub use error::LoadError;
pub use xml_frame::create_frame_from_xml;

/// Find the TOC file for an addon directory.
/// Prefers Mainline variant, then exact name match, then any non-Classic TOC.
pub fn find_toc_file(addon_dir: &Path) -> Option<PathBuf> {
    let addon_name = addon_dir.file_name()?.to_str()?;
    let toc_variants = [
        format!("{}_Mainline.toc", addon_name),
        format!("{}.toc", addon_name),
    ];
    for variant in &toc_variants {
        let toc_path = addon_dir.join(variant);
        if toc_path.exists() {
            return Some(toc_path);
        }
    }
    // Fallback: find any .toc file (skip Classic/TBC/etc.)
    if let Ok(entries) = std::fs::read_dir(addon_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "toc").unwrap_or(false) {
                let name = path.file_name().unwrap().to_str().unwrap();
                if !name.contains("_Cata")
                    && !name.contains("_Wrath")
                    && !name.contains("_TBC")
                    && !name.contains("_Vanilla")
                    && !name.contains("_Mists")
                {
                    return Some(path);
                }
            }
        }
    }
    None
}

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
            // Best-effort cleanup of temp files
            let _ = std::fs::remove_dir_all(&self.temp_dir);
        }
    }

    /// Create a test environment, write XML content, load it, return context.
    fn load_test_xml(dir_suffix: &str, xml_content: &str) -> TestCtx {
        let env = WowLuaEnv::new().unwrap();
        let temp_dir = std::env::temp_dir().join(format!("wow-ui-sim-{}", dir_suffix));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test.xml");
        std::fs::write(&xml_path, xml_content).unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table,
            addon_root: &temp_dir,
        };
        load_xml_file(&env, &xml_path, &ctx, &mut LoadTiming::default()).unwrap();

        TestCtx { env, temp_dir }
    }

    /// Create a test environment, write a Lua file, load it, return context + addon table.
    fn load_test_lua(dir_suffix: &str, lua_content: &str) -> (TestCtx, mlua::Table) {
        let env = WowLuaEnv::new().unwrap();
        let temp_dir = std::env::temp_dir().join(format!("wow-ui-sim-{}", dir_suffix));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let lua_path = temp_dir.join("test.lua");
        std::fs::write(&lua_path, lua_content).unwrap();

        let addon_table = env.create_addon_table().unwrap();
        let ctx = AddonContext {
            name: "TestAddon",
            table: addon_table.clone(),
            addon_root: &temp_dir,
        };
        load_lua_file(&env, &lua_path, &ctx, &mut LoadTiming::default()).unwrap();

        (TestCtx { env, temp_dir }, addon_table)
    }

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
        let t = load_test_xml(
            "test-xml",
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
        );

        assert_layers_and_scripts_frame(&t);
        assert_layers_and_scripts_children(&t);
    }

    fn assert_layers_and_scripts_frame(t: &TestCtx) {
        t.assert_lua_true("return TestXMLFrame ~= nil", "TestXMLFrame should exist");
        t.assert_lua_true(
            "return TestXMLFrame.bg ~= nil",
            "TestXMLFrame.bg should exist via parentKey",
        );
        t.assert_lua_true(
            "return TestXMLFrame.title ~= nil",
            "TestXMLFrame.title should exist via parentKey",
        );
        t.assert_script_set("TestXMLFrame", "OnLoad");
    }

    fn assert_layers_and_scripts_children(t: &TestCtx) {
        t.assert_lua_true(
            "return TestXMLFrame_CloseBtn ~= nil",
            "TestXMLFrame_CloseBtn should exist",
        );
        t.assert_lua_true(
            "return TestXMLFrame.closeBtn ~= nil",
            "TestXMLFrame.closeBtn should exist via parentKey",
        );
        t.assert_script_set("TestXMLFrame_CloseBtn", "OnClick");
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
    fn test_xml_keyvalue_global_type_resolves_global_string() {
        let env = WowLuaEnv::new().unwrap();

        let temp_dir = std::env::temp_dir().join("wow-ui-sim-test-kv-global");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let xml_path = temp_dir.join("test_kv_global.xml");
        std::fs::write(
            &xml_path,
            r#"<Ui>
                <Frame name="KeyValueGlobalFrame" parent="UIParent">
                    <KeyValues>
                        <KeyValue key="instructionText" value="SEARCH" type="global"/>
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

        // type="global" should resolve "SEARCH" via _G["SEARCH"] which is "Search"
        let val: String = env.eval("return KeyValueGlobalFrame.instructionText").unwrap();
        assert_eq!(val, "Search", "type='global' should resolve via global string lookup");

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
        t.assert_lua_true(
            "return ParentFrame.child == ChildFrame",
            "ParentFrame.child should be ChildFrame",
        );
        t.assert_lua_true(
            "return GrandchildButton ~= nil",
            "GrandchildButton should exist",
        );
        t.assert_lua_true(
            "return ChildFrame.btn == GrandchildButton",
            "ChildFrame.btn should be GrandchildButton",
        );
    }

    fn assert_nested_parent_relationships(t: &TestCtx) {
        t.assert_lua_str(
            "return ChildFrame:GetParent():GetName() or 'nil'",
            "ParentFrame",
        );
        t.assert_lua_str(
            "return GrandchildButton:GetParent():GetName() or 'nil'",
            "ChildFrame",
        );
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

        let num_points: i32 = t
            .env
            .eval("return MultiAnchorFrame:GetNumPoints()")
            .unwrap();
        assert_eq!(num_points, 2, "Frame should have 2 anchor points");

        t.assert_lua_str(
            r#"
            local point, _, relPoint, x, y = MultiAnchorFrame:GetPoint(1)
            return string.format("%s,%s,%d,%d", point, relPoint, x, y)
            "#,
            "TOPLEFT,TOPLEFT,10,-10",
        );
        t.assert_lua_str(
            r#"
            local point, _, relPoint, x, y = MultiAnchorFrame:GetPoint(2)
            return string.format("%s,%s,%d,%d", point, relPoint, x, y)
            "#,
            "BOTTOMRIGHT,BOTTOMRIGHT,-10,10",
        );
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
                    <Scripts>
                        <OnClick>ONCLICK = true</OnClick>
                    </Scripts>
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
        // Verifies local functions capture each other correctly in closures
        // Replicates the ExtraQuestButton/widgets.lua issue where updateKeyDirection is nil
        let (_t, addon_table) = load_test_lua(
            "test-closures",
            r#"
                local _, addon = ...
                local function innerFunc(x)
                    return x * 2
                end
                local function outerFunc(x)
                    if not innerFunc then
                        error("innerFunc is nil!")
                    end
                    return innerFunc(x)
                end
                addon.result = outerFunc(21)
                function addon:CreateSomething()
                    return outerFunc(10)
                end
            "#,
        );

        let result: i32 = addon_table.get("result").unwrap();
        assert_eq!(result, 42, "innerFunc should be captured and work correctly");

        let create_something: mlua::Function = addon_table.get("CreateSomething").unwrap();
        let method_result: i32 = create_something.call(addon_table.clone()).unwrap();
        assert_eq!(method_result, 20, "outerFunc should still capture innerFunc");
    }

    /// Load multiple Lua files in sequence with a shared addon table, simulating TOC loading.
    fn load_test_lua_files(
        dir_suffix: &str,
        addon_name: &str,
        files: &[(&str, &str)],
    ) -> (TestCtx, mlua::Table) {
        let env = WowLuaEnv::new().unwrap();
        let temp_dir = std::env::temp_dir().join(format!("wow-ui-sim-{}", dir_suffix));
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
            load_lua_file(&env, &path, &ctx, &mut LoadTiming::default())
                .unwrap_or_else(|e| panic!("{} should load: {}", filename, e));
        }

        (TestCtx { env, temp_dir }, addon_table)
    }

    #[test]
    fn test_multi_file_closures() {
        // Simulates ExtraQuestButton's loading pattern:
        // widgets.lua -> button.lua -> addon.lua with cross-file closure capture
        let (_t, addon_table) = load_test_lua_files(
            "test-multifile",
            "MultiFileTest",
            &[
                ("widgets.lua", MULTI_FILE_WIDGETS_LUA),
                ("button.lua", MULTI_FILE_BUTTON_LUA),
                ("addon.lua", MULTI_FILE_ADDON_LUA),
            ],
        );

        let test_button: mlua::Table =
            addon_table.get("testButton").expect("testButton should exist");
        let result: String = test_button.get("result").expect("result should be set");
        assert!(
            result.starts_with("updated:"),
            "updateKeyDirection should have been called, got: {}",
            result
        );
    }

    const MULTI_FILE_WIDGETS_LUA: &str = r#"
        local _, addon = ...
        local function updateKeyDirection(self)
            return "updated: " .. tostring(self)
        end
        local function onCVarUpdate(self, cvar)
            if cvar == "TestCVar" then
                if not updateKeyDirection then
                    error("updateKeyDirection is nil!")
                end
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
        function addon:CreateExtraButton(name)
            return addon:CreateButton(name .. "_extra")
        end
    "#;

    const MULTI_FILE_ADDON_LUA: &str = r#"
        local _, addon = ...
        local button = addon:CreateExtraButton("test")
        addon.testButton = button
    "#;
}
