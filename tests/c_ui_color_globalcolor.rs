#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use wow_ui_sim::lua_api::WowLuaEnv;

use sim_data_profile::global_colors_retail::{GlobalColorRow, RETAIL_GLOBAL_COLORS};

#[test]
fn retail_c_uicolor_get_colors_matches_the_deduplicated_catalog() {
    assert_eq!(RETAIL_GLOBAL_COLORS.len(), 361);
    assert_strict_tag_order(RETAIL_GLOBAL_COLORS);
    assert_sample_row("AREA_DESCRIPTION_FONT_COLOR", 56, 0xffffffff);
    assert_sample_row("HOUSING_REWARD_TOAST_LABEL_FONT_COLOR", 328, 0xffdfbd99);
    assert_sample_row("INVASION_DESCRIPTION_FONT_COLOR", 58, 0xfffff809);
    assert_sample_row("INVASION_FONT_COLOR", 57, 0xffc7ff00);
    assert_sample_row("PURE_RED_COLOR", 30, 0xffcc0000);
    assert_sample_row("REFUND_TOOLTIP_COLOR", 145, 0xff00ccff);
    assert_sample_row("ZULJARRA_MAJOR_FACTION_COLOR", 409, 0x0054e6aa);
    assert_sample_row("Prey Toast Text Color", 387, 0x00ff2b2b);

    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    assert_color_mixin_error(&env, "ColorMixin = nil");
    assert_color_mixin_error(&env, "ColorMixin = false");

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
            if #first[1] ~= 361 or #first[1] ~= #with_ignored_arguments[1] then
              return "array-count"
            end
            if rawequal(first[1], with_ignored_arguments[1]) then return "fresh-array" end

            local seen = {}
            for index, entry in ipairs(first[1]) do
              local other = with_ignored_arguments[1][index]
              if type(entry) ~= "table" or type(entry.baseTag) ~= "string" then return "record-shape" end
              if type(entry.color) ~= "table" then return "color-table" end
              if entry.color.marker ~= ColorMixin.marker then return "color-mixin-member-copy" end
              if rawequal(entry, other) then return "fresh-record" end
              if rawequal(entry.color, other.color) then return "fresh-color" end
              if entry.baseTag ~= other.baseTag then return "argument-tag-change" end
              if entry.color.r ~= other.color.r or entry.color.g ~= other.color.g
                or entry.color.b ~= other.color.b or entry.color.a ~= other.color.a then
                return "argument-color-change"
              end
              if seen[entry.baseTag] then return "duplicate-tag" end
              seen[entry.baseTag] = true
              for _, key in ipairs({ "r", "g", "b", "a" }) do
                local channel = entry.color[key]
                if type(channel) ~= "number" or channel < 0 or channel > 1 then return "channel-range" end
              end
            end
            return "ok"
            "##,
        )
        .expect("C_UIColor.GetColors API probe should run");

    assert_eq!(result, "ok", "GetColors API contract failure: {result}");

    let returned_tags: String = env
        .eval(
            r#"
            local tags = {}
            for index, row in ipairs(C_UIColor.GetColors()) do tags[index] = row.baseTag end
            return table.concat(tags, "\0")
            "#,
        )
        .expect("catalog tags should be readable");
    let returned_tags = returned_tags.split('\0').collect::<Vec<_>>();
    let expected_tags = RETAIL_GLOBAL_COLORS
        .iter()
        .map(|row| row.base_tag)
        .collect::<Vec<_>>();
    assert_eq!(
        returned_tags, expected_tags,
        "API rows must follow native tag order"
    );

    let (red, green, blue, alpha): (f64, f64, f64, f64) = env
        .eval(
            r#"
            for _, row in ipairs(C_UIColor.GetColors()) do
              if row.baseTag == "ZULJARRA_MAJOR_FACTION_COLOR" then
                return row.color.r, row.color.g, row.color.b, row.color.a
              end
            end
            return -1, -1, -1, -1
            "#,
        )
        .expect("sample color channels should be readable");
    let expected_channels = channels_from_argb(0x0054e6aa);
    assert_eq!((red, green, blue, alpha), expected_channels);
}

