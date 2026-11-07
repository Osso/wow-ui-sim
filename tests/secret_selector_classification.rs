//! P2: original consumers with authentic numeric-secret selectors.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
use crate::common;
use rilua::LuaApiMut;
use wow_ui_sim::c_api::c_spell_classification::SpellClassification;
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.spell_classifications.clear();
        sim.aura_filter_facts.important_spell_ids.clear();
        sim.spell_classifications.insert(
            19750,
            SpellClassification {
                consumable: true,
                crowd_control: true,
                external_defensive: true,
            },
        );
        sim.aura_filter_facts.important_spell_ids.insert(19750);
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

fn assert_classification_facts(env: &WowLuaEnv) {
    let sim = env.state().borrow();
    assert_eq!(sim.spell_classifications.len(), 1);
    let flags = sim.spell_classifications.get(&19750).unwrap();
    assert_eq!(
        (
            flags.consumable,
            flags.crowd_control,
            flags.external_defensive
        ),
        (true, true, true)
    );
    assert_eq!(sim.aura_filter_facts.important_spell_ids.len(), 1);
    assert!(sim.aura_filter_facts.important_spell_ids.contains(&19750));
}

fn load_original(env: &WowLuaEnv, file: &str, start: &str, end: &str) {
    let path = common::blizzard_ui_dir().join(file);
    let source = std::fs::read_to_string(path).unwrap();
    let begin = source.find(start).unwrap();
    let finish = begin + source[begin..].find(end).unwrap();
    env.exec(&source[begin..finish])
        .expect("unchanged original consumer block");
}

#[test]
fn original_nameplate_classifies_tainted_secret_spell() {
    let env = fixture();
    env.exec("NamePlateAurasMixin = {}").unwrap();
    load_original(
        &env,
        "Blizzard_NamePlates/Blizzard_NamePlateAuras.lua",
        "function NamePlateAurasMixin:IsAuraCrowdControl(",
        "function NamePlateAurasMixin:AddAura(",
    );
    env.exec(
        r#"
        local function probe(expectedTaint)
            local before = debug.getstacktaint()
            assert(before == expectedTaint, 'explicit caller context')
            for _, id in ipairs({19750, Selector}) do
                assert(NamePlateAurasMixin:IsAuraCrowdControl({spellId=id}) == true)
            end
            assert(issecretvalue(Selector))
            assert(debug.getstacktaint() == before)
        end
        probe(nil)
        debug.setobjecttaint(probe, 'P2Probe')
        probe('P2Probe')
    "#,
    )
    .expect("original nameplate resolves configured true classification");
    assert_classification_facts(&env);
}

#[test]
fn original_castbar_highlights_tainted_secret_spell() {
    let env = fixture();
    env.exec("CastingBarMixin = {}").unwrap();
    load_original(
        &env,
        "Blizzard_UIPanels_Game/Shared/CastingBarFrame.lua",
        "function CastingBarMixin:GetHighlightImportantCasts(",
        "function CastingBarMixin:SetIsHighlightedImportantCast(",
    );
    load_original(
        &env,
        "Blizzard_UIPanels_Game/Shared/CastingBarFrame.lua",
        "function CastingBarMixin:SetIsHighlightedImportantCast(",
        "function CastingBarMixin:GetIsHighlightedImportantCast(",
    );
    env.exec(
        r#"
        local function probe(expectedTaint)
            local before = debug.getstacktaint()
            assert(before == expectedTaint, 'explicit caller context')
            for _, id in ipairs({19750, Selector}) do
                local frame = CreateFrame('Frame')
                Mixin(frame, CastingBarMixin)
                frame.ImportantCastIndicator = frame:CreateTexture()
                frame.ImportantCastIndicator:Hide()
                frame.highlightImportantCasts = true
                frame.spellID = id
                frame:UpdateHighlightImportantCast()
                assert(frame.isHighlightedImportantCast == true)
                assert(frame.ImportantCastIndicator:IsShown())
            end
            assert(issecretvalue(Selector))
            assert(debug.getstacktaint() == before)
        end
        probe(nil)
        debug.setobjecttaint(probe, 'P2Probe')
        probe('P2Probe')
    "#,
    )
    .expect("original cast bar resolves important fact");
    assert_classification_facts(&env);
}

#[test]
fn classification_selector_controls_and_external_permission() {
    let env = fixture();
    env.exec(r#"
        local function probe(expectedTaint)
            local before = debug.getstacktaint()
            assert(before == expectedTaint, 'explicit caller context')
            for _, query in ipairs({C_Spell.IsSpellImportant, C_Spell.IsSpellCrowdControl, C_Spell.IsConsumableSpell}) do
                for _, id in ipairs({19750, Selector}) do
                    assert(select('#', query(id)) == 1)
                    local value = query(id)
                    assert(value == true and not issecretvalue(value))
                end
                for _, id in ipairs({900002, Missing, -1, Invalid}) do
                    assert(select('#', query(id)) == 1 and query(id) == false)
                end
                assert(query(19750.5) == query(Fraction), 'preserve numeric conversion')
                assert(query({}) == false)
                if before then
                    assert(not pcall(query, SecretText), 'secret strings not widened')
                end
                assert(debug.getstacktaint() == before)
            end
            assert(issecretvalue(Selector) and issecretvalue(Missing))
            if before then
                assert(not canaccessvalue(Selector))
                assert(not pcall(function() return Selector + 0 end))
                assert(not pcall(C_Spell.IsExternalDefensive, Selector))
            else
                assert(C_Spell.IsExternalDefensive(Selector) == true)
            end
            assert(C_Spell.IsExternalDefensive(19750) == true)
            assert(debug.getstacktaint() == before)
        end
        probe(nil)
        debug.setobjecttaint(probe, 'P2Probe')
        probe('P2Probe')
    "#).expect("P2 controls and unchanged AllowedWhenUntainted endpoint");
    assert_classification_facts(&env);
}
