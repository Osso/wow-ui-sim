//! End-to-end fenv isolation tests for secureenv.
//!
//! Secureenv is a shallow copy of `_G` retargeted onto the function
//! environment of chunks marked `[LoadIntoEnvironment secure]`. Writes
//! from secure code to env-level bindings must stay in secureenv — they
//! must not replicate to `_G`, because `_G` is what insecure addons see.
//! These tests exercise the property via the real load path (`mark_secure`
//! → `call_function`).
//!
//! Probe variable names use a lowercase prefix on purpose: our `_G`
//! metatable synthesises uppercase-only names into their own string
//! (so undefined `FOO_BAR` reads `"FOO_BAR"` instead of `nil`), which
//! defeats the "absent from _G" half of these assertions. Mixed-case
//! names skip that fallback.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn toc_multiannotations_load_files_in_order_with_secure_environment() {
    let env = env();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("AnnotationProbe.toc");
    std::fs::write(
        &toc,
        "## Title: AnnotationProbe\n\
         Before.lua\n\
         Secure.lua\t\t[Bootstrap] [LoadIntoEnvironment secure]\n\
         After.lua\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("Before.lua"),
        "local _, addon = ...; addon.order = {'before'}",
    )
    .unwrap();
    std::fs::write(
        root.path().join("Secure.lua"),
        "local _, addon = ...; assert(addon.order[1] == 'before'); \
         addon.order[#addon.order+1] = 'secure'; annotationSecureValue = 37",
    )
    .unwrap();
    std::fs::write(
        root.path().join("After.lua"),
        "local _, addon = ...; assert(table.concat(addon.order, ',') == 'before,secure'); \
         assert(rawget(_G, 'annotationSecureValue') == nil); \
         assert(__secureenv.annotationSecureValue == 37); annotationLoadComplete = true",
    )
    .unwrap();
    let result = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    env.exec("assert(annotationLoadComplete == true)").unwrap();
}

#[test]
fn secure_primitive_write_stays_inside_secureenv() {
    let env = env();

    // Before the secure chunk runs, the probe name must be absent from
    // both environments. This pins the starting point so the post-run
    // assertions prove the write landed exactly in secureenv.
    let (initial_in_g, initial_in_secureenv): (String, String) = env
        .eval(
            r#"
            return type(rawget(_G, "secureenvPrimitiveProbe")),
                   type(rawget(__secureenv, "secureenvPrimitiveProbe"))
            "#,
        )
        .unwrap();
    assert_eq!(initial_in_g, "nil");
    assert_eq!(initial_in_secureenv, "nil");

    // Run a chunk under secureenv that rebinds a primitive global.
    // Because the chunk's fenv is secureenv, the assignment lands in
    // secureenv's hash, not in _G.
    env.exec_rilua_secure(
        r#"
            secureenvPrimitiveProbe = 42
        "#,
    )
    .unwrap();

    // Verify the split: _G stays nil, secureenv now holds 42.
    let (g_type, secureenv_type, secureenv_value): (String, String, f64) = env
        .eval(
            r#"
            return type(rawget(_G, "secureenvPrimitiveProbe")),
                   type(rawget(__secureenv, "secureenvPrimitiveProbe")),
                   rawget(__secureenv, "secureenvPrimitiveProbe")
            "#,
        )
        .unwrap();
    assert_eq!(g_type, "nil", "_G must not have been mutated");
    assert_eq!(secureenv_type, "number");
    assert!((secureenv_value - 42.0).abs() < f64::EPSILON);
}

#[test]
fn secure_string_write_is_not_visible_in_global_env() {
    let env = env();

    env.exec_rilua_secure(
        r#"
            secureenvStringProbe = "only-in-secureenv"
        "#,
    )
    .unwrap();

    let (g_probe_type, secureenv_probe): (String, String) = env
        .eval(
            r#"
            return type(rawget(_G, "secureenvStringProbe")),
                   tostring(rawget(__secureenv, "secureenvStringProbe"))
            "#,
        )
        .unwrap();
    assert_eq!(g_probe_type, "nil");
    assert_eq!(secureenv_probe, "only-in-secureenv");
}

