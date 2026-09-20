-- Use the same per-type metatable lookup as GetFontStringMetatable.
function GetTextureMetatable(texture)
  if texture == nil then
    local frame = CreateFrame("Frame")
    texture = frame:CreateTexture()
  end
  return getmetatable(texture)
end
