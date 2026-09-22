//! Console enumeration exposes only CVars known to the simulator.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn console_catalog_lists_supported_cvars_with_bounded_metadata() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, query in ipairs({ConsoleGetAllCommands, C_Console.GetAllCommands}) do
            local commands = query()
            assert(#commands > 0, "modeled catalog must not be empty")
            local found, previous = false, ""
            for _, entry in ipairs(commands) do
                assert(entry.command:lower() > previous, "sorted unique names")
                previous = entry.command:lower()
                assert(C_CVar.GetCVar(entry.command) ~= nil)
                assert(entry.commandType == Enum.ConsoleCommandType.Cvar)
                assert(entry.category == Enum.ConsoleCategory.None)
                assert(entry.help == "" and entry.scriptContents == "" and entry.scriptParameters == "")
                if entry.command == "Sound_EnableAllSound" then found = true end
            end
            assert(found, "modeled Sound_EnableAllSound missing")
        end
        assert(type(ConsoleGetAllCommands) == "function", "legacy global missing")
        "#,
    )
    .unwrap();
}

#[test]
fn console_catalog_refreshes_registration_without_mutable_result_aliases() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local original = ConsoleGetAllCommands()
        C_CVar.RegisterCVar("ConsoleProbeMixedCase", "first")
        SetCVar("ConsoleProbeMixedCase", "override")
        C_CVar.RegisterCVar("consoleprobemixedcase", "second")
        local function count(commands)
            local n = 0
            for _, entry in ipairs(commands) do
                if entry.command:lower() == "consoleprobemixedcase" then n = n + 1 end
            end
            return n
        end
        assert(count(original) == 0)
        assert(count(ConsoleGetAllCommands()) == 1)
        local fresh = C_Console.GetAllCommands()
        assert(count(fresh) == 1)
        fresh[1].command = "corrupted"
        fresh[2] = nil
        assert(ConsoleGetAllCommands()[1].command ~= "corrupted")
        assert(C_Console.GetAllCommands()[2] ~= nil)
        assert(C_CVar.GetCVar("ConsoleProbeMixedCase") == "override")
        "#,
    )
    .unwrap();
    let other = WowLuaEnv::new().unwrap();
    other
        .exec(
            r#"
        for _, entry in ipairs(ConsoleGetAllCommands()) do
            assert(entry.command:lower() ~= "consoleprobemixedcase")
        end
        "#,
        )
        .unwrap();
}