#[test]
fn insecure_chunk_still_lands_in_global_env() {
    // Contrast test: the same assignment from a non-secure chunk DOES
    // land on _G. Keeps the first test honest — if mark_secure were a
    // no-op, the two writes would behave identically.
    let env = env();

    env.exec(
        r#"
            insecureProbe = "landed-on-_G"
        "#,
    )
    .unwrap();

    let on_g: String = env
        .eval(r#"return tostring(rawget(_G, "insecureProbe"))"#)
        .unwrap();
    assert_eq!(on_g, "landed-on-_G");
}

#[test]
fn two_secure_chunks_share_one_secureenv() {
    // Blizzard's restricted addon loads several secure files in sequence
    // (RestrictedEnvironment, RestrictedExecution, etc.). They cooperate
    // by writing to secureenv-scope globals that later files read. That
    // only works if every secure chunk resolves globals through the same
    // secureenv table.
    let env = env();

    // First secure chunk defines a binding that cannot exist in _G
    // (per the fenv-isolation test above) and only lives in secureenv.
    env.exec_rilua_secure(
        r#"
            secureenvSharedBetweenChunks = "defined-by-first-chunk"
        "#,
    )
    .unwrap();

    // Second secure chunk reads the same name through its own fenv and
    // republishes the result under a separate key so we can pull it out
    // from the registry-stored secureenv without trusting _G.
    env.exec_rilua_secure(
        r#"
            secureenvSharedBetweenChunksCopy = secureenvSharedBetweenChunks
        "#,
    )
    .unwrap();

    let (first_binding, copy_binding, has_g_fallback): (String, String, bool) = env
        .eval(
            r#"
            return tostring(rawget(__secureenv, "secureenvSharedBetweenChunks")),
                   tostring(rawget(__secureenv, "secureenvSharedBetweenChunksCopy")),
                   (getmetatable(__secureenv) and getmetatable(__secureenv).__index == _G) or false
            "#,
        )
        .unwrap();

    assert_eq!(
        first_binding, "defined-by-first-chunk",
        "first chunk's write must be present in secureenv"
    );
    assert_eq!(
        copy_binding, "defined-by-first-chunk",
        "second chunk must see the first chunk's binding through the shared secureenv"
    );
    assert!(!has_g_fallback, "secureenv must not fall back to live _G");
}

#[test]
fn exec_maybe_secure_true_matches_exec_rilua_secure() {
    // The --exec-lua-secure CLI flag routes exec-lua code through
    // `exec_maybe_secure(code, true)`. Confirm that path is a pure
    // pass-through to `exec_rilua_secure`: a write lands in secureenv,
    // not in _G.
    let env = env();

    env.exec_maybe_secure(r#"execMaybeSecureProbe = "via-flag""#, true)
        .unwrap();

    let (g_probe_type, secureenv_value): (String, String) = env
        .eval(
            r#"
            return type(rawget(_G, "execMaybeSecureProbe")),
                   tostring(rawget(__secureenv, "execMaybeSecureProbe"))
            "#,
        )
        .unwrap();
    assert_eq!(g_probe_type, "nil");
    assert_eq!(secureenv_value, "via-flag");
}

#[test]
fn exec_maybe_secure_false_matches_exec() {
    // Without the flag, exec-lua routes through `exec_maybe_secure(code,
    // false)`, which must behave like plain `exec` — writes land on _G.
    let env = env();

    env.exec_maybe_secure(r#"execMaybeSecureInsecure = "plain""#, false)
        .unwrap();

    let on_g: String = env
        .eval(r#"return tostring(rawget(_G, "execMaybeSecureInsecure"))"#)
        .unwrap();
    assert_eq!(on_g, "plain");
}

#[test]
fn shared_table_mutation_propagates_both_ways() {
    let env = env();

    let (shared_key, from_g): (String, String) = env
        .eval(
            r#"
            local sharedKey
            for key, value in pairs(__secureenv) do
                if type(key) == "string"
                   and key ~= "_G"
                   and type(value) == "table"
                   and rawget(_G, key) == value then
                    sharedKey = key
                    break
                end
            end
            if not sharedKey then
                return "", "missing"
            end

            local mutate = function()
                rawget(_G, sharedKey).secureenvSharedContainer = "from-secure"
            end
            debug.setfenv(mutate, __secureenv)
            mutate()
            return sharedKey, tostring(rawget(_G, sharedKey).secureenvSharedContainer)
            "#,
        )
        .unwrap();

    assert!(
        !shared_key.is_empty(),
        "expected one table shared by the shallow copy"
    );
    assert_eq!(
        from_g, "from-secure",
        "mutations to a table still shared after initialization should be visible from _G"
    );
}

/// Read a global through a function whose fenv is secureenv, mirroring how
/// a `[LoadIntoEnvironment secure]` chunk resolves names.
const SECURE_READ_PROBE: &str = r#"
    local result
    local probe = function() result = %NAME% end
    debug.setfenv(probe, __secureenv)
    probe()
    return type(result)
"#;

#[test]
fn late_global_is_not_visible_to_secure_without_explicit_export() {
    // A global registered on _G AFTER secureenv was created is not in
    // secureenv's own slot and does not fall through to _G.
    let env = env();

    // Insecure write → lands on _G (genv), absent from secureenv's own slot.
    env.exec(r#"lateGlobalProbe = 6"#).unwrap();

    let (in_secureenv_own_slot, secure_sees_type): (String, String) = env
        .eval(
            r#"
            return type(rawget(__secureenv, "lateGlobalProbe")),
                   (function()
                       local result
                       local probe = function() result = lateGlobalProbe end
                       debug.setfenv(probe, __secureenv)
                       probe()
                       return type(result)
                   end)()
            "#,
        )
        .unwrap();

    assert_eq!(
        in_secureenv_own_slot, "nil",
        "late global must NOT be in secureenv's own slot"
    );
    assert_eq!(
        secure_sees_type, "nil",
        "secure code must not see late _G globals"
    );
}

#[test]
fn late_global_rebind_stays_invisible_to_secure() {
    let env = env();

    env.exec(r#"liveLinkProbe = 6"#).unwrap();
    let first: String = env
        .eval(&SECURE_READ_PROBE.replace("%NAME%", "liveLinkProbe"))
        .unwrap();
    assert_eq!(first, "nil");

    env.exec(r#"liveLinkProbe = 7"#).unwrap();
    let second: String = env
        .eval(&SECURE_READ_PROBE.replace("%NAME%", "liveLinkProbe"))
        .unwrap();
    assert_eq!(
        second, "nil",
        "secure reads must remain independent from _G rebinds"
    );
}

#[test]
fn secure_write_severs_the_link_to_global_env() {
    // Once secure code writes the name, the value lands in secureenv's own
    // slot. The secure side is decoupled from _G.
    let env = env();

    env.exec(r#"severProbe = 10"#).unwrap();

    // Secure write → secureenv own slot.
    env.exec_rilua_secure(r#"severProbe = 99"#).unwrap();

    // Insecure re-bind on _G afterwards.
    env.exec(r#"severProbe = 11"#).unwrap();

    let (secure_sees, g_value): (f64, f64) = env
        .eval(
            r#"
            local result
            local probe = function() result = severProbe end
            debug.setfenv(probe, __secureenv)
            probe()
            return result, rawget(_G, "severProbe")
            "#,
        )
        .unwrap();

    assert_eq!(
        secure_sees, 99.0,
        "after a secure write, secure reads its own slot, not _G"
    );
    assert_eq!(g_value, 11.0, "_G keeps its own independent value");
}

#[test]
fn global_copied_at_creation_is_frozen_against_later_g_rebind() {
    // A primitive global present at secureenv creation was copied
    // into secureenv's own slot. Re-binding it on _G later does NOT change
    // the secure read — the copy is decoupled.
    let env = env();

    // Discover a primitive global that exists in BOTH envs' own slots with
    // equal value (i.e. it was copied at creation). pairs() walks raw keys,
    // so it ignores the __index metatable and yields only copied keys.
    let key: String = env
        .eval(
            r#"
            for k, v in pairs(__secureenv) do
                if (type(v) == "number" or type(v) == "string")
                   and type(k) == "string"
                   and rawget(_G, k) == v then
                    return k
                end
            end
            return ""
            "#,
        )
        .unwrap();
    assert!(
        !key.is_empty(),
        "expected at least one copied primitive global at creation"
    );

    // Rebind the _G slot to a sentinel string the original could not equal.
    let probe = format!(
        r#"
            local sentinel = "__frozen_sentinel__"
            rawset(_G, {key:?}, sentinel)
            local result
            local probe = function() result = rawget(__secureenv, {key:?}) end
            probe()
            return tostring(result), tostring(rawget(_G, {key:?})), tostring(result == sentinel)
            "#,
    );
    let (secure_value, g_value, secure_equals_sentinel): (String, String, String) =
        env.eval(&probe).unwrap();

    assert_eq!(
        g_value, "__frozen_sentinel__",
        "_G rebind must have applied"
    );
    assert_eq!(
        secure_equals_sentinel, "false",
        "secureenv's copied value (key {key}) must NOT follow the _G rebind"
    );
    assert_ne!(secure_value, "__frozen_sentinel__");
}

#[test]
fn secureenv_does_not_read_late_table_from_global_env() {
    let env = env();

    env.exec(r#"lateSecureTableProbe = { value = 39512 }"#)
        .unwrap();

    let table_type: String = env
        .eval(
            r#"
            local result
            local probe = function()
                result = lateSecureTableProbe
            end
            debug.setfenv(probe, __secureenv)
            probe()
            return type(result)
            "#,
        )
        .unwrap();

    assert_eq!(table_type, "nil");
}
