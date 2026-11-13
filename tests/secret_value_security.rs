use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn addon_tainted_function_is_not_secret() {
    let env = env();
    let (secret, inserted): (bool, bool) = env
        .eval(
            r#"
            local f = function() end
            debug.setobjecttaint(f, "TestAddon")
            local array = SecureTypes.CreateSecureArray()
            local ok = pcall(function() array:Insert(f) end)
            return issecretvalue(f), ok and array[1] == f
            "#,
        )
        .unwrap();

    assert!(
        !secret,
        "addon-tainted closures should not be secret values"
    );
    assert!(inserted, "SecureArray should accept addon-tainted closures");
}

#[test]
fn secure_loadstring_function_is_not_secret() {
    let env = env();
    let (secret, inserted): (bool, bool) = env
        .eval(
            r#"
            local f = loadstring("return 1")
            local array = SecureTypes.CreateSecureArray()
            local ok = pcall(function() array:Insert(f) end)
            return issecretvalue(f), ok and array[1] == f
            "#,
        )
        .unwrap();

    assert!(
        !secret,
        "secure loadstring closures should not be secret values"
    );
    assert!(
        inserted,
        "SecureArray should accept secure generated closures"
    );
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn explicit_secret_aspects_accumulate_without_affecting_other_frames() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame")
        local other = CreateFrame("Frame")
        assert(not frame:HasAnySecretAspect())
        assert(not frame:HasSecretValues())
        assert(not frame:HasSecretAspect(8))

        frame:AddSecretAspect(8) -- SecretAspect.Text
        assert(frame:HasSecretAspect(8))
        assert(not frame:HasSecretAspect(32768)) -- SecretAspect.Cooldown
        assert(frame:HasAnySecretAspect() and frame:HasSecretValues())
        assert(not frame:IsAnchoringRestricted() and not frame:IsAnchoringSecret())

        frame:AddSecretAspect(32768)
        frame:AddSecretAspect(8)
        frame:AddSecretAspect(0)
        assert(frame:HasSecretAspect(8) and frame:HasSecretAspect(32768))
        assert(not frame:HasSecretAspect(2) and not frame:HasSecretAspect(0))
        assert(not other:HasAnySecretAspect() and not other:HasSecretValues())

        frame:AddSecretAspect(8388608) -- SecretAspect.RadialProgress
        assert(frame:HasSecretAspect(8388608))
        assert(frame:HasSecretAspect(8) and frame:HasSecretAspect(32768))
        "#,
    )
    .expect("explicit secret aspects retain each added bit independently");
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-0"))]
#[test]
fn explicit_secret_aspects_reflect_recorded_bits_on_retail() {
    let env = env();
    env.exec(
        "SecretAspectProtectedFrame = CreateFrame('Frame', 'SecretAspectProtectedFrame') \
         SecretAspectPreventFrame = CreateFrame('Frame', 'SecretAspectPreventFrame') \
         SecretAspectEmptyPreventFrame = CreateFrame('Frame', 'SecretAspectEmptyPreventFrame')",
    )
    .expect("create protected-state and prevent-secret fixtures");
    env.exec("SecretAspectPreventFrame:AddSecretAspect(32768)")
        .expect("seed an existing recorded aspect");
    // SetPreventSecretValues was removed in 12.0.1; prevent-secret state is host input.
    let set_prevent = |env: &wow_ui_sim::lua_api::WowLuaEnv, name: &str, on: bool| {
        let mut state = env.state().borrow_mut();
        let id = state.widgets.get_id_by_name(name).unwrap();
        state.widgets.get_mut(id).unwrap().prevent_secret_values = on;
    };
    set_prevent(&env, "SecretAspectPreventFrame", true);
    set_prevent(&env, "SecretAspectEmptyPreventFrame", true);
    {
        let mut state = env.state().borrow_mut();
        let id = state
            .widgets
            .get_id_by_name("SecretAspectProtectedFrame")
            .unwrap();
        state.widgets.get_mut(id).unwrap().is_protected = true;
    }
    env.exec(
        r#"
        -- VERIFIED — Retail WM61 S6: aspect queries read recorded sets. Prevention
        -- blocks additions and preserves recorded bits; see docs/specs/secret-auras.md.
        local frame = SecretAspectPreventFrame
        assert(not frame:HasSecretAspect(1))
        assert(frame:HasSecretAspect(32768) and frame:HasAnySecretAspect())
        pcall(frame.AddSecretAspect, frame, 8)
        assert(frame:HasSecretAspect(32768) and not frame:HasSecretAspect(8))

        local empty = SecretAspectEmptyPreventFrame
        assert(not empty:HasSecretAspect(1) and not empty:HasAnySecretAspect())
        pcall(empty.AddSecretAspect, empty, 8)
        assert(not empty:HasSecretAspect(8) and not empty:HasAnySecretAspect())

        local protected = SecretAspectProtectedFrame
        assert(protected:IsProtected())
        assert(not protected:HasSecretAspect(1) and not protected:HasAnySecretAspect())
        protected:AddSecretAspect(32768)
        assert(not protected:HasSecretAspect(1) and protected:HasSecretAspect(32768))
        "#,
    )
    .expect("recorded aspects remain separate from prevention and protection");
    set_prevent(&env, "SecretAspectPreventFrame", false);
    set_prevent(&env, "SecretAspectEmptyPreventFrame", false);
    env.exec(
        r#"
        local frame = SecretAspectPreventFrame
        frame:AddSecretAspect(8)
        frame:AddSecretAspect(8)
        assert(frame:HasSecretAspect(32768) and frame:HasSecretAspect(8))

        local empty = SecretAspectEmptyPreventFrame
        empty:AddSecretAspect(8)
        assert(empty:HasSecretAspect(8) and empty:HasAnySecretAspect())

        local protected = SecretAspectProtectedFrame
        assert(protected:IsProtected())
        assert(protected:HasSecretAspect(32768))
        local forbidden = CreateFrame("Frame")
        forbidden:SetForbidden()
        assert(forbidden:IsForbidden())
        forbidden:AddSecretAspect(8)
        assert(forbidden:HasSecretAspect(8) and forbidden:IsForbidden())
        "#,
    )
    .expect("explicit aspect writes succeed after prevention is disabled");
}

