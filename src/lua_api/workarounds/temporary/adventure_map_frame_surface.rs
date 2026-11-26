//! Temporary AdventureMap frame-surface bootstrap repair.
//!
//! The AdventureMap panel is lazy-loaded by Blizzard UI, but simulator startup
//! and runtime addon loads can reach map-canvas paths before the frame surface,
//! border frame, inset pool, and providers are fully wired.

use crate::lua_api::{LoaderEnv, WowLuaEnv};

const ADVENTURE_MAP_FRAME_SURFACE_LUA: &str = r#"
local function __wow_seed_adventure_map_canvas_state(frame)
    frame.dataProviders = frame.dataProviders or {}
    frame.dataProviderEventsCount = frame.dataProviderEventsCount or {}
    frame.pinPools = frame.pinPools or {}
    frame.pinTemplateTypes = frame.pinTemplateTypes or {}
    frame.activeAreaTriggers = frame.activeAreaTriggers or {}
    frame.lockReasons = frame.lockReasons or {}
    frame.pinsToNudge = frame.pinsToNudge or {}
    frame.pinSuppressors = frame.pinSuppressors or {}

    if type(frame.pinFrameLevelsManager) ~= "table" then
        if type(CreateFromMixins) == "function" and type(MapCanvasPinFrameLevelsManagerMixin) == "table" then
            local ok, manager = pcall(CreateFromMixins, MapCanvasPinFrameLevelsManagerMixin)
            if ok then
                frame.pinFrameLevelsManager = manager
            end
        end

        frame.pinFrameLevelsManager = frame.pinFrameLevelsManager or {}
    end

    if type(frame.pinFrameLevelsManager.Initialize) == "function" then
        pcall(frame.pinFrameLevelsManager.Initialize, frame.pinFrameLevelsManager)
    end

    frame.pinFrameLevelsManager.definitions = frame.pinFrameLevelsManager.definitions or {}
end

local function __wow_seed_adventure_map_border_frame(frame)
    if type(frame) ~= "table" or type(CreateFrame) ~= "function" then
        return
    end

    if type(frame.BorderFrame) ~= "table" then
        frame.BorderFrame = CreateFrame("Frame", nil, frame)
    end

    local borderFrame = frame.BorderFrame
    if type(borderFrame.SetPortraitToAsset) ~= "function" then
        borderFrame.SetPortraitToAsset = function() end
    end
    if type(borderFrame.Underlay) ~= "table" then
        borderFrame.Underlay = CreateFrame("Frame", nil, borderFrame)
    end
    if type(borderFrame.TitleText) ~= "table" and type(borderFrame.CreateFontString) == "function" then
        borderFrame.TitleText = borderFrame:CreateFontString(nil, "ARTWORK")
    end
    if type(borderFrame.Bg) ~= "table" and type(borderFrame.CreateTexture) == "function" then
        borderFrame.Bg = borderFrame:CreateTexture(nil, "BACKGROUND")
    end
    if type(borderFrame.TopTileStreaks) ~= "table" and type(borderFrame.CreateTexture) == "function" then
        borderFrame.TopTileStreaks = borderFrame:CreateTexture(nil, "ARTWORK")
    end
end

local function __wow_adventure_map_has_provider(frame, mixin)
    if type(frame.dataProviders) ~= "table" or type(mixin) ~= "table" then
        return true
    end

    for provider in pairs(frame.dataProviders) do
        if provider.OnAdded == mixin.OnAdded then
            return true
        end
    end

    return false
end

local function __wow_add_adventure_map_provider(frame, mixin)
    if type(frame.AddDataProvider) ~= "function"
        or type(CreateFromMixins) ~= "function"
        or __wow_adventure_map_has_provider(frame, mixin)
    then
        return
    end

    local ok, provider = pcall(CreateFromMixins, mixin)
    if ok and type(provider) == "table" then
        pcall(frame.AddDataProvider, frame, provider)
    end
end

local function __wow_seed_adventure_map_inset_pool(frame)
    if type(frame) ~= "table"
        or frame.mapInsetPool ~= nil
        or type(CreateFramePool) ~= "function"
        or type(frame.GetCanvas) ~= "function"
        or type(frame.SetMapInsetPool) ~= "function"
    then
        return
    end

    local canvasOk, canvas = pcall(frame.GetCanvas, frame)
    if not canvasOk or type(canvas) ~= "table" then
        return
    end

    local function releaseMapInset(pool, mapInset)
        if type(mapInset) == "table" and type(mapInset.OnReleased) == "function" then
            mapInset:OnReleased()
        end
    end

    local poolOk, mapInsetPool = pcall(CreateFramePool, "FRAME", canvas, "AdventureMapInsetTemplate", releaseMapInset)
    if poolOk and type(mapInsetPool) == "table" then
        pcall(frame.SetMapInsetPool, frame, mapInsetPool)
    end
