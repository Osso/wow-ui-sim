-- Forever 70707: Blizzard_PlayerSpells.toc:31-46 selects the Camelot templates,
-- shared spellbook frame/item XML and Camelot sizing overrides.
local EPS = 0.2
local function near(actual, expected, label)
    assert(type(actual) == "number" and math.abs(actual - expected) <= EPS,
        label .. " expected " .. expected .. ", got " .. tostring(actual))
end

local function rect(frame)
    local l, b, w, h = frame:GetRect()
    assert(l and b and w and h, "missing rect")
    return { l = l, b = b, w = w, h = h, r = l + w, t = b + h }
end

local function lock_rect(frame, l, b, w, h, label)
    local actual = rect(frame)
    near(actual.l, l, label .. " left")
    near(actual.b, b, label .. " bottom")
    near(actual.w, w, label .. " width")
    near(actual.h, h, label .. " height")
    return actual
end

local panel = assert(PlayerSpellsFrame)
local book = assert(panel.SpellBookFrame)
local paged = assert(book.PagedSpellsFrame)
assert(panel:IsShown() and book:IsShown() and paged:IsShown())

-- Camelot/Blizzard_PlayerSpellsFrame.lua:37-54,94-110: first open minimizes,
-- changes area to left. Registration.lua:4-11 gives the fit buffers and y offset;
-- UIParentPanelManager/Shared/UIParentPanelManager.lua:244-258,281-288,656-681,
-- 950-989,1007-1020 and UIPanelLayoutFrame.lua:1-9 give first-panel position.
-- SharedXMLBase/FrameUtil.lua:169-178 supplies the fit scale.
assert(panel:IsMinimized() and book.isMinimized)
assert(GetUIPanel("center") == panel and not GetUIPanel("left"))
assert(panel:GetAttribute("UIPanelLayout-area") == "left")
local screenWidth, screenHeight = UIParent:GetSize()
near(screenWidth, 1024, "screen width")
near(screenHeight, 768, "screen height")
local scale = math.min(1, screenWidth / (577 + 200), screenHeight / (720 + 140))
near(panel:GetScale() * 100, scale * 100, "fit scale percent")
local top = math.max(10, math.min(116 - 75, screenHeight - 140 - 720 * scale))
local p = lock_rect(panel, (16 + 32) / scale, (screenHeight - top) / scale - 720,
    577, 720, "minimized panel")
lock_rect(panel.NineSlice, p.l, p.b, p.w, p.h, "NineSlice")
assert(panel.NineSlice:IsShown())
-- Camelot/Blizzard_PlayerSpellsFrame.lua:57-62 removes the outer tab system.
assert(not panel.TabSystem:IsShown())

-- Camelot/Blizzard_PlayerSpellsFrame.xml:51-59 defines the book bottom inset,
-- minimized width and authored height. Shared SpellBookFrame.lua:285-319
-- switches the visible art and view count when the panel is minimized.
local s = lock_rect(book, p.l, p.b + 4, 574, 702, "minimized book")
local q = lock_rect(paged, s.l, s.b, s.w, s.h - 50, "paged spells")
assert(paged.viewsPerPage == 1 and paged.columnsPerRow == 2)
assert(q.l >= s.l - EPS and q.b >= s.b - EPS and q.r <= s.r + EPS and q.t <= s.t + EPS)

-- Camelot/SpellBook/Blizzard_SpellBookTemplates.xml:19-54:
-- one 448x520 view, its hidden second view, zero-height top bar and the
-- full-width minimized paper with two separately anchored hidden half pages.
assert(paged.View1:IsShown() and not paged.View2:IsShown())
local view = lock_rect(paged.View1, q.l + 85, q.t - 45 - 520, 448, 520, "first view")
lock_rect(paged.View2, q.r - 50 - 448, q.t - 45 - 520, 448, 520, "second view")
assert(book.BookBGHalved:IsShown() and not book.BookBGLeft:IsShown() and not book.BookBGRight:IsShown())
lock_rect(book.BookBGHalved, s.l, s.b, 574, 702, "minimized paper")
lock_rect(book.BookBGLeft, s.l, s.b, 287, 702, "left half")
lock_rect(book.BookBGRight, s.l + 287, s.b, 287, 702, "right half")
assert(book.BookBGHalved:GetAtlas():lower() == "spellbook-page-right-c60")
assert(book.TopBar:IsShown())
lock_rect(book.TopBar, s.l, s.t, 574, 0, "top bar")
-- Shared SpellBookFrame.xml:16-21 supplies the corner rect; Camelot
-- SpellBookFrame.lua:49-56 deliberately hides the flipbook on show.
assert(not book.BookCornerFlipbook:IsShown())
lock_rect(book.BookCornerFlipbook, s.r - 15 - 150, s.b + 6, 150, 155, "book corner")

-- Shared SpellBookFrame.xml:143-153 anchors paging -75,+40 with spacing 8.
-- Blizzard_PagedContent/Blizzard_PagingControls.xml:4-25,39-78 and .lua:110-127
-- supply the two 32px arrows and formatted single-page controls.
-- Blizzard_SharedXML/LayoutFrame.lua:374-439 sums child widths plus spacing;
-- PageText width is font-dependent, not the Retail fixed observation (148).
local paging = paged.PagingControls
local prev, next, text = paging.PrevPageButton, paging.NextPageButton, paging.PageText
assert(paging:IsShown() and prev:IsShown() and next:IsShown() and text:IsShown())
assert(text:GetText() == "Page 1/1" and text:GetStringWidth() > 0)
assert(not prev:IsEnabled() and not next:IsEnabled())
local t = rect(text)
local pagingWidth = t.w + 32 + 32 + 2 * 8
local controls = lock_rect(paging, q.r - 75 - pagingWidth, q.b + 40, pagingWidth, 32, "paging")
near(t.l, controls.l, "page label left")
lock_rect(prev, t.r + 8, controls.b, 32, 32, "previous arrow")
lock_rect(next, controls.r - 32, controls.b, 32, 32, "next arrow")

-- Shared SpellBookItem.xml:101-154,201-210 and SpellBookItem.lua:681-731:
-- General's known Auto Attack uses the active square art, a centered 36px
-- icon/mask within a 40px button, and border offsets -11,+1 and +1,-7.
local firstSpell
for _, item in paged:EnumerateFrames() do
    if item:IsShown() and item.HasValidData and item:HasValidData()
        and item.spellBookItemInfo.spellID == 6603 then
        firstSpell = item
        break
    end
end
assert(firstSpell and firstSpell.Button:IsShown(), "visible Auto Attack item")
local button = firstSpell.Button
local b = rect(button)
near(b.w, 40, "button width")
near(b.h, 40, "button height")
assert(b.l >= view.l - EPS and b.b >= view.b - EPS and b.r <= view.r + EPS and b.t <= view.t + EPS)
lock_rect(button.Icon, b.l + 2, b.b + 2, 36, 36, "icon")
lock_rect(button.IconMask, b.l + 2, b.b + 2, 36, 36, "icon mask")
assert(button.IconMask:IsShown() and button.Border:IsShown())
assert(button.Border:GetAtlas() == "spellbook-item-iconframe")
lock_rect(button.Border, b.l - 11, b.b - 7, 52, 48, "active border")
return "ok"
