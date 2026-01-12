use tracing_subscriber::EnvFilter;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::run_ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let env = WowLuaEnv::new()?;

    // Create some demo frames
    env.exec(
        r#"
        -- Create a main panel with WoW-style backdrop
        local mainFrame = CreateFrame("Frame", "MainPanel", UIParent)
        mainFrame:SetSize(350, 250)
        mainFrame:SetPoint("CENTER", 0, 0)
        mainFrame:SetBackdrop({
            bgFile = "Interface\\DialogFrame\\UI-DialogBox-Background",
            edgeFile = "Interface\\DialogFrame\\UI-DialogBox-Border",
            edgeSize = 3,
            insets = { left = 4, right = 4, top = 4, bottom = 4 }
        })
        mainFrame:SetBackdropColor(0.1, 0.1, 0.15, 0.95)
        mainFrame:SetBackdropBorderColor(0.7, 0.55, 0.2, 1.0)

        -- Create a title fontstring
        local title = mainFrame:CreateFontString("MainTitle", "OVERLAY")
        title:SetSize(300, 20)
        title:SetPoint("TOP", 0, -15)
        title:SetText("WoW UI Simulator")
        title:SetTextColor(1.0, 0.82, 0.0, 1.0)  -- Gold text

        -- Status text (created before buttons so they can reference it)
        local status = mainFrame:CreateFontString("StatusText", "OVERLAY")
        status:SetSize(200, 16)
        status:SetPoint("BOTTOM", 0, 60)
        status:SetText("Status: Ready")
        status:SetTextColor(0.2, 1.0, 0.2, 1.0)  -- Green

        -- Create buttons with mouse interaction
        local btn1 = CreateFrame("Button", "AcceptButton", mainFrame)
        btn1:SetSize(100, 28)
        btn1:SetPoint("BOTTOMLEFT", 30, 25)
        btn1:SetText("Accept")
        btn1:EnableMouse(true)
        btn1:SetScript("OnClick", function(self, button)
            status:SetText("Status: Accepted!")
            status:SetTextColor(0.2, 1.0, 0.2, 1.0)
        end)
        btn1:SetScript("OnEnter", function(self)
            self:SetText("> Accept <")
        end)
        btn1:SetScript("OnLeave", function(self)
            self:SetText("Accept")
        end)

        local btn2 = CreateFrame("Button", "DeclineButton", mainFrame)
        btn2:SetSize(100, 28)
        btn2:SetPoint("BOTTOMRIGHT", -30, 25)
        btn2:SetText("Decline")
        btn2:EnableMouse(true)
        btn2:SetScript("OnClick", function(self, button)
            status:SetText("Status: Declined")
            status:SetTextColor(1.0, 0.3, 0.3, 1.0)
        end)
        btn2:SetScript("OnEnter", function(self)
            self:SetText("> Decline <")
        end)
        btn2:SetScript("OnLeave", function(self)
            self:SetText("Decline")
        end)

        -- Create a colored texture
        local icon = mainFrame:CreateTexture("IconTexture", "ARTWORK")
        icon:SetSize(64, 64)
        icon:SetPoint("TOP", 0, -50)
        icon:SetVertexColor(0.3, 0.7, 1.0, 1.0)  -- Blue tint

        -- Create a sidebar panel
        local sidebar = CreateFrame("Frame", "Sidebar", UIParent)
        sidebar:SetSize(150, 350)
        sidebar:SetPoint("LEFT", 30, 0)
        sidebar:SetBackdrop({
            bgFile = "Interface\\Tooltips\\UI-Tooltip-Background",
            edgeFile = "Interface\\Tooltips\\UI-Tooltip-Border",
            edgeSize = 2,
            insets = { left = 3, right = 3, top = 3, bottom = 3 }
        })
        sidebar:SetBackdropColor(0.05, 0.05, 0.1, 0.9)
        sidebar:SetBackdropBorderColor(0.5, 0.4, 0.15, 1.0)

        -- Sidebar title
        local sideTitle = sidebar:CreateFontString("SideTitle", "OVERLAY")
        sideTitle:SetSize(140, 18)
        sideTitle:SetPoint("TOP", 0, -10)
        sideTitle:SetText("Events")
        sideTitle:SetTextColor(1.0, 1.0, 1.0, 1.0)

        -- Event log text
        local eventLog = sidebar:CreateFontString("EventLog", "OVERLAY")
        eventLog:SetSize(140, 200)
        eventLog:SetPoint("TOP", 0, -35)
        eventLog:SetText("(no events)")
        eventLog:SetTextColor(0.7, 0.7, 0.7, 1.0)

        -- Register for events on the main frame
        mainFrame:RegisterEvent("ADDON_LOADED")
        mainFrame:RegisterEvent("PLAYER_LOGIN")
        mainFrame:RegisterEvent("PLAYER_ENTERING_WORLD")
        mainFrame:SetScript("OnEvent", function(self, event, ...)
            local args = {...}
            local argStr = ""
            for i, v in ipairs(args) do
                if i > 1 then argStr = argStr .. ", " end
                argStr = argStr .. tostring(v)
            end

            -- Print to console
            if argStr ~= "" then
                print(event .. ": " .. argStr)
                eventLog:SetText(event .. "\n" .. argStr)
            else
                print(event)
                eventLog:SetText(event)
            end
            eventLog:SetTextColor(0.3, 1.0, 0.3, 1.0)
        end)

        -- Fire Event button
        local fireBtn = CreateFrame("Button", "FireEventButton", sidebar)
        fireBtn:SetSize(120, 24)
        fireBtn:SetPoint("BOTTOM", 0, 40)
        fireBtn:SetText("Fire Event")
        fireBtn:EnableMouse(true)
        fireBtn:SetScript("OnClick", function(self)
            FireEvent("ADDON_LOADED", "TestAddon")
        end)
        fireBtn:SetScript("OnEnter", function(self)
            self:SetText("> Fire Event <")
        end)
        fireBtn:SetScript("OnLeave", function(self)
            self:SetText("Fire Event")
        end)

        -- Login button
        local loginBtn = CreateFrame("Button", "LoginButton", sidebar)
        loginBtn:SetSize(120, 24)
        loginBtn:SetPoint("BOTTOM", 0, 70)
        loginBtn:SetText("Player Login")
        loginBtn:EnableMouse(true)
        loginBtn:SetScript("OnClick", function(self)
            FireEvent("PLAYER_LOGIN")
        end)
        loginBtn:SetScript("OnEnter", function(self)
            self:SetText("> Login <")
        end)
        loginBtn:SetScript("OnLeave", function(self)
            self:SetText("Player Login")
        end)

        print("Demo frames with WoW styling created!")
        "#,
    )?;

    // Run the UI
    run_ui(env)?;

    Ok(())
}
