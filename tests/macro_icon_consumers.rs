//! R88l.20–23: original icon consumer and separately observable Lua API contracts.
#![cfg(all(
    any(feature = "client-retail", feature = "client-wowforever"),
    feature = "optional-tests"
))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_original_empty_icon_source(env: &WowLuaEnv) {
    env.exec(
        r#"
        local provider = CreateFromMixins(IconDataProviderMixin)
        provider:Init(IconDataProviderExtraType.None, false)
        local count = provider:GetNumIcons()
        local first, second = provider:GetIconByIndex(1), provider:GetIconByIndex(2)
        provider:Release()
        assert(first == [[INTERFACE\ICONS\INV_MISC_QUESTIONMARK]], tostring(first))
        assert(count == 1 and second == nil, "unsupplied simulator icon sources must not fabricate results")
        "#,
    )
    .expect("the original provider retains its own question mark for an empty host feed");
}

fn set_icon_catalog(env: &WowLuaEnv, catalog: &serde_json::Value) {
    env.exec(&format!("A_Admin.SetMacroIconCatalog([=[{catalog}]=])"))
        .expect("supplied macro icon catalog");
}

#[test]
fn macro_icon_api_returns_reused_table_and_rejects_wrong_type() {
    let env = WowLuaEnv::new().expect("Lua environment");
    assert_direct_icon_shapes(&env);
    for n in 1..=2 {
        let spell = format!("host_spell_{n}");
        let item = format!("host_item_{n}");
        let spell_id = 123 + n;
        let item_id = -45 - n;
        let catalog = serde_json::json!({
            "looseSpells": [spell.clone(), null, item.clone()],
            "looseItems": [item.clone()],
            "spells": [spell_id],
            "items": [item_id],
        });
        set_icon_catalog(&env, &catalog);
        assert_direct_icon_contents(&env, &spell, &item, spell_id, item_id);
    }
}

fn assert_direct_icon_shapes(env: &WowLuaEnv) {
    env.exec(
        r##"
        for _, name in ipairs({
            "GetLooseMacroIcons", "GetLooseMacroItemIcons",
            "GetMacroIcons", "GetMacroItemIcons",
        }) do
            local getter = assert(_G[name], name .. " missing")
            for _, arg in ipairs({false, 42, "bad"}) do
                local ok, err = pcall(getter, arg)
                assert(not ok and tostring(err):find("Usage: " .. name .. "([table])", 1, true),
                    name .. " accepted wrong argument or returned wrong Usage: " .. tostring(err))
            end
            assert(select("#", getter()) == 1, name)
            assert(select("#", getter(nil, true)) == 1, name)
            local fresh, other = getter(), getter(nil, true)
            assert(type(fresh) == "table" and type(other) == "table" and fresh ~= other, name)
            local output = {"existing", sentinel = true}
            assert(select("#", getter(output, false)) == 1, name)
            assert(getter(output, false) == output, name .. " did not reuse its output table")
            assert(output[1] == "existing" and output.sentinel, name .. " cleared the output")
        end
        "##,
    )
    .expect("R88l.20–23 ordinary argument, arity, and output table shapes");
}

fn assert_direct_icon_contents(
    env: &WowLuaEnv,
    spell: &str,
    item: &str,
    spell_id: i32,
    item_id: i32,
) {
    env.exec(&format!(r#"
        local function append(getter)
            local out = {{"existing", sentinel = true}}
            assert(getter(out, false) == out)
            assert(out[1] == "existing" and out.sentinel)
            return out
        end
        local out = append(GetLooseMacroIcons)
        assert(rawget(out, 2) == "{spell}" and rawget(out, 3) == nil and rawget(out, 4) == "{item}", "loose spell contents")
        assert(append(GetLooseMacroItemIcons)[2] == "{item}", "loose item contents")
        assert(append(GetMacroIcons)[2] == {spell_id}, "spell catalog contents")
        assert(append(GetMacroItemIcons)[2] == {item_id}, "item catalog contents")
        "#))
        .expect("all four getters must append two distinguishable host snapshots");
}

#[test]
fn original_icon_provider_observes_all_four_supplied_sources() {
    assert_original_icon_provider_observes_all_four_supplied_sources();
}

fn assert_original_icon_provider_observes_all_four_supplied_sources() {
    with_blizzard_addon_closure(&["Blizzard_FrameXMLBase"], &[], |env, loaded| {
        assert!(loaded.iter().any(|addon| addon == "Blizzard_FrameXMLBase"));
        for n in 1..=2 {
            let spell_name = format!("host_spell_{n}");
            let item_name = format!("host_item_{n}");
            let spell_id = 900_100 + n;
            let item_id = -900_200 - n;
            let catalog = serde_json::json!({
                "looseSpells": [spell_name.clone()],
                "looseItems": [item_name.clone()],
                "spells": [spell_id],
                "items": [item_id],
            });
            set_icon_catalog(env, &catalog);
            env.exec(&format!(
                r#"
                local source = debug.getinfo(IconDataProviderMixin.Init, "S").source
                assert(source:gsub("\\", "/"):find("Blizzard_FrameXMLBase/IconDataProvider.lua", 1, true), source)
                local provider = CreateFromMixins(IconDataProviderMixin)
                provider:Init(IconDataProviderExtraType.None, false, {{IconDataProviderIconType.Spell, IconDataProviderIconType.Item}})
                assert(provider:GetNumIcons() == 5, "one built-in question mark and four supplied icons")
                assert(provider:GetIconByIndex(2) == [[INTERFACE\ICONS\{spell_name}]], "supplied loose spell")
                assert(provider:GetIconByIndex(3) == {spell_id}, "supplied catalog spell")
                assert(provider:GetIconByIndex(4) == [[INTERFACE\ICONS\{item_name}]], "supplied loose item")
                assert(provider:GetIconByIndex(5) == {item_id}, "supplied signed catalog item")
                provider:Release()
                "#,
            ))
            .expect("original provider should distinguish both snapshots of all four host feeds");
        }
        env.exec("A_Admin.SetMacroIconCatalog([[{}]])")
            .expect("clear supplied icon catalog");
        assert_original_empty_icon_source(env);
    });
}
