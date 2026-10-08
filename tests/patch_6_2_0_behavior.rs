//! Spell resource-cost policy from the pinned 6.2.0 page, not native parity.
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_6_2_0_spell_link_excludes_cost(env: &WowLuaEnv) {
    check_spell_link_cost(env);
}
}

#[test]
fn patch_6_2_0_spell_link_excludes_cost_bare() {
    let env = WowLuaEnv::new().unwrap();
    check_spell_link_cost(&env);
}

fn check_spell_link_cost(env: &WowLuaEnv) {
    env.exec(r#"
        local tooltip = CreateFrame('GameTooltip', 'P620Tooltip', UIParent, 'GameTooltipTemplate')
        local function textLines()
            local lines = {}
            for i = 1, tooltip:NumLines() do
                lines[#lines + 1] = tooltip:GetLeftLine(i):GetText()
            end
            return table.concat(lines, '\n')
        end
        tooltip:SetSpellByID(19750)
        local direct = textLines()
        assert(direct:find('Mana'), direct)
        for _, link in ipairs({'spell:19750', '|Hspell:19750|h[Flash of Light]|h'}) do
            tooltip:SetHyperlink(link)
            local linked = textLines()
            assert(linked:find('Flash of Light'), linked)
            assert(not linked:find('Mana'), linked)
            local name, id = tooltip:GetSpell()
            assert(name == 'Flash of Light' and id == 19750)
        end
        tooltip:SetSpellByID(19750)
        assert(textLines():find('Mana'), textLines())
    "#).unwrap();
}
