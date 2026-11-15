//! Temporary MapCanvas scroll-container bootstrap repair.
//!
//! Some MapCanvas startup paths run before the simulator has fully connected
//! the live frame, scroll container, canvas child, and map art sizing. Keep this
//! isolated until MapCanvas XML/runtime construction matches Blizzard state.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::LoaderEnv;
#[cfg(test)]
use crate::lua_api::WowLuaEnv;
use crate::lua_api::loader_env::GeneratedChunkArg;

const MAP_CANVAS_SCROLL_CONTAINER_WORKAROUND_LUA: &str = r#"
local options = ...
local preserve_original_world_map_owners = options and options[1] == "preserve_original_world_map_owners"

if type(ClearCachedActivitiesForPlayer) ~= "function" then
  function ClearCachedActivitiesForPlayer() end
end

local function __wow_find_first_scroll_frame_child(parent)
  if type(parent) ~= "table" or type(parent.GetNumChildren) ~= "function" or type(parent.GetChildren) ~= "function" then
    return nil
  end
  local count = parent:GetNumChildren()
  for index = 1, count do
    local child = select(index, parent:GetChildren())
    if type(child) == "table" then
      local isScrollFrame =
        (type(child.IsObjectType) == "function" and child:IsObjectType("ScrollFrame")) or
        (type(child.GetObjectType) == "function" and child:GetObjectType() == "ScrollFrame")
      if isScrollFrame then
        return child
      end
    end
  end
  return nil
end

local function __wow_ensure_map_canvas_scroll_container(frame)
  if type(frame) ~= "table" then
    return nil
  end

  local existing = rawget(frame, "ScrollContainer")
  if existing ~= nil then
    return existing
  end

  local scroll = __wow_find_first_scroll_frame_child(frame)
  if scroll ~= nil then
    rawset(frame, "ScrollContainer", scroll)
  end
  return scroll
end

local function __wow_ensure_map_canvas_scroll_size(frame, scroll)
  if type(frame) ~= "table" or type(scroll) ~= "table" or type(scroll.SetSize) ~= "function" then
    return
  end

  local scrollWidth = type(scroll.GetWidth) == "function" and scroll:GetWidth() or 0
  local scrollHeight = type(scroll.GetHeight) == "function" and scroll:GetHeight() or 0
  if scrollWidth > 0 and scrollHeight > 0 then
    return
  end

  local frameWidth = type(frame.GetWidth) == "function" and frame:GetWidth() or 0
  local frameHeight = type(frame.GetHeight) == "function" and frame:GetHeight() or 0
  if frameWidth <= 0 or frameHeight <= 0 then
    return
  end

  scroll:SetSize(frameWidth, frameHeight)
end

local function __wow_ensure_map_canvas_controller_size(scroll)
  if type(scroll) ~= "table" then
    return
  end

  local frame = nil
  if type(scroll.GetMap) == "function" then
    frame = scroll:GetMap()
  elseif type(scroll.GetParent) == "function" then
    frame = scroll:GetParent()
  end
  __wow_ensure_map_canvas_scroll_size(frame, scroll)
end

local function __wow_repair_zero_map_canvas_scale(scroll)
  if type(scroll) ~= "table" or type(scroll.zoomLevels) ~= "table" or type(scroll.zoomLevels[1]) ~= "table" then
    return
  end
  if scroll.zoomLevels[1].scale > 0 then
    return
  end

  __wow_ensure_map_canvas_controller_size(scroll)
  local alreadyRepairing = rawget(scroll, "__wow_repairing_zero_canvas_scale")
  if not alreadyRepairing and type(scroll.CreateZoomLevels) == "function" then
    rawset(scroll, "__wow_repairing_zero_canvas_scale", true)
    scroll.zoomLevels = nil
    scroll:CreateZoomLevels()
    rawset(scroll, "__wow_repairing_zero_canvas_scale", nil)
  end

  if type(scroll.zoomLevels) ~= "table" or type(scroll.zoomLevels[1]) ~= "table" or scroll.zoomLevels[1].scale <= 0 then
    scroll.baseScale = 1
    scroll.zoomLevels = { { scale = 1, layerIndex = 1 } }
  end

  if type(scroll.targetScale) ~= "number" or scroll.targetScale <= 0 then
    scroll.targetScale = scroll.zoomLevels[1].scale
  end
  if type(scroll.currentScale) == "number" and scroll.currentScale <= 0 then
    scroll.currentScale = nil
  end
