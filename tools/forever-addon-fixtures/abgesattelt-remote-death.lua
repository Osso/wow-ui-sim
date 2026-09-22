-- Abgesattelt 8931441 bounded remote-death/ranking workflow.
-- Run after normal isolated startup; uses simulator event injection only.
local function check(label, value)
    assert(value, label)
    print('ABGESATTELT_CHECK', label, 'PASS')
end

check('addon loaded', C_AddOns.IsAddOnLoaded('Abgesattelt'))
check('database initialized', AbgesatteltDB and type(AbgesatteltDB.Players) == 'table')

-- Abgesattelt.lua:343-351 HandleRemoteDeath updates only newer counts,
-- updates lastDeath, and plays the supplied sound.  The addon receives this
-- through CHAT_MSG_ADDON at Abgesattelt.lua:754-773 after prefix/channel parsing.
local function remote_death(name, deaths, sound)
    A_Admin.FireEvent('CHAT_MSG_ADDON', 'ABGESATTELT',
        name .. ';' .. deaths .. ';' .. sound, 'GUILD', 'GuildWitness')
end

remote_death('NamedRaider', 3, 1)
local first = AbgesatteltDB.Players.NamedRaider
check('remote death creates named entry', first ~= nil)
check('remote death records count', first.deaths == 3)
check('remote death records timestamp', type(first.lastDeath) == 'number')

-- Lower reports must not regress the ranking/count (HandleRemoteDeath:347).
remote_death('NamedRaider', 2, 1)
check('lower report does not regress count', AbgesatteltDB.Players.NamedRaider.deaths == 3)

-- A newer report advances the same named player.
remote_death('NamedRaider', 5, 1)
check('newer report advances count', AbgesatteltDB.Players.NamedRaider.deaths == 5)

-- A second player makes ranking input nontrivial; the addon stores both entries.
remote_death('SecondRaider', 2, 1)
check('second player recorded', AbgesatteltDB.Players.SecondRaider.deaths == 2)
check('ranking has two named players', AbgesatteltDB.Players.NamedRaider.deaths > AbgesatteltDB.Players.SecondRaider.deaths)

print('ABGESATTELT_MAJOR_INTERACTION_PASS',
    AbgesatteltDB.Players.NamedRaider.deaths,
    AbgesatteltDB.Players.SecondRaider.deaths)