#[cfg(all(
    feature = "optional-tests",
    feature = "client-retail",
    feature = "retail-12-1-0"
))]
#[test]
fn secret_value_state_correlations_remain_unknown() {
    let env = env();
    env.exec(
        "SecretAspectProtectedFrame = CreateFrame('Frame', 'SecretAspectProtectedFrame') \
         SecretAspectPreventFrame = CreateFrame('Frame', 'SecretAspectPreventFrame')",
    )
    .expect("create security-state fixtures");
    let set_prevent = |env: &wow_ui_sim::lua_api::WowLuaEnv, on: bool| {
        let mut state = env.state().borrow_mut();
        let id = state
            .widgets
            .get_id_by_name("SecretAspectPreventFrame")
            .unwrap();
        state.widgets.get_mut(id).unwrap().prevent_secret_values = on;
    };
    set_prevent(&env, true);
    {
        let mut state = env.state().borrow_mut();
        let id = state
            .widgets
            .get_id_by_name("SecretAspectProtectedFrame")
            .unwrap();
        state.widgets.get_mut(id).unwrap().is_protected = true;
    }
    env.exec(
        r#"
        -- UNKNOWN — WM61 S6 / R18 do not establish these query/state relationships.
        local frame = SecretAspectPreventFrame
        assert(frame:HasSecretValues())
        assert(frame:IsAnchoringSecret())
        "#,
    )
    .expect("recorded UNKNOWN prevention correlations");
    set_prevent(&env, false);
    env.exec(
        r#"
        -- UNKNOWN — WM61 S6 / R18 do not establish these query/state relationships.
        assert(not SecretAspectPreventFrame:IsAnchoringSecret())
        local protected = SecretAspectProtectedFrame
        assert(not protected:HasSecretValues())
        protected:AddSecretAspect(32768)
        assert(protected:HasSecretValues())
        local forbidden = CreateFrame("Frame")
        forbidden:SetForbidden()
        assert(not forbidden:HasSecretValues())
        forbidden:AddSecretAspect(8)
        assert(forbidden:HasSecretValues())
        "#,
    )
    .expect("recorded UNKNOWN protected and forbidden correlations");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn explicit_secret_aspects_are_supported_on_aura_presentation_regions() {
    let env = env();
    env.exec(
        r#"
        local parent = CreateFrame("Frame")
        local cooldown = CreateFrame("Cooldown", nil, parent)
        local texture = parent:CreateTexture()
        local text = parent:CreateFontString()
        for _, object in ipairs({ cooldown, texture, text }) do
            object:AddSecretAspect(32) -- SecretAspect.Shown
            assert(object:HasSecretAspect(32))
            assert(object:HasAnySecretAspect() and object:HasSecretValues())
        end
        assert(not parent:HasAnySecretAspect() and not parent:HasSecretValues())
        "#,
    )
    .expect("aura presentation objects have independent explicit secret masks");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn custom_aura_button_update_mode_is_published_before_secure_environment_copy() {
    let env = env();
    env.exec(
        r#"
        for _, enums in ipairs({ Enum, __secureenv.Enum }) do
            local mode = enums.CustomAuraButtonUpdateMode
            assert(type(mode) == "table")
            assert(mode.Assignment == 0 and mode.Update == 1)
            local metadata = enums.CustomAuraButtonUpdateModeMeta
            assert(metadata.MinValue == 0)
            assert(metadata.MaxValue == 1)
            assert(metadata.NumValues == 2)
        end
        "#,
    )
    .expect("public and secure consumers receive the native update-mode enum");
}