end

local function __wow_patch_map_canvas_scroll_instance(scroll)
  if type(scroll) ~= "table" or rawget(scroll, "__wow_zero_canvas_scale_patched") then
    return
  end

  if type(scroll.CreateZoomLevels) == "function" then
    local originalCreateZoomLevels = scroll.CreateZoomLevels
    scroll.CreateZoomLevels = function(self, ...)
      __wow_ensure_map_canvas_controller_size(self)
      local result = originalCreateZoomLevels(self, ...)
      __wow_repair_zero_map_canvas_scale(self)
      return result
    end
  end

  if type(scroll.InstantPanAndZoom) == "function" then
    local originalInstantPanAndZoom = scroll.InstantPanAndZoom
    scroll.InstantPanAndZoom = function(self, scale, ...)
      __wow_ensure_map_canvas_controller_size(self)
      if type(scale) == "number" and scale <= 0 then
        __wow_repair_zero_map_canvas_scale(self)
        scale = type(self.targetScale) == "number" and self.targetScale > 0 and self.targetScale or 1
      end
      return originalInstantPanAndZoom(self, scale, ...)
    end
  end

  if type(scroll.ResetZoom) == "function" then
    scroll.ResetZoom = function(self)
      __wow_ensure_map_canvas_controller_size(self)
      __wow_repair_zero_map_canvas_scale(self)
      local scale = type(self.zoomLevels) == "table"
        and type(self.zoomLevels[1]) == "table"
        and self.zoomLevels[1].scale
        or 1
      if type(scale) ~= "number" or scale <= 0 then
        scale = 1
      end
      self:InstantPanAndZoom(scale, 0.5, 0.5)
    end
  end

  rawset(scroll, "__wow_zero_canvas_scale_patched", true)
end

local function __wow_try_init_map_canvas(frame)
  if type(frame) ~= "table" then
    return
  end

  __wow_ensure_map_canvas_scroll_container(frame)
  if rawget(frame, "__wow_map_canvas_onload_ran") then
    return
  end

  local scroll = rawget(frame, "ScrollContainer")
  if scroll == nil then
    return
  end

  __wow_patch_map_canvas_scroll_instance(scroll)
  __wow_ensure_map_canvas_scroll_size(frame, scroll)
  rawset(frame, "__wow_map_canvas_onload_ran", true)
  local originalOnLoad = rawget(_G, "__wow_map_canvas_original_onload")
  if type(originalOnLoad) == "function" then
    originalOnLoad(frame)
  end
end

local function __wow_refresh_map_canvas_size(frame)
  if type(frame) ~= "table" then
    return
  end

  local scroll = rawget(frame, "ScrollContainer")
  if type(scroll) ~= "table" then
    return
  end
  __wow_patch_map_canvas_scroll_instance(scroll)
  __wow_ensure_map_canvas_scroll_size(frame, scroll)

  local child = rawget(scroll, "Child")
  local childWidth = type(child) == "table" and type(child.GetWidth) == "function" and child:GetWidth() or 0
  local childHeight = type(child) == "table" and type(child.GetHeight) == "function" and child:GetHeight() or 0
  if childWidth ~= 0 and childHeight ~= 0 then
    return
  end

  local mapID = rawget(frame, "mapID")
  if (mapID == nil or mapID == 0) and type(frame.GetMapID) == "function" then
    mapID = frame:GetMapID()
  end

  if mapID ~= nil and mapID ~= 0 and type(scroll.SetMapID) == "function" then
    scroll:SetMapID(mapID)
    __wow_repair_zero_map_canvas_scale(scroll)
  elseif type(scroll.OnCanvasSizeChanged) == "function" then
    scroll:OnCanvasSizeChanged()
    __wow_repair_zero_map_canvas_scale(scroll)
  end

  childWidth = type(child) == "table" and type(child.GetWidth) == "function" and child:GetWidth() or 0
  childHeight = type(child) == "table" and type(child.GetHeight) == "function" and child:GetHeight() or 0
  if childWidth ~= 0 and childHeight ~= 0 then
    return
  end

  local layers = mapID ~= nil
    and mapID ~= 0
    and C_Map ~= nil
    and type(C_Map.GetMapArtLayers) == "function"
    and C_Map.GetMapArtLayers(mapID)
    or nil
  local layer = type(layers) == "table" and layers[1] or nil
  if type(child) ~= "table" or type(layer) ~= "table" then
    return
  end

  local layerWidth = layer.layerWidth or 0
  local layerHeight = layer.layerHeight or 0
  if layerWidth == 0 or layerHeight == 0 then
    return
  end

  if type(child.SetSize) == "function" then
    child:SetSize(layerWidth, layerHeight)
  end

  local tiledBackground = rawget(child, "TiledBackground")
  if type(tiledBackground) == "table" and type(tiledBackground.SetSize) == "function" then
    tiledBackground:SetSize(layerWidth * 2, layerHeight * 2)
  end

  if type(scroll.CalculateScaleExtents) == "function" then
    scroll:CalculateScaleExtents()
  end
  if type(scroll.CalculateScrollExtents) == "function" then
    scroll:CalculateScrollExtents()
  end
  if type(frame.OnCanvasSizeChanged) == "function" then
    frame:OnCanvasSizeChanged()
  end
