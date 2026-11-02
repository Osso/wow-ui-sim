//! User-approved Forever offline credits, not a native credits catalog.
//!
//! The original credits selector requires at least one row. This explicit offline
//! entry uses existing Classic artwork and a simulator notice, without attributing
//! fabricated credits to Blizzard. Retire these defaults when installed-product
//! credits metadata and file-data-ID text loading provide the real catalog.

const OFFLINE_CREDITS_LUA: &str = r#"
if rawget(_G, "GetCreditsDisplayInfo") == nil then
    function GetCreditsDisplayInfo()
        return {
            {
                isDefault = true,
                creditsFileID = 0,
                keyArt = "CreditsScreen-Keyart-0",
                background = "CreditsScreen-Background-0",
                name = "Offline simulator",
                logo = 0,
            },
        }
    end
end

if rawget(_G, "GetCreditsTextByFileDataID") == nil then
    function GetCreditsTextByFileDataID(fileDataID)
        if fileDataID == 0 then
            return "<html><body><p>Credits are unavailable in the offline simulator.</p></body></html>"
        end
        return nil
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(OFFLINE_CREDITS_LUA)?;
    Ok(())
}
