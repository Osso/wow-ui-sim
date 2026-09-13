//! Native-identity curve handles used by C_CurveUtil and secure aura options.

const CURVES_LUA: &str = r#"
local objects = setmetatable({}, {__mode='k'})
debug.getregistry().__wow_curve_objects = objects
local nextId = 0

local function copy_value(value, isColor)
    if not isColor then return value end
    return CreateColor(value.r, value.g, value.b, value.a)
end

local function interpolate(left, right, fraction, isColor)
    if not isColor then return left + (right - left) * fraction end
    return CreateColor(
        left.r + (right.r - left.r) * fraction,
        left.g + (right.g - left.g) * fraction,
        left.b + (right.b - left.b) * fraction,
        left.a + (right.a - left.a) * fraction)
end

local function evaluate_color_step(points, target)
    if target < points[1].x then error('Color curve lower extrapolation is not modeled', 2) end
    local value = points[1].y
    for index = 2, #points do
        if target < points[index].x then break end
        value = points[index].y
    end
    return copy_value(value, true)
end

local function evaluate_linear(points, target, isColor)
    for index = 1, #points - 1 do
        local left, right = points[index], points[index + 1]
        if target <= right.x then
            local dx = right.x - left.x
            if dx == 0 then return copy_value(right.y, isColor) end
            return interpolate(left.y, right.y, (target - left.x) / dx, isColor)
        end
    end
    return copy_value(points[#points].y, isColor)
end

local function evaluate_curve(s, x, isColor)
    local points = s.points
    if #points == 0 then
        if isColor then return CreateColor(0, 0, 0, 0) end
        return 0
    end
    if #points == 1 then return copy_value(points[1].y, isColor) end
    local target = x or 0
    if isColor and s.curveType == 1 then
        return evaluate_color_step(points, target)
    end
    if isColor and s.curveType ~= 0 then error('Color curve interpolation type is not modeled', 2) end
    if isColor and target < points[1].x then error('Color curve lower extrapolation is not modeled', 2) end
    return evaluate_linear(points, target, isColor)
end

local function install_object_access(prototype, methods, state)
    local mt = getmetatable(prototype)
    mt.__index = function(object, key)
        return methods[key] or state(object).fields[key]
    end
    mt.__newindex = function(object, key, value)
        if methods[key] then error('read-only key: ' .. tostring(key), 2) end
        state(object).fields[key] = value
    end
    mt.__tostring = function(object) return state(object).label end
    mt.__metatable = false
end

local function install_curve_methods(methods, state, create, isColor)
    function methods:AddPoint(x, y)
        local s = state(self)
        s.points[#s.points + 1] = {x=x or 0, y=copy_value(y or 0, isColor)}
    end
    function methods:ClearPoints() state(self).points = {} end
    function methods:SetType(value) state(self).curveType = value or 0 end
    function methods:GetType() return state(self).curveType end
    function methods:GetPointCount() return #state(self).points end
    function methods:Copy()
        local s = state(self)
        local result = create()
        local copy = state(result)
        copy.curveType = s.curveType
        for index, point in ipairs(s.points) do
            copy.points[index] = {x=point.x, y=copy_value(point.y, isColor)}
        end
        return result
    end
    function methods:Evaluate(x)
        return evaluate_curve(state(self), x, isColor)
    end
end

local function make_factory(kind, isColor)
    local methods = {}
    local prototype = newproxy(true)
    local function state(object)
        local value = objects[object]
        if not value or value.kind ~= kind then error(kind .. ' expected', 3) end
        return value
    end
    local function create()
        nextId = nextId + 1
        local object = newproxy(prototype)
        objects[object] = {kind=kind, label=kind .. ':' .. nextId, fields={}, points={}, curveType=0}
        return object
    end
    install_object_access(prototype, methods, state)
    install_curve_methods(methods, state, create, isColor)
    if isColor then
        function methods:GetPoint(index)
            -- Best-effort: one-based lookup and independent output snapshots.
            local point = state(self).points[index]
            if not point then return nil end
            return {x=point.x, y=copy_value(point.y, true)}
        end
    end
    return create
end

C_CurveUtil = C_CurveUtil or __wow_namespace()
C_CurveUtil.CreateCurve = make_factory('LuaCurveObject', false)
C_CurveUtil.CreateColorCurve = make_factory('LuaColorCurveObject', true)
"#;

#[cfg(feature = "retail-12-0-0")]
const BOOLEAN_SELECTION_LUA: &str = r#"
local channels = {'r', 'g', 'b', 'a'}

local function require_color(value)
    if type(value) ~= 'table' then error('color table expected', 3) end
    for _, channel in ipairs(channels) do
        if type(value[channel]) ~= 'number' then
            error('color must contain numeric RGBA channels', 3)
        end
    end
end

local function select_boolean_value(condition, valueIfTrue, valueIfFalse)
    if type(condition) ~= 'boolean' then error('boolean expected', 3) end
    if condition then return valueIfTrue end
    return valueIfFalse
end

function C_CurveUtil.EvaluateColorFromBoolean(condition, valueIfTrue, valueIfFalse)
    require_color(valueIfTrue)
    require_color(valueIfFalse)
    local value = select_boolean_value(condition, valueIfTrue, valueIfFalse)
    return CreateColor(value.r, value.g, value.b, value.a)
end

function C_CurveUtil.EvaluateColorValueFromBoolean(condition, valueIfTrue, valueIfFalse)
    if type(valueIfTrue) ~= 'number' or type(valueIfFalse) ~= 'number' then
        error('numeric color components expected', 2)
    end
    return select_boolean_value(condition, valueIfTrue, valueIfFalse)
end
"#;

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CURVES_LUA)?;
    #[cfg(feature = "retail-12-0-0")]
    lua.exec(BOOLEAN_SELECTION_LUA)?;
    Ok(())
}

pub(crate) fn evaluate_curve_value(
    state: &mut rilua::vm::state::LuaState,
    curve: rilua::Val,
    value: f64,
) -> rilua::LuaResult<rilua::Val> {
    use crate::lua_api::methods::{call_function_state, create_string};
    let is_curve = is_curve_object(state, curve, "LuaCurveObject")
        || is_curve_object(state, curve, "LuaColorCurveObject");
    if !is_curve {
        return Err(rilua::runtime_error("expected LuaCurveObjectBase"));
    }
    let key = create_string(state, "Evaluate");
    let evaluate = state.gettable(curve, key)?;
    call_function_state(state, evaluate, &[curve, rilua::Val::Num(value)])
}

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
