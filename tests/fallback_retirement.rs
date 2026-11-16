use std::path::Path;

use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::loader::{
    StartupAddonLoadKind, discover_blizzard_startup_addons_for_screen, find_toc_file,
    load_startup_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

const SOURCE_OWNER_CONSUMER_LUA: &str = r#"
local function original(fn, path)
    local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
    assert(source:find(path, 1, true), source)
end
original(FlagsUtil.IsSet, 'Blizzard_SharedXMLBase/Flags.lua')
original(FlagsMixin.IsSet, 'Blizzard_SharedXMLBase/Flags.lua')
assert(FlagsUtil.IsSet(3, 1) == true)
assert(FlagsUtil.IsSet(3, 5) == false)
assert(FlagsUtilConstants.CombineShouldSet == true)
original(CreateTemplateInfoCache, 'Blizzard_SharedXMLBase/TemplateInfoCache.lua')
original(CreateFrameFactory, 'Blizzard_SharedXMLBase/FrameFactory.lua')
local factory = CreateFrameFactory()
assert(factory:GetTemplateInfoCache():GetTemplateInfos() ~= nil)
if type(LoadAddOnWithErrorHandling) == 'function' then
    original(LoadAddOnWithErrorHandling, 'Blizzard_SharedXML/AddOnUtil.lua')
end
local probe = rawget(_G, '__fallback_retire_probe')
if probe then
    for _, name in ipairs({
        'FlagsUtil.IsSet', 'CreateTemplateInfoCache', 'LoadAddOnWithErrorHandling'
    }) do
        assert((probe.calls[name] or 0) == 0, name .. ' fallback reached a consumer')
    end
    for name, fallback in pairs(probe.fallbacks) do
        if name == 'FlagsUtil.IsSet' then
            assert(FlagsUtil.IsSet ~= fallback, 'Flags.lua must replace fallback IsSet')
        elseif name == 'FlagsUtilConstants.CombineShouldSet' then
            assert(FlagsUtilConstants ~= fallback, 'Flags.lua must replace fallback constants')
        elseif name == 'CreateTemplateInfoCache' then
            assert(CreateTemplateInfoCache ~= fallback, 'TemplateInfoCache.lua must replace fallback')
        elseif name == 'LoadAddOnWithErrorHandling' then
            assert(LoadAddOnWithErrorHandling ~= fallback, 'AddOnUtil.lua must replace fallback')
        end
    end
    print('fallback-retire-owners ' .. tostring(probe.branches['FlagsUtil.IsSet'] or 0)
        .. ',' .. tostring(probe.branches['FlagsUtilConstants.CombineShouldSet'] or 0)
        .. ',' .. tostring(probe.branches['CreateTemplateInfoCache'] or 0)
        .. ',' .. tostring(probe.branches['LoadAddOnWithErrorHandling'] or 0)
        .. '; fallback calls=' .. tostring(probe.calls['FlagsUtil.IsSet'] or 0)
        .. ',' .. tostring(probe.calls['CreateTemplateInfoCache'] or 0)
        .. ',' .. tostring(probe.calls['LoadAddOnWithErrorHandling'] or 0))
end
"#;

const RETAIL_CONSUMER_LUA: &str = r#"
local function original(fn, path)
    local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
    assert(source:find(path, 1, true), source)
end
local moneyDrop = GetMoneyTypeInfoField('PLAYER', 'DropFunc')
original(moneyDrop, 'Blizzard_MoneyFrame/Shared/MoneyFrame.lua')
original(ContainerFrameItemButton_OnClick, 'Blizzard_UIPanels_Game/Mainline/ContainerFrame.lua')
PickupPlayerMoney(500)
moneyDrop({})
assert(GetCursorMoney() == 0)
PickupPlayerMoney(500)
ContainerFrameItemButton_OnClick({}, 'LeftButton')
assert(GetCursorMoney() == 0)
local readyCalls = 0
EventUtil.ContinueOnVariablesLoaded(function() readyCalls = readyCalls + 1 end)
assert(readyCalls == 1)
original(ToggleCollectionsJournal, 'Blizzard_Collections/Blizzard_Collections_Bootstrap.lua')
original(ToggleEncounterJournal, 'Blizzard_EncounterJournal/Blizzard_EncounterJournal_Bootstrap.lua')
ToggleCollectionsJournal()
assert(CollectionsJournal and CollectionsJournal:IsShown())
ToggleCollectionsJournal()
assert(not CollectionsJournal:IsShown())
assert(ToggleEncounterJournal() == true)
assert(EncounterJournal:IsShown())
assert(ToggleEncounterJournal() == true)
assert(not EncounterJournal:IsShown())
"#;

fn load_profile_startup(env: &WowLuaEnv, ui: &Path) -> Vec<String> {
    let addons = discover_blizzard_startup_addons_for_screen(ui, ScreenKind::Game);
    let mut names = Vec::with_capacity(addons.len());
    for addon in addons {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
            .unwrap_or_else(|error| panic!("startup addon {} failed: {error}", addon.name));
        names.push(addon.name);
    }
    names
}

fn assert_addon_precedes(addons: &[String], owner: &str, consumer: &str) {
    let position = |name| {
        addons
            .iter()
            .position(|addon| addon == name)
            .unwrap_or_else(|| panic!("{name} is absent from the selected startup order"))
    };
    assert!(
        position(owner) < position(consumer),
        "{owner} must load before {consumer}"
    );
}

fn assert_base_toc_order(ui: &Path) {
    let toc_path = find_toc_file(&ui.join("Blizzard_SharedXMLBase"))
        .expect("active profile must select Blizzard_SharedXMLBase TOC");
    let toc = TocFile::from_file(&toc_path).expect("selected SharedXMLBase TOC parses");
    let files = toc.files.iter().map(|path| path.to_string_lossy());
    let files: Vec<String> = files.map(|path| path.replace('\\', "/")).collect();
    let position = |suffix: &str| {
        files
            .iter()
            .position(|file| file.ends_with(suffix))
            .unwrap_or_else(|| panic!("{suffix} is absent from {}", toc_path.display()))
    };
    assert!(position("Flags.lua") < position("FrameFactory.lua"));
    assert!(position("TemplateInfoCache.lua") < position("FrameFactory.lua"));
}

fn assert_startup_producers(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(debug.iscfunction(DropCursorMoney))
        assert(debug.iscfunction(ToggleCollectionsJournal))
        assert(debug.iscfunction(ToggleEncounterJournal))
        if C_CVar and C_CVar.AreCVarsLoaded then
            assert(debug.iscfunction(C_CVar.AreCVarsLoaded))
            assert(C_CVar.AreCVarsLoaded() == true)
        end
        if type(settablesecurity) == 'function' then
            assert(debug.iscfunction(settablesecurity))
        end
        local probe = rawget(_G, '__fallback_retire_probe')
        if probe then
            for _, name in ipairs({
                'DropCursorMoney', 'ToggleCollectionsJournal', 'ToggleEncounterJournal',
                'C_CVar.AreCVarsLoaded'
            }) do
                assert((probe.branches[name] or 0) == 0, name .. ' fallback branch ran')
            end
            print('fallback-retire-startup ' .. tostring(probe.branches['DropCursorMoney'] or 0)
                .. ',' .. tostring(probe.branches['ToggleCollectionsJournal'] or 0)
                .. ',' .. tostring(probe.branches['ToggleEncounterJournal'] or 0)
                .. ',' .. tostring(probe.branches['C_CVar.AreCVarsLoaded'] or 0))
        end
        "#,
    )
    .expect("registered startup API producers must be present");
}

fn assert_source_owner_consumers(env: &WowLuaEnv) {
    env.exec(SOURCE_OWNER_CONSUMER_LUA)
        .expect("original SharedXML owners must serve their consumers");
}

fn assert_retail_consumers(env: &WowLuaEnv) {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        return;
    }
    env.state().borrow_mut().player.money = 10_000;
    env.exec(RETAIL_CONSUMER_LUA)
        .expect("original Retail consumers must use the registered producers and owners");
}

#[test]
fn fallback_boundaries_and_source_owners_preserve_consumers() {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    wow_ui_sim::xml::register_intrinsic_templates();
    assert_startup_producers(&env);

    let addons = load_profile_startup(&env, &ui);
    assert_base_toc_order(&ui);
    assert_addon_precedes(&addons, "Blizzard_SharedXMLBase", "Blizzard_FrameXMLBase");
    if ACTIVE != ClientProfile::Mists {
        assert_addon_precedes(&addons, "Blizzard_SharedXML", "Blizzard_TrainerUI");
    }
    assert_source_owner_consumers(&env);

    env.apply_post_load_workarounds();
    assert_retail_consumers(&env);
}
