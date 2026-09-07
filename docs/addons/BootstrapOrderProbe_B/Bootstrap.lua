-- Standalone: bootstrap may precede A. Keep collector identical to A.lua.
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
BootstrapOrderProbeRecord("B:bootstrap")
