#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn retail_c_uicolor_get_colors_publishes_shared_fresh_color_mixin_rows() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let result: String = env
        .eval(
            r##"
            ColorMixin = { marker = function() return "from ColorMixin" end }
            local function pack(...)
              return { n = select("#", ...), ... }
            end

            if not rawequal(_G.C_UIColor.GetColors, __secureenv.C_UIColor.GetColors) then
              return "dual-publication"
            end

            local first = pack(C_UIColor.GetColors())
            local with_ignored_arguments = pack(C_UIColor.GetColors(nil, false, {}))
            if first.n ~= 1 or with_ignored_arguments.n ~= 1 then return "return-count" end
            if type(first[1]) ~= "table" or type(with_ignored_arguments[1]) ~= "table" then
              return "return-table"
            end
            if #first[1] == 0 or #first[1] ~= #with_ignored_arguments[1] then return "array-count" end
            if rawequal(first[1], with_ignored_arguments[1]) then return "fresh-array" end
            if rawequal(first[1][1], with_ignored_arguments[1][1]) then return "fresh-record" end
            if rawequal(first[1][1].color, with_ignored_arguments[1][1].color) then return "fresh-color" end

            for _, entry in ipairs(first[1]) do
              if type(entry) ~= "table" or type(entry.baseTag) ~= "string" then return "record-shape" end
              if type(entry.color) ~= "table" then return "color-table" end
              if entry.color.marker ~= ColorMixin.marker then return "color-mixin-member-copy" end
              for _, channel in ipairs({ entry.color.r, entry.color.g, entry.color.b, entry.color.a }) do
                if type(channel) ~= "number" or channel < 0 or channel > 1 then return "channel-range" end
              end
            end
            return "ok"
            "##,
        )
        .expect("C_UIColor.GetColors API probe should run");

    assert_eq!(result, "ok", "GetColors API contract failure: {result}");
}
