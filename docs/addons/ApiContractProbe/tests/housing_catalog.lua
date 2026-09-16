local root = assert(arg[1])
local passed, excluded = 0, 0
local secret = newproxy(true)
getmetatable(secret).__index = function() error("secret lookup") end
getmetatable(secret).__tostring = function() error("secret stringify") end
local function forbidden() excluded = excluded + 1; error("excluded operation") end
local categoryKeys = { "ID", "displayName", "iconTexture", "linkTag", "isDisabled", "showPersistentRefundButton" }
local refundKeys = { "decorGUID", "timeRemainingSeconds", "name", "price" }
local function setup(products, category, refunds, featured)
    ApiContractProbeDB, SlashCmdList, excluded = nil, {}, 0
    Constants = nil
    issecretvalue = function(v) return rawequal(v, secret) end
    canaccessvalue = function(v) return not rawequal(v, secret) end
    GetBuildInfo, time = function() return "fixture" end, function() return 42 end
    C_HousingCatalog = { HasFeaturedEntries = featured, RequestHousingMarketInfo = forbidden }
    C_CatalogShop = { GetNewProducts = products, GetFirstCategoryByProductID = category,
        GetRefundableDecors = refunds, GetVirtualCurrencyBalance = forbidden,
        RefreshRefundableDecors = forbidden, RefundProduct = forbidden,
        BulkPurchaseProducts = forbidden, ConfirmHousingPurchase = forbidden,
        RefreshVirtualCurrencyBalance = forbidden, RequestCatalogShop = forbidden }
    C_AddOns, LoadAddOn = { LoadAddOn = forbidden }, forbidden
    local toc = assert(io.open(root .. "/ApiContractProbe.toc"))
    for line in toc:lines() do
        if line:match("%.lua$") then assert(loadfile(root .. "/" .. line))() end
    end
    toc:close()
