-- Proposed read-only CustomSearch-1.0 interaction fixture.
-- Requires the exact cached CustomSearch-1.0 package plus LibStub.
local Search = LibStub('CustomSearch-1.0')
assert(Search, 'CustomSearch-1.0 unavailable')

local Filters = {
  text = {
    tags = {'n', 'name'},
    canSearch = function(self, operator, word)
      if not operator then return word end
    end,
    match = function(self, article, operator, word)
      return Search:Find(word, article.name, article.category)
    end,
  },
  count = {
    tags = {'c', 'count'},
    canSearch = function(self, operator, word, article)
      if operator then return tonumber(word) end
    end,
    match = function(self, article, operator, wanted)
      return Search:Compare(operator, article.count, wanted)
    end,
  },
}

local apple = {name = 'Golden Apple', category = 'Fruit', count = 12}
local sword = {name = 'Iron Sword', category = 'Weapon', count = 3}
local apple_small = {name = 'Apple Seed', category = 'Reagent', count = 2}

local function check(label, actual, expected)
  actual = not not actual
  print('CUSTOMSEARCH', label, actual == expected and 'PASS' or 'FAIL', tostring(actual), tostring(expected))
  assert(actual == expected, label)
end

-- Direct utility contracts from the README/source.
check('find-accent-insensitive', Search:Find('joao', 'Violets are jóaô'), true)
check('find-negative', Search:Find('banana', 'Roses are red'), false)
check('compare-ge', Search:Compare('>=', 5, 5), true)
check('compare-negative', Search:Compare('>', 3, 4), false)

-- Matches: whitespace-separated terms are AND; the localized OR token is OR;
-- the localized NOT token (normally "Not" on enUS) negates the next term.
check('and-positive', Search:Matches(apple, 'apple fruit', Filters), true)
check('and-negative', Search:Matches(sword, 'apple fruit', Filters), false)
check('or-positive', Search:Matches(sword, 'apple or sword', Filters), true)
check('or-negative', Search:Matches(sword, 'apple or banana', Filters), false)
check('not-positive', Search:Matches(apple, 'apple Not sword', Filters), true)
check('not-negative', Search:Matches(apple, 'apple Not fruit', Filters), false)

-- A tag selects only filters whose tags start with the query tag.
check('tag-positive', Search:Matches(apple, 'n:apple', Filters), true)
check('tag-negative', Search:Matches(apple, 'c:apple', Filters), false)

-- Numeric operators are passed to the filter and compared by Search:Compare.
check('numeric-greater-positive', Search:Matches(apple, '>10', Filters), true)
check('numeric-greater-negative', Search:Matches(apple_small, '>10', Filters), false)
check('numeric-tag-positive', Search:Matches(apple, 'c:>=10', Filters), true)
check('numeric-tag-negative', Search:Matches(apple_small, 'c:>=10', Filters), false)

print('CUSTOMSEARCH DONE')
