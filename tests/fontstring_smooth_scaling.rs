#![cfg(feature = "retail-12-0-5")]

use std::cell::RefCell;
use std::rc::Rc;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::font::WowFontSystem;

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.set_font_system(Rc::new(RefCell::new(WowFontSystem::new_without_casc())));
    env
}

#[test]
fn smooth_scaling_roundtrip_isolated_and_animation_mode_independent() {
    env()
        .exec(
            r#"
        local parent = CreateFrame('Frame')
        local a, b = parent:CreateFontString(), parent:CreateFontString()
        assert(a:GetSmoothScaling() == false and b:GetSmoothScaling() == false)
        assert(select('#', a:GetSmoothScaling()) == 1)
        a:SetScaleAnimationMode(Enum.FontStringScaleAnimationMode.Vertex)
        assert(select('#', a:SetSmoothScaling(true)) == 0)
        assert(a:GetSmoothScaling() == true and b:GetSmoothScaling() == false)
        assert(a:GetScaleAnimationMode() == Enum.FontStringScaleAnimationMode.Vertex)
        a:SetSmoothScaling(false)
        assert(a:GetSmoothScaling() == false)
        a:SetScaleAnimationMode(Enum.FontStringScaleAnimationMode.FontSize)
        assert(a:GetSmoothScaling() == false)
    "#,
        )
        .unwrap();
}

#[test]
fn smooth_scaling_bad_arguments_preserve_state() {
    env()
        .exec(
            r#"
        local fs = CreateFrame('Frame'):CreateFontString()
        fs:SetSmoothScaling(true)
        for _, value in ipairs({0, 1, 'true', {}, fs}) do
            assert(not pcall(fs.SetSmoothScaling, fs, value))
            assert(fs:GetSmoothScaling() == true)
        end
        assert(not pcall(fs.SetSmoothScaling, fs))
        assert(not pcall(fs.SetSmoothScaling, fs, nil))
        assert(fs:GetSmoothScaling() == true)
    "#,
        )
        .unwrap();
}

#[test]
#[cfg(feature = "forbidden-aspects")]
fn smooth_scaling_secret_boolean_requires_untainted_caller() {
    env()
        .exec(
            r#"
        local fs = CreateFrame('Frame'):CreateFontString()
        local yes, no = secretwrap(true), secretwrap(false)
        fs:SetSmoothScaling(yes)
        assert(fs:GetSmoothScaling() == true and not issecretvalue(fs:GetSmoothScaling()))
        local function denied()
            assert(not pcall(fs.SetSmoothScaling, fs, no))
            assert(fs:GetSmoothScaling() == true)
            fs:SetSmoothScaling(false)
            assert(fs:GetSmoothScaling() == false)
        end
        debug.setobjecttaint(denied, 'SmoothScalingTest')
        denied()
    "#,
        )
        .unwrap();
}

#[test]
fn smooth_scaling_fractional_height_and_auto_height_follow_mode_flips() {
    env()
        .exec(
            r#"
        local fs = CreateFrame('Frame'):CreateFontString()
        fs:SetFont('Fonts\\FRIZQT__.TTF', 12 * 1.1)
        fs:SetText('H')
        local function near(a, b) assert(math.abs(a-b) < .001, tostring(a)..' vs '..tostring(b)) end
        near(fs:GetStringHeight(), 16)
        fs:SetSmoothScaling(true)
        near(fs:GetStringHeight(), 15.84)
        near(fs:GetHeight(), 15.84)
        fs:SetSmoothScaling(false)
        near(fs:GetStringHeight(), 16)
        near(fs:GetHeight(), 16)
        fs:SetSmoothScaling(true)
        fs:SetText('H\nH')
        local height = fs:GetStringHeight()
        assert(height > 2 * 15.84)
        near(fs:GetHeight(), height)
        fs:SetTextScale(1.1)
        near(fs:GetStringHeight(), height * 1.1)
    "#,
        )
        .unwrap();
}

#[test]
fn smooth_scaling_wrap_keeps_line_count_and_fractional_height() {
    env()
        .exec(
            r#"
        local fs = CreateFrame('Frame'):CreateFontString()
        fs:SetFont('Fonts\\FRIZQT__.TTF', 12 * 1.1)
        fs:SetWidth(24)
        fs:SetText('Hello Hello Hello Hello')
        local snapped, count = fs:GetStringHeight(), fs:GetNumLines()
        assert(count > 1)
        fs:SetSmoothScaling(true)
        assert(fs:GetNumLines() == count)
        assert(fs:GetStringHeight() < snapped)
        assert(math.abs(fs:GetHeight() - fs:GetStringHeight()) < .001)
    "#,
        )
        .unwrap();
}

#[test]
fn smooth_scaling_xml_ordinary_and_runtime_template() {
    let env = env();
    let directory = tempfile::tempdir().unwrap();
    let toc = directory.path().join("SmoothFixture.toc");
    std::fs::write(&toc, "## Title: SmoothFixture\nfixture.xml\n").unwrap();
    std::fs::write(
        directory.path().join("fixture.xml"),
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
          <FontString name="SmoothRegionTemplate" virtual="true" smoothScaling="true"/>
          <FontString name="SnappedRegionTemplate" virtual="true" inherits="SmoothRegionTemplate" smoothScaling="false"/>
          <Frame name="SmoothTemplate" virtual="true"><Layers><Layer>
            <FontString parentKey="Smooth" smoothScaling="true"/>
            <FontString parentKey="Plain" smoothScaling="false"/>
          </Layer></Layers></Frame>
          <Frame name="SmoothOrdinary" inherits="SmoothTemplate"/>
        </Ui>
    "#,
    )
    .unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    env.exec(
        r#"
        assert(SmoothOrdinary.Smooth:GetSmoothScaling() == true)
        assert(SmoothOrdinary.Plain:GetSmoothScaling() == false)
        local runtime = CreateFrame('Frame', nil, UIParent, 'SmoothTemplate')
        assert(runtime.Smooth:GetSmoothScaling() == true)
        assert(runtime.Plain:GetSmoothScaling() == false)
        local inherited = runtime:CreateFontString(nil, 'ARTWORK', 'SmoothRegionTemplate')
        local overridden = runtime:CreateFontString(nil, 'ARTWORK', 'SnappedRegionTemplate')
        assert(inherited:GetSmoothScaling() == true)
        assert(overridden:GetSmoothScaling() == false)
    "#,
    )
    .unwrap();
}
