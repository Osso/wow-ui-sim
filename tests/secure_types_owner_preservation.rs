//! PRESERVATION: the selected Blizzard owner supplies SecureTypes before its consumers.
#![cfg(all(
    feature = "optional-tests",
    any(
        feature = "client-retail",
        feature = "client-ptr",
        feature = "client-wowforever",
        feature = "client-mists"
    )
))]

use std::path::Path;

use wow_ui_sim::client_profile::ACTIVE;
use wow_ui_sim::loader::{
    discover_blizzard_addon_closure_for_screen_with_overrides, find_toc_file,
};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

const OWNER: &str = "Blizzard_SharedXMLBase";
const CONSUMERS: &[(&str, &[&str])] = &[
    (
        "Blizzard_ChatFrameBase",
        &[
            "Shared/ChatFrameFiltersSecure.lua",
            "Shared/ChatFrameFilters.lua",
        ],
    ),
    ("Blizzard_Menu", &["Menu.lua"]),
    ("Blizzard_SharedXML", &["CircularBuffer.lua"]),
];
const ROOTS: &[&str] = &[
    "Blizzard_ChatFrameBase",
    "Blizzard_Menu",
    "Blizzard_SharedXML",
];

fn cache_dir() -> std::path::PathBuf {
    wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .expect("active profile Blizzard UI cache")
}

fn normalized_file(file: &Path) -> String {
    file.to_string_lossy().replace('\\', "/")
}

fn assert_selected_toc_file(ui: &Path, addon: &str, suffix: &str) -> TocFile {
    let toc_path = find_toc_file(&ui.join(addon)).unwrap_or_else(|| {
        panic!("selected TOC for {addon} must exist in the active profile cache")
    });
    let toc = TocFile::from_file(&toc_path)
        .unwrap_or_else(|error| panic!("selected TOC {}: {error}", toc_path.display()));

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "{addon} selected TOC must allow Game: {}",
        toc_path.display()
    );
    assert!(
        toc.files
            .iter()
            .any(|file| normalized_file(file).ends_with(suffix)),
        "{addon} selected TOC {} must select {suffix}; selected files: {:?}",
        toc_path.display(),
        toc.files
    );
    eprintln!(
        "PRESERVATION {ACTIVE:?}: selected {addon} TOC {}",
        toc_path.display()
    );
    toc
}

fn assert_owner_precedes_consumer(ui: &Path, consumer: &str) {
    let closure = discover_blizzard_addon_closure_for_screen_with_overrides(
        ui,
        ScreenKind::Game,
        &[consumer],
        &[],
    );
    let names: Vec<&str> = closure.iter().map(|(name, _)| name.as_str()).collect();
    let owner_index = names
        .iter()
        .position(|name| *name == OWNER)
        .unwrap_or_else(|| panic!("{OWNER} absent from {consumer} closure: {names:?}"));
    let consumer_index = names
        .iter()
        .position(|name| *name == consumer)
        .unwrap_or_else(|| panic!("{consumer} absent from its selected closure: {names:?}"));

    assert!(
        owner_index < consumer_index,
        "{OWNER} must load before {consumer}: {names:?}"
    );
    eprintln!("PRESERVATION {ACTIVE:?}: {OWNER} precedes {consumer} in selected closure");
}

