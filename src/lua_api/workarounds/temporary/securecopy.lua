-- Compatibility copy shared by the 12.1 bootstrap and Forever.
-- Retire when native securecopy security/taint semantics are modeled.
-- Copies table graphs without metatables; non-table handles retain identity.
if securecopy == nil then
  local function __wow_securecopy(value, seen)
    if type(value) ~= "table" then
      return value
    end
    if seen[value] ~= nil then
      return seen[value]
    end
    local copy = {}
    seen[value] = copy
    for k, v in pairs(value) do
      copy[__wow_securecopy(k, seen)] = __wow_securecopy(v, seen)
    end
    return copy
  end

  function securecopy(value)
    return __wow_securecopy(value, {})
  end
end
