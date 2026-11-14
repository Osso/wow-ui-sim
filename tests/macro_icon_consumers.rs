//! R88l.20–23: original icon consumer and separately observable Lua API contracts.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

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

#[test]
fn macro_icon_api_returns_reused_table_and_rejects_wrong_type() {
    let env = WowLuaEnv::new().expect("Lua environment");
    {
        let mut sim = env.state().borrow_mut();
        sim.macro_icons.loose_spells =
            vec![Some("host_alpha".into()), None, Some("host_beta".into())];
        sim.macro_icons.items = vec![-45, 123];
    }
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
            assert(select("#", getter()) == 1 and type(getter()) == "table", name)
            assert(select("#", getter(nil, true)) == 1 and type(getter(nil, true)) == "table", name)
            local output = {"existing", sentinel = true}
            assert(select("#", getter(output, false)) == 1, name)
            assert(getter(output, false) == output, name .. " did not reuse its output table")
            assert(output[1] == "existing" and output.sentinel, name .. " cleared the output")
        end
        local out = {"existing"}
        assert(GetLooseMacroIcons(out) == out)
        assert(rawget(out, 1) == "existing" and rawget(out, 2) == "host_alpha")
        assert(rawget(out, 3) == nil and rawget(out, 4) == "host_beta")
        out = {"existing"}
        assert(GetMacroItemIcons(out) == out)
        assert(out[1] == "existing" and out[2] == -45 and out[3] == 123)
        "##,
    )
    .expect("API-level VERIFIED argument, arity, and output table semantics from R88l.20–23");
}

#[test]
fn original_icon_provider_observes_all_four_supplied_sources() {
    with_blizzard_addon_closure(&["Blizzard_FrameXMLBase"], &[], |env, loaded| {
        assert!(loaded.iter().any(|addon| addon == "Blizzard_FrameXMLBase"));
        for n in 1..=2 {
            let spell_name = format!("host_spell_{n}");
            let item_name = format!("host_item_{n}");
            let spell_id = 900_100 + n;
            let item_id = -900_200 - n;
            {
                let mut sim = env.state().borrow_mut();
                sim.macro_icons.loose_spells = vec![Some(spell_name.clone())];
                sim.macro_icons.loose_items = vec![Some(item_name.clone())];
                sim.macro_icons.spells = vec![spell_id];
                sim.macro_icons.items = vec![item_id];
            }
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
        env.state().borrow_mut().macro_icons = Default::default();
        assert_original_empty_icon_source(env);
    });
}
