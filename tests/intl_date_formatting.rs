//! Explicit-zone ICU4C model; native WoW defaults and output versions are unverified.
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_dates_epoch_styles_and_explicit_zones() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local c = C_Intl.CreateLocaleContext("en-GB")
        assert(c:FormatDate(0, 1, "UTC") == "01/01/1970")
        assert(c:FormatTime(0, 2, "UTC") == "00:00:00")
        assert(c:FormatTime(-0.001, 2, "UTC") == "23:59:59")
        assert(c:FormatTime(0.999, 2, "UTC") == "00:00:00")
        assert(c:FormatTime(1.001, 2, "UTC") == "00:00:01")
        assert(c:FormatDate(0, 1, "America/New_York") == "31/12/1969")
        assert(c:FormatTime(0, 1, "GMT+05:30") == "05:30")
        assert(c:FormatTime(0, 1, "") == c:FormatTime(0, 1, "UTC"))
        assert(c:FormatDateTime(0, 0, 0, "UTC") == "")
        for style = 0, 4 do
            local date = c:FormatDate(0, style, "UTC")
            local time = c:FormatTime(0, style, "UTC")
            assert(c:FormatDateTime(0, style, 0, "UTC") == date)
            assert(c:FormatDateTime(0, 0, style, "UTC") == time)
            assert(select('#', c:FormatDateTime(0, style, style, "UTC")) == 1)
            if style > 0 then
                assert(#date > 0 and #time > 0)
                local both = c:FormatDateTime(0, style, style, "UTC")
                assert(both:find(date, 1, true) and both:find(time, 1, true))
            else assert(date == "" and time == "") end
        end
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_dates_dst_and_locale_selection() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local c = C_Intl.CreateLocaleContext("en-GB")
        assert(c:FormatTime(1710053999, 2, "America/New_York") == "01:59:59")
        assert(c:FormatTime(1710054000, 2, "America/New_York") == "03:00:00")
        local current = C_Intl.CreateLocaleContext(C_Intl.GetCurrentLocale())
        assert(C_Intl.FormatDate(0, 1, "UTC") == current:FormatDate(0, 1, "UTC"))
        assert(C_Intl.FormatTime(0, 1, "UTC") == current:FormatTime(0, 1, "UTC"))
        assert(C_Intl.FormatDateTime(0, 1, 2, "UTC") == current:FormatDateTime(0, 1, 2, "UTC"))
        local english = c:FormatDate(0, 3, "UTC")
        assert(c:SetLocale("de-DE"))
        assert(c:FormatDate(0, 3, "UTC") ~= english)
        assert(c:GetLocale() == "de-DE")
        local arab = C_Intl.CreateLocaleContext("en-GB-u-nu-arab")
        assert(arab:FormatDate(0, 1, "UTC") ~= "01/01/1970")
        assert(arab:GetLocale() == "en-GB-u-nu-arab")
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_dates_reject_invalid_arguments_without_zone_fallback() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local c = C_Intl.CreateLocaleContext("en-GB")
        for _, zone in ipairs({"not/a/zone", "UTC trailing", "UTC\0", "GMT+25:00", string.char(255)}) do
            assert(not pcall(c.FormatDateTime, c, 0, 1, 1, zone), zone)
            assert(not pcall(c.FormatDateTime, c, 0, 0, 0, zone), zone)
        end
        for _, value in ipairs({math.huge, -math.huge, 0/0, 1e308, -1e308}) do
            assert(not pcall(c.FormatDateTime, c, value, 1, 1, "UTC"))
        end
        for _, style in ipairs({-1, 5, 1.5, "1"}) do
            assert(not pcall(c.FormatDate, c, 0, style, "UTC"))
            assert(not pcall(c.FormatTime, c, 0, style, "UTC"))
        end
        assert(not pcall(c.FormatDate, c, 0, 1))
        assert(not pcall(c.FormatTime, c, "0", 1, "UTC"))
        assert(not pcall(c.FormatDate, {}, 0, 1, "UTC"))
        assert(c:FormatDate(0, 1, "UTC") == "01/01/1970")
    "#).unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_dates_preserve_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_Intl == nil)").unwrap();
    wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
    env.exec("assert(C_Intl == nil)").unwrap();
}
