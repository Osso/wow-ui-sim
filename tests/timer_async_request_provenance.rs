//! Original Retail AsyncRequest closure: owner/ID correlation, not a new timeout model.
#![cfg(feature = "client-retail")]

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use sha2::{Digest, Sha256};
use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "common/bounded_process.rs"]
mod bounded_process;

const CHILD: &str = "WOW_SIM_ASYNC_REQUEST_PROVENANCE_CHILD";
const TIMINGS: &str = "WOW_SIM_LOG_HANDLER_TIMINGS";
const MUTATION: &str = "WOW_SIM_SHARED_INPUT_MUTATION";
const VENDOR_SHA256: &str = "1648c842902f6657ae1e10de7366cd0650c77a5eb41bc1c9c7e8030256fa8acb";
const CALLBACK: &str = "@Interface/AddOns/Blizzard_AsyncRequest/Blizzard_AsyncRequest.lua:60";
const OWNERS: [&str; 2] = ["AsyncRequestOwnerA", "AsyncRequestOwnerB"];
const COMPLETE: &str =
    "asyncreq-result requests=4 responses=2 timeouts=2 cancelled_timeouts=0 errors=0";

#[test]
fn original_async_request_correlates_timeout_and_cancellation_in_both_modes() {
    if std::env::var_os(CHILD).is_some() {
        exercise_original_requests();
        return;
    }
    for threshold in [None, Some("0")] {
        let stderr = run_isolated(threshold);
        eprintln!(
            "asyncreq-diagnostic-mode={}\n{stderr}",
            threshold.unwrap_or("unset")
        );
        assert_eq!(
            stderr.lines().filter(|line| *line == COMPLETE).count(),
            1,
            "{stderr}"
        );
        assert!(!stderr.contains("asyncreq-cancelled-timeout"), "{stderr}");
        assert_registration_ids_are_distinct(&stderr);
        if threshold.is_some() {
            assert_correlated_records(&stderr);
        } else {
            assert!(!stderr.contains("[handler-entry]"), "{stderr}");
            assert!(!stderr.contains("callback_source="), "{stderr}");
            assert!(!stderr.contains("timer_id="), "{stderr}");
        }
    }
}

fn run_isolated(threshold: Option<&str>) -> String {
    let thread = std::thread::current();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            thread.name().unwrap(),
            "--exact",
            "--test-threads=1",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env_remove(TIMINGS);
    if let Some(threshold) = threshold {
        command.env(TIMINGS, threshold);
    }
    let output = bounded_process::output(90, &mut command).expect("run bounded AsyncRequest child");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "child failed: {stderr}\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    stderr
}

fn exercise_original_requests() {
    let cache = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    let original = cache.join("Blizzard_AsyncRequest/Blizzard_AsyncRequest.lua");
    let bytes = std::fs::read(&original).expect("read original Retail AsyncRequest Lua");
    let digest = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(
        digest,
        VENDOR_SHA256,
        "consumer drift: {}",
        original.display()
    );
    let env = WowLuaEnv::new().expect("isolated original AsyncRequest environment");
    let toc = cache.join("Blizzard_AsyncRequest/Blizzard_AsyncRequest.toc");
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    exercise_request_transitions(&env);
}

fn exercise_request_transitions(env: &WowLuaEnv) {
    env.exec(
        r#"
        AsyncRequestProvenance = { errors = 0 }
        seterrorhandler(function()
            AsyncRequestProvenance.errors = AsyncRequestProvenance.errors + 1
        end)
    "#,
    )
    .unwrap();
    install_mutation_control(env);
    let temp = tempfile::tempdir().unwrap();
    for owner in OWNERS {
        load_request_addon(env, temp.path(), owner);
    }
    let ids = OWNERS.map(|owner| request_ids(env, owner));
    env.exec(
        r#"
        FireEvent("PLAYER_LOGIN", AsyncRequestResponseOverride or "AsyncRequestOwnerA-response")
        FireEvent("PLAYER_LOGOUT", "AsyncRequestOwnerB-response")
    "#,
    )
    .unwrap();
    assert_request_state(env, 0, 0);
    make_timer_ready(env, ids[0].0);
    env.process_timers().unwrap();
    assert_request_state(env, 1, 0);
    make_timer_ready(env, ids[1].0);
    env.process_timers().unwrap();
    assert_request_state(env, 1, 1);
    drain_cancelled_deadlines(env, &[ids[0].1, ids[1].1]);
    assert_request_state(env, 1, 1);
    eprintln!("{COMPLETE}");
}

