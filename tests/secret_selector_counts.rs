//! P4: original viewer count/color consumers; numeric selectors only.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
use rilua::LuaApiMut;
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture(restricted: bool) -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.spell_cast_counts.clear();
        sim.spell_max_cumulative_aura_applications.clear();
        sim.spell_cast_counts.insert(19750, 3);
        sim.spell_max_cumulative_aura_applications.insert(19750, 7);
        sim.known_spells.clear();
        sim.known_spells.insert(19750);
        sim.spell_charges.clear();
        sim.spell_id_aliases.clear();
        sim.cooldowns_restricted = restricted;
        sim.restricted_actions[0] = if restricted { 2 } else { 0 };
    }
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    for (name, number) in [
        ("Selector", 19750.0),
        ("Missing", 900002.0),
        ("Invalid", -1.0),
        ("Fraction", 19750.5),
    ] {
        let value = rilua::table_security::wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        lua.set_global_val(name, value).unwrap();
        lua.state_mut().pop();
    }
    let text = rilua::table_security::wrap_host_secret_string(lua.state_mut(), "19750");
    lua.state_mut().push(text);
    lua.set_global_val("SecretText", text).unwrap();
    lua.state_mut().pop();
    drop(lua);
    env
}

fn original(env: &WowLuaEnv, file: &str, start: &str, end: &str) {
    let profile = if cfg!(feature = "client-ptr") {
        "ptr"
    } else {
        "retail"
    };
    let root = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
    let path = root.join(format!(
        ".cache/wow-ui-sim/blizzard-ui/{profile}/AddOns/Blizzard_CooldownViewer/{file}"
    ));
    let source = std::fs::read_to_string(path).unwrap();
    let begin = source.find(start).unwrap();
    let finish = begin + source[begin..].find(end).unwrap();
    env.exec(&source[begin..finish])
        .expect("unchanged original consumer block");
}