end

local function __wow_patch_live_map_canvas(frame)
  if type(frame) ~= "table" or type(MapCanvasMixin) ~= "table" then
    return
  end

  if type(MapCanvasMixin.SetMapID) == "function" then
    frame.SetMapID = MapCanvasMixin.SetMapID
  end
  if type(MapCanvasMixin.GetCanvas) == "function" then
    frame.GetCanvas = MapCanvasMixin.GetCanvas
  end
  if type(MapCanvasMixin.GetCanvasContainer) == "function" then
    frame.GetCanvasContainer = MapCanvasMixin.GetCanvasContainer
  end
  if type(MapCanvasMixin.OnFrameSizeChanged) == "function" then
    frame.OnFrameSizeChanged = MapCanvasMixin.OnFrameSizeChanged
  end
  if type(MapCanvasMixin.OnShow) == "function" then
    frame.OnShow = MapCanvasMixin.OnShow
  end

  __wow_try_init_map_canvas(frame)
  __wow_refresh_map_canvas_size(frame)
end

local function __wow_patch_world_map_display_state(frame)
  if preserve_original_world_map_owners then
    return
  end
  if type(frame) ~= "table" or type(frame.SetDisplayState) ~= "function" then
    return
  end
  if rawget(frame, "__wow_display_state_refresh_patched") then
    return
  end

  local originalSetDisplayState = frame.SetDisplayState
  frame.SetDisplayState = function(self, ...)
    local result = originalSetDisplayState(self, ...)
    __wow_try_init_map_canvas(self)
    __wow_refresh_map_canvas_size(self)
    return result
  end

  rawset(frame, "__wow_display_state_refresh_patched", true)
end

local function __wow_ensure_map_canvas_zoom_levels(scroll)
  if type(scroll) ~= "table" or type(scroll.zoomLevels) == "table" then
    return
  end

  local mapID = rawget(scroll, "mapID")
  if (mapID == nil or mapID == 0) and type(scroll.GetMap) == "function" then
    local map = scroll:GetMap()
    if type(map) == "table" and type(map.GetMapID) == "function" then
      mapID = map:GetMapID()
    end
  end

  local layers = mapID ~= nil
    and mapID ~= 0
    and C_Map ~= nil
    and type(C_Map.GetMapArtLayers) == "function"
    and C_Map.GetMapArtLayers(mapID)
    or nil
  if type(layers) ~= "table" or type(layers[1]) ~= "table" then
    scroll.zoomLevels = { { scale = 1.0, layerIndex = 1 } }
    scroll.targetScale = scroll.targetScale or 1.0
    return
  end

  local zoomLevels = {}
  for index, layer in ipairs(layers) do
    zoomLevels[index] = {
      scale = layer.minScale or 1.0,
      layerIndex = index,
    }
  end
  scroll.zoomLevels = zoomLevels
  scroll.targetScale = scroll.targetScale or zoomLevels[1].scale or 1.0
end

rawset(_G, "__wow_ensure_map_canvas_zoom_levels", __wow_ensure_map_canvas_zoom_levels)

local function __wow_refresh_world_map_canvas()
  __wow_patch_live_map_canvas(WorldMapFrame)
  __wow_patch_world_map_display_state(WorldMapFrame)
end