end

if type(AdventureMapFrame) ~= "table"
    and type(UIParent) == "table"
    and type(CreateFrame) == "function"
    and type(MapCanvasMixin) == "table"
then
    AdventureMapFrame = CreateFrame("Frame", "AdventureMapFrame", UIParent)
    AdventureMapFrame:SetFrameStrata("DIALOG")
    AdventureMapFrame:SetSize(1004, 689)
    __wow_seed_adventure_map_canvas_state(AdventureMapFrame)
    __wow_seed_adventure_map_border_frame(AdventureMapFrame)

    if type(Mixin) == "function" then
        pcall(Mixin, AdventureMapFrame, MapCanvasMixin)
        if type(AdventureMapMixin) == "table" then
            pcall(Mixin, AdventureMapFrame, AdventureMapMixin)
        end
    end

    local scrollContainer = CreateFrame("ScrollFrame", nil, AdventureMapFrame)
    scrollContainer.Child = CreateFrame("Frame", nil, scrollContainer)
    AdventureMapFrame.ScrollContainer = scrollContainer

    __wow_seed_adventure_map_canvas_state(AdventureMapFrame)
    __wow_seed_adventure_map_border_frame(AdventureMapFrame)
    __wow_seed_adventure_map_inset_pool(AdventureMapFrame)

    if type(AdventureMapFrame.RegisterEvent) == "function" then
        pcall(AdventureMapFrame.RegisterEvent, AdventureMapFrame, "ADVENTURE_MAP_UPDATE_INSETS")
    end

    __wow_add_adventure_map_provider(AdventureMapFrame, AdventureMap_QuestChoiceDataProviderMixin)
    __wow_add_adventure_map_provider(AdventureMapFrame, AdventureMap_QuestOfferDataProviderMixin)
    __wow_add_adventure_map_provider(AdventureMapFrame, QuestSessionDataProviderMixin)
end

if type(AdventureMapFrame) == "table" then
    __wow_seed_adventure_map_border_frame(AdventureMapFrame)
    __wow_seed_adventure_map_inset_pool(AdventureMapFrame)
end
"#;

pub(crate) fn patch(env: &WowLuaEnv) {
    let _ = env.exec(ADVENTURE_MAP_FRAME_SURFACE_LUA);
}

pub(crate) fn patch_loader(env: &LoaderEnv<'_>) {
    let _ = env.exec(ADVENTURE_MAP_FRAME_SURFACE_LUA);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_missing_adventure_map_frame_surface() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            MapCanvasMixin = {}
            AdventureMapMixin = {}
            AdventureMap_QuestChoiceDataProviderMixin = { name = "choice" }
            AdventureMap_QuestOfferDataProviderMixin = { name = "offer" }
            QuestSessionDataProviderMixin = { name = "session" }
            "#,
        )
        .expect("adventure-map fixture should install");

        patch(&env);

        let (
            has_frame,
            has_scroll_container,
            has_scroll_child,
            has_border_frame,
            has_underlay,
            has_pin_definitions,
        ): (bool, bool, bool, bool, bool, bool) = env
            .eval(
                r#"
                return type(AdventureMapFrame) == "table",
                    type(AdventureMapFrame.ScrollContainer) == "table",
                    type(AdventureMapFrame.ScrollContainer.Child) == "table",
                    type(AdventureMapFrame.BorderFrame) == "table",
                    type(AdventureMapFrame.BorderFrame.Underlay) == "table",
                    type(AdventureMapFrame.pinFrameLevelsManager.definitions) == "table"
                "#,
            )
            .expect("created adventure-map surface should be readable");

        assert!(has_frame);
        assert!(has_scroll_container);
        assert!(has_scroll_child);
        assert!(has_border_frame);
        assert!(has_underlay);
        assert!(has_pin_definitions);
    }

    #[test]
    fn preserves_existing_adventure_map_border_frame() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            existingBorderFrame = {
                SetPortraitToAsset = function() end,
                Underlay = {},
            }
            AdventureMapFrame = {
                BorderFrame = existingBorderFrame,
                pinFrameLevelsManager = { definitions = { existing = true } },
            }
            "#,
        )
        .expect("existing adventure-map fixture should install");

        patch(&env);

        let (same_border, kept_definitions): (bool, bool) = env
            .eval(
                r#"
                return AdventureMapFrame.BorderFrame == existingBorderFrame,
                    AdventureMapFrame.pinFrameLevelsManager.definitions.existing == true
                "#,
            )
            .expect("preserved adventure-map surface should be readable");

        assert!(same_border);
        assert!(kept_definitions);
    }
}