fn drain_cancelled_deadlines(env: &WowLuaEnv, ids: &[u64]) {
    // Cross the cancelled deadlines too; absence before expiry is insufficient.
    make_retained_timers_ready(env, ids);
    env.process_timers().unwrap();
    env.process_timers().unwrap();
}

fn install_mutation_control(env: &WowLuaEnv) {
    match std::env::var(MUTATION).as_deref() {
        Ok("response") => env
            .exec("AsyncRequestResponseOverride = 'wrong-response'")
            .unwrap(),
        Ok("source") => env
            .exec(
                r#"
            local associations = debug.getregistry().__wow_timer_original_callbacks
            if associations then
                getmetatable(associations).__newindex = function(store, wrapper)
                    rawset(store, wrapper, wrapper)
                end
            end
        "#,
            )
            .unwrap(),
        _ => {}
    }
}

fn request_fixture() -> String {
    let source = include_str!("fixtures/AsyncRequestProvenance/Requests.lua");
    if std::env::var(MUTATION).as_deref() == Ok("timeout") {
        source.replace("probe.timeouts + 1", "probe.timeouts + 2")
    } else {
        source.to_owned()
    }
}

fn load_request_addon(env: &WowLuaEnv, root: &Path, owner: &str) {
    let directory = root.join("Interface/AddOns").join(owner);
    std::fs::create_dir_all(&directory).unwrap();
    let toc = directory.join(format!("{owner}.toc"));
    std::fs::write(
        &toc,
        format!("## Interface: 120005\n## Title: {owner}\nRequests.lua\n"),
    )
    .unwrap();
    std::fs::write(directory.join("Requests.lua"), request_fixture()).unwrap();
    let result = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(result.warnings.is_empty(), "{owner}: {:?}", result.warnings);
}

fn request_ids(env: &WowLuaEnv, owner: &str) -> (u64, u64) {
    let (live, cancelled): (i64, i64) = env.eval(&format!(
        "local p = AsyncRequestProvenance['{owner}']; return p.liveHandle.__id, p.cancelledHandle.__id"
    )).unwrap();
    assert!(live > 0 && cancelled > 0);
    eprintln!("asyncreq-registration owner={owner} live_id={live} cancelled_id={cancelled}");
    (live as u64, cancelled as u64)
}

fn make_timer_ready(env: &WowLuaEnv, id: u64) {
    let state = env.state();
    let mut state = state.borrow_mut();
    let timer = state
        .rilua_timers
        .iter_mut()
        .find(|timer| timer.id == id)
        .expect("positive live timer must still be pending");
    assert!(!timer.cancelled);
    timer.fire_at = Instant::now();
}

fn make_retained_timers_ready(env: &WowLuaEnv, ids: &[u64]) {
    let now = Instant::now();
    let state = env.state();
    let mut state = state.borrow_mut();
    for timer in &mut state.rilua_timers {
        if ids.contains(&timer.id) {
            timer.fire_at = now;
        }
    }
}