fn load_viewer(env: &WowLuaEnv) {
    env.exec("CooldownViewerCooldownItemMixin = {}; CooldownViewerItemDataMixin = {}; CooldownViewerItemMixin = {}").unwrap();
    original(
        env,
        "CooldownViewerItemData.lua",
        "function CooldownViewerItemDataMixin:GetSpellChargeInfo(",
        "function CooldownViewerItemDataMixin:GetFallbackSpellTexture(",
    );
    original(
        env,
        "CooldownViewer.lua",
        "CooldownViewerConstants = {",
        "-- Some ways to generate fake data",
    );
    original(
        env,
        "CooldownViewer.lua",
        "function CooldownViewerItemMixin:GetIconTexture(",
        "function CooldownViewerItemMixin:MarkDirty(",
    );
    original(
        env,
        "CooldownViewer.lua",
        "function CooldownViewerCooldownItemMixin:GetOutOfRangeTexture(",
        "function CooldownViewerCooldownItemMixin:OnLoad(",
    );
    original(
        env,
        "CooldownViewer.lua",
        "function CooldownViewerCooldownItemMixin:SetCachedChargeValues(",
        "function CooldownViewerCooldownItemMixin:IsExpired(",
    );
    original(
        env,
        "CooldownViewer.lua",
        "function CooldownViewerCooldownItemMixin:RefreshIconColor(",
        "function CooldownViewerCooldownItemMixin:RefreshOverlayGlow(",
    );
    env.exec(r#"
        function NewViewer(id)
            local frame = CreateFrame('Frame')
            Mixin(frame, CooldownViewerItemMixin, CooldownViewerItemDataMixin, CooldownViewerCooldownItemMixin)
            frame.Icon = frame:CreateTexture()
            frame.Icon:SetVertexColor(0.2, 0.3, 0.4)
            frame.OutOfRange = frame:CreateTexture()
            frame.OutOfRange:Show()
            frame.spellID = id
            function frame:GetSpellID() return self.spellID end
            function frame:GetCooldownInfo() return {spellID=self.spellID} end
            return frame
        end
    "#).unwrap();
}

#[test]
fn original_viewer_count_uses_configured_three() {
    for (restricted, tainted) in [(false, false), (false, true), (true, false)] {
        let env = fixture(restricted);
        load_viewer(&env);
        env.exec(&format!(
            r#"
            local expectedTaint = nil
        local function probe()
                local before = debug.getstacktaint()
            assert(before == expectedTaint, 'explicit caller context')
                for _, id in ipairs({{19750, Selector}}) do
                    local frame = NewViewer(id)
                    assert(frame:GetSpellChargeInfo() == nil, 'genuinely absent charges')
                    frame:CacheChargeValues()
                    assert(secretunwrap(frame.cooldownChargesCount) == 3, 'cached configured quantity')
                    assert(frame.cooldownChargesShown == true, 'cached shown state')
                    assert(issecretvalue(frame.cooldownChargesCount) == {restricted})
                end
                assert(issecretvalue(Selector) and debug.getstacktaint() == before)
            end
            if {tainted} then debug.setobjecttaint(probe, 'P4Probe'); expectedTaint = 'P4Probe' end
            probe()
        "#
        ))
        .expect("original viewer caches configured cast count");
        assert_eq!(env.state().borrow().spell_cast_counts[&19750], 3);
        assert!(env.state().borrow().spell_charges.is_empty());
        assert_count_facts(&env);
    }
}

#[test]
fn original_viewer_colors_known_spell_usable() {
    let env = fixture(false);
    load_viewer(&env);
    env.exec(
        r#"
        local expectedTaint = nil
        local function probe()
            local before = debug.getstacktaint()
            assert(before == expectedTaint, 'explicit caller context')
            for _, id in ipairs({19750, Selector}) do
                local frame = NewViewer(id)
                frame:RefreshIconColor()
                local r,g,b,a = frame.Icon:GetVertexColor()
                assert(r == 1 and g == 1 and b == 1 and a == 1)
                assert(not frame.OutOfRange:IsShown())
            end
            assert(issecretvalue(Selector) and debug.getstacktaint() == before)
        end
        probe()
        debug.setobjecttaint(probe, 'P4Probe'); expectedTaint = 'P4Probe'
        probe()
    "#,
    )
    .expect("original viewer renders usable white");
    assert_count_facts(&env);
}

fn assert_count_facts(env: &WowLuaEnv) {
    let sim = env.state().borrow();
    assert_eq!(sim.spell_cast_counts.len(), 1);
    assert_eq!(sim.spell_cast_counts[&19750], 3);
    assert_eq!(sim.spell_max_cumulative_aura_applications.len(), 1);
    assert_eq!(sim.spell_max_cumulative_aura_applications[&19750], 7);
    assert_eq!(sim.known_spells.len(), 1);
    assert!(sim.known_spells.contains(&19750));
    assert!(sim.spell_charges.is_empty());
    assert!(sim.spell_id_aliases.is_empty());
}

fn assert_scalar_selector_contract(api: &str, expected: u32) {
    for restricted in [false, true] {
        let env = fixture(restricted);
        env.exec(&format!(
            r#"
            local getter = C_Spell['{api}']
            local function probe(expectedTaint)
                local before = debug.getstacktaint()
                assert(before == expectedTaint, 'explicit caller context')
                for _,id in ipairs({{19750,Selector,900002,Missing}}) do
                    assert(select('#',getter(id)) == 1)
                    assert(issecretvalue(getter(id)) == {restricted})
                end
                if not {restricted} or not before then
                    assert(secretunwrap(getter(19750)) == {expected}, 'public selector quantity')
                    assert(secretunwrap(getter(Selector)) == {expected}, 'secret selector quantity')
                    assert(secretunwrap(getter(900002)) == 0, 'public selector miss')
                    assert(secretunwrap(getter(Missing)) == 0, 'secret selector miss')
                end
                for _,id in ipairs({{-1,Invalid,19750.5,Fraction,{{}},SecretText}}) do
                    assert(not pcall(getter,id))
                end
                assert(issecretvalue(Selector) and debug.getstacktaint() == before)
                if before then
                    assert(not canaccessvalue(Selector))
                    assert(not pcall(function() return Selector + 0 end))
                end
            end
            probe(nil)
            debug.setobjecttaint(probe,'P4Probe')
            probe('P4Probe')
        "#
        ))
        .expect("one scalar with configured quantity, own restriction and unchanged strict domain");
        assert_count_facts(&env);
    }
}

#[test]
fn cast_count_selector_arity_restrictions_misses_and_invalid_controls() {
    assert_scalar_selector_contract("GetSpellCastCount", 3);
}

#[test]
fn maximum_aura_selector_arity_restrictions_misses_and_invalid_controls() {
    assert_scalar_selector_contract("GetSpellMaxCumulativeAuraApplications", 7);
}

#[test]
fn usability_selector_arity_misses_invalid_controls_and_plain_outputs() {
    for restricted in [false, true] {
        let env = fixture(restricted);
        env.exec(r#"
            local function probe(expectedTaint)
                local before = debug.getstacktaint()
                assert(before == expectedTaint, 'explicit caller context')
                for _,id in ipairs({19750,Selector}) do
                    assert(select('#',C_Spell.IsSpellUsable(id)) == 2)
                    local usable,power = C_Spell.IsSpellUsable(id)
                    assert(usable and not power and not issecretvalue(usable) and not issecretvalue(power))
                end
                for _,id in ipairs({900002,Missing}) do
                    assert(select('#',C_Spell.IsSpellUsable(id)) == 2)
                    local usable,power = C_Spell.IsSpellUsable(id)
                    assert(not usable and not power)
                end
                for _,id in ipairs({-1,Invalid,19750.5,Fraction,{},SecretText}) do
                    assert(not pcall(C_Spell.IsSpellUsable,id))
                end
                assert(issecretvalue(Selector) and debug.getstacktaint() == before)
                if before then
                    assert(not canaccessvalue(Selector))
                    assert(not pcall(function() return Selector + 0 end))
                end
            end
            probe(nil)
            debug.setobjecttaint(probe,'P4Probe')
            probe('P4Probe')
        "#).expect("exactly two ordinary usability flags from known membership");
        assert_count_facts(&env);
    }
}

#[test]
fn count_and_aura_output_restrictions_transition_independently_for_secret_selectors() {
    let env = fixture(false);
    for (count_restricted, aura_restricted) in [
        (false, false),
        (true, false),
        (false, true),
        (true, true),
        (false, false),
    ] {
        {
            let mut sim = env.state().borrow_mut();
            sim.cooldowns_restricted = count_restricted;
            sim.restricted_actions[0] = if aura_restricted { 2 } else { 0 };
        }
        env.exec(&format!(
            r#"
            local function probe(expectedTaint)
                local before = debug.getstacktaint()
                assert(before == expectedTaint, 'explicit caller context')
                for _,id in ipairs({{19750,Selector}}) do
                    local count = C_Spell.GetSpellCastCount(id)
                    local maximum = C_Spell.GetSpellMaxCumulativeAuraApplications(id)
                    assert(issecretvalue(count) == {count_restricted})
                    assert(issecretvalue(maximum) == {aura_restricted})
                    if not {count_restricted} or not before then
                        assert(secretunwrap(count) == 3, 'count quantity after restriction transition')
                    end
                    if not {aura_restricted} or not before then
                        assert(secretunwrap(maximum) == 7, 'maximum quantity after restriction transition')
                    end
                end
                assert(issecretvalue(Selector) and debug.getstacktaint() == before)
            end
            probe(nil)
            debug.setobjecttaint(probe,'P4Probe')
            probe('P4Probe')
        "#
        ))
        .expect("live independent output restrictions, not selector-driven secrecy");
        let sim = env.state().borrow();
        assert_eq!(sim.cooldowns_restricted, count_restricted);
        assert_eq!(sim.restricted_actions[0] == 2, aura_restricted);
        assert_eq!(sim.spell_cast_counts[&19750], 3);
        assert_eq!(sim.spell_max_cumulative_aura_applications[&19750], 7);
    }
}
