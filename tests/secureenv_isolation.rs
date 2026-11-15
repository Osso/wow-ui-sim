//! End-to-end fenv isolation tests for secureenv.
//!
//! Secureenv is a shared table seeded from the fixed 39-name base and used
//! as the function environment of chunks marked `[LoadIntoEnvironment
//! secure]`. Table-valued base globals are shallow-copied into separate root
//! tables; later writes stay in the environment where they occur.
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
         ## UseSecureEnvironment: 1\n\
         Before.lua\n\
         Secure.lua\t\t[Bootstrap] [LoadIntoEnvironment secure]\n\
         Secure.xml\n\
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
         assert(debug == nil and coroutine == nil, 'secure files use the fixed environment'); \
         addon.order[#addon.order+1] = 'secure'; annotationSecureValue = 37; \
         annotationSecureBaseOnly = true",
    )
    .unwrap();
    std::fs::write(
        root.path().join("Secure.xml"),
        "<Ui>\n\
         <Script file=\"SecureXmlScript.lua\"/>\n\
         <Script>assert(debug == nil and coroutine == nil, 'secure XML uses the fixed environment'); annotationSecureXmlInlineRead = true</Script>\n\
         <Frame name=\"AnnotationProbeFrame\">\n\
         <Size x=\"20\" y=\"20\"/>\n\
         </Frame>\n\
         </Ui>",
    )
    .unwrap();
    std::fs::write(
        root.path().join("SecureXmlScript.lua"),
        "assert(debug == nil and coroutine == nil, 'secure XML uses the fixed environment'); annotationSecureXmlFileRead = true",
    )
    .unwrap();
    std::fs::write(
        root.path().join("After.lua"),
        "local _, addon = ...; assert(table.concat(addon.order, ',') == 'before,secure'); \
         assert(debug == nil and coroutine == nil, 'secure files use the fixed environment'); \
         annotationLoadComplete = true",
    )
    .unwrap();

    let snapshot_names_start_private: bool = env
        .eval(
            r#"
                return type(rawget(_G, 'debug')) == 'table'
                    and rawget(_G, 'coroutine') ~= nil
                    and rawget(__secureenv, 'debug') == nil
                    and rawget(__secureenv, 'coroutine') == nil
            "#,
        )
        .unwrap();
    assert!(snapshot_names_start_private);

    let result = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    let lua_errors = env.state().borrow().lua_errors.clone();
    assert!(lua_errors.is_empty(), "{:?}", lua_errors);

    let secure_file_results: (bool, bool, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
                return rawget(__secureenv, 'annotationSecureValue') == 37,
                    rawget(__secureenv, 'annotationSecureBaseOnly') == true,
                    rawget(__secureenv, 'annotationLoadComplete') == true,
                    rawget(__secureenv, 'annotationSecureXmlInlineRead') == true,
                    rawget(__secureenv, 'annotationSecureXmlFileRead') == true,
                    rawget(_G, 'annotationSecureValue') == nil,
                    rawget(_G, 'annotationSecureBaseOnly') == nil,
                    rawget(__secureenv, 'debug') == nil,
                    rawget(__secureenv, 'coroutine') == nil
            "#,
        )
        .unwrap();
    assert_eq!(
        secure_file_results,
        (true, true, true, true, true, true, true, true, true)
    );
}

