-- Pure-Lua taint system stubs replacing Elune's C security library.
--
-- Provides permissive implementations: issecure() always true,
-- issecurevariable() always true, securecall just calls the function.
-- Taint tracking is a no-op — all code is treated as secure.

-- Taint query
function issecure()
    return true
end

function issecurevariable(tblOrName, nameOrNil)
    return true, nil
end

-- Secure calling (just forwards to the function)
function securecall(func, ...)
    return func(...)
end

function securecallfunction(func, ...)
    return func(...)
end

-- Taint manipulation (no-ops)
function forceinsecure()
end

-- Hook a function while preserving taint semantics
function hooksecurefunc(tblOrName, nameOrHook, hookOrNil)
    local obj, name, hook
    if hookOrNil == nil then
        -- 2-arg: hooksecurefunc("globalName", hookfn)
        obj = _G
        name = tblOrName
        hook = nameOrHook
    else
        -- 3-arg: hooksecurefunc(obj, "method", hookfn)
        obj = tblOrName
        name = nameOrHook
        hook = hookOrNil
    end
    local orig = obj[name]
    if type(orig) ~= "function" then return end
    rawset(obj, name, function(...)
        local results = { orig(...) }
        hook(...)
        return unpack(results)
    end)
end

-- Execute a function for each key in a table with clean taint
function secureexecuterange(tbl, func, ...)
    if type(tbl) ~= "table" then return end
    for k, v in pairs(tbl) do
        securecallfunction(func, k, v, ...)
    end
end

-- Debug taint functions
-- newsecurefunction wraps a function to run with clean (nil) taint.
-- Without Elune, just return the function as-is.
if not debug.newsecurefunction then
    debug.newsecurefunction = function(func) return func end
end

-- iscfunction checks if a value is a C function (used by CreateCallback validation)
if not debug.iscfunction then
    debug.iscfunction = function(func) return false end
end

if not debug.setobjecttaint then
    debug.setobjecttaint = function() end
end
if not debug.getstacktaint then
    debug.getstacktaint = function() return nil end
end
if not debug.setstacktaint then
    debug.setstacktaint = function() end
end
if not debug.settaintmode then
    debug.settaintmode = function() end
end

-- WoW globals that Elune provided from the os library
if not time then time = os.time end
if not date then date = os.date end
if not difftime then difftime = os.difftime end

-- Value access helpers (permissive stubs)
if not issecretvalue then
    issecretvalue = function() return false end
end
if not canaccessvalue then
    canaccessvalue = function() return true end
end
if not canaccessallvalues then
    canaccessallvalues = function() return true end
end
if not canaccesstable then
    canaccesstable = function() return true end
end
if not scrub then
    scrub = function(...) return ... end
end
if not scrubsecretvalues then
    scrubsecretvalues = function(...) return ... end
end

-- Error handler (WoW expects these to exist)
if not geterrorhandler then
    local _errorHandler
    function seterrorhandler(handler)
        _errorHandler = handler
    end
    function geterrorhandler()
        return _errorHandler
    end
end
