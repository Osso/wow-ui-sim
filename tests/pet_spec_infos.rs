//! Supplied catalogs through the selected original StableUI specialization callbacks.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn load_original_specialization() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let path = find_toc_file(&ui.join("Blizzard_StableUI")).unwrap();
    let toc = TocFile::from_file(&path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    let selected = if ACTIVE == ClientProfile::Ptr {
        "Standard/Blizzard_StableUI.lua"
    } else {
        "Blizzard_StableUI.lua"
    };
    assert!(
        toc.files
            .iter()
            .any(|file| file == std::path::Path::new(selected))
    );
    eprintln!("selected StableUI {} {:?}", path.display(), toc.files);
    let env = new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_FrameXML", "Blizzard_StableUI"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_StableUI"));
    eprintln!(
        "unfiltered setup diagnostics: {:?}",
        env.state().borrow().lua_errors
    );
    env.exec(&format!("expectedPetSpecSource = {selected:?}"))
        .unwrap();
    env.exec(ORIGINAL_OBSERVER).unwrap();
    env
}

const ORIGINAL_OBSERVER: &str = r#"
    dropdown = assert(StableFrame.PetModelScene.PetInfo.Specialization)
    assert(dropdown:GetParent() == StableFrame.PetModelScene.PetInfo)
    assert(dropdown.OnLoad == StablePetSpecializationMixin.OnLoad)
    assert(dropdown.Refresh == StablePetSpecializationMixin.Refresh)
    for _, method in ipairs({dropdown.OnLoad, dropdown.Refresh}) do
        local info = debug.getinfo(method, 'S')
        local source = info.source:gsub('\\', '/')
        assert(info.what == 'Lua' and source:sub(-#expectedPetSpecSource) == expectedPetSpecSource)
        print('original specialization', source, info.linedefined, info.lastlinedefined)
    end
    function suppliedSpec(id, index, name)
        return {specID=id, specIndex=index, specializationName=name}
    end
    function observeRefresh(records, selectedID)
        StableFrame.selectedPet = {specID=selectedID}
        dropdown.activeSpecIndex = nil
        dropdown:Hide() -- Defer only menu generation, using the original SetupMenu behavior.
        A_Admin.SetAvailablePetSpecInfos(records)
        dropdown:Refresh()
        assert(#dropdown.options == #records, 'original option count')
        local expected = {}
        for _, record in ipairs(records) do expected[#expected+1] = record end
        table.sort(expected, function(a,b) return strcmputf8i(a.specializationName,b.specializationName)<0 end)
        local active
        for i, record in ipairs(expected) do
            local option = dropdown.options[i]
            assert(option.name == record.specializationName, 'original option name')
            assert(option.specIndex == record.specIndex, 'original option index')
            assert(option.isActive == (record.specID == selectedID), 'original specID comparison')
            if record.specID == selectedID then active = record.specIndex end
        end
        assert(dropdown.activeSpecIndex == active, 'original active index')
        assert(type(dropdown.menuGenerator) == 'function', 'original Refresh completed SetupMenu')
    end
"#;

#[test]
fn pet_spec_admin_accepts_complete_records_and_rejects_invalid_inputs() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetAvailablePetSpecInfos({{specID=917, specIndex=7, specializationName='Supplied'}})
        A_Admin.SetAvailablePetSpecInfos({})
        A_Admin.SetAvailablePetSpecInfos(nil)
        for _, input in ipairs({false, 4, 'bad', {false}, {{specID=1}},
            {{specID=1,specIndex=0,specializationName='x'}},
            {{specID=1,specIndex=1.5,specializationName='x'}},
            {{specID=0/0,specIndex=1,specializationName='x'}},
            {{specID=1,specIndex=1,specializationName=false}},
            {[2]={specID=1,specIndex=1,specializationName='x'}}, {unexpected=true}}) do
            local ok, reason = pcall(A_Admin.SetAvailablePetSpecInfos, input)
            assert(not ok and tostring(reason):find('pet specialization',1,true), tostring(reason))
        end
    "#,
    )
    .unwrap();
}

#[test]
#[cfg(feature = "client-ptr")]
fn original_pet_spec_onload_tracks_supplied_cardinality() {
    let env = load_original_specialization();
    env.exec(r#"
        local first = suppliedSpec(917,7,'Observed first')
        local second = suppliedSpec(811,2,'Observed second')
        local third = suppliedSpec(602,9,'Observed third')
        local installed = assert(dropdown:GetScript('OnLoad'))
        for _, records in ipairs({{}, {first,second}, {third}, {third,first,second}}) do
            A_Admin.SetAvailablePetSpecInfos(records)
            dropdown:Show()
            dropdown:SetWidth(51)
            dropdown.activeSpecIndex = 88
            local reached = false
            debug.sethook(function()
                if debug.getinfo(2,'f').func == StablePetSpecializationMixin.OnLoad then reached = true end
            end, 'c')
            installed(dropdown)
            debug.sethook()
            assert(reached, 'installed script dispatched original OnLoad')
            if #records <= 1 then
                assert(not dropdown:IsShown(), 'original short-catalog Hide')
                assert(dropdown:GetWidth() == 51 and dropdown.activeSpecIndex == 88)
            else
                assert(dropdown:IsShown(), 'original multi-catalog branch')
                assert(dropdown:GetWidth() == 135 and dropdown.activeSpecIndex == nil)
            end
            local observed = C_StableInfo.GetAvailablePetSpecInfos()
            assert(#observed == #records)
            for i, record in ipairs(records) do
                assert(observed[i].specID == record.specID and observed[i].specIndex == record.specIndex)
                assert(observed[i].specializationName == record.specializationName)
            end
        end
    "#).expect("original PTR OnLoad cardinality transitions");
}

#[test]
fn original_pet_spec_refresh_tracks_records_and_isolation() {
    let first = load_original_specialization();
    first
        .exec(
            r#"
        alpha = suppliedSpec(917,7,'Zulu observation')
        beta = suppliedSpec(811,2,'Alpha observation')
        observeRefresh({alpha,beta},811)
        observeRefresh({suppliedSpec(602,9,'Replacement observation')},602)
        observeRefresh({},602)
        observeRefresh({alpha,beta},917)
    "#,
        )
        .expect("original Refresh consumes distinct catalogs and selected IDs");
    let second = load_original_specialization();
    second
        .exec("observeRefresh({suppliedSpec(401,4,'Independent observation')},401)")
        .unwrap();
    first.exec("observeRefresh({alpha,beta},811)").unwrap();
    second
        .exec(
            r#"
        dropdown:Refresh()
        assert(#dropdown.options == 1 and dropdown.options[1].name == 'Independent observation')
        assert(dropdown.activeSpecIndex == 4)
    "#,
        )
        .unwrap();
}

#[test]
fn original_pet_spec_unset_and_clear_use_empty_default() {
    let env = load_original_specialization();
    let method = if ACTIVE == ClientProfile::Ptr {
        "OnLoad"
    } else {
        "Refresh"
    };
    env.exec(&format!("originalPetSpecEntry = dropdown.{method}"))
        .unwrap();
    env.exec(
        r#"
        local function assertDefault()
            StableFrame.selectedPet = {specID=917}
            dropdown:Hide()
            originalPetSpecEntry(dropdown)
            assert(#C_StableInfo.GetAvailablePetSpecInfos() == 0)
            if originalPetSpecEntry == dropdown.Refresh then
                assert(#dropdown.options == 0)
            else
                assert(not dropdown:IsShown())
            end
        end
        assertDefault()
        observeRefresh({suppliedSpec(917,7,'Supplied observation')},917)
        A_Admin.SetAvailablePetSpecInfos(nil)
        assertDefault()
        observeRefresh({suppliedSpec(811,2,'Resupplied observation')},811)
    "#,
    )
    .expect("empty default through original entry and recovery");
}

#[test]
fn pet_spec_observations_are_fresh_copies_and_invalid_supply_is_atomic() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local record = {specID=917,specIndex=7,specializationName='Original observation'}
        local input = {record}
        A_Admin.SetAvailablePetSpecInfos(input)
        record.specializationName = 'Caller mutation'
        input[1] = nil
        local output = C_StableInfo.GetAvailablePetSpecInfos()
        assert(#output == 1 and output[1].specializationName == 'Original observation')
        output[1].specIndex = 99
        output[2] = record
        local ok = pcall(A_Admin.SetAvailablePetSpecInfos, {
            {specID=811,specIndex=2,specializationName='Must not commit'}, {specID=602}})
        assert(not ok)
        local again = C_StableInfo.GetAvailablePetSpecInfos()
        assert(again ~= output and again[1] ~= output[1] and #again == 1)
        assert(again[1].specID == 917 and again[1].specIndex == 7)
        assert(again[1].specializationName == 'Original observation')
    "#,
    )
    .unwrap();
}
