#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn retail_c_uicolor_get_colors_publishes_shared_fresh_color_mixin_rows() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let result: bool = env
        .eval(
            r##"
            ColorMixin = { marker = function() return "from ColorMixin" end }
            local function pack(...)
              return { n = select("#", ...), ... }
            end

            if not rawequal(_G.C_UIColor.GetColors, __secureenv.C_UIColor.GetColors) then
              return false
            end

            local first = pack(C_UIColor.GetColors())
            local with_ignored_arguments = pack(C_UIColor.GetColors(nil, false, {}))
            if first.n ~= 1 or with_ignored_arguments.n ~= 1 then return false end
            if type(first[1]) ~= "table" or type(with_ignored_arguments[1]) ~= "table" then return false end
            if #first[1] == 0 or #first[1] ~= #with_ignored_arguments[1] then return false end
            if rawequal(first[1], with_ignored_arguments[1]) then return false end
            if rawequal(first[1][1], with_ignored_arguments[1][1]) then return false end
            if rawequal(first[1][1].color, with_ignored_arguments[1][1].color) then return false end

            for _, entry in ipairs(first[1]) do
              if type(entry) ~= "table" or type(entry.baseTag) ~= "string" then return false end
              if type(entry.color) ~= "table" or entry.color.marker ~= ColorMixin.marker then return false end
              for _, channel in ipairs({ entry.color.r, entry.color.g, entry.color.b, entry.color.a }) do
                if type(channel) ~= "number" or channel < 0 or channel > 1 then return false end
              end
            end
            return true
            "##,
        )
        .expect("C_UIColor.GetColors API probe should run");

    assert!(
        result,
        "GetColors must be flag-3-published, argument-agnostic, fresh, and copy ColorMixin rows"
    );
}