if type(MapCanvasMixin) == "table" and not rawget(_G, "__wow_map_canvas_scroll_container_advanced_patched") then
  if rawget(_G, "__wow_map_canvas_original_onload") == nil and type(MapCanvasMixin.OnLoad) == "function" then
    _G.__wow_map_canvas_original_onload = MapCanvasMixin.OnLoad
    MapCanvasMixin.OnLoad = function(self, ...)
      if rawget(self, "__wow_map_canvas_onload_ran") then
        return
      end
      __wow_try_init_map_canvas(self)
    end
  end

  if type(MapCanvasMixin.SetMapID) == "function" then
    local originalSetMapID = MapCanvasMixin.SetMapID
    MapCanvasMixin.SetMapID = function(self, ...)
      __wow_try_init_map_canvas(self)
      if rawget(self, "ScrollContainer") == nil then
        local mapID = ...
        self.mapID = mapID
        if C_Map and type(C_Map.GetMapArtID) == "function" then
          self.mapArtID = C_Map.GetMapArtID(mapID)
        end
        return
      end
      local result = originalSetMapID(self, ...)
      __wow_refresh_map_canvas_size(self)
      return result
    end
  end

  if type(MapCanvasMixin.OnShow) == "function" then
    local originalOnShow = MapCanvasMixin.OnShow
    MapCanvasMixin.OnShow = function(self, ...)
      __wow_try_init_map_canvas(self)
      local result = originalOnShow(self, ...)
      __wow_refresh_map_canvas_size(self)
      return result
    end
  end

  if type(MapCanvasMixin.GetCanvas) == "function" then
    MapCanvasMixin.GetCanvas = function(self, ...)
      __wow_try_init_map_canvas(self)
      local scroll = rawget(self, "ScrollContainer")
      return scroll and scroll.Child or nil
    end
  end

  if type(MapCanvasMixin.GetCanvasContainer) == "function" then
    MapCanvasMixin.GetCanvasContainer = function(self, ...)
      __wow_try_init_map_canvas(self)
      return rawget(self, "ScrollContainer")
    end
  end

  if type(MapCanvasMixin.OnFrameSizeChanged) == "function" then
    local originalOnFrameSizeChanged = MapCanvasMixin.OnFrameSizeChanged
    MapCanvasMixin.OnFrameSizeChanged = function(self, ...)
      __wow_try_init_map_canvas(self)
      if rawget(self, "ScrollContainer") == nil then
        return
      end
      return originalOnFrameSizeChanged(self, ...)
    end
  end

  if type(MapCanvasScrollControllerMixin) == "table"
    and type(MapCanvasScrollControllerMixin.ResetZoom) == "function"
  then
    MapCanvasScrollControllerMixin.ResetZoom = function(self)
      __wow_ensure_map_canvas_controller_size(self)
      __wow_repair_zero_map_canvas_scale(self)
      local scale = type(self.zoomLevels) == "table"
        and type(self.zoomLevels[1]) == "table"
        and self.zoomLevels[1].scale
        or 1
      if type(scale) ~= "number" or scale <= 0 then
        scale = 1
      end
      self:InstantPanAndZoom(scale, 0.5, 0.5)
    end
  end

  if type(MapCanvasScrollControllerMixin) == "table"
    and type(MapCanvasScrollControllerMixin.CreateZoomLevels) == "function"
  then
    local originalCreateZoomLevels = MapCanvasScrollControllerMixin.CreateZoomLevels
    MapCanvasScrollControllerMixin.CreateZoomLevels = function(self, ...)
      __wow_ensure_map_canvas_controller_size(self)
      local result = originalCreateZoomLevels(self, ...)
      if type(self.zoomLevels) == "table" and type(self.zoomLevels[1]) == "table" and self.zoomLevels[1].scale <= 0 then
        self.baseScale = 1
        for _, zoomLevel in ipairs(self.zoomLevels) do
          if type(zoomLevel) == "table" and zoomLevel.scale <= 0 then
            zoomLevel.scale = 1
          end
        end
      end
      return result
    end
  end

  if type(MapCanvasScrollControllerMixin) == "table"
    and type(MapCanvasScrollControllerMixin.InstantPanAndZoom) == "function"
  then
    local originalInstantPanAndZoom = MapCanvasScrollControllerMixin.InstantPanAndZoom
    MapCanvasScrollControllerMixin.InstantPanAndZoom = function(self, scale, ...)
      __wow_ensure_map_canvas_controller_size(self)
      if type(scale) == "number" and scale <= 0 then
        scale = 1
      end
      return originalInstantPanAndZoom(self, scale, ...)
    end
  end

  if type(MapCanvasScrollControllerMixin) == "table"
    and type(MapCanvasScrollControllerMixin.GetZoomLevelIndexForScale) == "function"
  then
    local originalGetZoomLevelIndexForScale = MapCanvasScrollControllerMixin.GetZoomLevelIndexForScale
    MapCanvasScrollControllerMixin.GetZoomLevelIndexForScale = function(self, scale)
      __wow_ensure_map_canvas_zoom_levels(self)
      return originalGetZoomLevelIndexForScale(self, scale)
    end
  end

  if type(MapCanvasScrollControllerMixin) == "table"
    and type(MapCanvasScrollControllerMixin.GetCurrentLayerIndex) == "function"
  then
    local originalGetCurrentLayerIndex = MapCanvasScrollControllerMixin.GetCurrentLayerIndex
    MapCanvasScrollControllerMixin.GetCurrentLayerIndex = function(self, ...)
      __wow_ensure_map_canvas_zoom_levels(self)
      local zoomLevels = rawget(self, "zoomLevels")
      if type(zoomLevels) ~= "table" or type(zoomLevels[1]) ~= "table" then
        return 1
      end
      local ok, layerIndex = pcall(originalGetCurrentLayerIndex, self, ...)
      if ok and type(layerIndex) == "number" and layerIndex >= 1 then
        return layerIndex
      end
      return zoomLevels[1].layerIndex or 1
    end
  end

  rawset(_G, "__wow_map_canvas_scroll_container_advanced_patched", true)
  rawset(_G, "__wow_map_canvas_scroll_container_patched", true)