end
local function capture(label)
    SlashCmdList.APICONTRACTPROBE("housing-catalog " .. (label or "fixture"))
    assert(ApiContractProbeDB and #ApiContractProbeDB.captures > 0, "housing-catalog mode absent")
    return assert(ApiContractProbeDB.captures[#ApiContractProbeDB.captures].housingCatalog)
end
local function entries(r) return r.products.values[1].entries end
local function test(name, fn)
    fn(); assert(excluded == 0); passed = passed + 1; print("PASS " .. name)
end

test("original IDs exact arguments declared fields and omitted refund filter", function()
    local calls, ids = 0, {}
    setup(function(...) assert(select("#", ...) == 0); calls = calls + 1; return { 1.25, -2.5 } end,
        function(...) assert(select("#", ...) == 1); local id = ...; ids[#ids + 1] = id
            return { ID = id, displayName = "category", iconTexture = "atlas", linkTag = "tag",
                isDisabled = false, showPersistentRefundButton = true }, nil end,
        function(...) assert(select("#", ...) == 0); calls = calls + 1
            return { { decorGUID = "guid", timeRemainingSeconds = 22, name = "decor", price = "12 gold" } }, 19 end,
        function(...) assert(select("#", ...) == 0); calls = calls + 1; return calls end)
    local r = capture(); local e = entries(r)
    assert(calls == 4 and #ids == 2 and ids[1] == 1.25 and ids[2] == -2.5)
    assert(#e == 8 and e[1].category.n == 2 and e[1].category.values[2].kind == "nil")
    local f = e[1].category.values[1].fields
    for _, key in ipairs(categoryKeys) do assert(f[key].status == "observed") end
    assert(f.ID.value == 1.25 and f.showPersistentRefundButton.value and f.isDisabled.value == false)
    f = r.refundable.values[1].entries[1].fields
    assert(f.decorGUID.value == "guid" and f.price.value == "12 gold" and f.standaloneDecorProductID == nil)
    assert(r.refundable.n == 2 and r.refundable.values[2].value == 19)
end)

test("missing inaccessible and throwing APIs do not suppress peers", function()
    setup(nil, nil, nil, nil)
    local r = capture(); assert(r.featured[1].status == "missing-api" and r.products.status == "missing-api")
    C_CatalogShop = secret; r = capture(); assert(r.products.status == "field-error")
    C_CatalogShop = setmetatable({}, { __index = function() error(secret) end })
    assert(capture().refundable.status == "field-error")
    setup(function() return { 1 } end, secret, function() return {} end, function() error(secret) end)
    r = capture(); assert(r.featured[2].status == "call-error" and r.refundable.status == "observed")
    assert(entries(r)[1].category.status == "missing-api")
end)

test("invalid IDs remain unavailable while original peers continue", function()
    for _, bad in ipairs({ false, "1", math.huge, -math.huge, 0/0, secret, {}, newproxy(true) }) do
        local calls = 0
        setup(function() return { bad, 2 } end, function(id) assert(id == 2); calls = calls + 1 end)
        local e = entries(capture()); assert(calls == 1 and e[1].category.status ~= "observed")
        assert(e[2].category.status == "observed")
    end
end)

test("zero nil holes opaque errors and only first producer returns", function()
    setup(function() return nil, { 1 }, nil end, forbidden, function() return nil, 12, nil end)
    local r = capture(); assert(r.products.n == 3 and r.products.values[2].entries == nil)
    assert(r.refundable.n == 3 and r.refundable.values[2].value == 12)
    setup(function() return { 1, 2, 3 } end, function(id)
        if id == 1 then error(secret) elseif id == 2 then return nil, { ID = 7 }, nil end
    end, function() error(secret) end, function() end)
    r = capture(); local e = entries(r)
    assert(r.featured[1].n == 0 and r.refundable.status == "call-error")
    assert(e[1].category.status == "call-error" and e[2].category.n == 3)
    assert(e[2].category.values[2].fields == nil and e[3].category.n == 0)
end)

test("each ID rechecked after namespace lookup and both function guards", function()
    for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
        for selected = 1, 8 do
            local revoked, calls = false, 0
            local target = selected + 0.25
            local ids = {}; for i = 1, 8 do ids[i] = i + 0.25 end
            local fn = function(id) assert(not (revoked and id == target)); calls = calls + 1; return {} end
            setup(function() return ids end, nil)
            local ns = C_CatalogShop
            setmetatable(ns, { __index = function(_, key)
                if key ~= "GetFirstCategoryByProductID" then return nil end
                if phase == "lookup" then revoked = true end
                return fn
            end })
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if (phase == "namespace" and rawequal(v, ns)) or (phase == "access" and rawequal(v, fn)) then revoked = true end
                return not rawequal(v, secret) and not (revoked and v == target)
            end
            local e = entries(capture()); assert(calls == 7 and e[selected].category.status == "restricted-input")
        end
    end
end)

test("every list index rechecks its receiver", function()
    for _, which in ipairs({ "products", "refundable" }) do
        for selected = 1, 8 do
            local revoked, reads = false, 0
            local list = setmetatable({}, { __index = function(_, index)
                assert(not revoked and index <= 8); reads = reads + 1
                if index == selected then revoked = true end
                return which == "products" and index or {}
            end, __len = forbidden, __pairs = forbidden })
            setup(function() return which == "products" and list or {} end, function() return {} end,
                function() return which == "refundable" and list or {} end)
            canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, list)) end
            local r = capture(); assert(reads == selected)
            if selected < 8 then assert(r[which].values[1].entries[selected + 1].status == "field-error") end
        end
    end
end)

test("every category and refundable field rechecks the object", function()
    for _, which in ipairs({ "category", "refund" }) do
        local keys = which == "category" and categoryKeys or refundKeys
        for selected = 1, #keys do
            local revoked, reads = false, 0
            local object = setmetatable({}, { __index = function(_, key)
                assert(not revoked and key == keys[reads + 1]); reads = reads + 1
                if reads == selected then revoked = true end
                return key
            end, __pairs = forbidden, __tostring = forbidden })
            setup(function() return { 1 } end, function() return which == "category" and object or {} end,
                function() return { which == "refund" and object or {} } end)
            canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, object)) end
            local r = capture(); assert(reads == selected)
            local fields = which == "category" and entries(r)[1].category.values[1].fields or r.refundable.values[1].entries[1].fields
            if selected < #keys then assert(fields[keys[selected + 1]].status == "field-error") end
        end
    end
end)

test("serialization and peer results can revoke downstream originals", function()
    local revoked = false
    setup(function() return { 1, 2 }, "revoke" end, function(id) assert(id == 2); return {} end)
    canaccessvalue = function(v)
        if v == "revoke" then revoked = true end
        return not rawequal(v, secret) and not (revoked and v == 1)
    end
    assert(entries(capture())[1].category.status == "restricted-input")
    revoked = false
    setup(function() return { 1, 2 } end, function(id) assert(id == 1); return nil, "revoke" end)
    canaccessvalue = function(v)
        if v == "revoke" then revoked = true end
        return not rawequal(v, secret) and not (revoked and v == 2)
    end
    assert(entries(capture())[2].category.status == "restricted-input")
end)

test("secret and wrong shaped results stay opaque and guards fail closed", function()
    for _, value in ipairs({ secret, false, 1, "list", newproxy(true) }) do
        setup(function() return value end, forbidden, function() return value end)
        local r = capture(); assert(r.products.values[1].entries == nil and r.refundable.values[1].entries == nil)
    end
    setup(function() return { 1 } end, function() return secret end, function() return { secret } end)
    local r = capture(); assert(entries(r)[1].category.values[1].status == "restricted")
    assert(r.refundable.values[1].entries[1].status == "restricted")
    setup(forbidden, forbidden, forbidden, forbidden); canaccessvalue = nil
    SlashCmdList.APICONTRACTPROBE("housing-catalog missing"); assert(ApiContractProbeDB.captures[1].status == "missing-access-api")
end)

test("twelve calls bounded tuples strings labels and ten shared snapshots", function()
    local calls, many, list = 0, {}, {}
    for i = 1, 20 do many[i] = string.rep("x", 300); list[i] = i end
    setup(function() calls = calls + 1; return list, unpack(many) end,
        function() calls = calls + 1; return { displayName = many[1] }, unpack(many) end,
        function() calls = calls + 1; return { { price = many[1] } }, unpack(many) end,
        function() calls = calls + 1; return unpack(many) end)
    for i = 1, 11 do capture(string.rep("L", 180)) end
    assert(calls == 120 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local r = ApiContractProbeDB.captures[1]; assert(#r.label == 128)
    assert(r.housingCatalog.products.n == 21 and r.housingCatalog.products.truncated)
    assert(#r.housingCatalog.products.values == 16 and #r.housingCatalog.products.values[2].value == 256)
    assert(#entries(r.housingCatalog) == 8 and #entries(r.housingCatalog)[1].category.values[1].fields.displayName.value == 256)
end)

test("manual mode excluded from all and raw objects collectible", function()
    local weak = setmetatable({}, { __mode = "v" })
    setup(function() local list = { 1 }; weak[1] = list; return list end,
        function() local c = { ID = 1 }; weak[2] = c; return c end,
        function() local entry = { price = "string" }; weak[3] = entry; return { entry } end)
    capture(); collectgarbage(); collectgarbage(); assert(weak[1] == nil and weak[2] == nil and weak[3] == nil)
    setup(forbidden, forbidden, forbidden, forbidden)
    SlashCmdList.APICONTRACTPROBE("all fixture"); assert(ApiContractProbeDB.captures[1].housingCatalog == nil)
end)
test("category followups preserve original fractional IDs without deduplication", function()
    local calls = {}
    setup(function() return { 1, 2 } end, function() return { ID = 7.25 } end)
    C_CatalogShop.GetProductIDsForCategory = function(...)
        assert(select("#", ...) == 1 and (...) == 7.25)
        calls[#calls + 1] = ...; return { 9.5, nil, false }, nil, "tail"
    end
    local e = entries(capture())
    assert(#calls == 2, "category followup absent")
    assert(e[1].categoryProducts.n == 3 and e[1].categoryProducts.values[2].kind == "nil")
    local list = e[1].categoryProducts.values[1].entries
    assert(#list == 8 and list[1].value == 9.5 and list[2].kind == "nil" and list[3].value == false)
end)

test("category followup rejects invalid first category objects and IDs independently", function()
    for _, bad in ipairs({ false, "id", math.huge, -math.huge, 0/0, secret, {} }) do
        local calls = 0
        setup(function() return { 1, 2 } end, function(id)
            if id == 1 then return { ID = bad } end
            return { ID = 4.5 }
        end)
        C_CatalogShop.GetProductIDsForCategory = function(id) assert(id == 4.5); calls = calls + 1 end
        local e = entries(capture()); assert(calls == 1 and e[1].categoryProducts.status ~= "observed")
    end
    for _, bad in ipairs({ false, "category", 3, secret }) do
        setup(function() return { 1 } end, function() return bad, { ID = 2 } end)
        C_CatalogShop.GetProductIDsForCategory = forbidden
        assert(entries(capture())[1].categoryProducts.status ~= "observed")
    end
end)

test("category receiver and original ID rechecked after field serialization", function()
    for _, revokeObject in ipairs({ false, true }) do
        local revoked, object = false, { ID = 9.25, displayName = "revoke-category" }
        setup(function() return { 1 } end, function() return object end)
        C_CatalogShop.GetProductIDsForCategory = forbidden
        canaccessvalue = function(v)
            if v == "revoke-category" then revoked = true end
            return not rawequal(v, secret) and not (revoked and (revokeObject and rawequal(v, object) or not revokeObject and v == 9.25))
        end
        assert(entries(capture())[1].categoryProducts.status ~= "observed")
    end
end)

test("category IDs rechecked after every followup lookup and function guard", function()
    for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
        for selected = 1, 8 do
            local revoked, armed, calls = false, false, 0
            local target = selected + 100.25
            setup(function() return { 1,2,3,4,5,6,7,8 } end, function(id) return { ID = id + 100.25 } end)
            local ns = C_CatalogShop
            local fn = function(id) assert(not (revoked and id == target)); calls = calls + 1; return {} end
            setmetatable(ns, { __index = function(_, key)
                if key == "GetProductIDsForCategory" then
                    if phase == "lookup" then revoked = true end
                    return fn
                end
            end })
            local oldCategory = ns.GetFirstCategoryByProductID
            ns.GetFirstCategoryByProductID = function(id) armed = true; return oldCategory(id) end
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret)
            end
            canaccessvalue = function(v)
                if phase == "namespace" and armed and rawequal(v, ns) then revoked = true end
                if phase == "access" and rawequal(v, fn) then revoked = true end
                return not rawequal(v, secret) and not (revoked and v == target)
            end
            local e = entries(capture()); assert(calls == 7 and e[selected].categoryProducts.status == "restricted-input")
        end
    end
end)

test("followup missing errors and nil tuples do not suppress peers", function()
    setup(function() return { 1,2,3,4 } end, function(id) return { ID = id } end)
    C_CatalogShop.GetProductIDsForCategory = function(id)
        if id == 1 then error(secret) elseif id == 2 then return nil, { 99 }, nil
        elseif id == 3 then return else return { 5 } end
    end
    local e = entries(capture())
    assert(e[1].categoryProducts.status == "call-error" and e[2].categoryProducts.n == 3)
    assert(e[2].categoryProducts.values[2].entries == nil and e[3].categoryProducts.n == 0)
    assert(e[4].categoryProducts.values[1].entries[1].value == 5)
    C_CatalogShop.GetProductIDsForCategory = nil
    assert(entries(capture())[1].categoryProducts.status == "missing-api")
    C_CatalogShop.GetProductIDsForCategory = secret
    assert(entries(capture())[1].categoryProducts.status == "missing-api")
end)

test("followup list receivers rechecked at every bounded index", function()
    for selected = 1, 8 do
        local revoked, reads = false, 0
        local list = setmetatable({}, { __index = function(_, index)
            assert(not revoked and index <= 8); reads = reads + 1
            if index == selected then revoked = true end
            return index
        end, __len = forbidden, __pairs = forbidden })
        setup(function() return { 1 } end, function() return { ID = 2 } end)
        C_CatalogShop.GetProductIDsForCategory = function() return list end
        canaccessvalue = function(v) return not rawequal(v, secret) and not (revoked and rawequal(v, list)) end
        local e = entries(capture())[1].categoryProducts.values[1].entries
        assert(reads == selected)
        if selected < 8 then assert(e[selected + 1].status == "field-error") end
    end
end)

test("twenty calls bounded followup tuples and no downstream recursion", function()
    local calls, ids, many = 0, {}, {}
    for i = 1, 20 do ids[i] = i; many[i] = string.rep("x", 300) end
    setup(function() calls = calls + 1; return ids end,
        function(id) calls = calls + 1; return { ID = id + 0.25 } end,
        function() calls = calls + 1; return {} end,
        function() calls = calls + 1; return true end)
    C_CatalogShop.GetProductIDsForCategory = function() calls = calls + 1; return ids, unpack(many) end
    C_CatalogShop.GetProductInfo = forbidden
    for i = 1, 11 do capture(string.rep("L", 180)) end
    assert(calls == 200 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local q = entries(ApiContractProbeDB.captures[1].housingCatalog)[1].categoryProducts
    assert(q.n == 21 and q.truncated and #q.values == 16 and #q.values[2].value == 256)
    assert(#q.values[1].entries == 8 and #ApiContractProbeDB.captures[1].label == 128)
end)
local currencyKeys = { "HEARTHSTEEL_VC_CURRENCY_CODE", "TRADERS_TENDER_VC_CURRENCY_CODE" }
local function currencies(fn)
    setup()
    Constants = { CatalogShopVirtualCurrencyConstants = {
        [currencyKeys[1]] = string.rep("custom", 70), [currencyKeys[2]] = "other-code",
    } }
    C_CatalogShop.GetVirtualCurrencyBalance = fn
end

test("currency publication forwards original long strings and exact tuples", function()
    local seen = {}
    currencies(function(...) assert(select("#", ...) == 1); seen[#seen + 1] = ...; return nil, "balance", nil end)
    local r = capture().currencies
    assert(#seen == 2 and #seen[1] == 420 and seen[2] == "other-code")
    assert(r[currencyKeys[1]].n == 3 and r[currencyKeys[1]].values[1].kind == "nil")
    assert(r[currencyKeys[2]].values[2].value == "balance")
    assert(#r[currencyKeys[1]].input.value == 256)
end)

test("currency missing restricted invalid and throwing publications keep peers independent", function()
    for _, bad in ipairs({ false, 4, secret, {} }) do
        local calls = 0
        currencies(function(code) assert(code == "other-code"); calls = calls + 1 end)
        Constants.CatalogShopVirtualCurrencyConstants[currencyKeys[1]] = bad
        local r = capture().currencies
        assert(calls == 1 and r[currencyKeys[1]].status ~= "observed" and r[currencyKeys[2]].n == 0)
    end
    for _, container in ipairs({ secret, setmetatable({}, { __index = function() error(secret) end }) }) do
        currencies(forbidden); Constants = container
        assert(capture().currencies[currencyKeys[1]].status ~= "observed")
        currencies(forbidden); Constants.CatalogShopVirtualCurrencyConstants = container
        assert(capture().currencies[currencyKeys[2]].status ~= "observed")
    end
end)

test("currency original inputs rechecked after namespace lookup and function guards", function()
    for _, phase in ipairs({ "namespace", "lookup", "secret", "access" }) do
        for chosen = 1, 2 do
            local revoked, calls = false, 0
            local fn = function(code) assert(not revoked or code ~= "target"); calls = calls + 1 end
            currencies(fn)
            Constants.CatalogShopVirtualCurrencyConstants[currencyKeys[chosen]] = "target"
            local ns = C_CatalogShop
            ns.GetVirtualCurrencyBalance = nil
            setmetatable(ns, { __index = function(_, key)
                if key == "GetVirtualCurrencyBalance" then
                    if phase == "lookup" then revoked = true end
                    return fn
                end
            end })
            issecretvalue = function(v)
                if phase == "secret" and rawequal(v, fn) then revoked = true end
                return rawequal(v, secret) or (revoked and v == "target")
            end
            canaccessvalue = function(v)
                if (phase == "namespace" and rawequal(v, ns)) or (phase == "access" and rawequal(v, fn)) then revoked = true end
                return not rawequal(v, secret) and not (revoked and v == "target")
            end
            local r = capture().currencies
            assert(calls == 1 and r[currencyKeys[chosen]].status ~= "observed")
        end
    end
end)

test("currency container guards prevent lookup and member inspection", function()
    for _, level in ipairs({ "root", "members" }) do
        local touched = 0
        currencies(forbidden)
        local blocked = setmetatable({}, { __index = function() touched = touched + 1; error("lookup") end })
        if level == "root" then Constants = blocked else Constants.CatalogShopVirtualCurrencyConstants = blocked end
        canaccessvalue = function(v) return not rawequal(v, blocked) and not rawequal(v, secret) end
        capture(); assert(touched == 0)
    end
end)

test("currency errors zero arity and missing functions remain independent", function()
    currencies(function(code) if code == "other-code" then return end; error(secret) end)
    local r = capture().currencies
    assert(r[currencyKeys[1]].status == "call-error" and r[currencyKeys[2]].n == 0)
    C_CatalogShop.GetVirtualCurrencyBalance = secret
    assert(capture().currencies[currencyKeys[1]].status == "missing-api")
end)

test("currency extension retains twenty two call and shared serialization bounds", function()
    local calls, many = 0, {}
    for i = 1, 20 do many[i] = string.rep("x", 300) end
    currencies(function() calls = calls + 1; return unpack(many) end)
    local ids = {}; for i = 1, 8 do ids[i] = i end
    C_HousingCatalog.HasFeaturedEntries = function() calls = calls + 1 end
    C_CatalogShop.GetNewProducts = function() calls = calls + 1; return ids end
    C_CatalogShop.GetRefundableDecors = function() calls = calls + 1 end
    C_CatalogShop.GetFirstCategoryByProductID = function(id) calls = calls + 1; return { ID = id } end
    C_CatalogShop.GetProductIDsForCategory = function() calls = calls + 1; return ids end
    for i = 1, 11 do capture(string.rep("L", 180)) end
    assert(calls == 220 and #ApiContractProbeDB.captures == 10 and ApiContractProbeDB.dropped == 1)
    local q = ApiContractProbeDB.captures[1].housingCatalog.currencies[currencyKeys[1]]
    assert(q.n == 20 and q.truncated and #q.values == 16 and #q.values[1].value == 256)
    assert(#ApiContractProbeDB.captures[1].label == 128)
    ApiContractProbeDB = nil; calls = 0
    SlashCmdList.APICONTRACTPROBE("all fixture")
    assert(calls == 0)
end)
print("housing-catalog fixtures passed: " .. passed)
