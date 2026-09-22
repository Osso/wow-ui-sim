//! Tests for Slider widget methods.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().unwrap()
}

const VALUE_BINDINGS_XML: &str = r#"
<Ui>
    <Slider name="SliderValuePrecall" intrinsic="true">
        <Scripts>
            <OnValueChanged intrinsicOrder="precall">SliderValueProbe(self, 'pre', value)</OnValueChanged>
        </Scripts>
    </Slider>
    <Slider name="SliderValuePostcall" virtual="true">
        <Scripts>
            <OnValueChanged intrinsicOrder="postcall">SliderValueProbe(self, 'post', value)</OnValueChanged>
        </Scripts>
    </Slider>
</Ui>
"#;

fn value_binding_env() -> WowLuaEnv {
    let env = env();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("SliderValueBindings.toc");
    std::fs::write(&toc, "## Title: Slider value bindings\nBindings.xml\n").unwrap();
    std::fs::write(root.path().join("Bindings.xml"), VALUE_BINDINGS_XML).unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env
}

#[test]
fn set_value_dispatches_synchronously_with_mouse_payload() {
    env()
        .exec(
            r#"
        local slider = CreateFrame('Slider')
        slider:SetMinMaxValues(0, 10)
        slider:Hide()
        local calls = {}
        slider:SetScript('OnValueChanged', function(...)
            local self, value, mouse = ...
            calls[#calls + 1] = {self, value, mouse, select('#', ...), self:GetValue()}
        end)
        slider:SetValue(3)
        assert(#calls == 1, 'SetValue must dispatch before returning, even when hidden')
        assert(calls[1][1] == slider and calls[1][2] == 3 and calls[1][5] == 3)
        assert(calls[1][3] == false and calls[1][4] == 3, 'default mouse flag must be false')
        slider:SetValue(7, true)
        assert(#calls == 2 and calls[2][2] == 7 and calls[2][5] == 7)
        assert(calls[2][3] == true and calls[2][4] == 3, 'explicit mouse flag must be forwarded')
    "#,
        )
        .unwrap();
}

#[test]
fn set_value_clamps_suppresses_unchanged_and_allows_reentry() {
    env()
        .exec(
            r#"
        local slider = CreateFrame('Slider')
        slider:SetMinMaxValues(0, 10)
        local calls, errors = {}, {}
        seterrorhandler(function(message) errors[#errors + 1] = message end)
        slider:SetScript('OnValueChanged', function(self, value)
            calls[#calls + 1] = value
            self:SetValue(value)
        end)
        slider:SetValue(5)
        slider:SetValue(5, true)
        slider:SetValue(20)
        slider:SetValue(30)
        slider:SetValue(-3)
        slider:SetValue(-10)
        assert(#calls == 3, 'only changed clamped values must dispatch')
        assert(calls[1] == 5 and calls[2] == 10 and calls[3] == 0)
        assert(slider:GetValue() == 0)
        assert(#errors == 0, errors[1] or 'callback reentry must not fail')
    "#,
        )
        .unwrap();
}

#[test]
fn set_value_dispatches_intrinsic_bindings_after_reported_errors() {
    value_binding_env().exec(r#"
        local calls, errors = {}, {}
        local failPre = true
        seterrorhandler(function(message) errors[#errors + 1] = message end)
        function SliderValueProbe(self, binding, value)
            calls[#calls + 1] = binding .. ':' .. value
            assert(self:GetValue() == value, 'callback must observe committed value')
            if binding == 'pre' and failPre then error('slider precall failure') end
        end
        local slider = CreateFrame('Slider', nil, UIParent, 'SliderValuePrecall,SliderValuePostcall')
        slider:SetMinMaxValues(0, 10)
        slider:SetScript('OnValueChanged', function(self, value)
            SliderValueProbe(self, 'normal', value)
        end)
        slider:SetValue(3)
        assert(table.concat(calls, ',') == 'pre:3,normal:3,post:3', 'binding order or error continuation failed')
        assert(#errors == 1 and string.find(errors[1], 'slider precall failure', 1, true))
        failPre = false
        calls = {}
        slider:SetScript('OnValueChanged', nil)
        slider:SetValue(4)
        assert(table.concat(calls, ',') == 'pre:4,post:4', 'intrinsic bindings must run without normal script')
        assert(#errors == 1, 'unexpected later callback error')
    "#).unwrap();
}

#[test]
fn set_value_updates_castbar_style_scale_and_cvar_consumer() {
    env().exec(r#"
        local db = {scale = 1}
        local bar = CreateFrame('StatusBar')
        local slider = CreateFrame('Slider')
        slider:SetMinMaxValues(0.7, 1.8)
        C_CVar.RegisterCVar('__slider_castbar_scale', '')
        local calls = 0
        slider:SetScript('OnValueChanged', function(_, value)
            calls = calls + 1
            db.scale = tonumber(string.format('%.2f', value))
            bar:SetScale(db.scale)
            C_CVar.SetCVar('__slider_castbar_scale', tostring(db.scale))
        end)
        slider:SetValue(1.35)
        assert(calls == 1 and db.scale == 1.35, 'slider callback must update addon configuration')
        assert(math.abs(bar:GetScale() - 1.35) < 0.000001, 'castbar scale must update synchronously')
        assert(C_CVar.GetCVar('__slider_castbar_scale') == '1.35', 'callback CVar write missing')
        slider:SetValue(1)
        assert(calls == 2 and db.scale == 1 and bar:GetScale() == 1)
        assert(C_CVar.GetCVar('__slider_castbar_scale') == '1')
    "#).unwrap();
}

// ============================================================================
// GetThumbTexture / SetThumbTexture
// ============================================================================

#[test]
fn test_get_thumb_texture_defaults_to_thumb_child() {
    let env = env();
    let matches: bool = env
        .eval(
            r#"
        local s = CreateFrame("Slider")
        return s:GetThumbTexture() == s.ThumbTexture
    "#,
        )
        .unwrap();
    assert!(
        matches,
        "GetThumbTexture should return the default thumb child"
    );
}

#[test]
fn test_set_and_get_thumb_texture() {
    let env = env();
    let matches: bool = env
        .eval(
            r#"
        local s = CreateFrame("Slider")
        local t = s:CreateTexture()
        s:SetThumbTexture(t)
        return s:GetThumbTexture() == t
    "#,
        )
        .unwrap();
    assert!(
        matches,
        "GetThumbTexture should return the texture set via SetThumbTexture"
    );
}

#[test]
fn test_set_thumb_texture_fileid_keeps_same_object() {
    let env = env();
    let still_same: bool = env
        .eval(
            r#"
        local s = CreateFrame("Slider")
        local t = s:CreateTexture()
        s:SetThumbTexture(t)
        s:SetThumbTexture(12345)
        return s:GetThumbTexture() == t
    "#,
        )
        .unwrap();
    assert!(
        still_same,
        "SetThumbTexture with fileID should keep same texture object"
    );
}

#[test]
fn test_slider_set_thumb_texture_file_id_get_texture() {
    let env = env();
    let tex_id: i32 = env
        .eval(
            r#"
        local s = CreateFrame("Slider")
        s:SetThumbTexture(12345)
        local t = s:GetThumbTexture()
        return t:GetTexture()
    "#,
        )
        .unwrap();
    assert_eq!(tex_id, 12345);
}

#[test]
fn test_obey_step_on_drag_round_trip() {
    let env = env();
    let obeys: (bool, bool, bool) = env
        .eval(
            r#"
        local s = CreateFrame("Slider")
        local initial = s:GetObeyStepOnDrag()
        s:SetObeyStepOnDrag(true)
        local enabled = s:GetObeyStepOnDrag()
        s:SetObeyStepOnDrag(false)
        local disabled = s:GetObeyStepOnDrag()
        return initial, enabled, disabled
    "#,
        )
        .unwrap();
    assert_eq!(
        obeys,
        (false, true, false),
        "GetObeyStepOnDrag should round-trip the persisted slider flag"
    );
}
