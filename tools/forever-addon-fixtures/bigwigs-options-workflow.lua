assert(SlashCmdList.BigWigs, "BigWigs slash missing")()
for _, name in ipairs({"BigWigs_Core", "BigWigs_Plugins", "BigWigs_Options"}) do
    local _, loaded = C_AddOns.IsAddOnLoaded(name)
    assert(loaded, name .. " did not load")
    print("BIGWIGS_WORKFLOW", "loaded", name)
end
assert(BigWigsOptions and BigWigsOptions:IsOpen(), "options did not open")
print("BIGWIGS_WORKFLOW", "options-open")
print("BIGWIGS_WORKFLOW", "DONE")