fn assert_request_state(env: &WowLuaEnv, first: u32, second: u32) {
    env.exec(&format!(
        r#"
        assert(AsyncRequestProvenance.errors == 0, 'unexpected callback error')
        local expected = {{ AsyncRequestOwnerA = {first}, AsyncRequestOwnerB = {second} }}
        for owner, timeoutCount in pairs(expected) do
            local p = AsyncRequestProvenance[owner]
            assert(p.requests == 2 and p.responses == 1, owner)
            assert(p.timeouts == timeoutCount and p.cancelledTimeouts == 0, owner)
            assert(p.cancelledHandle:IsCancelled(), owner)
            assert(not p.cancelled.isRunning, owner)
            assert(p.cancelled.timeoutTimer == nil, owner)
            assert(not p.cancelled:IsEventRegistered(p.responseEvent), owner)
            if timeoutCount == 0 then
                assert(p.live.isRunning and not p.liveHandle:IsCancelled(), owner)
                assert(p.live:IsEventRegistered('PLAYER_ENTERING_WORLD'), owner)
            else
                assert(not p.live.isRunning and p.liveHandle:IsCancelled(), owner)
                assert(p.live.timeoutTimer == nil, owner)
                assert(not p.live:IsEventRegistered('PLAYER_ENTERING_WORLD'), owner)
            end
        end
    "#
    ))
    .unwrap();
}

fn registration<'a>(stderr: &'a str, owner: &str) -> &'a str {
    let prefix = format!("asyncreq-registration owner={owner} ");
    let matches: Vec<_> = stderr
        .lines()
        .filter(|line| line.starts_with(&prefix))
        .collect();
    assert_eq!(matches.len(), 1, "{stderr}");
    matches[0]
}

fn assert_registration_ids_are_distinct(stderr: &str) {
    let ids: Vec<_> = OWNERS
        .iter()
        .flat_map(|owner| {
            let record = registration(stderr, owner);
            [field(record, "live_id"), field(record, "cancelled_id")]
        })
        .collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), 4, "{stderr}");
}

fn assert_correlated_records(stderr: &str) {
    let label = format!("callback_source={CALLBACK}");
    let entries: Vec<_> = stderr
        .lines()
        .filter(|line| line.starts_with("[handler-entry]") && line.contains(&label))
        .collect();
    let returns: Vec<_> = stderr
        .lines()
        .filter(|line| line.starts_with("[handler]") && line.contains(&label))
        .collect();
    assert_eq!(entries.len(), 2, "{stderr}");
    assert_eq!(returns.len(), 2, "{stderr}");
    for owner in OWNERS {
        assert_owner_invocation(stderr, owner);
    }
}

fn assert_owner_invocation(stderr: &str, owner: &str) {
    let registered = registration(stderr, owner);
    let live_id = field(registered, "live_id");
    let cancelled_id = field(registered, "cancelled_id");
    let records: Vec<_> = stderr
        .lines()
        .filter(|line| line.starts_with("[handler-entry]") || line.starts_with("[handler]"))
        .filter(|line| line.contains("handler=C_Timer"))
        .collect();
    assert!(
        !records
            .iter()
            .any(|line| field(line, "timer_id") == cancelled_id),
        "{stderr}"
    );
    let live: Vec<_> = records
        .iter()
        .copied()
        .filter(|line| field(line, "timer_id") == live_id)
        .collect();
    assert_eq!(live.len(), 2, "{stderr}");
    assert!(live[0].starts_with("[handler-entry]"), "{stderr}");
    assert!(live[1].starts_with("[handler]"), "{stderr}");
    for record in &live {
        assert_eq!(field(record, "addon"), owner, "{record}");
        assert_eq!(field(record, "frame"), "#0", "{record}");
        assert_eq!(field(record, "callback_source"), CALLBACK, "{record}");
        assert_ne!(
            field(record, "source"),
            CALLBACK,
            "invoker and original must remain distinct"
        );
    }
    assert_eq!(field(live[1], "outcome"), "ok");
    let body = format!("asyncreq-timeout owner={owner}");
    assert_eq!(
        stderr.lines().filter(|line| *line == body).count(),
        1,
        "{stderr}"
    );
    assert!(
        stderr.find(live[0]).unwrap() < stderr.find(&body).unwrap(),
        "{stderr}"
    );
    assert!(
        stderr.find(&body).unwrap() < stderr.find(live[1]).unwrap(),
        "{stderr}"
    );
}

fn field<'a>(record: &'a str, name: &str) -> &'a str {
    let prefix = format!("{name}=");
    record
        .split_whitespace()
        .find_map(|part| part.strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("missing {name}: {record}"))
}
