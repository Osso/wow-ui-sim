#![cfg(feature = "aura-containers")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn spell_aura_secrecy_uses_native_attribute_flags_and_contextual_default() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_Secrets.GetSpellAuraSecrecy(1126) == Enum.SecrecyLevel.NeverSecret)
        assert(C_Secrets.GetSpellAuraSecrecy(343960) == Enum.SecrecyLevel.AlwaysSecret)
        assert(C_Secrets.GetSpellAuraSecrecy(35395) == Enum.SecrecyLevel.ContextuallySecret)
        assert(Enum.SecrecyLevel.NeverSecret == 0)
        assert(Enum.SecrecyLevel.AlwaysSecret == 1)
        assert(Enum.SecrecyLevel.ContextuallySecret == 2)
        assert(Enum.SecrecyLevelMeta.MinValue == 0)
        assert(Enum.SecrecyLevelMeta.MaxValue == 2)
        assert(Enum.SecrecyLevelMeta.NumValues == 3)
        "#,
    )
    .expect("base aura secrecy follows the native spell attributes");
    env.exec_maybe_secure(
        "assert(C_Secrets.GetSpellAuraSecrecy(1126) == Enum.SecrecyLevel.NeverSecret)",
        true,
    )
    .expect("namespace and enum are available to secure aura code");
}

#[test]
fn spell_aura_secrecy_reuses_existing_spell_identifier_resolution() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local info = C_Spell.GetSpellInfo("Crusader Strike")
        assert(info and info.spellID, "fixture spell must resolve through C_Spell")
        assert(C_Secrets.GetSpellAuraSecrecy("Crusader Strike") ==
            C_Secrets.GetSpellAuraSecrecy(info.spellID))
        assert(C_Secrets.GetSpellAuraSecrecy("cRuSaDeR sTrIkE") ==
            Enum.SecrecyLevel.ContextuallySecret)
        assert(C_Secrets.GetSpellAuraSecrecy("1126") == Enum.SecrecyLevel.NeverSecret)
        "#,
    )
    .expect("spell names and numeric strings use the modeled resolver");
}

// WM61 S8 establishes AlwaysSecret precedence when both SpellMisc flags are set.
// Retail SecretPredicateAPIDocumentation.lua:45 declares GetSpellAuraSecrecy.
#[test]
fn spell_aura_secrecy_prefers_always_secret_for_both_native_flags() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_Secrets.GetSpellAuraSecrecy(1317008) == Enum.SecrecyLevel.AlwaysSecret)
        assert(C_Secrets.GetSpellAuraSecrecy(1126) == Enum.SecrecyLevel.NeverSecret)
        "#,
    )
    .expect("AlwaysSecret takes precedence over NeverSecret");
}

#[test]
fn spell_aura_secrecy_drives_blizzard_identity_candidate_filtering() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec_maybe_secure(
                    r#"
                    local never = { spellId = 1126, isHelpful = false, isHarmful = true }
                    local contextual = { spellId = 35395, isHelpful = false, isHarmful = true }
                    local always = { spellId = 343960, isHelpful = false, isHarmful = true }
                    assert(AuraContainerUtil.CanApplyIdentityCandidateFilters("player", never))
                    assert(not AuraContainerUtil.CanApplyIdentityCandidateFilters("player", contextual))
                    assert(not AuraContainerUtil.CanApplyIdentityCandidateFilters("player", always))
                    "#,
                    true,
                )
                .expect("actual aura filter honors native never-secret exemptions");
            },
        );
    });
}
