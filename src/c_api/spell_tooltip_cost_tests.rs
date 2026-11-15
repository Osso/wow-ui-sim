//! WM98 outer eligibility, exercised through both public data getters.
use super::spell_tooltip_cost::SpellTooltipCostContext;
use crate::lua_api::WowLuaEnv;

fn has_cost(env: &WowLuaEnv, spell_id: u32) -> (bool, bool) {
    env.eval(&format!(
        r#"
        local function hasCost(data)
            for _, line in ipairs(data.lines) do
                if line.leftText and line.leftText:find('MANA') then return true end
            end
            return false
        end
        return hasCost(C_TooltipInfo.GetSpellByID({spell_id})),
            hasCost(C_TooltipInfo.GetHyperlink('spell:{spell_id}'))
    "#
    ))
    .unwrap()
}

#[test]
fn spell_cost_eligibility_matches_wm98() {
    let env = WowLuaEnv::new().unwrap();
    // R81 Q2: final resource helper display text is not a computed percentage.
    assert_eq!(has_cost(&env, 19750), (false, false));
    env.state()
        .borrow_mut()
        .spell_tooltip_inputs
        .entry(19750)
        .or_default()
        .cost = Some("host MANA cost".into());
    for (mode, suppress_content, attributes_9, expected) in [
        (0, false, 0, true),
        (1, false, 0, false),
        (2, false, 0, true),
        (0, true, 0, false),
        (0, false, 32768, false),
        (0, false, 65536, true),
    ] {
        env.state().borrow_mut().spell_tooltip_cost_contexts.insert(
            19750,
            SpellTooltipCostContext {
                mode,
                suppress_content,
                attributes_9: Some(attributes_9),
            },
        );
        assert_eq!(
            has_cost(&env, 19750),
            (expected, expected),
            "mode={mode} suppressed={suppress_content} attributes_9={attributes_9}"
        );
    }
    env.state().borrow_mut().spell_tooltip_cost_contexts.clear();
    assert_eq!(has_cost(&env, 19750), (true, true));
}

#[test]
fn spells_without_resource_fixture_do_not_manufacture_costs() {
    let env = WowLuaEnv::new().unwrap();
    assert_eq!(has_cost(&env, 100), (false, false));
    assert_eq!(has_cost(&env, 116), (false, false));
}
