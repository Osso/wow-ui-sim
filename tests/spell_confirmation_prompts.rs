//! Bounded retail 12.0.5 pending-prompt contract; native lifecycle remains unknown.
#![cfg(all(feature = "retail-12-0-5", feature = "profile-retail"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const PROMPT_FIXTURES: &str = r#"
    promptA = {
        spellID = 460101, confirmType = 1, text = "Fixture bonus roll",
        duration = 37.5, currencyID = 2801, currencyCost = 13,
        difficultyID = 16, displayItemID = 220081,
        itemContext = 23, treasureContextLevel = 71,
    }
    promptB = {
        spellID = 460202, confirmType = 3, text = "Fixture confirmation alert",
        duration = 62.25, currencyID = 2802, currencyCost = 29,
        difficultyID = 8, displayItemID = 220092,
        itemContext = 24, treasureContextLevel = 83,
    }
    promptFields = {
        "spellID", "confirmType", "text", "duration", "currencyID", "currencyCost",
        "difficultyID", "displayItemID", "itemContext", "treasureContextLevel",
    }
    function assertPrompt(actual, expected)
        assert(type(actual) == "table", "query must return a prompt record")
        for _, field in ipairs(promptFields) do
            assert(actual[field] == expected[field], "prompt field mismatch: " .. field)
        end
    end
    function findPrompt(rows, spellID)
        for _, row in ipairs(rows) do
            if row.spellID == spellID then return row end
        end
        return nil
    end
"#;

fn create_prompt_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create prompt environment");
    env.eval::<()>(PROMPT_FIXTURES)
        .expect("install concrete prompt fixtures without replacing APIs");
    env
}

fn require_prompt_actions(env: &WowLuaEnv) {
    env.eval::<()>(
        r#"
        assert(type(AcceptSpellConfirmationPrompt) == "function",
            "AcceptSpellConfirmationPrompt must be callable")
        assert(type(DeclineSpellConfirmationPrompt) == "function",
            "DeclineSpellConfirmationPrompt must be callable")
        "#,
    )
    .expect("require callable actions before testing removal or absence");
}

#[test]
fn empty_default_and_absent_actions_keep_no_pending_prompts() {
    let env = create_prompt_env();
    require_prompt_actions(&env);
    env.eval::<()>(
        r#"
        assert(type(GetSpellConfirmationPromptsInfo) == "function")
        assert(select('#', GetSpellConfirmationPromptsInfo()) == 1)
        local rows = GetSpellConfirmationPromptsInfo()
        assert(type(rows) == "table" and next(rows) == nil)
        AcceptSpellConfirmationPrompt(promptA.spellID)
        DeclineSpellConfirmationPrompt(promptB.spellID)
        assert(next(GetSpellConfirmationPromptsInfo()) == nil)
        "#,
    )
    .expect("empty simulator default and inferred absent-ID no-op");
}

