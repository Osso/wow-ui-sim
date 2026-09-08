//! Native-identity curve handles used by C_CurveUtil and secure aura options.

const CURVES_LUA: &str = r#"
local objects = setmetatable({}, {__mode='k'})
debug.getregistry().__wow_curve_objects = objects
local nextId = 0

local function make_factory(kind, isColor)
    local methods = {}
    local prototype = newproxy(true)
    local mt = getmetatable(prototype)
    local function state(object)
        local value = objects[object]
        if not value or value.kind ~= kind then error(kind .. ' expected', 3) end
        return value
    end
    mt.__index = function(object, key)
        return methods[key] or state(object).fields[key]
    end
    mt.__newindex = function(object, key, value)
        if methods[key] then error('read-only key: ' .. tostring(key), 2) end
        state(object).fields[key] = value
    end
    mt.__tostring = function(object) return state(object).label end
    mt.__metatable = false
    local function create()
        nextId = nextId + 1
        local object = newproxy(prototype)
        objects[object] = {kind=kind, label=kind .. ':' .. nextId, fields={}, points={}, curveType=0}
        return object
    end
    local function copy_value(value)
        if not isColor then return value end
        return CreateColor(value.r, value.g, value.b, value.a)
    end
    function methods:AddPoint(x, y)
        local s = state(self)
        s.points[#s.points + 1] = {x=x or 0, y=copy_value(y or 0)}
    end
    function methods:ClearPoints() state(self).points = {} end
    function methods:SetType(value) state(self).curveType = value or 0 end
    function methods:GetPointCount() return #state(self).points end
    function methods:Copy()
        local s = state(self)
        local result = create()
        local copy = state(result)
        copy.curveType = s.curveType
        for index, point in ipairs(s.points) do
            copy.points[index] = {x=point.x, y=copy_value(point.y)}
        end
        return result
    end
    local function interpolate(left, right, fraction)
        if not isColor then return left + (right - left) * fraction end
        return CreateColor(
            left.r + (right.r - left.r) * fraction,
            left.g + (right.g - left.g) * fraction,
            left.b + (right.b - left.b) * fraction,
            left.a + (right.a - left.a) * fraction)
    end
    function methods:Evaluate(x)
        local s = state(self)
        local points = s.points
        if #points == 0 then
            if isColor then return CreateColor(0, 0, 0, 0) end
            return 0
        end
        if #points == 1 then return copy_value(points[1].y) end
        local target = x or 0
        if isColor and s.curveType == 1 then
            if target < points[1].x then error('Color curve lower extrapolation is not modeled', 2) end
            local value = points[1].y
            for index = 2, #points do
                if target < points[index].x then break end
                value = points[index].y
            end
            return copy_value(value)
        end
        if isColor and s.curveType ~= 0 then error('Color curve interpolation type is not modeled', 2) end
        if isColor and target < points[1].x then error('Color curve lower extrapolation is not modeled', 2) end
        for index = 1, #points - 1 do
            local left, right = points[index], points[index + 1]
            if target <= right.x then
                local dx = right.x - left.x
                if dx == 0 then return copy_value(right.y) end
                return interpolate(left.y, right.y, (target - left.x) / dx)
            end
        end
        return copy_value(points[#points].y)
    end
    return create
end

C_CurveUtil = C_CurveUtil or __wow_namespace()
C_CurveUtil.CreateCurve = make_factory('LuaCurveObject', false)
C_CurveUtil.CreateColorCurve = make_factory('LuaColorCurveObject', true)
"#;

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CURVES_LUA)?;
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
pub(crate) fn is_curve_object(
    state: &mut rilua::vm::state::LuaState,
    value: rilua::Val,
    kind: &str,
) -> bool {
    use crate::lua_api::methods::{create_string, registry_get, table_get};
    let rilua::Val::Userdata(_) = value else {
        return false;
    };
    let rilua::Val::Table(objects) = registry_get(state, "__wow_curve_objects") else {
        return false;
    };
    let entry = state
        .gc
        .tables
        .get(objects)
        .map(|table| table.get(value, &state.gc.string_arena));
    let Some(entry) = entry else { return false };
    table_get(state, entry, "kind") == create_string(state, kind)
}
