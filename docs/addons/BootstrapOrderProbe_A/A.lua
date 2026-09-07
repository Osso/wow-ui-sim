-- Also installed by B's standalone bootstrap: either may execute first.
if not BootstrapOrderProbeRecord then
    local session = { events = {}, counts = {}, expectedBuild = "69594" }
    BootstrapOrderProbeSession = session
    BootstrapOrderProbeRecord = function(tag)
        local version, build, date, interface = GetBuildInfo()
        session.counts[tag] = (session.counts[tag] or 0) + 1
        local event = {
            tag = tag, sequence = #session.events + 1, count = session.counts[tag],
            build = { version = version, build = build, date = date, interface = interface,
                matchesExpectedBuild = build == "69594" and interface == 120105 },
            addons = {}, errors = {},
            helpers = { clickBinding = type(InClickBindingMode), collections = type(ToggleCollectionsJournal) },
        }
        for _, name in ipairs({ "BootstrapOrderProbe_B", "Blizzard_ClickBindingUI", "Blizzard_Collections" }) do
            local ok, loaded, finished = pcall(C_AddOns.IsAddOnLoaded, name)
            if ok then
                event.addons[name] = { loaded = loaded, finished = finished }
            else
                event.errors[name] = tostring(loaded)
            end
        end
        session.events[#session.events + 1] = event
        print("[BootstrapOrderProbe] " .. event.sequence .. " " .. tag .. " #" .. event.count .. " build " .. tostring(build))
        return event
    end
end
BootstrapOrderProbeRecord("A:eager")

local function loadCount(capture)
    return capture and capture.counts and capture.counts["load:after"] or 0
end

-- Publish only after the result payload is attached, even if the saved table was detached.
function BootstrapOrderProbePublish()
    local session = BootstrapOrderProbeSession
    session.complete = loadCount(session) >= 2
    if loadCount(BootstrapOrderProbeDB) >= 2 and not session.complete then
        BootstrapOrderProbeDB.pendingCapture = session
    else
        BootstrapOrderProbeDB = session
    end
end

-- Retain saved load results across reloads; startup-only data remains replaceable.
local frame = CreateFrame("Frame")
frame:RegisterEvent("ADDON_LOADED")
frame:RegisterEvent("PLAYER_LOGIN")
frame:SetScript("OnEvent", function(_, event, name)
    if event == "ADDON_LOADED" and name == "BootstrapOrderProbe_A" then
        if loadCount(BootstrapOrderProbeDB) == 0 then
            BootstrapOrderProbeDB = BootstrapOrderProbeSession
        end
    elseif event == "PLAYER_LOGIN" then
        BootstrapOrderProbeRecord("PLAYER_LOGIN")
    end
end)
