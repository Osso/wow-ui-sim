#![cfg(feature = "client-mists")]

use std::path::PathBuf;
use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

fn wow_sim_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_wow-sim")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("debug")
                .join("wow-sim")
        })
}

#[test]
fn mists_inspect_and_guild_control_panels_support_interaction_flows() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            INTERACTION_FLOW_LUA,
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
    assert_no_lua_errors(&stdout, &stderr);
}

fn assert_no_lua_errors(stdout: &str, stderr: &str) {
    assert!(
        !stdout.contains("Lua error") && !stderr.contains("Lua error"),
        "Inspect/GuildControl flow emitted Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

const INTERACTION_FLOW_LUA: &str = r#"
    local function fail(message)
        error(message, 0)
    end

    local function captureRadios(generator, dropdown)
        local radios = {}
        local rootDescription = {
            SetTag = function() end,
            CreateRadio = function(_, text, isSelected, setSelected, value)
                table.insert(radios, {
                    text = text,
                    isSelected = isSelected,
                    setSelected = setSelected,
                    value = value,
                })
            end,
        }
        generator(dropdown, rootDescription)
        return radios
    end

    A_Admin.SetGuildInfo("Parity Guild", "Officer", 7)
    A_Admin.SetGuildRanks({
        { name = "Guild Leader", flags = { true, true, true, true, true } },
        { name = "Officer", flags = { true, false, true, false, true } },
        { name = "Member", flags = { false, false, false, false, true } },
    })

    local ok, reason = LoadAddOn("Blizzard_InspectUI")
    if ok == false then
        fail("Blizzard_InspectUI failed to load: " .. tostring(reason))
    end

    InspectFrame_Show("player")
    FireEvent("INSPECT_READY", UnitGUID("player"))
    if not InspectFrame:IsShown() then
        fail("InspectFrame did not show after INSPECT_READY")
    end
    if not InspectPaperDollFrame:IsShown() then
        fail("InspectFrame did not start on PaperDoll tab")
    end

    InspectFrameTab3:Click()
    if not InspectTalentFrame:IsShown() or InspectPaperDollFrame:IsShown() then
        fail("Inspect talent tab did not switch visible subframes")
    end

    InspectFrameTab4:Click()
    if not InspectGuildFrame:IsShown() or InspectTalentFrame:IsShown() then
        fail("Inspect guild tab did not switch visible subframes")
    end
    if InspectGuildFrame.guildName:GetText() ~= "Parity Guild" then
        fail("Inspect guild tab did not render guild name")
    end
    if not string.find(InspectGuildFrame.guildNumMembers:GetText() or "", "7") then
        fail("Inspect guild tab did not render guild member count")
    end

    ok, reason = LoadAddOn("Blizzard_GuildControlUI")
    if ok == false then
        fail("Blizzard_GuildControlUI failed to load: " .. tostring(reason))
    end

    ShowUIPanel(GuildControlUI)
    GuildControlUI_RankOrder_Update(GuildControlUIRankOrderFrame)
    if not GuildControlUIRankOrderFrame:IsShown() then
        fail("GuildControlUI did not start on rank-order view")
    end

    local mainRadios = captureRadios(GuildControlUI.dropdown.menuGenerator, GuildControlUI.dropdown)
    mainRadios[2].setSelected(mainRadios[2].value)
    if GuildControlUI.selectedTab ~= 2 then
        fail("GuildControlUI rank-permissions tab was not selected")
    end
    if not GuildControlUIRankSettingsFrame:IsShown() or GuildControlUIRankOrderFrame:IsShown() then
        fail("GuildControlUI rank-permissions tab did not switch visible frames")
    end

    local rankRadios = captureRadios(
        GuildControlUIRankSettingsFrame.dropdown.menuGenerator,
        GuildControlUIRankSettingsFrame.dropdown
    )
    rankRadios[2].setSelected(rankRadios[2].value)
    if GuildControlUI.currentRank ~= 3 or GuildControlGetRankName() ~= "Member" then
        fail("GuildControlUI rank dropdown did not select Member rank")
    end

    local checkbox = GuildControlUIRankSettingsFrameCheckbox5
    if checkbox:GetChecked() ~= true then
        fail("GuildControlUI rank permission checkbox did not reflect seeded flags")
    end
    checkbox:Click()
    if checkbox:GetChecked() ~= false then
        fail("GuildControlUI rank permission checkbox did not visibly toggle")
    end
    if C_GuildInfo.GuildControlGetRankFlags(3)[5] ~= false then
        fail("GuildControlUI rank permission click did not update backing rank flags")
    end
"#;
