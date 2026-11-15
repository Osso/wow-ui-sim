//! Temporary map runtime state and helper surface.
//!
//! Core map data is partly Rust-backed in `c_api::c_map` and
//! `c_api::c_map_exploration_info`, but quest-log map selection and the
//! `UiMapPoint` helper shape still live as Lua compatibility behavior.

const MAP_RUNTIME_STATE_LUA: &str = r#"
if type(C_Map) ~= "table" then
    C_Map = {}
end
if type(UiMapPoint) ~= "table" then
    UiMapPoint = {}
end
if type(C_MapExplorationInfo) ~= "table" then
    C_MapExplorationInfo = {}
end

local __wow_map_area_names = {
  [1] = "Dun Morogh",
  [2248] = "The Isle of Dorn",
}

local function __wow_map_layer_dimensions(mapID)
  if C_Map == nil or type(C_Map.GetMapArtLayers) ~= "function" then
    return nil, nil
  end
  local layers = C_Map.GetMapArtLayers(mapID)
  if type(layers) ~= "table" then
    return nil, nil
  end
  local layer = layers[1]
  if type(layer) ~= "table" then
    return nil, nil
  end
  return layer.layerWidth, layer.layerHeight
end

local function __wow_map_make_overlay(offsetX, offsetY, textureWidth, textureHeight, fileDataIDs)
  return {
    offsetX = offsetX,
    offsetY = offsetY,
    textureWidth = textureWidth,
    textureHeight = textureHeight,
    isShownByMouseOver = false,
    isDrawOnTopLayer = false,
    fileDataIDs = fileDataIDs,
    hitRect = {
      top = offsetY,
      bottom = offsetY + textureHeight,
      left = offsetX,
      right = offsetX + textureWidth,
    },
  }
end

local function __wow_map_exploration_overlays(mapID)
  local layerWidth, layerHeight = __wow_map_layer_dimensions(mapID)
  if layerWidth == nil or layerHeight == nil then
    return {}
  end

  local topOffset = math.floor(layerHeight * 0.02)
  local overlayHeight = math.max(math.floor(layerHeight * 0.94), 1)
  local leftWidth = math.floor(layerWidth * 0.55)
  local rightOffset = math.floor(layerWidth * 0.82)
  local rightWidth = math.max(math.floor(layerWidth * 0.18), 1)
  return {
    __wow_map_make_overlay(0, topOffset, leftWidth, overlayHeight, { 4556093, 4741460 }),
    __wow_map_make_overlay(rightOffset, topOffset, rightWidth, overlayHeight, { 4556094 }),
  }
end

local function __wow_map_point_from_table(mapID, pos)
  if type(pos) ~= "table" then
    return nil
  end
  return {
    uiMapID = mapID,
    x = tonumber(pos.x) or 0.5,
    y = tonumber(pos.y) or 0.5,
  }
end

-- Retail owns these constructors in the original game ObjectAPI. Other
-- profiles retain the fallback until their ownership boundary is proven.
if retainUiMapPointFallback then
  UiMapPoint.CreateFromVector2D = function(mapID, pos)
    return __wow_map_point_from_table(mapID, pos)
  end

  UiMapPoint.CreateFromCoordinates = function(mapID, x, y)
    return { uiMapID = mapID, x = tonumber(x) or 0, y = tonumber(y) or 0 }
  end
end

if type(C_MapExplorationInfo.GetExploredAreaIDsAtPosition) ~= "function" then
  C_MapExplorationInfo.GetExploredAreaIDsAtPosition = function(mapID, pos)
    local areas = {}
    local point = __wow_map_point_from_table(mapID, pos)
    if point == nil then
      return areas
    end

    if mapID == C_Map.GetCurrentMapID() then
      if point.x < 0.10 or point.y < 0.05 then
        return areas
      end
      if point.x >= 0.68 and point.x <= 0.74 and point.y >= 0.20 and point.y <= 0.50 then
        return areas
      end
      if point.x <= 0.55 and point.y >= 0.05 and point.y <= 0.95 then
        areas[1] = 1
        areas[2] = 2
        return areas
      end
      if point.x >= 0.82 and point.y >= 0.05 and point.y <= 0.95 then
        areas[1] = 3
        return areas
      end
    end

    return areas
  end
end

if type(C_MapExplorationInfo.GetExploredMapTextures) ~= "function" then
  C_MapExplorationInfo.GetExploredMapTextures = function(mapID)
    if mapID ~= C_Map.GetCurrentMapID() and mapID ~= 1 then
      return {}
    end
    return __wow_map_exploration_overlays(mapID)
  end
end

local __wow_map_runtime_state = rawget(_G, "__wow_map_runtime_state")
if type(__wow_map_runtime_state) ~= "table" then
  __wow_map_runtime_state = {
    currentMapID = type(C_Map.GetCurrentMapID) == "function" and C_Map.GetCurrentMapID() or 2248,
  }
  rawset(_G, "__wow_map_runtime_state", __wow_map_runtime_state)
end

