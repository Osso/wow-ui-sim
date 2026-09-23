//! Datamine-shaped nested XML lifecycle: child initialization precedes first show.

use std::fs;
use std::path::{Path, PathBuf};

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::xml::clear_templates;

fn write_addon(root: &Path, name: &str, lua: &str, xml: &str) -> PathBuf {
    let addon = root.join(name);
    fs::create_dir_all(&addon).unwrap();
    fs::write(addon.join("Lifecycle.lua"), lua).unwrap();
    fs::write(addon.join("Lifecycle.xml"), xml).unwrap();
    let toc = addon.join(format!("{name}.toc"));
    fs::write(
        &toc,
        "## Title: XML lifecycle probe\nLifecycle.lua\nLifecycle.xml\n",
    )
    .unwrap();
    toc
}

#[test]
fn virtual_workspace_child_onload_initializes_search_mode_before_first_show() {
    clear_templates();
    let root = tempfile::tempdir().unwrap();
    let toc = write_addon(
        root.path(),
        "DatamineExplorerLifecycle",
        r#"
        ExplorerLoadCount = 0
        ExplorerShowCount = 0
        ExplorerMixin = {}
        function ExplorerMixin:OnLoad()
            ExplorerLoadCount = ExplorerLoadCount + 1
            self.SearchMode = 1
        end
        ViewerMixin = {}
        function ViewerMixin:OnShow()
            ExplorerShowCount = ExplorerShowCount + 1
            assert(self:GetParent().SearchMode == 1, 'Explorer OnLoad must precede DataFrame OnShow')
        end
        "#,
        r#"<Ui>
            <Frame name="DatamineWorkspaceTemplate" virtual="true">
                <Frames>
                    <Frame parentKey="ExplorerTab" mixin="ExplorerMixin">
                        <Frames>
                            <Frame parentKey="DataFrame" mixin="ViewerMixin">
                                <Scripts><OnShow method="OnShow"/></Scripts>
                            </Frame>
                        </Frames>
                        <Scripts><OnLoad method="OnLoad"/></Scripts>
                    </Frame>
                </Frames>
            </Frame>
            <Frame name="DatamineLifecycleRoot" parent="UIParent" hidden="true">
                <Frames>
                    <Frame parentKey="Workspace" inherits="DatamineWorkspaceTemplate"/>
                </Frames>
            </Frame>
        </Ui>"#,
    );
    let env = WowLuaEnv::new().unwrap();
    let loaded = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(
        r#"
        local explorer = DatamineLifecycleRoot.Workspace.ExplorerTab
        assert(explorer and explorer.DataFrame, 'virtual workspace children missing')
        assert(ExplorerLoadCount == 1, 'Explorer OnLoad must run once during XML load')
        assert(explorer.SearchMode == 1, 'Explorer SearchMode was not initialized')
        DatamineLifecycleRoot:Show()
        assert(ExplorerShowCount == 1, 'DataFrame OnShow must run on first show')
        assert(explorer.SearchMode == 1, 'first show lost initialized SearchMode')
        "#,
    )
    .unwrap();
}

