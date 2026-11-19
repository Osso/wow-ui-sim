#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use wow_ui_sim::lua_api::WowLuaEnv;

struct PinnedGlobalColorSample {
    tag: &'static str,
    packed_argb: u32,
    channels: (f64, f64, f64, f64),
}

// Count and sample literals are pinned 70783 snapshot expectations, not native-catalog
// parity.
// The 361-row count follows the CSV under the inferred feed/dedup rules.
// Source: data/db2/retail-12.1.5.70783/GlobalColor.csv
// SHA-256: e5fe982cf72c9d77af9d5a82360e0a06537b10daf9940c6a56705674029b38c2
// Channel literals use the R88o.3 f32 conversion; applying it to 70783 is INFERRED.
const PINNED_GLOBAL_COLOR_SAMPLES: &[PinnedGlobalColorSample] = &[
    PinnedGlobalColorSample {
        tag: "AREA_DESCRIPTION_FONT_COLOR",
        packed_argb: 0xffffffff,
        channels: (1.0, 1.0, 1.0, 1.0),
    },
    PinnedGlobalColorSample {
        tag: "HOUSING_REWARD_TOAST_LABEL_FONT_COLOR",
        packed_argb: 0xffdfbd99,
        channels: (
            0.874509871006012,
            0.7411764860153198,
            0.6000000238418579,
            1.0,
        ),
    },
    PinnedGlobalColorSample {
        tag: "INVASION_DESCRIPTION_FONT_COLOR",
        packed_argb: 0xfffff809,
        channels: (1.0, 0.9725490808486938, 0.03529411926865578, 1.0),
    },
    PinnedGlobalColorSample {
        tag: "INVASION_FONT_COLOR",
        packed_argb: 0xffc7ff00,
        channels: (0.7803922295570374, 1.0, 0.0, 1.0),
    },
    PinnedGlobalColorSample {
        tag: "PURE_RED_COLOR",
        packed_argb: 0xffcc0000,
        channels: (0.8000000715255737, 0.0, 0.0, 1.0),
    },
    PinnedGlobalColorSample {
        tag: "REFUND_TOOLTIP_COLOR",
        packed_argb: 0xff00ccff,
        channels: (0.0, 0.8000000715255737, 1.0, 1.0),
    },
    PinnedGlobalColorSample {
        tag: "ZULJARRA_MAJOR_FACTION_COLOR",
        packed_argb: 0x0054e6aa,
        channels: (
            0.3294117748737335,
            0.9019608497619629,
            0.6666666865348816,
            0.0,
        ),
    },
    PinnedGlobalColorSample {
        tag: "Prey Toast Text Color",
        packed_argb: 0x00ff2b2b,
        channels: (1.0, 0.16862745583057404, 0.16862745583057404, 0.0),
    },
];

#[test]
fn retail_c_uicolor_get_colors_matches_the_deduplicated_catalog() {
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

    for sample in PINNED_GLOBAL_COLOR_SAMPLES {
        assert_sample_row(&env, sample);
    }

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
    assert_eq!(
        returned_tags.len(),
        361,
        "pinned snapshot row count changed"
    );
    assert_strict_tag_order(&returned_tags);

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
    let expected_channels = sample_channels("ZULJARRA_MAJOR_FACTION_COLOR");
    assert_eq!((red, green, blue, alpha), expected_channels);
    assert_color_mixin_error(&env, "ColorMixin = nil");
    assert_color_mixin_error(&env, "ColorMixin = false");
}

prefork_full_ui_case! {
fn retail_full_ui_tooltip_color_measurement(env: &WowLuaEnv) {
    let values: String = env
        .eval(
            r#"
            local names = {"ERROR_COLOR", "DISABLED_FONT_COLOR", "SOULBIND_CONDUIT_ENHANCED_COLOR",
                "TRIVIAL_DIFFICULTY_COLOR", "HIGHLIGHT_FONT_COLOR", "NORMAL_FONT_COLOR",
                "GREEN_FONT_COLOR", "FRIENDLY_STATUS_COLOR", "INCREASE_STAT_COLOR",
                "ITEM_ARTIFACT_COLOR"}
            local rows = {}
            for _, name in ipairs(names) do
                local color = _G[name]
                if color then
                    local indexed = {tostring(color.r), tostring(color.g), tostring(color.b), tostring(color.a)}
                    local raw = {tostring(rawget(color, "r")), tostring(rawget(color, "g")),
                        tostring(rawget(color, "b")), tostring(rawget(color, "a"))}
                    rows[#rows + 1] = name .. "=" .. table.concat(indexed, ",")
                        .. " raw=" .. table.concat(raw, ",")
                else
                    rows[#rows + 1] = name .. "=<nil>"
                end
            end
            return table.concat(rows, "\n")
            "#,
        )
        .expect("Retail full-UI tooltip colors should be readable");
    println!("Retail tooltip color measurement:\n{values}");
}
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
    let expected_rgba = sample_channels("ZULJARRA_MAJOR_FACTION_COLOR");
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

fn assert_sample_row(env: &WowLuaEnv, sample: &PinnedGlobalColorSample) {
    let query = format!(
        r#"
        for _, row in ipairs(C_UIColor.GetColors()) do
          if row.baseTag == {tag:?} then
            return row.color.r, row.color.g, row.color.b, row.color.a
          end
        end
        return -1, -1, -1, -1
        "#,
        tag = sample.tag
    );
    let actual: (f64, f64, f64, f64) = env
        .eval(&query)
        .expect("pinned sample color channels should be readable");
    assert_eq!(
        actual, sample.channels,
        "unexpected channels for pinned row {} packed 0x{:08x}",
        sample.tag, sample.packed_argb
    );
}

fn assert_strict_tag_order(tags: &[&str]) {
    for pair in tags.windows(2) {
        assert!(
            pair[0].as_bytes() < pair[1].as_bytes(),
            "API tags are not strictly byte-sorted: {:?}, {:?}",
            pair[0],
            pair[1]
        );
    }
}

fn sample_channels(tag: &str) -> (f64, f64, f64, f64) {
    PINNED_GLOBAL_COLOR_SAMPLES
        .iter()
        .find(|sample| sample.tag == tag)
        .unwrap_or_else(|| panic!("no pinned color sample for {tag}"))
        .channels
}
