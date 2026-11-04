//! WM90: supplied data, not claimed Forever 70707 database contents.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::forever_talents::{SpellIcons, TalentGroup, TalentRow};
use wow_ui_sim::lua_api::WowLuaEnv;

fn supplied_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.player.class_index = 2;
        let grid = &mut sim.talents.forever;
        grid.player_present = true;
        grid.allowance = Some(7);
        grid.groups = vec![
            TalentGroup { spec_id: 70, selected_ids: [901].into(), tier_mask: 0 },
            TalentGroup { spec_id: 65, selected_ids: [902].into(), tier_mask: 1 },
        ];
        grid.rows = vec![
            TalentRow { id: 901, class_id: 2, tier_id: 0, column_index: 0, spec_id: 70, spell_id: 9001 },
            TalentRow { id: 902, class_id: 2, tier_id: 0, column_index: 0, spec_id: 65, spell_id: 9002 },
            TalentRow { id: 903, class_id: 2, tier_id: 0, column_index: 1, spec_id: 0, spell_id: 9003 },
        ];
        grid.spell_names.extend([(9001, "Supplied selected".into()), (9002, "Supplied alternate".into()), (9003, "".into())]);
        grid.spell_icons.insert(9001, SpellIcons { visual_icon: 111, misc_icon: 222 });
        grid.spell_icons.insert(9002, SpellIcons { visual_icon: 0, misc_icon: 333 });
    }
    env
}

#[test]
fn forever_talent_frame_consumes_supplied_grid() {
    let env = supplied_env();
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    env.exec(&std::fs::read_to_string(addons.join("Blizzard_FrameXML/Mainline/TalentFrameBase.lua")).unwrap()).unwrap();
    // Only the adjacent tier-unlock API is scaffolded. The query, frames and
    // original TalentFrame_Update body are real; no producer is replaced.
    env.exec(r#"
        function GetTalentTierInfo() return true, 0, 10 end
        function SetItemButtonTexture(button, icon) button.icon:SetTexture(icon) end
        wm90Frame = { talentGroup = 1, talentInfo = {} }
        for tier = 1, Constants.TalentTierConstants.MAX_TALENT_TIERS do
            local row = {}
            wm90Frame['tier'..tier] = row
            for column = 1, Constants.TalentConsts.NumTalentColumns do
                local b = CreateFrame('Button')
                b.icon = b:CreateTexture()
                b.name = b:CreateFontString()
                b.Cover = b:CreateTexture()
                b.highlight = b:CreateTexture()
                row['talent'..column] = b
            end
        end
        TalentFrame_Update(wm90Frame, 'player')
        local a, b = wm90Frame.tier1.talent1, wm90Frame.tier1.talent2
        assert(a:IsShown(), 'WM90 populated cell was hidden')
        assert(a:GetID() == 901 and a.name:GetText() == 'Supplied selected')
        assert(a.icon:GetTexture() == 111 and not a.shouldGlow)
        assert(b:IsShown() and b:GetID() == 903 and b.shouldGlow)
        assert(not wm90Frame.tier1.talent3:IsShown())
    "#).unwrap();
    env.state().borrow_mut().talents.forever.current_group = 1;
    env.exec(r#"
        wm90Frame.talentGroup = 2
        TalentFrame_Update(wm90Frame, 'player')
        local a = wm90Frame.tier1.talent1
        assert(a:IsShown() and a:GetID() == 902)
        assert(a.name:GetText() == 'Supplied alternate' and a.icon:GetTexture() == 333)
    "#).unwrap();
}

#[test]
fn forever_talent_record_and_current_group_semantics() {
    let env = supplied_env();
    env.exec(r#"
        local q = C_SpecializationInfo.GetTalentInfo
        local a = assert(q{tier=1, column=1})
        assert(a.talentID == 901 and a.name == 'Supplied selected' and a.icon == 111)
        assert(a.tier == 1 and a.column == 1 and a.spellID == 9001)
        assert(a.selected and a.known and a.available and not a.grantedByAura)
        assert(a.rank == 1 and a.maxRank == 1 and a.previewRank == 1)
        assert(not a.meetsPrereq and not a.meetsPreviewPrereq and not a.isExceptional)
        assert(not a.hasGoldBorder and not a.isPVPTalentUnlocked)
        local count = 0 for _ in pairs(a) do count = count + 1 end assert(count == 18)
        local b = assert(q{tier=1, column=1, groupIndex=2, specializationIndex=999, talentIndex=999, isPet=0})
        assert(b.talentID == 902 and b.selected and not b.known and b.available and b.icon == 333)
        local c = assert(q{tier=1, column=2})
        assert(c.name == '' and c.icon == nil and c.available and not c.selected and not c.known)
        assert(q{tier=1, column=3} == nil)
        for _, t in ipairs({0, 8}) do assert(q{tier=t, column=1} == nil) end
        for _, c in ipairs({0, 4}) do assert(q{tier=1, column=c} == nil) end
        assert(q{tier=1, column=1, groupIndex=0} == nil)
        assert(q{tier=1, column=1, groupIndex=3} == nil)
        assert(q{tier=1, column=1, isInspect=0} == nil)
    "#).unwrap();
    {
        let mut sim = env.state().borrow_mut();
        let grid = &mut sim.talents.forever;
        grid.current_group = 1;
        grid.aura_grants.insert(903);
    }
    env.exec(r#"
        local q = C_SpecializationInfo.GetTalentInfo
        local a = assert(q{tier=1, column=1, groupIndex=1})
        assert(a.selected and a.available and not a.known)
        local b = assert(q{tier=1, column=2})
        assert(b.grantedByAura and not b.available)
    "#).unwrap();
}

#[test]
fn forever_talent_missing_rows_and_specialization_order() {
    let env = supplied_env();
    {
        let mut sim = env.state().borrow_mut();
        let grid = &mut sim.talents.forever;
        let row = grid.rows[0].clone();
        grid.rows = vec![
            TalentRow { id: 903, spec_id: 0, spell_id: 9003, ..row.clone() },
            TalentRow { id: 902, spec_id: 0, spell_id: 9002, ..row.clone() },
            row.clone(),
            TalentRow { id: 904, ..row },
        ];
    }
    env.exec("assert(C_SpecializationInfo.GetTalentInfo{tier=1,column=1}.talentID == 901)").unwrap();
    env.state().borrow_mut().talents.forever.groups[0].spec_id = 66;
    env.exec("assert(C_SpecializationInfo.GetTalentInfo{tier=1,column=1}.talentID == 902)").unwrap();
    env.state().borrow_mut().talents.forever.spell_names.remove(&9002);
    env.exec("assert(C_SpecializationInfo.GetTalentInfo{tier=1,column=1} == nil)").unwrap();
    env.state().borrow_mut().talents.forever.player_present = false;
    env.exec("assert(C_SpecializationInfo.GetTalentInfo{tier=1,column=1} == nil)").unwrap();
}

#[test]
fn forever_talent_missing_coordinates_have_wm90_diagnostics() {
    let env = supplied_env();
    for (query, diagnostic) in [
        ("{}", "C_SpecializationInfo.GetTalent: query.tier must be specified."),
        ("{tier=1}", "C_SpecializationInfo.GetTalent: query.column must be specified."),
    ] {
        let error = env.exec(&format!("C_SpecializationInfo.GetTalentInfo({query})")).unwrap_err().to_string();
        assert!(error.contains(diagnostic), "{error}");
    }
}
