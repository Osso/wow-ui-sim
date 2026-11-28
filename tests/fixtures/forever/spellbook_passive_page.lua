-- Camelot/SpellBook/Blizzard_SpellBookFrame.lua:115-128 builds one real tab per
-- skill line. Shared SpellBookFrame.lua:285-295 minimizes to one view. The
-- Protection catalog (spellbook_data.rs:79-94) puts 85043 on page two.
local book = PlayerSpellsFrame.SpellBookFrame
local skillLine = C_SpellBook.GetSpellBookSkillLineInfo(3)
assert(skillLine.name == "Protection")
book.CategoryTabSystem:GetTabButton(3):Click()
local paged = book.PagedSpellsFrame
local controls = paged.PagingControls
assert(book:GetTab() == 3 and controls.PageText:GetText() == "Page 1/2")
assert(controls.NextPageButton:IsEnabled())
controls.NextPageButton:Click()
assert(controls.PageText:GetText() == "Page 2/2" and not controls.NextPageButton:IsEnabled())
for _, item in paged:EnumerateFrames() do
    if item:IsShown() and item.spellBookItemInfo and item.spellBookItemInfo.spellID == 85043 then
        assert(item.spellBookItemInfo.isPassive and item.Button:IsShown())
        return
    end
end
error("Grand Crusader must be visible on Protection page two")
