#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_currency_token_ui_populates_rows_and_watched_tokens() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            if GetCurrencyListSize() ~= C_CurrencyInfo.GetCurrencyListSize() then
                error("legacy currency list size wrapper diverged")
            end
            if GetCurrencyListSize() <= 0 then
                error("currency list is empty")
            end

            ToggleCharacter("TokenFrame")
            if CharacterFrame == nil or CharacterFrame:IsShown() ~= true then
                error("CharacterFrame did not open")
            end
            if TokenFrame == nil or TokenFrame:IsShown() ~= true then
                error("TokenFrame did not open")
            end
            if CharacterFrameTab4 == nil or CharacterFrameTab4:IsShown() ~= true then
                error("currency tab did not show")
            end

            TokenFrame_Update()
            if TokenFrameContainer == nil or TokenFrameContainer.buttons == nil then
                error("TokenFrame buttons were not created")
            end

            local populatedRows = 0
            for _, button in ipairs(TokenFrameContainer.buttons) do
                if button:IsShown() and button.name and button.name:GetText() then
                    populatedRows = populatedRows + 1
                end
            end
            if populatedRows == 0 then
                error("TokenFrame has no populated currency rows")
            end

            local firstName, firstIsHeader = GetCurrencyListInfo(1)
            if type(firstName) ~= "string" or firstIsHeader ~= true then
                error("legacy currency list info did not return first header")
            end

            BackpackTokenFrame_Update()
            if GetNumWatchedTokens() <= 0 then
                error("watched backpack currencies are empty")
            end
            if BackpackTokenFrameToken1 == nil or BackpackTokenFrameToken1.currencyID == nil then
                error("first backpack token did not populate")
            end
            "#,
            "lua-errors",
        ]),
    )
    .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("Currency/Token UI", &stdout, &stderr);
}