C_Map.GetCurrentMapID = function()
  return __wow_map_runtime_state.currentMapID or 2248
end

C_Map.SetMapForQuestLog = function(mapID)
  if type(mapID) ~= "number" then
    return
  end

  __wow_map_runtime_state.currentMapID = mapID

  if WorldMapFrame and type(WorldMapFrame.SetMapID) == "function" then
    WorldMapFrame:SetMapID(mapID)
  end

  if QuestMapFrame and type(QuestMapFrame.SetMapID) == "function" then
    QuestMapFrame:SetMapID(mapID)
  end
end

C_Map.GetBestMapForUnit = function(unitToken)
  if unitToken ~= nil and unitToken ~= "player" then
    return nil
  end
  if C_Map.GetCurrentMapID ~= nil then
    local currentMapID = C_Map.GetCurrentMapID()
    if currentMapID ~= nil then
      return currentMapID
    end
  end
  return 2248
end

C_Map.GetFallbackWorldMapID = function()
  if C_Map.GetCurrentMapID ~= nil then
    local currentMapID = C_Map.GetCurrentMapID()
    if currentMapID ~= nil then
      return currentMapID
    end
  end
  return 2248
end

C_Map.GetAreaInfo = function(areaID)
  if areaID == nil then
    return nil
  end
  return __wow_map_area_names[areaID]
end

C_Map.GetMapWorldSize = function(mapID)
  local layerWidth, layerHeight = __wow_map_layer_dimensions(mapID)
  if layerWidth == nil or layerHeight == nil then
    return nil
  end
  return layerWidth, layerHeight
end

C_Map.MapHasArt = function(mapID)
  if mapID == nil then
    return false
  end
  if C_Map.GetMapArtID ~= nil then
    local artID = C_Map.GetMapArtID(mapID)
    if artID ~= nil then
      return artID ~= 0
    end
  end
  return true
end

if MapUtil == nil then
  MapUtil = {}
end

local __wow_maputil_child_map_cache = {}

local function __wow_maputil_safe_map_info(mapID)
  if C_Map == nil or C_Map.GetMapInfo == nil or mapID == nil then
    return nil
  end
  return C_Map.GetMapInfo(mapID)
end

local function __wow_maputil_safe_map_has_art(mapID)
  if C_Map == nil then
    return false
  end
  if C_Map.MapHasArt ~= nil then
    local ok, hasArt = pcall(C_Map.MapHasArt, mapID)
    if ok and hasArt ~= nil then
      return hasArt
    end
  end
  if C_Map.GetMapArtID ~= nil then
    local ok, artID = pcall(C_Map.GetMapArtID, mapID)
    if ok and artID ~= nil then
      return artID ~= 0
    end
  end
  return mapID ~= nil
end

if MapUtil.IsMapTypeZone == nil then
  function MapUtil.IsMapTypeZone(mapID)
    local mapInfo = __wow_maputil_safe_map_info(mapID)
    return mapInfo ~= nil and mapInfo.mapType == Enum.UIMapType.Zone
  end
end

if MapUtil.GetMapParentInfo == nil then
  function MapUtil.GetMapParentInfo(mapID, mapType, topMost)
    local candidate = nil
    local mapInfo = __wow_maputil_safe_map_info(mapID)
    while mapInfo do
      if mapInfo.mapType == mapType then
        if topMost then
          candidate = mapInfo
        else
          return mapInfo
        end
      end
      mapInfo = __wow_maputil_safe_map_info(mapInfo.parentMapID)
    end
    return candidate
  end
end

if MapUtil.ShouldMapTypeShowQuests == nil then
  function MapUtil.ShouldMapTypeShowQuests(mapType)
    return mapType ~= Enum.UIMapType.World
      and mapType ~= Enum.UIMapType.Continent
      and mapType ~= Enum.UIMapType.Cosmic
  end
end

if MapUtil.GetDisplayableMapForPlayer == nil then
  function MapUtil.GetDisplayableMapForPlayer()
    if C_Map == nil then
      return 1
    end
    local mapID = C_Map and C_Map.GetBestMapForUnit and C_Map.GetBestMapForUnit("player")
    if mapID == nil then
      if C_Map.GetFallbackWorldMapID then
        mapID = C_Map.GetFallbackWorldMapID()
      elseif C_Map.GetCurrentMapID then
        mapID = C_Map.GetCurrentMapID()
      else
        return 1
      end
    end
    while mapID and mapID ~= 0 do
      if __wow_maputil_safe_map_has_art(mapID) then
        return mapID
      end
      local mapInfo = __wow_maputil_safe_map_info(mapID)
      mapID = mapInfo and mapInfo.parentMapID or 0
    end
    if C_Map and C_Map.GetFallbackWorldMapID then
      local fallbackMapID = C_Map.GetFallbackWorldMapID()
      if fallbackMapID ~= nil then
        return fallbackMapID
      end
    end
    if C_Map and C_Map.GetCurrentMapID then
      local currentMapID = C_Map.GetCurrentMapID()
      if currentMapID ~= nil then
        return currentMapID
      end
    end
    return 1
  end