#[test]
fn secure_scoped_modifier_onload_resolves_secure_mixin() {
    let env = env();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("SecureMixinProbe.toc");
    std::fs::write(
        &toc,
        "## Title: SecureMixinProbe\n\
         ## UseSecureEnvironment: 1\n\
         Mixin.lua\n\
         Mixin.xml\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("Mixin.lua"),
        "SecureMixinProbe = { OnLoad = function() secureMixinProbeOnLoad = true end }",
    )
    .unwrap();
    std::fs::write(
        root.path().join("Mixin.xml"),
        "<Ui><ScopedModifier forbidden=\"true\" hideFromGlobalEnv=\"true\">\
         <Frame name=\"SecureMixinProbeFrame\" mixin=\"SecureMixinProbe\">\
         <Scripts><OnLoad method=\"OnLoad\"/></Scripts>\
         </Frame></ScopedModifier></Ui>",
    )
    .unwrap();

    let result = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    let observed: (bool, bool) = env
        .eval(
            "return rawget(__secureenv, 'secureMixinProbeOnLoad') == true, \
             rawget(_G, 'secureMixinProbeOnLoad') == nil",
        )
        .unwrap();
    assert_eq!(observed, (true, true));
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn create_texture_source_location_skips_excluded_callsite() {
    let env = env();
    env.exec("sourceLocationProbeResults = {}").unwrap();
    let root = tempfile::tempdir().unwrap();
    let addon_dir = root.path().join("SourceLocationProbe");
    std::fs::create_dir_all(&addon_dir).unwrap();
    let toc = addon_dir.join("SourceLocationProbe.toc");
    std::fs::write(
        &toc,
        "## Title: SourceLocationProbe\nSourceLocationProbe.lua\n",
    )
    .unwrap();
    std::fs::write(
        addon_dir.join("SourceLocationProbe.lua"),
        r##"
local first = UIParent:CreateTexture()
local firstLocation = first:GetSourceLocation()
local returnCount = select("#", AddSourceLocationExclude("SourceLocationProbe.lua"))
local firstAfterExclusion = first:GetSourceLocation()
local excluded = UIParent:CreateTexture()
local excludedLocation = excluded:GetSourceLocation()
sourceLocationProbeResults.first = firstLocation
sourceLocationProbeResults.firstAfterExclusion = firstAfterExclusion
sourceLocationProbeResults.returnCount = returnCount
sourceLocationProbeResults.excluded = excludedLocation
"##,
    )
    .unwrap();

    let result = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    assert!(
        result.lua_files > 0,
        "the source probe Lua file must execute"
    );
    let addon_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('SourceLocationProbe')")
        .unwrap();
    assert!(addon_loaded, "the source-location probe addon must load");
    let lua_errors = env.state().borrow().lua_errors.clone();
    assert!(lua_errors.is_empty(), "probe Lua errors: {lua_errors:?}");

    let (first, first_after_exclusion, return_count, excluded): (
        Option<String>,
        Option<String>,
        Option<i32>,
        Option<String>,
    ) = env
        .eval(
            "return sourceLocationProbeResults.first, sourceLocationProbeResults.firstAfterExclusion, sourceLocationProbeResults.returnCount, sourceLocationProbeResults.excluded",
        )
        .unwrap();
    assert!(
        first
            .as_deref()
            .is_some_and(|location| location.ends_with("SourceLocationProbe.lua:2")),
        "CreateTexture must capture its Lua file and line: {first:?}"
    );
    assert_eq!(
        first_after_exclusion, first,
        "stored locations do not rescan"
    );
    assert_eq!(
        return_count,
        Some(0),
        "AddSourceLocationExclude returns no values"
    );
    assert!(
        excluded
            .as_deref()
            .is_some_and(|location| location.starts_with("UnknownFile:")),
        "an excluded callsite without another file source uses UnknownFile: {excluded:?}"
    );
}

#[test]
fn secure_addon_loads_nonsecure_dependency_in_ordinary_then_secure_environment() {
    let env = env();
    let root = tempfile::tempdir().unwrap();
    let dependency_dir = root.path().join("DependencyProbe");
    let consumer_dir = root.path().join("SecureConsumerProbe");
    std::fs::create_dir_all(&dependency_dir).unwrap();
    std::fs::create_dir_all(&consumer_dir).unwrap();
    env.exec(
        r#"
        local trace = {}
        rawset(_G, "r88tSecureDependencyTrace", trace)
        rawset(__secureenv, "r88tSecureDependencyTrace", trace)
        rawset(_G, "r88tSecureEnvironment", __secureenv)
        rawset(__secureenv, "r88tSecureEnvironment", __secureenv)
        "#,
    )
    .unwrap();
    std::fs::write(
        dependency_dir.join("DependencyProbe.toc"),
        "## Title: DependencyProbe\nDependency.lua\n",
    )
    .unwrap();
    std::fs::write(
        dependency_dir.join("Dependency.lua"),
        "local mode = getfenv(1) == r88tSecureEnvironment and 'secure' or 'ordinary'; \
         table.insert(r88tSecureDependencyTrace, 'dependency:' .. mode); \
         dependencyLoadCount = (dependencyLoadCount or 0) + 1; \
         dependencyToken = 'secure-visible'; \
         function getDependencyToken() return dependencyToken end",
    )
    .unwrap();
    let consumer_toc = consumer_dir.join("SecureConsumerProbe.toc");
    std::fs::write(
        &consumer_toc,
        "## Title: SecureConsumerProbe\n\
         ## UseSecureEnvironment: 1\n\
         ## Dependencies: DependencyProbe\n\
         Consumer.lua\n",
    )
    .unwrap();
    std::fs::write(
        consumer_dir.join("Consumer.lua"),
        "assert(rawget(_G, 'debug') == nil); \
         assert(rawget(_G, 'coroutine') == nil); \
         assert(getfenv(getDependencyToken) == _G); \
         assert(getDependencyToken() == 'secure-visible'); \
         table.insert(r88tSecureDependencyTrace, 'consumer:secure'); \
         secureConsumerRunCount = (secureConsumerRunCount or 0) + 1; \
         secureConsumerSawDependency = true",
    )
    .unwrap();

    let result = wow_ui_sim::loader::load_addon(&env.loader_env(), &consumer_toc).unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    let trace: String = env
        .eval("return table.concat(r88tSecureDependencyTrace, ',')")
        .unwrap();
    let ordinary_request = trace
        .find("dependency:ordinary")
        .expect("ordinary dependency request should execute its file");
    let secure_request = trace
        .find("dependency:secure")
        .expect("secure dependency request should execute its file");
    let consumer = trace
        .find("consumer:secure")
        .expect("secure consumer file should execute after its dependency");
    assert!(
        ordinary_request < secure_request && secure_request < consumer,
        "ordinary request, secure request, and consumer must stay ordered: {trace}"
    );
    let observed: (i32, i32, i32, bool, bool, bool, bool) = env
        .eval(
            r#"
                return rawget(_G, 'dependencyLoadCount') or 0,
                    rawget(__secureenv, 'dependencyLoadCount') or 0,
                    rawget(__secureenv, 'secureConsumerRunCount') or 0,
                    rawget(_G, 'dependencyToken') == 'secure-visible',
                    rawget(__secureenv, 'dependencyToken') == 'secure-visible',
                    rawget(__secureenv, 'secureConsumerSawDependency') == true,
                    C_AddOns.IsAddOnLoaded('SecureConsumerProbe') == true
            "#,
        )
        .unwrap();
    // R88t marks the initial default-file execution count as INFERRED.
    let (
        ordinary_count,
        secure_count,
        consumer_count,
        ordinary_token,
        secure_token,
        consumer_saw_dependency,
        consumer_loaded,
    ) = observed;
    assert!(ordinary_count > 0 && secure_count > 0 && consumer_count > 0);
    assert_eq!(
        (
            ordinary_token,
            secure_token,
            consumer_saw_dependency,
            consumer_loaded,
        ),
        (true, true, true, true)
    );

    {
        let state = env.state().borrow();
        let dependency = state
            .addons
            .iter()
            .find(|addon| addon.folder_name == "DependencyProbe")
            .expect("dependency load state");
        assert!(dependency.loaded);
        assert!(dependency.secure_loaded);
        let consumer = state
            .addons
            .iter()
            .find(|addon| addon.folder_name == "SecureConsumerProbe")
            .expect("consumer load state");
        assert!(!consumer.loaded);
        assert!(consumer.secure_loaded);
    }

    let counts_before: (i32, i32, i32) = (ordinary_count, secure_count, consumer_count);
    wow_ui_sim::loader::load_addon(&env.loader_env(), &consumer_toc).unwrap();
    let counts_after: (i32, i32, i32) = env
        .eval(
            "return rawget(_G, 'dependencyLoadCount'), \
             rawget(__secureenv, 'dependencyLoadCount'), \
             rawget(__secureenv, 'secureConsumerRunCount')",
        )
        .unwrap();
    assert_eq!(counts_after, counts_before);
}

#[test]
fn secure_dependencies_load_optional_before_required_before_consumer() {
    let env = env();
    let root = tempfile::tempdir().unwrap();
    let optional_dir = root.path().join("OptionalOrderProbe");
    let required_dir = root.path().join("RequiredOrderProbe");
    let consumer_dir = root.path().join("SecureOrderConsumer");
    std::fs::create_dir_all(&optional_dir).unwrap();
    std::fs::create_dir_all(&required_dir).unwrap();
    std::fs::create_dir_all(&consumer_dir).unwrap();
    env.exec(
        r#"
        local trace = {}
        rawset(_G, "r88tSecureDependencyOrderTrace", trace)
        rawset(__secureenv, "r88tSecureDependencyOrderTrace", trace)
        rawset(_G, "r88tSecureEnvironment", __secureenv)
        rawset(__secureenv, "r88tSecureEnvironment", __secureenv)
        "#,
    )
    .unwrap();

    std::fs::write(
        optional_dir.join("OptionalOrderProbe.toc"),
        "## Title: OptionalOrderProbe\nOptional.lua\n",
    )
    .unwrap();
    std::fs::write(
        optional_dir.join("Optional.lua"),
        "local mode = getfenv(1) == r88tSecureEnvironment and 'secure' or 'ordinary'; \
         table.insert(r88tSecureDependencyOrderTrace, 'optional:' .. mode)",
    )
    .unwrap();
    std::fs::write(
        required_dir.join("RequiredOrderProbe.toc"),
        "## Title: RequiredOrderProbe\nRequired.lua\n",
    )
    .unwrap();
    std::fs::write(
        required_dir.join("Required.lua"),
        "local mode = getfenv(1) == r88tSecureEnvironment and 'secure' or 'ordinary'; \
         table.insert(r88tSecureDependencyOrderTrace, 'required:' .. mode)",
    )
    .unwrap();
    let consumer_toc = consumer_dir.join("SecureOrderConsumer.toc");
    std::fs::write(
        &consumer_toc,
        "## Title: SecureOrderConsumer\n\
         ## UseSecureEnvironment: 1\n\
         ## OptionalDeps: OptionalOrderProbe\n\
         ## Dependencies: RequiredOrderProbe\n\
         Consumer.lua\n",
    )
    .unwrap();
    std::fs::write(
        consumer_dir.join("Consumer.lua"),
        "local mode = getfenv(1) == r88tSecureEnvironment and 'secure' or 'ordinary'; \
         table.insert(r88tSecureDependencyOrderTrace, 'consumer:' .. mode)",
    )
    .unwrap();

    let _ = wow_ui_sim::loader::load_addon(&env.loader_env(), &consumer_toc);
    let trace: String = env
        .eval("return table.concat(r88tSecureDependencyOrderTrace, ',')")
        .unwrap();
    let optional_ordinary = trace.find("optional:ordinary").unwrap();
    let optional_secure = trace.find("optional:secure").unwrap();
    let required_ordinary = trace.find("required:ordinary").unwrap();
    let required_secure = trace.find("required:secure").unwrap();
    let consumer_secure = trace.find("consumer:secure").unwrap();
    assert!(
        optional_ordinary < optional_secure
            && optional_secure < required_ordinary
            && required_ordinary < required_secure
            && required_secure < consumer_secure,
        "optional and required dependency requests must precede the secure consumer: {trace}"
    );
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
fn secure_base_library_mutations_stay_in_separate_tables() {
    let env = env();

    env.exec_rilua_secure(
        r#"
            rawset(math, "__secureenv_table_probe", "secure")
        "#,
    )
    .unwrap();

    let public_value_type: String = env
        .eval("return type(rawget(math, '__secureenv_table_probe'))")
        .unwrap();
    assert_eq!(public_value_type, "nil");

    env.exec("rawset(math, '__secureenv_table_probe', 'global')")
        .unwrap();
    let (secure_value, public_value): (String, String) = env
        .eval(
            r#"
            return rawget(rawget(__secureenv, "math"), "__secureenv_table_probe"),
                   rawget(rawget(_G, "math"), "__secureenv_table_probe")
            "#,
        )
        .unwrap();

    assert_eq!(secure_value, "secure");
    assert_eq!(public_value, "global");
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
fn copied_base_function_is_frozen_against_later_g_rebind() {
    let env = env();

    let initially_copied: bool = env
        .eval(
            "return type(rawget(__secureenv, 'geterrorhandler')) == 'function' \
             and rawget(__secureenv, 'geterrorhandler') == rawget(_G, 'geterrorhandler')",
        )
        .unwrap();
    assert!(
        initially_copied,
        "geterrorhandler is part of the fixed secure base"
    );

    env.exec("rawset(_G, 'geterrorhandler', '__global_rebound__')")
        .unwrap();
    let (secure_type, public_type, secure_kept_copy): (String, String, bool) = env
        .eval(
            r#"
            local result
            local probe = function() result = geterrorhandler end
            debug.setfenv(probe, __secureenv)
            probe()
            return type(result), type(rawget(_G, "geterrorhandler")),
                   result == rawget(__secureenv, "geterrorhandler")
            "#,
        )
        .unwrap();

    assert_eq!(secure_type, "function");
    assert_eq!(public_type, "string");
    assert!(
        secure_kept_copy,
        "secureenv must keep its initial function value"
    );
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

#[cfg(feature = "client-retail")]
#[test]
fn get_frame_metatable_is_present_in_both_environments() {
    let env = env();

    let result: (bool, bool) = env
        .eval(
            r#"
            return type(rawget(_G, "GetFrameMetatable")) == "function",
                   type(rawget(__secureenv, "GetFrameMetatable")) == "function"
            "#,
        )
        .unwrap();

    assert_eq!(result, (true, true));
}

#[cfg(all(
    feature = "client-retail",
    feature = "retail-12-1-5",
    feature = "timed-signal-maps"
))]
#[test]
fn retail_r88n11b_timed_signal_map_shares_function_with_secure_namespace() {
    let env = env();

    let shared: bool = env
        .eval(
            r#"
            local public = rawget(_G, "C_Timer")
            local secure = rawget(__secureenv, "C_Timer")
            local publicFactory = type(public) == "table" and rawget(public, "NewTimedSignalMap")
            local secureFactory = type(secure) == "table" and rawget(secure, "NewTimedSignalMap")
            return type(publicFactory) == "function"
                and rawequal(publicFactory, secureFactory)
                and not rawequal(public, secure)
            "#,
        )
        .expect("R88n.11b NewTimedSignalMap publication should be observable");

    assert!(shared);
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_c_timer_dual_members_are_registered_in_both_namespace_tables() {
    let env = env();

    let result: Vec<bool> = env
        .eval(
            r#"
            local public = rawget(_G, "C_Timer")
            local secure = rawget(__secureenv, "C_Timer")
            local function hasFunction(table, member)
                return type(table) == "table" and type(rawget(table, member)) == "function"
            end
            return {
                hasFunction(public, "After"), hasFunction(secure, "After"),
                hasFunction(public, "NewTicker"), hasFunction(secure, "NewTicker"),
                hasFunction(public, "NewTimer"), hasFunction(secure, "NewTimer"),
                public ~= secure,
            }
            "#,
        )
        .unwrap();

    assert_eq!(result, vec![true, true, true, true, true, true, true]);
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_native_constants_and_enum_tables_are_separate() {
    let env = env();

    env.exec(
        r#"
        local publicConstants = rawget(_G, "Constants")
        local secureConstants = rawget(__secureenv, "Constants")
        assert(type(publicConstants) == "table")
        assert(type(secureConstants) == "table")
        assert(not rawequal(publicConstants, secureConstants))

        local publicRoles = rawget(publicConstants, "LFG_ROLEConstants")
        local secureRoles = rawget(secureConstants, "LFG_ROLEConstants")
        assert(type(publicRoles) == "table")
        assert(type(secureRoles) == "table")
        assert(not rawequal(publicRoles, secureRoles))
        assert(rawget(publicRoles, "LFG_ROLE_ANY") == 3)
        assert(rawget(secureRoles, "LFG_ROLE_ANY") == 3)
        assert(rawget(publicRoles, "LFG_ROLE_NO_ROLE") == -1)
        assert(rawget(secureRoles, "LFG_ROLE_NO_ROLE") == -1)

        local publicEnum = rawget(_G, "Enum")
        local secureEnum = rawget(__secureenv, "Enum")
        assert(type(publicEnum) == "table")
        assert(type(secureEnum) == "table")
        assert(not rawequal(publicEnum, secureEnum))

        local publicPingStates = rawget(publicEnum, "PingSetTargetState")
        local securePingStates = rawget(secureEnum, "PingSetTargetState")
        assert(type(publicPingStates) == "table")
        assert(type(securePingStates) == "table")
        assert(not rawequal(publicPingStates, securePingStates))
        assert(rawget(publicPingStates, "Ok") == 0)
        assert(rawget(securePingStates, "Ok") == 0)
        "#,
    )
    .expect("native Constants and Enum installers must create separate tables");
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn retail_r88n11b_global_functions_share_values_with_secure_environment() {
    const GLOBALS: [&str; 15] = [
        "securecallmethod",
        "issecretvalue",
        "canaccessvalue",
        "canaccessallvalues",
        "canaccesstable",
        "scrub",
        "scrubsecretvalues",
        "secretunwrap",
        "canaccesssecrets",
        "hasanysecretvalues",
        "dropsecretaccess",
        "issecrettable",
        "CreateSecureDelegate",
        "securecopy",
        "secretwrap",
    ];
    let names = GLOBALS
        .iter()
        .map(|name| format!("{name:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    let failures: Vec<String> = env()
        .eval(&format!(
            r#"
                local names = {{ {names} }}
                local failures = {{}}
                for _, name in ipairs(names) do
                    local public = rawget(_G, name)
                    local secure = rawget(__secureenv, name)
                    if type(public) ~= "function" then
                        failures[#failures + 1] = name .. ": ordinary function missing"
                    elseif not rawequal(public, secure) then
                        failures[#failures + 1] = name .. ": secure value differs"
                    end
                end
                table.sort(failures)
                return failures
            "#
        ))
        .expect("R88n.11b global publication should be observable");

    assert!(
        failures.is_empty(),
        "verified flag-7 globals must share one function value: {failures:?}"
    );
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn retail_r88n11b_members_share_functions_with_secure_namespaces() {
    const MEMBERS: [(&str, &str); 37] = [
        ("C_ClassColor", "GetClassColor"),
        ("C_ColorUtil", "ConvertHSLToHSV"),
        ("C_ColorUtil", "ConvertHSVToHSL"),
        ("C_ColorUtil", "ConvertHSVToRGB"),
        ("C_ColorUtil", "ConvertRGBToHSV"),
        ("C_ColorUtil", "GenerateTextColorCode"),
        ("C_ColorUtil", "WrapTextInColor"),
        ("C_ColorUtil", "WrapTextInColorCode"),
        ("C_CurveUtil", "CreateColorCurve"),
        ("C_CurveUtil", "CreateCurve"),
        ("C_CurveUtil", "EvaluateColorFromBoolean"),
        ("C_CurveUtil", "EvaluateColorValueFromBoolean"),
        ("C_CurveUtil", "EvaluateGameCurve"),
        ("C_FunctionContainers", "CreateCallback"),
        ("C_GameRules", "AutoConnectToGameModeRealm"),
        ("C_GameRules", "DoesGameModeHavePromo"),
        ("C_GameRules", "GetActiveGameMode"),
        ("C_GameRules", "GetCurrentEventRealmQueues"),
        ("C_GameRules", "GetCurrentGameModeDisplayInfo"),
        ("C_GameRules", "GetCurrentGameModeRecordID"),
        ("C_GameRules", "GetDisplayedGameModeRecordIDAtIndex"),
        ("C_GameRules", "GetGameModeDisplayInfoByRecordID"),
        ("C_GameRules", "GetGameModeGlueScreenName"),
        ("C_GameRules", "GetGameModePromoGlobalString"),
        ("C_GameRules", "GetGameRuleAsFloat"),
        ("C_GameRules", "GetGameRuleAsFrameStrata"),
        ("C_GameRules", "GetNumDisplayedGameModes"),
        ("C_GameRules", "IsCharacterlessLoginActive"),
        ("C_GameRules", "IsClassAllowedForGameMode"),
        ("C_GameRules", "IsGameModeEnabled"),
        ("C_GameRules", "IsGameRuleActive"),
        ("C_GameRules", "IsMultiActionBarVisibilityForced"),
        ("C_GameRules", "IsPersonalResourceDisplayEnabled"),
        ("C_GameRules", "IsPlunderstorm"),
        ("C_GameRules", "IsStandard"),
        ("C_GameRules", "IsWoWHack"),
        ("C_ScriptedAnimations", "GetAllScriptedAnimationEffects"),
    ];
    let entries = MEMBERS
        .iter()
        .map(|(namespace, member)| format!("{{{namespace:?}, {member:?}}}"))
        .collect::<Vec<_>>()
        .join(", ");
    let env = env();
    let (present, namespaces, missing, failures): (i64, i64, Vec<String>, Vec<String>) = env
        .eval(&format!(
            r#"
                local entries = {{ {entries} }}
                local present, seen, missing, failures = 0, {{}}, {{}}, {{}}
                for _, entry in ipairs(entries) do
                    local namespace, member = entry[1], entry[2]
                    local public = rawget(_G, namespace)
                    local secure = rawget(__secureenv, namespace)
                    if type(public) ~= "table" then
                        failures[#failures + 1] = namespace .. ": ordinary namespace missing"
                    elseif type(secure) ~= "table" then
                        failures[#failures + 1] = namespace .. ": secure namespace missing"
                    elseif rawequal(public, secure) then
                        failures[#failures + 1] = namespace .. ": namespace tables are shared"
                    else
                        seen[namespace] = true
                        local publicValue = rawget(public, member)
                        local secureValue = rawget(secure, member)
                        if publicValue == nil then
                            missing[#missing + 1] = namespace .. "." .. member
                            if secureValue ~= nil then
                                failures[#failures + 1] = namespace .. "." .. member .. ": secure-only value"
                            end
                        elseif type(publicValue) ~= "function" then
                            failures[#failures + 1] = namespace .. "." .. member .. ": ordinary value is not a function"
                        elseif not rawequal(publicValue, secureValue) then
                            failures[#failures + 1] = namespace .. "." .. member .. ": values differ"
                        else
                            present = present + 1
                        end
                    end
                end
                local namespaceCount = 0
                for _ in pairs(seen) do namespaceCount = namespaceCount + 1 end
                table.sort(missing)
                table.sort(failures)
                return present, namespaceCount, missing, failures
            "#
        ))
        .expect("R88n.11b namespace publication should be observable");

    assert_eq!(
        namespaces, 6,
        "all verified namespace roots must be separate"
    );
    assert_eq!(present, 31, "expected all modeled members to publish");
    assert_eq!(
        missing,
        vec![
            "C_CurveUtil.EvaluateGameCurve".to_string(),
            "C_GameRules.AutoConnectToGameModeRealm".to_string(),
            "C_GameRules.GetCurrentGameModeRecordID".to_string(),
            "C_GameRules.GetGameModePromoGlobalString".to_string(),
            "C_GameRules.IsCharacterlessLoginActive".to_string(),
            "C_GameRules.IsClassAllowedForGameMode".to_string(),
        ],
        "only the six unimplemented simulator members should remain absent"
    );
    assert_eq!(present + missing.len() as i64, 37);
    assert!(
        failures.is_empty(),
        "invalid secure publications: {failures:?}"
    );
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn retail_r88n11b_private_aura_members_are_secure_only() {
    const MEMBERS: [&str; 13] = [
        "AddPrivateAuraUpdateCallback",
        "AnchorPrivateAura",
        "GetAllPrivateAuraInstanceIDs",
        "GetAllPrivateAuras",
        "GetAuraDataByAuraInstanceIDPrivate",
        "GetPrivateAuraAnchors",
        "GetPrivateAuraCasterGUID",
        "IsPrivateAuraFilteredOutByInstanceID",
        "RemovePrivateAuraUpdateCallback",
        "SetPrivateAuraAnchorAddedCallback",
        "SetPrivateAuraAnchorRemovedCallback",
        "SetPrivateRaidBossMessageCallback",
        "SetPrivateWarningTextFrame",
    ];
    let members = MEMBERS
        .iter()
        .map(|member| format!("{member:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    let (present, missing, failures, roots_are_separate): (i64, Vec<String>, Vec<String>, bool) =
        env()
            .eval(&format!(
                r#"
                local public = rawget(_G, "C_UnitAurasPrivate")
                local secure = rawget(__secureenv, "C_UnitAurasPrivate")
                local failures, missing = {{}}, {{}}
                if type(public) ~= "table" then
                    failures[#failures + 1] = "ordinary namespace missing"
                end
                if type(secure) ~= "table" then
                    failures[#failures + 1] = "secure namespace missing"
                end
                if #failures > 0 then
                    return 0, missing, failures, false
                end
                local rootsAreSeparate = not rawequal(public, secure)
                if not rootsAreSeparate then
                    failures[#failures + 1] = "namespace tables are shared"
                end
                local present = 0
                for _, member in ipairs({{{members}}}) do
                    local publicValue = rawget(public, member)
                    local secureValue = rawget(secure, member)
                    if type(secureValue) == "function" then
                        present = present + 1
                        if publicValue ~= nil then
                            failures[#failures + 1] = member .. ": ordinary value is present"
                        end
                    elseif publicValue == nil and secureValue == nil then
                        missing[#missing + 1] = "C_UnitAurasPrivate." .. member
                    else
                        failures[#failures + 1] = member .. ": member is not secure-only"
                    end
                end
                table.sort(missing)
                table.sort(failures)
                return present, missing, failures, rootsAreSeparate
            "#
            ))
            .expect("R88n.11b flag-2 publication should be observable");

    assert!(
        roots_are_separate,
        "private aura namespaces must remain separate"
    );
    assert_eq!(
        present, 12,
        "all modeled flag-2 members must be secure-only"
    );
    assert_eq!(
        missing,
        vec!["C_UnitAurasPrivate.AnchorPrivateAura".to_string()],
        "the unimplemented member must remain absent"
    );
    assert_eq!(present + missing.len() as i64, MEMBERS.len() as i64);
    assert!(
        failures.is_empty(),
        "invalid secure-only members: {failures:?}"
    );
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn retail_r88n11b_c_texture_members_share_functions_with_secure_namespace() {
    const MEMBERS: [&str; 11] = [
        "ClearTitleIconTexture",
        "GetAtlasElementID",
        "GetAtlasElements",
        "GetAtlasExists",
        "GetAtlasID",
        "GetAtlasInfo",
        "GetFilenameFromFileDataID",
        "GetTitleIconTexture",
        "IsTitleIconTextureReady",
        "SetTitleIconTexture",
        "SetURLTexture",
    ];
    let members = MEMBERS
        .iter()
        .map(|member| format!("{member:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    let (present, missing, failures, roots_are_separate): (i64, Vec<String>, Vec<String>, bool) =
        env()
            .eval(&format!(
                r#"
                local public = rawget(_G, "C_Texture")
                local secure = rawget(__secureenv, "C_Texture")
                local failures, missing = {{}}, {{}}
                if type(public) ~= "table" then
                    failures[#failures + 1] = "ordinary namespace missing"
                end
                if type(secure) ~= "table" then
                    failures[#failures + 1] = "secure namespace missing"
                end
                if #failures > 0 then
                    return 0, missing, failures, false
                end
                local rootsAreSeparate = not rawequal(public, secure)
                if not rootsAreSeparate then
                    failures[#failures + 1] = "namespace tables are shared"
                end
                local present = 0
                for _, member in ipairs({{{members}}}) do
                    local publicValue = rawget(public, member)
                    local secureValue = rawget(secure, member)
                    if type(publicValue) == "function" and rawequal(publicValue, secureValue) then
                        present = present + 1
                    elseif publicValue == nil and secureValue == nil then
                        missing[#missing + 1] = "C_Texture." .. member
                    else
                        failures[#failures + 1] = member .. ": values are not paired functions"
                    end
                end
                table.sort(missing)
                table.sort(failures)
                return present, missing, failures, rootsAreSeparate
            "#
            ))
            .expect("R88n.11b C_Texture publication should be observable");

    assert!(
        roots_are_separate,
        "C_Texture namespaces must remain separate"
    );
    assert_eq!(present, 3, "publish existing flag-3 functions only");
    assert_eq!(
        missing,
        vec![
            "C_Texture.ClearTitleIconTexture".to_string(),
            "C_Texture.GetAtlasElementID".to_string(),
            "C_Texture.GetAtlasElements".to_string(),
            "C_Texture.GetAtlasID".to_string(),
            "C_Texture.GetTitleIconTexture".to_string(),
            "C_Texture.IsTitleIconTextureReady".to_string(),
            "C_Texture.SetTitleIconTexture".to_string(),
            "C_Texture.SetURLTexture".to_string(),
        ],
        "unimplemented members must remain absent"
    );
    assert_eq!(present + missing.len() as i64, MEMBERS.len() as i64);
    assert!(
        failures.is_empty(),
        "invalid C_Texture routes: {failures:?}"
    );
}
