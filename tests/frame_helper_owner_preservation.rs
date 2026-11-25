#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn game_probe(script: &str) {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    env.exec(script).expect("original owner probe");
}

#[test]
fn world_map_original_provider_lifecycle() {
    game_probe(
        r#"
        local map = assert(WorldMapFrame)
        assert(map.AddDataProvider == MapCanvasMixin.AddDataProvider)
        assert(map.RemoveDataProvider == MapCanvasMixin.RemoveDataProvider)
        local provider = CreateFromMixins(MapCanvasDataProviderMixin)
        map:AddDataProvider(provider)
        assert(map.dataProviders[provider] and provider:GetMap() == map)
        map:RemoveDataProvider(provider)
        assert(not map.dataProviders[provider] and provider:GetMap() == nil)
    "#,
    );
}

#[test]
fn objective_tracker_original_module_lifecycle() {
    game_probe(
        r#"
        local tracker = assert(ObjectiveTrackerFrame, "original container missing")
        assert(tracker.AddModule == ObjectiveTrackerContainerMixin.AddModule, "add owner")
        assert(tracker.RemoveModule == ObjectiveTrackerContainerMixin.RemoveModule, "remove owner")
        assert(tracker.HasModule == ObjectiveTrackerContainerMixin.HasModule, "membership owner")
        assert(tracker.RemoveAllModules == ObjectiveTrackerContainerMixin.RemoveAllModules, "clear owner")
        local module = assert(QuestObjectiveTracker, "original quest module missing")
        if tracker:HasModule(module) then tracker:RemoveModule(module) end
        local count = #tracker.modules
        tracker:AddModule(module)
        tracker:AddModule(module)
        assert(tracker:HasModule(module) and #tracker.modules == count + 1, "deduplicated add")
        tracker:RemoveModule(module)
        assert(not tracker:HasModule(module) and #tracker.modules == count, "remove membership")
        tracker:AddModule(module)
        tracker:RemoveAllModules()
        assert(not tracker:HasModule(module) and #tracker.modules == 0, "clear membership")
    "#,
    );
}

#[test]
fn titled_panel_original_title_widget() {
    game_probe(
        r#"
        local panel = CreateFrame("Frame", nil, UIParent, "DefaultPanelTemplate")
        assert(panel.SetTitle == TitledPanelMixin.SetTitle)
        panel:SetTitle("first frame helper proof")
        assert(panel:GetTitleText():GetText() == "first frame helper proof")
        panel:SetTitle("second frame helper proof")
        assert(panel:GetTitleText():GetText() == "second frame helper proof")
    "#,
    );
}

#[test]
fn layout_frame_original_dirty_transition() {
    game_probe(
        r#"
        local layout = CreateFrame("Frame", nil, UIParent, "VerticalLayoutFrame")
        assert(layout.MarkDirty == BaseLayoutMixin.MarkDirty)
        layout:MarkClean()
        assert(not layout:IsDirty())
        layout:MarkDirty()
        assert(layout:IsDirty())
        layout:MarkClean()
        assert(not layout:IsDirty())
    "#,
    );
}