prefork_full_ui_case! {
fn retail_color_lua_publishes_catalog_globals_and_codes_in_both_environments(env: &WowLuaEnv) {
    let global_types: (String, String) = env
        .eval("return type(_G.ZULJARRA_MAJOR_FACTION_COLOR), type(_G.ZULJARRA_MAJOR_FACTION_COLOR_CODE)")
        .expect("Retail color globals should be readable");
    let secure_types: (String, String) = env
        .eval("return type(__secureenv.ZULJARRA_MAJOR_FACTION_COLOR), type(__secureenv.ZULJARRA_MAJOR_FACTION_COLOR_CODE)")
        .expect("secure Retail color globals should be readable");
    assert_eq!(global_types, ("table".to_owned(), "string".to_owned()));
    assert_eq!(secure_types, global_types);

    let global_rgba: (f64, f64, f64, f64) = env
        .eval("local c = _G.ZULJARRA_MAJOR_FACTION_COLOR; return c.r, c.g, c.b, c.a")
        .expect("global color channels should be readable");
    let secure_rgba: (f64, f64, f64, f64) = env
        .eval("local c = __secureenv.ZULJARRA_MAJOR_FACTION_COLOR; return c.r, c.g, c.b, c.a")
        .expect("secure color channels should be readable");
    let expected_rgba = channels_from_argb(0x0054e6aa);
    assert_eq!(global_rgba, expected_rgba);
    assert_eq!(secure_rgba, expected_rgba);

    let (global_code, global_generated): (String, String) = env
        .eval("local c = _G.ZULJARRA_MAJOR_FACTION_COLOR; return _G.ZULJARRA_MAJOR_FACTION_COLOR_CODE, c:GenerateHexColorMarkup()")
        .expect("global color code should be readable");
    let (secure_code, secure_generated): (String, String) = env
        .eval("local c = __secureenv.ZULJARRA_MAJOR_FACTION_COLOR; return __secureenv.ZULJARRA_MAJOR_FACTION_COLOR_CODE, c:GenerateHexColorMarkup()")
        .expect("secure color code should be readable");
    assert_eq!(global_code, global_generated);
    assert_eq!(secure_code, secure_generated);
    assert_eq!(global_code, secure_code);
}
}

fn assert_color_mixin_error(env: &WowLuaEnv, setup: &str) {
    env.exec(setup)
        .expect("ColorMixin state should be set for the probe");
    let error = env
        .eval::<bool>("C_UIColor.GetColors()")
        .expect_err("GetColors must reject a missing or non-table ColorMixin");
    assert!(
        error
            .to_string()
            .contains("unable to find mixin or metatable (ColorMixin)"),
        "unexpected ColorMixin diagnostic: {error}"
    );
}

fn assert_sample_row(tag: &str, id: u32, packed_color: u32) {
    let row = RETAIL_GLOBAL_COLORS
        .iter()
        .find(|row| row.base_tag == tag)
        .unwrap_or_else(|| panic!("deduplicated catalog has no {tag}"));
    assert_eq!(row.id, id, "unexpected winning DB2 row for {tag}");
    assert_eq!(
        row.packed_color, packed_color,
        "unexpected packed color for {tag}"
    );
}

fn assert_strict_tag_order(rows: &[GlobalColorRow]) {
    for pair in rows.windows(2) {
        assert!(
            pair[0].base_tag.as_bytes() < pair[1].base_tag.as_bytes(),
            "catalog tags are not strictly byte-sorted: {:?}, {:?}",
            pair[0].base_tag,
            pair[1].base_tag
        );
    }
}

fn channels_from_argb(packed_color: u32) -> (f64, f64, f64, f64) {
    let [alpha, red, green, blue] = packed_color.to_be_bytes();
    let channel = |byte| f64::from(f32::from(byte) * (1.0_f32 / 255.0_f32));
    (channel(red), channel(green), channel(blue), channel(alpha))
}
