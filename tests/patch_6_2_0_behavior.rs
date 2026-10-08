//! Spell resource-cost policy from the pinned 6.2.0 page, not native parity.
#![cfg(feature = "profile-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_6_2_0_difficulty_identifiers(env: &WowLuaEnv) {
    env.exec(r#"
        assert(DifficultyUtil.ID.DungeonMythic == 23)
        assert(DifficultyUtil.ID.DungeonTimewalker == 24)
    "#).unwrap();
}
}

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
        local directData = C_TooltipInfo.GetSpellByID(19750)
        local linkedData = C_TooltipInfo.GetHyperlink('spell:19750')
        assert(#directData.lines == #linkedData.lines + 1)
        local nextLinked = 1
        for _, line in ipairs(directData.lines) do
            if not line.leftText:find('MANA') then
                assert(line.leftText == linkedData.lines[nextLinked].leftText)
                nextLinked = nextLinked + 1
            end
        end
        tooltip:SetSpellByID(19750)
        local direct = textLines()
        assert(direct:find('MANA'), direct)
        for _, link in ipairs({'spell:19750', '|Hspell:19750|h[Flash of Light]|h'}) do
            tooltip:SetHyperlink(link)
            local linked = textLines()
            assert(linked:find('Flash of Light'), linked)
            assert(not linked:find('MANA'), linked)
            local name, id = tooltip:GetSpell()
            assert(name == 'Flash of Light' and id == 19750)
        end
        tooltip:SetSpellByID(19750)
        assert(textLines():find('MANA'), textLines())
    "#).unwrap();
}
