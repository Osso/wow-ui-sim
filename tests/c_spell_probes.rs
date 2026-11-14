//! Integration tests for spell lookup and related C_Spell queries.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn spell_subtext_returns_catalog_text_and_distinguishes_empty_from_unknown() {
    let env = env();
    env.exec(
        r#"
        local getSubtext = assert(rawget(C_Spell, "GetSpellSubtext"))
        assert(getSubtext(395296) == "Black")
        assert(getSubtext("ebon might(Black)") == "Black")
        assert(getSubtext(C_Spell.GetSpellLink(395296)) == "Black")
        assert(getSubtext("Brown Horse") == "")
        assert(select('#', getSubtext(458)) == 1)
        assert(select('#', getSubtext(999999999)) == 0)
        assert(select('#', getSubtext("Ebon Might(Unknown)")) == 0)
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn spell_subtext_returns_rank_from_forever_spell_data() {
    let env = env();
    env.exec(
        r#"
        local getSubtext = assert(C_Spell.GetSpellSubtext)
        assert(getSubtext(467) == "Rank 1")
        assert(getSubtext(782) == "Rank 2")
        assert(getSubtext("Thorns(Rank 2)") == "Rank 2")
        "#,
    )
    .unwrap();
}

fn formatted_player_attack_power_damage(env: &WowLuaEnv, formula: &str) -> String {
    let lua = format!(
        r#"
        local function fmt(v)
            if math.abs(v) >= 100 or math.abs(v - math.floor(v + 0.5)) < 0.001 then
                return tostring(math.floor(v + 0.5))
            end
            return string.format('%.1f', v)
        end
        local ap = UnitAttackPower('player')
        return fmt({formula})
        "#
    );
    env.eval(&lua).unwrap()
}

// ── GetSpellInfo ─────────────────────────────────────────────────────────────

#[test]
fn test_get_spell_info_known_returns_table() {
    let env = env();
    // Spell 116 = Frostbolt (exists in crates/sim-data/data/spells.rs)
    let name: String = env
        .eval("local info = C_Spell.GetSpellInfo(116); return info.name")
        .unwrap();
    assert_eq!(name, "Frostbolt");
}

#[test]
fn test_get_spell_info_has_required_fields() {
    let env = env();
    let (icon, spell_id, cast_time): (i64, i64, f64) = env
        .eval(
            "local info = C_Spell.GetSpellInfo(116)
             return info.iconID, info.spellID, info.castTime",
        )
        .unwrap();
    assert!(icon > 0, "iconID should be nonzero, got {icon}");
    assert_eq!(spell_id, 116);
    assert!((cast_time - 0.0).abs() < 0.001);
}

#[test]
fn test_get_spell_info_ebon_might_self_buff() {
    let env = env();
    let (name, icon_id): (String, i64) = env
        .eval(
            r#"
            local info = C_Spell.GetSpellInfo(395296)
            return info.name, info.iconID
            "#,
        )
        .unwrap();
    assert_eq!(name, "Ebon Might");
    assert_eq!(icon_id, 5061347);
}

#[test]
fn test_get_spell_info_includes_cell_default_indicator_spells() {
    let env = env();
    let names: Vec<String> = env
        .eval(
            r#"
            local ids = {
                377509, -- Dream Projection
                52042,  -- Healing Stream Totem
                170906, -- Food & Drink
                167152, -- Refreshment
                430,    -- Drink
                43182,  -- Drink
                172786, -- Drink
                308433, -- Food & Drink
                369162, -- Drink
                456574, -- Cinder Nectar
                461063, -- Quiet Contemplation
                132403, -- Shield of the Righteous
                132404, -- Shield Block
            }
            local names = {}
            for i, spellID in ipairs(ids) do
                local info = C_Spell.GetSpellInfo(spellID)
                names[i] = info and info.name or ""
            end
            return names
            "#,
        )
        .unwrap();

    assert_eq!(
        names,
        vec![
            "Dream Projection",
            "Healing Stream Totem",
            "Food & Drink",
            "Refreshment",
            "Drink",
            "Drink",
            "Drink",
            "Food & Drink",
            "Drink",
            "Cinder Nectar",
            "Quiet Contemplation",
            "Shield of the Righteous",
            "Shield Block",
        ]
    );
}

#[test]
fn test_get_spell_info_includes_cell_click_casting_resurrections() {
    let env = env();
    let names: Vec<String> = env
        .eval(
            r#"
            local ids = {
                61999,  -- Raise Ally
                20484,  -- Rebirth
                361227, -- Return
                115178, -- Resuscitate
                391054, -- Intercession
                2006,   -- Resurrection
                2008,   -- Ancestral Spirit
                20707,  -- Soulstone
            }
            local names = {}
            for i, spellID in ipairs(ids) do
                local info = C_Spell.GetSpellInfo(spellID)
                names[i] = info and info.name or ""
            end
            return names
            "#,
        )
        .unwrap();

    assert_eq!(
        names,
        vec![
            "Raise Ally",
            "Rebirth",
            "Return",
            "Resuscitate",
            "Intercession",
            "Resurrection",
            "Ancestral Spirit",
            "Soulstone",
        ]
    );
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_supplemental_spell_info_preserves_catalog_metadata() {
    for (id, name, icon, target) in [
        (24705, "Invocation of the Wickerman", 135462, 21),
        (30754, "Cenarion Favor", 136086, 25),
    ] {
        let spell = wow_ui_sim::spell_lookup::get_spell(id).expect("native spell must resolve");
        assert_eq!(spell.name, name);
        assert_eq!(spell.subtext, "");
        assert_eq!(spell.icon_file_data_id, icon);
        assert_eq!(spell.school_mask, 1);
        assert_eq!(spell.implicit_target, target);
    }
    let env = env();
    env.exec(
        r#"
        for _, expected in ipairs({
            {24705, 'Invocation of the Wickerman', 135462},
            {30754, 'Cenarion Favor', 136086},
        }) do
            local info = C_Spell.GetSpellInfo(expected[1])
            assert(info.name == expected[2] and info.spellID == expected[1])
            assert(info.iconID == expected[3])
            assert(C_Spell.GetSpellPowerCost(expected[1]) == nil)
            local name, subtext, icon = GetSpellInfo(expected[1])
            assert(name == expected[2] and subtext == nil and icon == expected[3])
        end
        assert(C_Spell.GetSpellInfo(999999999) == nil)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_supplemental_spell_info_supports_installed_wowhead_lookup() {
    use std::path::{Path, PathBuf};
    use wow_ui_sim::screen::ScreenKind;
    use wow_ui_sim::toc::TocFile;

    let Some(addon_dir) = std::env::var_os("WOWHEAD_LOOTER_PATH").map(PathBuf::from) else {
        eprintln!(
            "skipping installed Wowhead consumer: set WOWHEAD_LOOTER_PATH to the addon directory"
        );
        return;
    };
    let toc_path = wow_ui_sim::loader::find_toc_file(&addon_dir)
        .expect("installed Wowhead addon must have a profile-selected TOC");
    let toc = TocFile::from_file(&toc_path).expect("parse selected Wowhead TOC");
    assert_eq!(
        toc_path.file_name().and_then(|s| s.to_str()),
        Some("+Wowhead_Looter.toc")
    );
    assert_eq!(
        toc.metadata.get("Interface").map(String::as_str),
        Some("16001")
    );
    assert!(toc.allows_screen(ScreenKind::Game));
    let index = toc
        .files
        .iter()
        .position(|p| p == Path::new("Wowhead_Looter.xml"))
        .expect("selected TOC must include the original XML");
    assert!(
        toc.file_screen_allows[index]
            .as_deref()
            .is_none_or(|screen| screen.eq_ignore_ascii_case("game"))
    );
    let xml = std::fs::read_to_string(addon_dir.join("Wowhead_Looter.xml")).unwrap();
    assert!(xml.contains("<Script file=\"Wowhead_Looter.lua\" />"));
    let source = std::fs::read_to_string(addon_dir.join("Wowhead_Looter.lua")).unwrap();
    let start = source
        .find("local GetSpellInfo = GetSpellInfo or function(spell)")
        .expect("original native spell wrapper must exist");
    let table_start = source[start..].find("local WL_REP_MODS = {").unwrap() + start;
    let end = table_start + source[table_start..].find("\n};").unwrap() + 3;
    let original = &source[start..end];
    assert!(original.contains("[GetSpellInfo(24705)]"));
    assert!(original.contains("[GetSpellInfo(30754)]"));
    let chunk = format!("{original}\nreturn WL_REP_MODS");
    let mut separator = String::from("=");
    while chunk.contains(&format!("]{separator}]")) {
        separator.push('=');
    }
    let env = env();
    env.exec(&format!(
        "WowheadReputationSpellMap = assert(loadstring([{separator}[{chunk}]{separator}], '@+Wowhead_Looter/Wowhead_Looter.lua'))()"
    )).expect("unchanged selected reputation table must have non-nil distinct spell-name keys");
    env.exec(
        r#"
        local map = WowheadReputationSpellMap
        assert(map['Invocation of the Wickerman'][2] == 0.1)
        assert(map['Cenarion Favor'][2] == 0.25)
        local count = 0
        for name in pairs(map) do
            assert(type(name) == 'string')
            count = count + 1
        end
        assert(count == 2, 'constant spell names must not collapse the original keys')
        local first = C_Spell.GetSpellInfo(24705)
        local second = C_Spell.GetSpellInfo(30754)
        assert(first.name == 'Invocation of the Wickerman' and first.spellID == 24705)
        assert(second.name == 'Cenarion Favor' and second.spellID == 30754)
        assert(GetSpellInfo(24705) == first.name)
        assert(GetSpellInfo(30754) == second.name)
        assert(C_Spell.GetSpellInfo(999999999) == nil)
    "#,
    )
    .unwrap();
}

#[test]
fn test_get_spell_info_unknown_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Spell.GetSpellInfo(999999999) == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_get_spell_info_original_icon_matches_icon() {
    let env = env();
    let matches: bool = env
        .eval(
            "local info = C_Spell.GetSpellInfo(116)
             return info.iconID == info.originalIconID",
        )
        .unwrap();
    assert!(matches);
}

#[test]
fn test_get_spell_description_resolves_spell_placeholders() {
    let env = env();
    let avengers_shield_damage = formatted_player_attack_power_damage(&env, "ap * 1.55");
    let desc: String = env
        .eval("return C_Spell.GetSpellDescription(31935)")
        .unwrap();

    assert!(
        desc.contains(&format!("{avengers_shield_damage} Holy damage")),
        "spell description should resolve AP-scaled damage placeholders: expected {avengers_shield_damage}, got: {desc}"
    );
    assert!(
        !desc.contains('$'),
        "spell description should not expose raw Blizzard placeholders, got: {desc}"
    );
}

#[test]
fn test_get_spell_description_resolves_named_dmg_variable() {
    let env = env();
    let eye_beam_damage = formatted_player_attack_power_damage(&env, "ap * 0.4026 * 10");
    let desc: String = env
        .eval("return C_Spell.GetSpellDescription(198013)")
        .unwrap();

    assert!(
        desc.contains(&format!("{eye_beam_damage} Chaos damage")),
        "spell description should resolve $<dmg> variables from SimC formulas, got: {desc}"
    );
    assert!(
        !desc.contains("$<dmg>"),
        "spell description should not expose raw $<dmg> placeholders, got: {desc}"
    );
}

#[test]
fn test_get_spell_description_resolves_consecration_dmg_expression() {
    let env = env();
    let consecration_damage = formatted_player_attack_power_damage(&env, "ap * 0.05 * 12 * 1.05");
    let desc: String = env
        .eval("return C_Spell.GetSpellDescription(26573)")
        .unwrap();

    assert!(
        desc.contains(&format!("{consecration_damage} Holy damage")),
        "Consecration should resolve nested $<dmg> expression; expected {consecration_damage}, got: {desc}"
    );
    assert!(
        !desc.contains("Radiant"),
        "Consecration should choose the inactive Burning Crusade false branch, got: {desc}"
    );
    assert!(
        !desc.contains("$<dmg>") && !desc.contains("${"),
        "Consecration should not expose raw expression placeholders, got: {desc}"
    );
    assert!(
        desc.contains("Limit 1."),
        "Consecration should use its area-trigger limit value, got: {desc}"
    );
}

#[test]
fn test_get_spell_description_resolves_hammer_of_justice_duration() {
    let env = env();
    let desc: String = env.eval("return C_Spell.GetSpellDescription(853)").unwrap();

    assert_eq!(desc, "Stuns the target for 6.");
}

#[test]
fn test_get_spell_description_resolves_main_action_bar_durations() {
    let env = env();
    let duration_snippets: Vec<String> = env
        .eval(
            r#"
            local ids = {62124, 31850, 86659, 642}
            local snippets = {}
            for _, spellID in ipairs(ids) do
                local desc = C_Spell.GetSpellDescription(spellID)
                table.insert(snippets, string.match(desc, "for %d+"))
            end
            return snippets
            "#,
        )
        .unwrap();

    assert_eq!(duration_snippets, vec!["for 6", "for 8", "for 8", "for 8"]);
}

// ── GetSpellCooldown ─────────────────────────────────────────────────────────

#[test]
fn test_get_spell_cooldown_returns_table() {
    let env = env();
    let is_table: bool = env
        .eval("return type(C_Spell.GetSpellCooldown(12345)) == 'table'")
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_get_spell_cooldown_no_cooldown_has_zero_duration() {
    let env = env();
    let duration: f64 = env
        .eval(
            "local info = C_Spell.GetSpellCooldown(12345)
             return info.duration",
        )
        .unwrap();
    assert!(
        (duration - 0.0).abs() < 0.01,
        "fresh spell should have duration 0, got {duration}"
    );
}

#[test]
fn test_get_spell_cooldown_is_enabled_true() {
    let env = env();
    let enabled: bool = env
        .eval(
            "local info = C_Spell.GetSpellCooldown(12345)
             return info.isEnabled",
        )
        .unwrap();
    assert!(enabled);
}

#[test]
fn test_get_spell_cooldown_mod_rate_is_one() {
    let env = env();
    let mod_rate: f64 = env
        .eval(
            "local info = C_Spell.GetSpellCooldown(12345)
             return info.modRate",
        )
        .unwrap();
    assert!((mod_rate - 1.0).abs() < 0.001);
}

#[test]
fn test_get_spell_cooldown_active_after_set() {
    let env = env();
    let (duration, is_active): (f64, bool) = env
        .eval(
            "A_Admin.SetSpellCooldown(42, 8.0)
             local info = C_Spell.GetSpellCooldown(42)
             return info.duration, info.isActive",
        )
        .unwrap();
    assert!(
        (duration - 8.0).abs() < 0.01,
        "cooldown duration should be 8.0, got {duration}"
    );
    assert!(is_active, "isActive should be true when duration > 0");
}

// ── Empty count domains ──────────────────────────────────────────────────────

#[test]
fn test_spell_count_missing_data_matches_epoch_domains() {
    let env = env();
    let cast_count: i64 = env.eval("return C_Spell.GetSpellCastCount(116)").unwrap();
    assert_eq!(cast_count, 0);
    #[cfg(feature = "retail-12-0-5")]
    {
        let display_count: String = env
            .eval("return C_Spell.GetSpellDisplayCount(116, 99)")
            .unwrap();
        assert_eq!(display_count, "");
    }
    #[cfg(not(feature = "retail-12-0-5"))]
    {
        let display_count: i64 = env
            .eval("return C_Spell.GetSpellDisplayCount(116, 99)")
            .unwrap();
        assert_eq!(display_count, 0);
    }
}

// ── GetMountFromSpell ─────────────────────────────────────────────────────────

#[test]
fn test_get_mount_from_spell_known_mount_spell() {
    let env = env();
    // Mount ID 6 ("Brown Horse") uses spell_id 458 in state_defaults.rs
    let mount_id: i64 = env.eval("return C_Spell.GetMountFromSpell(458)").unwrap();
    assert_eq!(mount_id, 6);
}

#[test]
fn test_get_mount_from_spell_unknown_returns_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_Spell.GetMountFromSpell(999999) == nil")
        .unwrap();
    assert!(is_nil);
}

// ── GetVisibilityInfo ─────────────────────────────────────────────────────────

#[cfg(feature = "client-retail")]
#[test]
fn test_get_visibility_info_is_registered() {
    let env = env();
    let is_function: bool = env
        .eval("return type(C_Spell.GetVisibilityInfo) == 'function'")
        .unwrap();
    assert!(is_function, "Retail registers C_Spell.GetVisibilityInfo");
}

// ── IsPriorityAura ────────────────────────────────────────────────────────────

#[cfg(feature = "client-retail")]
#[test]
fn test_is_priority_aura_returns_false_for_zero_id() {
    let env = env();
    let result: bool = env.eval("return C_Spell.IsPriorityAura(0)").unwrap();
    assert!(!result);
}

// ── Temporary classification shims ───────────────────────────────────────────

#[cfg(feature = "client-retail")]
#[test]
fn test_is_spell_passive_returns_false_for_zero_id() {
    let env = env();
    let result: bool = env.eval("return C_Spell.IsSpellPassive(0)").unwrap();
    assert!(!result);
}

// ── IsSelfBuff ────────────────────────────────────────────────────────────────

#[test]
fn test_is_self_buff_true_for_self_target_spell() {
    let env = env();
    // Spell 1272138 "Keen Edge" has implicit_target=1 (self)
    let result: bool = env.eval("return C_Spell.IsSelfBuff(1272138)").unwrap();
    assert!(result, "spell with implicit_target=1 should be a self-buff");
}

#[test]
fn test_is_self_buff_false_for_enemy_spell() {
    let env = env();
    // Spell 116 "Frostbolt" has implicit_target=6 (enemy)
    let result: bool = env.eval("return C_Spell.IsSelfBuff(116)").unwrap();
    assert!(!result, "enemy-targeted spell should not be a self-buff");
}

#[test]
fn test_is_self_buff_false_for_unknown_spell() {
    let env = env();
    let result: bool = env.eval("return C_Spell.IsSelfBuff(999999999)").unwrap();
    assert!(!result);
}

// ── IsSpellUsable ─────────────────────────────────────────────────────────────

#[test]
fn test_is_spell_usable_false_for_unknown_spell() {
    let env = env();
    let (usable, insufficient): (bool, bool) = env
        .eval(
            "local u, ip = C_Spell.IsSpellUsable(99999)
             return u, ip",
        )
        .unwrap();
    assert!(!usable, "unknown spell should not be usable");
    assert!(!insufficient);
}

#[test]
fn test_is_spell_usable_true_for_known_spell() {
    let env = env();
    env.state().borrow_mut().known_spells.insert(12345);
    let (usable, insufficient): (bool, bool) = env
        .eval(
            "local u, ip = C_Spell.IsSpellUsable(12345)
             return u, ip",
        )
        .unwrap();
    assert!(usable, "known spell should be usable");
    assert!(!insufficient);
}

// ── TargetSpell stubs ─────────────────────────────────────────────────────────

#[test]
fn test_target_spell_is_enchanting_false() {
    let env = env();
    let result: bool = env
        .eval("return C_Spell.TargetSpellIsEnchanting()")
        .unwrap();
    assert!(!result);
}

#[test]
fn test_target_spell_jumps_upgrade_track_false() {
    let env = env();
    let result: bool = env
        .eval("return C_Spell.TargetSpellJumpsUpgradeTrack()")
        .unwrap();
    assert!(!result);
}

#[test]
fn test_target_spell_replaces_bonus_tree_false() {
    let env = env();
    let result: bool = env
        .eval("return C_Spell.TargetSpellReplacesBonusTree()")
        .unwrap();
    assert!(!result);
}
