-- Observe unchanged BigWigs registration and normal deferred Sounds activation.
assert(SlashCmdList.BigWigs, "BigWigs slash missing")()
assert(BigWigsOptions and BigWigsOptions:IsOpen(), "options did not open")
local media = LibStub("LibSharedMedia-3.0")
local sounds = assert(BigWigs:GetPlugin("Sounds"), "Sounds plugin missing")
local files = {
    {"Long", "Long.ogg"},
    {"Info", "Info.ogg"},
    {"Alarm", "Alarm.ogg"},
    {"Alert", "Alert.ogg"},
    {"underyou", "spell_under_you.ogg"},
}
for _, entry in ipairs(files) do
    local key, file = entry[1], entry[2]
    local path = "Interface\\AddOns\\BigWigs\\Media\\Sounds\\" .. file
    assert(C_UIFileAsset.IsKnownFile(path), file .. " not known")
    assert(C_UIFileAsset.IsLooseFile(path), file .. " not loose")
    assert(C_UIFileAsset.GetFileID(path) == nil, file .. " has fabricated file ID")
    local name = assert(sounds.defaultDB.media[key], "default sound missing")
    assert(media:IsValid("sound", name), name .. " not registered")
    assert(media:Fetch("sound", name, true) == path, name .. " resolves to wrong file")
    assert(sounds.db.profile.media[key] == name, name .. " profile changed")
    print("BIGWIGS_SOUND_ASSETS", file, "registered", name)
end
assert(media:IsValid("statusbar", "Otravi"), "extensionless Otravi not registered")
print("BIGWIGS_SOUND_ASSETS", "DONE")