end

if MapUtil.GetMapCenterOnMap == nil then
  function MapUtil.GetMapCenterOnMap(mapID, topMapID)
    if C_Map == nil or C_Map.GetMapRectOnMap == nil then
      return nil, nil
    end
    local left, right, top, bottom = C_Map.GetMapRectOnMap(mapID, topMapID)
    if left == nil then
      return nil, nil
    end
    return left + (right - left) * 0.5, top + (bottom - top) * 0.5
  end
end

if MapUtil.IsChildMap == nil then
  function MapUtil.IsChildMap(mapID, ancestorMapID)
    local mapInfo = __wow_maputil_safe_map_info(mapID)
    while mapInfo ~= nil and mapInfo.parentMapID ~= nil do
      if mapInfo.parentMapID == ancestorMapID then
        return true
      end
      mapInfo = __wow_maputil_safe_map_info(mapInfo.parentMapID)
    end
    return false
  end
end

if MapUtil.IsChildMapCached == nil then
  function MapUtil.IsChildMapCached(mapID, ancestorMapID)
    local key = tostring(mapID) .. ":" .. tostring(ancestorMapID)
    local cached = __wow_maputil_child_map_cache[key]
    if cached ~= nil then
      return cached
    end
    local result = MapUtil.IsChildMap(mapID, ancestorMapID)
    __wow_maputil_child_map_cache[key] = result
    return result
  end
end

if MapUtil.MapHasEmissaries == nil then
  function MapUtil.MapHasEmissaries(_mapID)
    return false
  end
end

if MapUtil.IsOribosMap == nil then
  function MapUtil.IsOribosMap(mapID)
    return mapID == 1670 or mapID == 1671 or mapID == 1672 or mapID == 1673
  end
end

if MapUtil.IsShadowlandsZoneMap == nil then
  function MapUtil.IsShadowlandsZoneMap(mapID)
    if mapID == 1550 or MapUtil.IsOribosMap(mapID) then
      return true
    end
    local mapInfo = __wow_maputil_safe_map_info(mapID)
    if mapInfo == nil then
      return false
    end
    if mapInfo.mapType ~= Enum.UIMapType.Zone and mapInfo.mapType ~= Enum.UIMapType.Continent then
      return false
    end
    return MapUtil.IsChildMap(mapID, 1550)
  end
end

if MapUtil.MapShouldShowWorldQuestFilters == nil then
  function MapUtil.MapShouldShowWorldQuestFilters(mapID)
    return MapUtil.MapHasEmissaries(mapID) or MapUtil.IsShadowlandsZoneMap(mapID)
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    let retain_fallback =
        crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail;
    lua.exec(&format!(
        "local retainUiMapPointFallback = {retain_fallback}\n{MAP_RUNTIME_STATE_LUA}"
    ))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn uimap_point_bootstrap_respects_profile_boundary() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let constructors: (bool, bool) = env
            .eval("return UiMapPoint.CreateFromCoordinates ~= nil, UiMapPoint.CreateFromVector2D ~= nil")
            .expect("constructor publication probe");
        let retained =
            crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail;
        assert_eq!(constructors, (retained, retained));
        if retained {
            env.exec(
                r#"
                local point = UiMapPoint.CreateFromCoordinates(2248, 0.25, 0.75)
                assert(point.uiMapID == 2248 and point.x == 0.25 and point.y == 0.75)
                local vector = UiMapPoint.CreateFromVector2D(1, {x=0.7, y=0.2})
                assert(vector.uiMapID == 1 and vector.x == 0.7 and vector.y == 0.2)
            "#,
            )
            .expect("other profiles retain their existing fallback");
        }
    }

    #[test]
    fn installs_map_runtime_state_and_helpers() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                C_Map.SetMapForQuestLog(1)
                if C_Map.GetCurrentMapID() ~= 1 then
                    return "bad_current_map"
                end
                if C_Map.GetBestMapForUnit("player") ~= 1 then
                    return "bad_best_map"
                end
                if C_Map.GetBestMapForUnit("target") ~= nil then
                    return "bad_non_player_map"
                end
                if C_Map.GetAreaInfo(1) ~= "Dun Morogh" then
                    return "bad_area"
                end
                if C_Map.GetFallbackWorldMapID() ~= 1 then
                    return "bad_fallback_map"
                end
                if C_Map.MapHasArt(1) ~= true then
                    return "bad_map_art"
                end
                if MapUtil.GetDisplayableMapForPlayer() ~= 1 then
                    return "bad_displayable_map"
                end
                if MapUtil.ShouldMapTypeShowQuests(Enum.UIMapType.World) ~= false then
                    return "bad_world_quests"
                end
                local width, height = C_Map.GetMapWorldSize(1)
                if type(width) ~= "number" or type(height) ~= "number" then
                    return "bad_world_size"
                end
                return "ok"
                "#,
            )
            .expect("map runtime probe should run");

        assert_eq!(result, "ok");
    }
}