#[test]
fn secure_types_owner_preservation_through_selected_consumers() {
    let ui = cache_dir();
    let owner_toc = assert_selected_toc_file(&ui, OWNER, "SecureTypes.lua");
    let owner_pool_position = owner_toc
        .files
        .iter()
        .position(|file| normalized_file(file).ends_with("SecureTypes.lua"))
        .expect("selected SecureTypes.lua");
    let pools_position = owner_toc
        .files
        .iter()
        .position(|file| normalized_file(file).ends_with("Pools.lua"))
        .expect("selected Pools.lua");
    assert!(
        owner_pool_position < pools_position,
        "SecureTypes.lua must load before its Pools.lua consumer"
    );

    for (consumer, files) in CONSUMERS {
        let toc = assert_selected_toc_file(&ui, consumer, files[0]);
        for file in files.iter().skip(1) {
            assert!(
                toc.files
                    .iter()
                    .any(|selected| normalized_file(selected).ends_with(file)),
                "{consumer} selected TOC must also select {file}: {:?}",
                toc.files
            );
        }
        assert_owner_precedes_consumer(&ui, consumer);
    }

    // The consumer closures below are loaded without file overrides. The original
    // SecureTypes.lua factory values must own every cited call site.
    let (env, loaded) =
        crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(&ui, ROOTS, &[]);
    for consumer in ROOTS {
        assert!(
            loaded.iter().any(|name| name == consumer),
            "{consumer} must load from its selected TOC: {loaded:?}"
        );
    }
    assert!(
        loaded.iter().any(|name| name == OWNER),
        "{OWNER} must load before original consumers: {loaded:?}"
    );
    let owner_position = loaded
        .iter()
        .position(|name| name == OWNER)
        .expect("owner in combined runtime closure");
    let chat_position = loaded
        .iter()
        .position(|name| name == ROOTS[0])
        .expect("ChatFrameBase in combined runtime closure");
    assert!(
        owner_position < chat_position,
        "owner must precede ChatFrameBase in the combined runtime closure: {loaded:?}"
    );

    env.exec(
        r##"
        local function requireSource(fn, suffix)
            local info = debug.getinfo(fn, "S")
            local source = info.source:gsub("\\", "/")
            assert(info.what == "Lua" and source:find(suffix, 1, true), source)
            print("PRESERVATION original source", source, info.linedefined)
        end

        requireSource(SecureTypes.CreateSecureMap, "Blizzard_SharedXMLBase/SecureTypes.lua")
        requireSource(SecureTypes.CreateSecureArray, "Blizzard_SharedXMLBase/SecureTypes.lua")
        requireSource(SecureTypes.CreateSecureNumber, "Blizzard_SharedXMLBase/SecureTypes.lua")
        requireSource(SecureTypes.CreateSecureFunction, "Blizzard_SharedXMLBase/SecureTypes.lua")
        requireSource(CreateObjectPool, "Blizzard_SharedXMLBase/Pools.lua")
        requireSource(CreateCircularBuffer, "Blizzard_SharedXML/CircularBuffer.lua")
        requireSource(ChatFrameUtil.CreateMessageEventFilterRegistry, "Blizzard_ChatFrameBase/Shared/ChatFrameFilters.lua")
        requireSource(Menu.CreateRootMenuDescription, "Blizzard_Menu/Menu.lua")

        -- SecureTypes.lua:29-98 owns the map and does not define the fallback-only
        -- Insert/Remove/Find/Contains/Clear methods. Pools.lua:198-262 consumes its
        -- SetValue/ClearValue/HasKey and SecureNumber values.
        local map = SecureTypes.CreateSecureMap()
        assert(map.Insert == nil and map.Remove == nil and map.Find == nil and map.Contains == nil and map.Clear == nil,
            "SecureTypes.lua:29-98 original SecureMap surface")
        assert(type(map.tbl) == "table", "SecureTypes.lua:95 creates the documented backing table")

        local pool = CreateObjectPool(function() return {} end, function() end)
        local first = pool:Acquire()
        local second = pool:Acquire()
        assert(first ~= second and pool:GetNumActive() == 2, "Pools.lua:198-262 original secure pool state")
        pool:Release(first)
        assert(pool:GetNumActive() == 1 and not pool:IsActive(first) and pool:IsActive(second),
            "Pools.lua:229-262 original secure pool release")

        -- CircularBuffer.lua:159-162 stores SecureNumber objects, not raw numbers.
        local buffer = CreateCircularBuffer(4)
        assert(type(buffer.headIndex) == "table" and buffer.headIndex.value == 0,
            "CircularBuffer.lua:160 SecureNumber wrapper")
        assert(type(buffer.maxElements) == "table" and buffer.maxElements.value == 4,
            "CircularBuffer.lua:161 SecureNumber wrapper and supplied value")

        -- ChatFrameFilters.lua:76-90 uses the original map and secure array.
        local filterRegistry = ChatFrameUtil.CreateMessageEventFilterRegistry()
        local callback = function() end
        filterRegistry:AddFilter("CHAT_MSG_SAY", callback)
        filterRegistry:RemoveFilter("CHAT_MSG_SAY", callback)

        -- Menu.lua:212-220 and :443-451 consume SecureArray and SecureFunction.
        local root = Menu.CreateRootMenuDescription({})
        local element = Menu.CreateMenuElementDescription()
        root:Insert(element)
        assert(root:HasElements(), "Menu.lua:427-430 stores descriptions in its SecureArray")
        local iterator, state, index = root:EnumerateElementDescriptions()
        local actualIndex, actualElement = iterator(state, index)
        assert(actualIndex == 1 and actualElement == element,
            "Menu.lua:419-420 preserves the SecureArray index/value iterator")
        local initializer = function() end
        root:SetFinalInitializer(initializer)

        -- SecureTypes.lua:388-392 returns an object; :356-364 stores a wrapper,
        -- unlike the fallback's identity-function result.
        local secureFunction = SecureTypes.CreateSecureFunction()
        assert(type(secureFunction) == "table" and type(secureFunction.SetFunction) == "function"
            and secureFunction.wrapper == nil, "SecureTypes.lua:388-392 secure function wrapper object")
        secureFunction:SetFunction(initializer)
        assert(type(secureFunction.wrapper) == "function" and secureFunction.wrapper ~= initializer,
            "SecureTypes.lua:356-364 stores a wrapper, not the identity function")

        -- SecureTypes.lua:131-134 returns the value at an array index, not a boolean.
        local array = SecureTypes.CreateSecureArray()
        array:Insert("first")
        array:Insert("second")
        assert(array:Remove(1) == "first", "SecureTypes.lua:131-134 Remove(index) returns the removed value")
        array:Insert("second")
        local arrayIterator, arrayState, arrayIndex = array:Enumerate()
        assert(select("#", array:Enumerate()) == 3 and type(arrayIterator) == "function"
            and arrayState == nil and arrayIndex == nil, "SecureTypes.lua:169-173 Enumerate return shape")
        local actualArrayIndex, actualArrayValue = arrayIterator(arrayState, arrayIndex)
        assert(actualArrayIndex == 1 and actualArrayValue == "second",
            "SecureTypes.lua:169-173 Enumerate yields index and value")
        "##,
    )
    .expect("selected original consumers preserve the original SecureTypes values");
}