end

for _, mapName in ipairs({ "WorldMapFrame", "BattlefieldMapFrame" }) do
  __wow_patch_live_map_canvas(_G[mapName])
end
__wow_patch_world_map_display_state(WorldMapFrame)

if not preserve_original_world_map_owners
  and type(ToggleWorldMap) == "function"
  and not rawget(_G, "__wow_toggle_world_map_refresh_patched")
then
  local originalToggleWorldMap = ToggleWorldMap
  ToggleWorldMap = function(...)
    local result = originalToggleWorldMap(...)
    __wow_refresh_world_map_canvas()
    return result
  end

  if type(OpenWorldMap) == "function" then
    local originalOpenWorldMap = OpenWorldMap
    OpenWorldMap = function(...)
      local result = originalOpenWorldMap(...)
      __wow_refresh_world_map_canvas()
      return result
    end
  end

  rawset(_G, "__wow_toggle_world_map_refresh_patched", true)
end
    "#;

pub(crate) fn patch(env: &LoaderEnv<'_>) -> Result<(), crate::Error> {
    if ACTIVE == ClientProfile::Retail {
        // Retail MapCanvas loads before the WorldMap XML frame and its original
        // QuestLogOwnerMixin. Keep the original display-state and map verbs;
        // their consumers run after WorldMap owns them. Other repairs stay.
        env.exec_with_args(
            MAP_CANVAS_SCROLL_CONTAINER_WORKAROUND_LUA,
            &[GeneratedChunkArg::String(
                "preserve_original_world_map_owners".into(),
            )],
        )
    } else {
        env.exec(MAP_CANVAS_SCROLL_CONTAINER_WORKAROUND_LUA)
    }
}

