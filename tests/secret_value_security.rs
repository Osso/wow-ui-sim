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

#[cfg(feature = "retail-12-1-0")]
#[test]
fn explicit_secret_aspects_union_with_existing_derived_state() {
    let env = env();
    env.exec("SecretAspectProtectedFrame = CreateFrame('Frame', 'SecretAspectProtectedFrame')")
        .expect("create protected-state fixture");
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
        local frame = CreateFrame("Frame")
        frame:SetPreventSecretValues(true)
        assert(frame:HasSecretAspect(1) and frame:HasAnySecretAspect())
        assert(frame:HasSecretValues() and frame:IsAnchoringSecret())
        frame:AddSecretAspect(8)
        assert(frame:HasSecretAspect(1) and frame:HasSecretAspect(8))
        frame:SetPreventSecretValues(false)
        assert(not frame:HasSecretAspect(1))
        assert(frame:HasSecretAspect(8) and frame:HasSecretValues())
        assert(not frame:IsAnchoringSecret())

        local protected = SecretAspectProtectedFrame
        assert(protected:IsProtected())
        assert(protected:HasSecretAspect(1) and protected:HasAnySecretAspect())
        assert(not protected:HasSecretValues())
        protected:AddSecretAspect(32768)
        assert(protected:HasSecretAspect(1) and protected:HasSecretAspect(32768))
        assert(protected:HasSecretValues() and protected:IsProtected())

        local forbidden = CreateFrame("Frame")
        forbidden:SetForbidden()
        assert(forbidden:HasSecretAspect(1) and forbidden:HasAnySecretAspect())
        assert(not forbidden:HasSecretValues())
        forbidden:AddSecretAspect(8)
        assert(forbidden:HasSecretAspect(1) and forbidden:HasSecretAspect(8))
        assert(forbidden:HasSecretValues() and forbidden:IsForbidden())
        "#,
    )
    .expect("explicit masks do not replace protected, forbidden, or prevent-secret state");
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