#[test]
fn nested_model_scene_runs_inherited_onload_before_prepended_custom_handler() {
    clear_templates();
    let root = tempfile::tempdir().unwrap();
    let toc = write_addon(
        root.path(),
        "DatamineSceneLifecycle",
        r#"
        SceneLoadOrder = {}
        BaseSceneMixin = {}
        function BaseSceneMixin:OnLoad()
            table.insert(SceneLoadOrder, 'base')
            self.tagToActor = {}
        end
        CustomSceneMixin = {}
        function CustomSceneMixin:OnLoad_Custom()
            table.insert(SceneLoadOrder, 'custom')
            assert(type(self.tagToActor) == 'table', 'base OnLoad must initialize actor map')
        end
        "#,
        r#"<Ui>
            <ModelScene name="NonInteractableModelSceneMixinTemplate"
                mixin="BaseSceneMixin" virtual="true">
                <Scripts><OnLoad method="OnLoad"/></Scripts>
            </ModelScene>
            <ModelScene name="ModelSceneMixinTemplate"
                inherits="NonInteractableModelSceneMixinTemplate" virtual="true"/>
            <ModelScene name="PanningModelSceneMixinTemplate"
                inherits="ModelSceneMixinTemplate" virtual="true"/>
            <ModelScene name="DatamineModelSceneTemplate"
                inherits="PanningModelSceneMixinTemplate"
                mixin="CustomSceneMixin" virtual="true">
                <Scripts><OnLoad method="OnLoad_Custom" inherit="prepend"/></Scripts>
            </ModelScene>
            <Frame name="DatamineSceneWorkspaceTemplate" virtual="true">
                <Frames>
                    <Frame parentKey="ModelViewTab">
                        <Frames>
                            <ModelScene parentKey="ModelScene" inherits="DatamineModelSceneTemplate"/>
                        </Frames>
                    </Frame>
                </Frames>
            </Frame>
            <Frame name="DatamineSceneRoot" parent="UIParent" hidden="true">
                <Frames>
                    <Frame parentKey="Workspace" inherits="DatamineSceneWorkspaceTemplate"/>
                </Frames>
            </Frame>
        </Ui>"#,
    );
    let env = WowLuaEnv::new().unwrap();
    let loaded = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(
        r#"
        local scene = DatamineSceneRoot.Workspace.ModelViewTab.ModelScene
        assert(scene, 'nested ModelScene missing')
        assert(type(scene.tagToActor) == 'table', 'inherited ModelScene OnLoad did not run')
        assert(#SceneLoadOrder == 2, 'ModelScene OnLoad should run exactly once per handler')
        assert(SceneLoadOrder[1] == 'base' and SceneLoadOrder[2] == 'custom',
            'inherited OnLoad must precede custom OnLoad')
        "#,
    )
    .unwrap();
}

#[test]
fn inherited_texture_instance_key_values_reach_parent_onload() {
    let root = tempfile::tempdir().unwrap();
    let toc = write_addon(
        root.path(),
        "DatamineAtlasLifecycle",
        r#"
        AtlasMixin = {}
        function AtlasMixin:ApplyAtlas()
            assert(self.FileName == 'Blizzard_UITools.blp', 'Missing FileName or FilePath')
            assert(self.AtlasName == 'uitools-icon-highlight')
            assert(self.TemplateFlag == true, 'inherited key lost')
            assert(self.Enabled == false, 'false override lost')
            self.Applied = true
        end
        ControlMixin = {}
        function ControlMixin:OnLoad()
            self.Button.Icon:ApplyAtlas()
            self.Initialized = true
        end
        "#,
        r#"<Ui>
            <Texture name="AtlasTextureTemplate" mixin="AtlasMixin" virtual="true">
                <KeyValues>
                    <KeyValue key="TemplateFlag" value="true" type="boolean"/>
                    <KeyValue key="Enabled" value="true" type="boolean"/>
                    <KeyValue key="FileName" value="wrong.blp"/>
                </KeyValues>
            </Texture>
            <Button name="AtlasButtonTemplate" virtual="true">
                <Layers><Layer level="OVERLAY">
                    <Texture parentKey="Icon" inherits="AtlasTextureTemplate">
                        <KeyValues>
                            <KeyValue key="FileName" value="Blizzard_UITools.blp"/>
                            <KeyValue key="AtlasName" value="uitools-icon-highlight"/>
                            <KeyValue key="Enabled" value="false" type="boolean"/>
                        </KeyValues>
                    </Texture>
                </Layer></Layers>
            </Button>
            <Frame name="AtlasControlTemplate" mixin="ControlMixin" virtual="true">
                <Frames><Button parentKey="Button" inherits="AtlasButtonTemplate"/></Frames>
                <Scripts><OnLoad method="OnLoad"/></Scripts>
            </Frame>
            <Frame name="AtlasRoot" parent="UIParent" hidden="true">
                <Frames><Frame parentKey="Controls" inherits="AtlasControlTemplate"/></Frames>
            </Frame>
        </Ui>"#,
    );
    let env = WowLuaEnv::new().unwrap();
    let loaded = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(
        r#"
        assert(AtlasRoot.Controls.Initialized, 'control initialization aborted')
        assert(AtlasRoot.Controls.Button.Icon.Applied, 'texture keyvalues did not reach consumer')
        "#,
    )
    .unwrap();
}

#[test]
fn texture_template_info_sizes_datamine_map_canvas() {
    let root = tempfile::tempdir().unwrap();
    let toc = write_addon(
        root.path(),
        "DatamineMapInfo",
        r#"
        MapCanvasMixin = {}
        function MapCanvasMixin:OnLoad()
            local info = C_XMLUtil.GetTemplateInfo('MapTileTemplate')
            self:SetSize(info.width * 64, info.height * 64)
            self.Initialized = true
        end
        "#,
        r#"<Ui>
            <Texture name="BaseMapTileTemplate" virtual="true">
                <Size x="32" y="64"/>
                <KeyValues><KeyValue key="TileRole" value="terrain"/></KeyValues>
            </Texture>
            <Texture name="MapTileTemplate" inherits="BaseMapTileTemplate" virtual="true">
                <Size x="64" y="64"/>
                <KeyValues><KeyValue key="TileIndex" value="0" type="number"/></KeyValues>
            </Texture>
            <Frame name="OrdinaryFrameTemplate" virtual="true"><Size x="90" y="45"/></Frame>
            <Frame name="MapCanvasProbe" mixin="MapCanvasMixin" parent="UIParent">
                <Scripts><OnLoad method="OnLoad"/></Scripts>
            </Frame>
        </Ui>"#,
    );
    let env = WowLuaEnv::new().unwrap();
    let loaded = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.exec(
        r#"
        local info = C_XMLUtil.GetTemplateInfo('MapTileTemplate')
        assert(info and info.type == 'Texture')
        assert(info.width == 64 and info.height == 64)
        local values = {}
        for _, entry in ipairs(info.keyValues) do values[entry.key] = entry end
        assert(values.TileRole.value == 'terrain')
        assert(values.TileIndex.value == '0' and values.TileIndex.type == 'number')
        assert(MapCanvasProbe.Initialized)
        assert(MapCanvasProbe:GetWidth() == 4096 and MapCanvasProbe:GetHeight() == 4096)
        info.width = 1
        assert(C_XMLUtil.GetTemplateInfo('MapTileTemplate').width == 64)
        local ordinary = C_XMLUtil.GetTemplateInfo('OrdinaryFrameTemplate')
        assert(ordinary.type == 'Frame' and ordinary.width == 90 and ordinary.height == 45)
        assert(C_XMLUtil.GetTemplateInfo('NeverRegisteredTemplate') == nil)
        "#,
    )
    .unwrap();
}