#[test]
fn queue_publishes_exact_ten_arguments_and_query_state_synchronously() {
    let env = create_prompt_env();
    env.eval::<()>(
        r#"
        assert(type(A_Admin.QueueSpellConfirmationPrompt) == "function",
            "A_Admin.QueueSpellConfirmationPrompt must be implemented")
        local events = {}
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("SPELL_CONFIRMATION_PROMPT")
        listener:SetScript("OnEvent", function(_, event, ...)
            local expected = {
                460101, 1, "Fixture bonus roll", 37.5, 2801, 13, 16,
                220081, 23, 71,
            }
            assert(event == "SPELL_CONFIRMATION_PROMPT")
            assert(select('#', ...) == 10, "event must have exactly ten arguments")
            for index = 1, 10 do
                local actual = select(index, ...)
                assert(actual ~= nil, "every event argument is nonnil")
                assert(actual == expected[index], "event argument mismatch: " .. index)
            end
            local rows = GetSpellConfirmationPromptsInfo()
            assert(#rows == 1, "pending state must precede callback")
            assertPrompt(rows[1], promptA)
            events[#events + 1] = rows
        end)
        A_Admin.QueueSpellConfirmationPrompt(promptA)
        assert(#events == 1, "actual admin transition must dispatch before returning")
        assertPrompt(GetSpellConfirmationPromptsInfo()[1], promptA)
        "#,
    )
    .expect("actual producer maps all ten event fields and commits state before dispatch");
}

#[test]
fn two_pending_spell_ids_retain_independent_complete_records() {
    let env = create_prompt_env();
    env.eval::<()>(
        r#"
        assert(type(A_Admin.QueueSpellConfirmationPrompt) == "function")
        A_Admin.QueueSpellConfirmationPrompt(promptA)
        A_Admin.QueueSpellConfirmationPrompt(promptB)
        local rows = GetSpellConfirmationPromptsInfo()
        assert(#rows == 2)
        assertPrompt(findPrompt(rows, promptA.spellID), promptA)
        assertPrompt(findPrompt(rows, promptB.spellID), promptB)
        "#,
    )
    .expect("two IDs coexist without asserting native query ordering");
}

#[test]
fn accept_and_decline_remove_only_matching_pending_spell() {
    let env = create_prompt_env();
    require_prompt_actions(&env);
    env.eval::<()>(
        r#"
        assert(type(A_Admin.QueueSpellConfirmationPrompt) == "function")
        A_Admin.QueueSpellConfirmationPrompt(promptA)
        A_Admin.QueueSpellConfirmationPrompt(promptB)
        AcceptSpellConfirmationPrompt(promptA.spellID)
        local afterAccept = GetSpellConfirmationPromptsInfo()
        assert(#afterAccept == 1)
        assert(findPrompt(afterAccept, promptA.spellID) == nil)
        assertPrompt(afterAccept[1], promptB)

        A_Admin.QueueSpellConfirmationPrompt(promptA)
        DeclineSpellConfirmationPrompt(promptB.spellID)
        local afterDecline = GetSpellConfirmationPromptsInfo()
        assert(#afterDecline == 1)
        assert(findPrompt(afterDecline, promptB.spellID) == nil)
        assertPrompt(afterDecline[1], promptA)

        AcceptSpellConfirmationPrompt(promptB.spellID)
        DeclineSpellConfirmationPrompt(promptB.spellID)
        assertPrompt(GetSpellConfirmationPromptsInfo()[1], promptA)
        assert(#GetSpellConfirmationPromptsInfo() == 1)
        "#,
    )
    .expect("inferred matching-ID removal and absent-ID no-op preserve another prompt");
}

#[test]
fn repeated_spell_id_replaces_one_record_without_duplicate() {
    let env = create_prompt_env();
    env.eval::<()>(
        r#"
        assert(type(A_Admin.QueueSpellConfirmationPrompt) == "function")
        A_Admin.QueueSpellConfirmationPrompt(promptA)
        A_Admin.QueueSpellConfirmationPrompt(promptB)
        local replacement = {
            spellID = 460101, confirmType = 4, text = "Replacement warning",
            duration = 19.75, currencyID = 2803, currencyCost = 41,
            difficultyID = 15, displayItemID = 220103,
            itemContext = 25, treasureContextLevel = 97,
        }
        A_Admin.QueueSpellConfirmationPrompt(replacement)
        local rows = GetSpellConfirmationPromptsInfo()
        assert(#rows == 2, "inferred replacement must not duplicate a spell ID")
        assertPrompt(findPrompt(rows, promptA.spellID), replacement)
        assertPrompt(findPrompt(rows, promptB.spellID), promptB)
        "#,
    )
    .expect("explicit inferred same-ID replacement changes every supplied field");
}

#[test]
fn input_and_query_mutations_do_not_cross_snapshots_or_environments() {
    let populated = create_prompt_env();
    let isolated = create_prompt_env();
    populated
        .eval::<()>(
            r#"
            assert(type(A_Admin.QueueSpellConfirmationPrompt) == "function")
            A_Admin.QueueSpellConfirmationPrompt(promptA)
            A_Admin.QueueSpellConfirmationPrompt(promptB)
            local first = GetSpellConfirmationPromptsInfo()
            local second = GetSpellConfirmationPromptsInfo()
            assert(first ~= second)
            assert(findPrompt(first, 460101) ~= findPrompt(second, 460101))
            local changed = findPrompt(first, 460101)
            changed.text = "Mutated snapshot"
            changed.displayItemID = -1
            first[1] = nil
            first[2] = nil
            promptA.text = "Mutated caller input"
            promptA.treasureContextLevel = -2
            assertPrompt(findPrompt(second, 460202), promptB)
            assert(findPrompt(second, 460101).text == "Fixture bonus roll")
            local fresh = GetSpellConfirmationPromptsInfo()
            assert(#fresh == 2)
            assertPrompt(findPrompt(fresh, 460202), promptB)
            local original = findPrompt(fresh, 460101)
            assert(original.text == "Fixture bonus roll")
            assert(original.displayItemID == 220081)
            assert(original.treasureContextLevel == 71)
            "#,
        )
        .expect("caller and result mutation cannot change retained prompt data");
    isolated
        .eval::<()>(
            r#"
            assert(next(GetSpellConfirmationPromptsInfo()) == nil)
            assert(type(A_Admin.QueueSpellConfirmationPrompt) == "function")
            A_Admin.QueueSpellConfirmationPrompt(promptB)
            assert(#GetSpellConfirmationPromptsInfo() == 1)
            assertPrompt(GetSpellConfirmationPromptsInfo()[1], promptB)
            "#,
        )
        .expect("separate environment starts empty and owns its pending prompts");
    populated
        .eval::<()>(
            r#"
            local rows = GetSpellConfirmationPromptsInfo()
            assert(#rows == 2)
            assert(findPrompt(rows, 460101).text == "Fixture bonus roll")
            assertPrompt(findPrompt(rows, 460202), promptB)
            "#,
        )
        .expect("second environment input cannot replace first environment state");
}
