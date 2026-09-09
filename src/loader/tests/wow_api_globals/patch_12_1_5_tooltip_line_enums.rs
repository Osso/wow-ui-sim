//! Publication only. Retail intentionally retains its actual 45-member layout,
//! which predates the pinned 50-member 12.1.0 baseline; PTR follows the 52-member target.

use crate::lua_api::WowLuaEnv;

fn expected_members() -> Vec<(String, i64)> {
    let register: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../data/patch-api/sources/12.1.5-register.json"
    ))
    .unwrap();
    let parent = register["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["symbol"] == "Enum.TooltipDataLineType")
        .unwrap();
    assert_eq!(parent["before"]["Fields"].as_array().unwrap().len(), 50);
    if cfg!(feature = "client-ptr") {
        return parent["after"]["Fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|field| {
                (
                    field["Name"].as_str().unwrap().to_owned(),
                    field["EnumValue"].as_i64().unwrap(),
                )
            })
            .collect();
    }
    let names = "None Blank UnitName GemSocket AzeriteEssenceSlot AzeriteEssencePower LearnableSpell UnitThreat QuestObjective AzeriteItemPowerDescription RuneforgeLegendaryPowerDescription SellPrice ProfessionCraftingQuality SpellName CurrencyTotal ItemEnchantmentPermanent UnitOwner QuestTitle QuestPlayer NestedBlock ItemBinding RestrictedRaceClass RestrictedFaction RestrictedSkill RestrictedPvPMedal RestrictedReputation RestrictedSpellKnown RestrictedLevel EquipSlot ItemName Separator ToyName ToyText ToyEffect ToyDuration RestrictedArena RestrictedBg ToyFlavorText ToyDescription ToySource GemSocketEnchantment ItemLevel ItemUpgradeLevel SpellPassive SpellDescription";
    names
        .split_whitespace()
        .enumerate()
        .map(|(value, name)| (name.to_owned(), value as i64))
        .collect()
}

fn assert_publication(env: &WowLuaEnv, expected: &[(String, i64)]) {
    for (name, value) in expected {
        let actual: Option<i64> = env
            .eval(&format!("return Enum.TooltipDataLineType.{name}"))
            .unwrap();
        assert_eq!(actual, Some(*value), "member {name}");
    }
    let actual: (i64, i64, i64, i64) = env
        .eval(
            r#"
        local count = 0
        for _ in pairs(Enum.TooltipDataLineType) do count = count + 1 end
        local meta = Enum.TooltipDataLineTypeMeta
        return count, meta.MinValue, meta.MaxValue, meta.NumValues
    "#,
        )
        .unwrap();
    assert_eq!(
        actual,
        (
            expected.len() as i64,
            0,
            expected.len() as i64 - 1,
            expected.len() as i64
        )
    );
    if cfg!(feature = "client-ptr") {
        env.exec(r#"
            for _, name in ipairs({"RestrictedRaceClass", "RestrictedFaction", "RestrictedSkill",
                "RestrictedPvPMedal", "RestrictedReputation", "RestrictedSpellKnown", "RestrictedLevel",
                "RestrictedArena", "RestrictedBg", "ToyFlavorText"}) do
                assert(Enum.TooltipDataLineType[name] == nil, name)
            end
        "#).unwrap();
    }
}

#[test]
fn patch_12_1_5_tooltip_line_enums_publication() {
    let env = WowLuaEnv::new().unwrap();
    let expected = expected_members();
    assert_publication(&env, &expected);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_publication(&env, &expected);
}
