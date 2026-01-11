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
        -- Create a main frame
        local mainFrame = CreateFrame("Frame", "MainFrame", UIParent)
        mainFrame:SetSize(300, 200)
        mainFrame:SetPoint("CENTER", 0, 0)

        -- Create a header frame
        local header = CreateFrame("Frame", "HeaderFrame", mainFrame)
        header:SetSize(280, 30)
        header:SetPoint("TOP", 0, -10)

        -- Create a button
        local button = CreateFrame("Button", "MyButton", mainFrame)
        button:SetSize(100, 30)
        button:SetPoint("BOTTOM", 0, 20)

        -- Create a sidebar
        local sidebar = CreateFrame("Frame", "Sidebar", UIParent)
        sidebar:SetSize(150, 300)
        sidebar:SetPoint("LEFT", 50, 0)

        -- Create a frame in the corner
        local corner = CreateFrame("Frame", "CornerFrame", UIParent)
        corner:SetSize(100, 100)
        corner:SetPoint("TOPRIGHT", -20, -20)

        print("Demo frames created!")
        "#,
    )?;

    // Run the UI
    run_ui(env)?;

    Ok(())
}
