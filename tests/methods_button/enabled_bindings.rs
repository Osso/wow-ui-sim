use wow_ui_sim::lua_api::WowLuaEnv;

const ENABLE_BINDINGS_XML: &str = r#"
<Ui>
    <Button name="ButtonEnablePrecall" intrinsic="true">
        <Scripts>
            <OnEnable intrinsicOrder="precall">ButtonEnableProbe(self, 'pre', true, ...)</OnEnable>
            <OnDisable intrinsicOrder="precall">ButtonEnableProbe(self, 'pre', false, ...)</OnDisable>
        </Scripts>
    </Button>
    <Button name="ButtonEnablePostcall" virtual="true">
        <Scripts>
            <OnEnable intrinsicOrder="postcall">ButtonEnableProbe(self, 'post', true, ...)</OnEnable>
            <OnDisable intrinsicOrder="postcall">ButtonEnableProbe(self, 'post', false, ...)</OnDisable>
        </Scripts>
    </Button>
</Ui>
"#;

fn enabled_binding_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("ButtonEnableBindings.toc");
    std::fs::write(&toc, "## Title: Button enabled bindings\nBindings.xml\n").unwrap();
    std::fs::write(root.path().join("Bindings.xml"), ENABLE_BINDINGS_XML).unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env
}

#[test]
fn enabled_transitions_dispatch_intrinsic_normal_hook_and_post_with_committed_state() {
    enabled_binding_env()
        .exec(
            r#"
        local calls = {}
        local button
        function ButtonEnableProbe(self, binding, enabled, ...)
            assert(self == button and select('#', ...) == 0, 'only self must be passed to scripts')
            assert(self:IsEnabled() == enabled, 'script must see committed enabled state')
            calls[#calls + 1] = binding .. ':' .. tostring(enabled)
        end
        button = CreateFrame('Button', nil, UIParent, 'ButtonEnablePrecall,ButtonEnablePostcall')
        for _, script in ipairs({'OnEnable', 'OnDisable'}) do
            assert(type(button:GetScript(script, 0)) == 'function', 'missing precall binding')
            assert(type(button:GetScript(script, 2)) == 'function', 'missing postcall binding')
            local enabled = script == 'OnEnable'
            button:SetScript(script, function(self, ...)
                ButtonEnableProbe(self, 'normal', enabled, ...)
            end)
            button:HookScript(script, function(self, ...)
                ButtonEnableProbe(self, 'hook', enabled, ...)
            end)
        end
        button:Disable()
        button:Disable()
        button:SetEnabled(false)
        button:Enable()
        button:Enable()
        button:SetEnabled(true)
        button:SetEnabled(false)
        button:SetEnabled(true)
        assert(table.concat(calls, ',') == table.concat({
            'pre:false', 'normal:false', 'hook:false', 'post:false',
            'pre:true', 'normal:true', 'hook:true', 'post:true',
            'pre:false', 'normal:false', 'hook:false', 'post:false',
            'pre:true', 'normal:true', 'hook:true', 'post:true',
        }, ','), 'only transitions must dispatch all bindings in order: ' .. table.concat(calls, ','))
    "#,
        )
        .unwrap();
}

#[test]
fn precall_error_reports_and_continues_normal_hook_and_post() {
    enabled_binding_env().exec(r#"
        local calls, errors = {}, {}
        local button
        seterrorhandler(function(message) errors[#errors + 1] = tostring(message) end)
        function ButtonEnableProbe(self, binding, enabled, ...)
            assert(self == button and select('#', ...) == 0)
            assert(self:IsEnabled() == enabled)
            calls[#calls + 1] = binding
            if binding == 'pre' then error('enabled precall sentinel') end
        end
        button = CreateFrame('Button', nil, UIParent, 'ButtonEnablePrecall,ButtonEnablePostcall')
        assert(type(button:GetScript('OnDisable', 0)) == 'function', 'missing precall binding')
        assert(type(button:GetScript('OnDisable', 2)) == 'function', 'missing postcall binding')
        button:SetScript('OnDisable', function(self, ...) ButtonEnableProbe(self, 'normal', false, ...) end)
        button:HookScript('OnDisable', function(self, ...) ButtonEnableProbe(self, 'hook', false, ...) end)
        button:Disable()
        assert(table.concat(calls, ',') == 'pre,normal,hook,post',
            'error must not stop later bindings: ' .. table.concat(calls, ','))
        assert(#errors == 1 and string.find(errors[1], 'enabled precall sentinel', 1, true),
            'precall error must be reported once')
    "#).unwrap();
}

#[test]
fn hook_only_enabled_transitions_dispatch_without_intrinsic_bindings() {
    WowLuaEnv::new().unwrap().exec(r#"
        local button = CreateFrame('Button', nil, UIParent)
        local calls = {}
        button:HookScript('OnDisable', function(self, ...)
            assert(self == button and select('#', ...) == 0 and not self:IsEnabled())
            calls[#calls + 1] = 'disable'
        end)
        button:HookScript('OnEnable', function(self, ...)
            assert(self == button and select('#', ...) == 0 and self:IsEnabled())
            calls[#calls + 1] = 'enable'
        end)
        button:Disable()
        button:SetEnabled(false)
        button:Enable()
        button:SetEnabled(true)
        assert(table.concat(calls, ',') == 'disable,enable', 'hook-only transitions must dispatch once')
    "#).unwrap();
}
