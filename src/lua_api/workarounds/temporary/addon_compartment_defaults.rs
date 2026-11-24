//! Temporary AddonCompartment frame defaults.
//!
//! Blizzard_Minimap owns the real addon-compartment frame and mixin. These
//! shallow methods keep startup and isolated tests functional until that frame
//! surface is modeled without generic runtime bootstrap help.

const ADDON_COMPARTMENT_DEFAULTS_LUA: &str = r#"
local function ensure_addon_compartment_frame()
    local frame = rawget(_G, "AddonCompartmentFrame")
    if frame ~= nil then
        return frame
    end
    if type(CreateFrame) ~= "function" then
        frame = {}
    else
        frame = CreateFrame("Button", "AddonCompartmentFrame", UIParent)
    end
    rawset(_G, "AddonCompartmentFrame", frame)
    return frame
end

local function remove_registered_addon(addons, addon)
    if addon == nil then
        table.remove(addons)
        return
    end
    for index = #addons, 1, -1 do
        if addons[index] == addon then
            table.remove(addons, index)
            return
        end
    end
end

local frame = ensure_addon_compartment_frame()
frame.registeredAddons = frame.registeredAddons or {}
if frame.RegisterAddon == nil then
    function frame:RegisterAddon(addon)
        self.registeredAddons = self.registeredAddons or {}
        self.registeredAddons[#self.registeredAddons + 1] = addon or true
    end
end
if frame.UnregisterAddon == nil then
    function frame:UnregisterAddon(addon)
        self.registeredAddons = self.registeredAddons or {}
        remove_registered_addon(self.registeredAddons, addon)
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(ADDON_COMPARTMENT_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_addon_compartment_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if type(AddonCompartmentFrame) ~= "table" then return "frame" end
                if type(AddonCompartmentFrame.RegisterAddon) ~= "function" then return "register" end
                if type(AddonCompartmentFrame.UnregisterAddon) ~= "function" then return "unregister" end
                AddonCompartmentFrame:RegisterAddon("One")
                AddonCompartmentFrame:RegisterAddon("Two")
                AddonCompartmentFrame:UnregisterAddon("One")
                if #AddonCompartmentFrame.registeredAddons ~= 1 then return "remove_named" end
                if AddonCompartmentFrame.registeredAddons[1] ~= "Two" then return "remaining" end
                AddonCompartmentFrame:UnregisterAddon()
                if #AddonCompartmentFrame.registeredAddons ~= 0 then return "remove_tail" end
                return "ok"
                "#,
            )
            .expect("addon compartment defaults probe should run");

        assert_eq!(result, "ok");
    }
}