#[cfg(test)]
pub(crate) fn patch_env(env: &WowLuaEnv) {
    patch(&env.loader_env()).expect("MapCanvas scroll-container workaround should install");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_scroll_container_before_running_map_canvas_onload_once() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            original_onload_calls = 0
            scroll = {
                Child = {},
                IsObjectType = function(self, objectType)
                    return objectType == "ScrollFrame"
                end,
            }
            frame = {
                GetNumChildren = function()
                    return 1
                end,
                GetChildren = function()
                    return scroll
                end,
            }
            MapCanvasMixin = {
                OnLoad = function(self)
                    original_onload_calls = original_onload_calls + 1
                    self.originalOnLoadSawScroll = self.ScrollContainer == scroll
                end,
            }
            "#,
        )
        .expect("map canvas onload fixture should install");

        patch_env(&env);

        let (onload_calls, has_scroll, onload_saw_scroll, onload_ran): (i64, bool, bool, bool) =
            env.eval(
                r#"
                MapCanvasMixin.OnLoad(frame)
                MapCanvasMixin.OnLoad(frame)
                return original_onload_calls,
                    frame.ScrollContainer == scroll,
                    frame.originalOnLoadSawScroll == true,
                    frame.__wow_map_canvas_onload_ran == true
                "#,
            )
            .expect("patched map canvas onload should run");

        assert_eq!(onload_calls, 1);
        assert!(has_scroll);
        assert!(onload_saw_scroll);
        assert!(onload_ran);
    }

    #[test]
    fn sizes_zero_canvas_from_map_art_layer_on_set_map_id() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            child = {
                TiledBackground = {
                    SetSize = function(self, width, height)
                        self.width = width
                        self.height = height
                    end,
                },
                GetWidth = function(self)
                    return self.width or 0
                end,
                GetHeight = function(self)
                    return self.height or 0
                end,
                SetSize = function(self, width, height)
                    self.width = width
                    self.height = height
                end,
            }
            scroll = {
                Child = child,
                CalculateScaleExtents = function(self)
                    self.calculatedScale = true
                end,
                CalculateScrollExtents = function(self)
                    self.calculatedScroll = true
                end,
            }
            frame = {
                ScrollContainer = scroll,
                OnCanvasSizeChanged = function(self)
                    self.canvasSizeChanged = true
                end,
            }
            C_Map = {
                GetMapArtLayers = function(mapID)
                    if mapID == 947 then
                        return { { layerWidth = 1000, layerHeight = 500 } }
                    end
                    return nil
                end,
            }
            MapCanvasMixin = {
                OnLoad = function() end,
                SetMapID = function(self, mapID)
                    self.mapID = mapID
                    return "map-set"
                end,
            }
            "#,
        )
        .expect("map canvas sizing fixture should install");

        patch_env(&env);

        let (
            result,
            child_width,
            child_height,
            background_width,
            background_height,
            calculated_scale,
            calculated_scroll,
            canvas_size_changed,
        ): (String, i64, i64, i64, i64, bool, bool, bool) = env
            .eval(
                r#"
                local result = MapCanvasMixin.SetMapID(frame, 947)
                return result,
                    child.width,
                    child.height,
                    child.TiledBackground.width,
                    child.TiledBackground.height,
                    scroll.calculatedScale == true,
                    scroll.calculatedScroll == true,
                    frame.canvasSizeChanged == true
                "#,
            )
            .expect("patched map canvas SetMapID should size the canvas");

        assert_eq!(result, "map-set");
        assert_eq!(child_width, 1000);
        assert_eq!(child_height, 500);
        assert_eq!(background_width, 2000);
        assert_eq!(background_height, 1000);
        assert!(calculated_scale);
        assert!(calculated_scroll);
        assert!(canvas_size_changed);
    }

    #[test]
    fn creates_zoom_levels_before_scroll_controller_queries() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_Map = {
                GetMapArtLayers = function(mapID)
                    if mapID == 947 then
                        return {
                            { minScale = 0.75 },
                            { minScale = 1.5 },
                        }
                    end
                    return nil
                end,
            }
            MapCanvasMixin = { OnLoad = function() end }
            MapCanvasScrollControllerMixin = {
                GetCurrentLayerIndex = function()
                    error("missing zoom levels")
                end,
            }
            scroll = { mapID = 947 }
            "#,
        )
        .expect("map canvas zoom fixture should install");

        patch_env(&env);

        let (layer_index, first_scale, second_layer, target_scale): (i64, f64, i64, f64) = env
            .eval(
                r#"
                local layerIndex = MapCanvasScrollControllerMixin.GetCurrentLayerIndex(scroll)
                return layerIndex,
                    scroll.zoomLevels[1].scale,
                    scroll.zoomLevels[2].layerIndex,
                    scroll.targetScale
                "#,
            )
            .expect("patched zoom layer query should create fallback zoom levels");

        assert_eq!(layer_index, 1);
        assert_eq!(first_scale, 0.75);
        assert_eq!(second_layer, 2);
        assert_eq!(target_scale, 0.75);
    }
}
