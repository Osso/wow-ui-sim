//! Bound generated-code compilation during real XML reconstruction after reload.

use super::{AddonContext, LoadTiming, WowLuaEnv, load_xml_file};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output};
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

const TEST_NAME: &str =
    "loader::tests::reload_xml_cost::reload_xml_layer_reconstruction_bounds_source_compiles";
const CHILD_TEST_NAME: &str =
    "loader::tests::reload_xml_cost::reload_xml_layer_reconstruction_child";
const RELOAD_BEGIN: &str = "[reload-xml-cost-begin]";
const RELOAD_END: &str = "[reload-xml-cost-end]";
const INSTANCE_COUNT: usize = 16;
const RELOAD_COUNT: usize = 2;
// One homogeneous XML operation may prepare code once in each fresh VM,
// but must not compile again for every instance's generated name/frame ID.
const MAX_SOURCE_COMPILES_PER_RELOAD: usize = 1;

fn write_xml_fixture(root: &Path) {
    let frames: String = (0..INSTANCE_COUNT)
        .map(|index| {
            format!(
                r#"<Frame name="ReloadCost{index}" parent="UIParent">
                    <Layers><Layer level="BACKGROUND">
                        <Texture parentKey="Ink">
                            <Size x="32" y="16"/>
                            <Color r="0.25" g="0.5" b="0.75" a="1"/>
                            <Anchors><Anchor point="TOPLEFT"/></Anchors>
                        </Texture>
                    </Layer></Layers>
                </Frame>"#
            )
        })
        .collect();
    std::fs::write(root.join("reload-cost.xml"), format!("<Ui>{frames}</Ui>")).unwrap();
}

fn load_xml_fixture(env: &WowLuaEnv, root: &Path) -> crate::Result<()> {
    let addon_table = env.create_addon_table()?;
    let context = AddonContext::new(env.lua(), "ReloadCost", addon_table, root, false, false)?;
    let mut warnings = Vec::new();
    let result = load_xml_file(
        &env.loader_env(),
        &root.join("reload-cost.xml"),
        &context,
        &mut LoadTiming::default(),
        &mut warnings,
    );
    assert!(warnings.is_empty(), "unexpected XML warnings: {warnings:?}");
    result
        .map(|_| ())
        .map_err(|error| crate::Error::Other(error.to_string()))
}

fn assert_fresh_regions(env: &WowLuaEnv) {
    env.exec(&format!(
        r#"
        assert(ReloadCostOldSentinel == nil,
            'ReloadCostOldSentinel: expected nil, actual ' .. tostring(ReloadCostOldSentinel))
        local seen = {{}}
        for index = 0, {} do
            local name = 'ReloadCost' .. index
            local frame = _G[name]
            assert(frame, name .. ': expected frame, actual ' .. tostring(frame))
            assert(frame:GetParent() == UIParent,
                name .. ': expected parent UIParent, actual ' .. tostring(frame:GetParent()))
            local ink = frame.Ink
            local region = name .. '.Ink'
            assert(ink, region .. ': expected region, actual ' .. tostring(ink))
            assert(ink:GetParent() == frame,
                region .. ': expected parent ' .. tostring(frame) .. ', actual ' .. tostring(ink:GetParent()))
            assert(not seen[ink],
                region .. ': expected distinct region, actual reused ' .. tostring(ink))
            seen[ink] = true
            local width, height = ink:GetWidth(), ink:GetHeight()
            assert(width == 32 and height == 16,
                region .. ': expected size 32x16, actual ' .. tostring(width) .. 'x' .. tostring(height))
            local layer = ink:GetDrawLayer()
            assert(layer == 'BACKGROUND',
                region .. ': expected draw layer BACKGROUND, actual ' .. tostring(layer))
        end
        ReloadCost0.Ink:SetWidth(7)
        local width = ReloadCost1.Ink:GetWidth()
        assert(width == 32,
            'ReloadCost1.Ink: expected independent width 32, actual ' .. tostring(width))
        "#,
        INSTANCE_COUNT - 1
    ))
    .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}

fn reconstruct_xml_across_reloads() {
    let fixture = tempfile::tempdir().unwrap();
    write_xml_fixture(fixture.path());
    let mut env = WowLuaEnv::new().unwrap();
    let root = fixture.path().to_path_buf();
    env.set_reload_loader(move |env, _| {
        eprintln!("{RELOAD_BEGIN}");
        let result = load_xml_fixture(env, &root);
        eprintln!("{RELOAD_END}");
        result
    });
    load_xml_fixture(&env, fixture.path()).unwrap();
    assert_fresh_regions(&env);
    for _ in 0..RELOAD_COUNT {
        let old_lua = Rc::downgrade(&env.lua);
        env.exec("ReloadCostOldSentinel = {}; ReloadUI()").unwrap();
        assert!(env.service_reload(&mut None).unwrap());
        assert!(old_lua.upgrade().is_none());
        assert_fresh_regions(&env);
    }
}

fn assert_reload_compile_budget(stderr: &str) {
    let reloads: Vec<_> = stderr.split(RELOAD_BEGIN).skip(1).collect();
    assert_eq!(reloads.len(), RELOAD_COUNT, "missing reload observations");
    for (index, reload) in reloads.iter().enumerate() {
        let (loading, _) = reload.split_once(RELOAD_END).expect("reload must finish");
        let compiles = loading
            .lines()
            .filter(|line| line.contains("[generated-chunk]"))
            .filter(|line| line.split_whitespace().any(|field| field == "mode=compile"))
            .filter(|line| {
                line.split_whitespace()
                    .any(|field| field == "phase=xml-frame")
            })
            .count();
        eprintln!(
            "[reload-xml-cost] reload={} source_compiles={compiles} budget={MAX_SOURCE_COMPILES_PER_RELOAD}",
            index + 1
        );
        assert!(
            compiles <= MAX_SOURCE_COMPILES_PER_RELOAD,
            "reload {} compiled {compiles} generated XML chunks for {INSTANCE_COUNT} identical \
             layer operations; budget is {MAX_SOURCE_COMPILES_PER_RELOAD}\n{loading}",
            index + 1
        );
    }
}

#[test]
fn reload_xml_layer_reconstruction_child() {
    reconstruct_xml_across_reloads();
}

fn exact_test_command(test_name: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", test_name, "--nocapture", "--test-threads=1"]);
    command
}

fn read_test_output(mut pipe: impl Read + Send + 'static) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes).unwrap();
        bytes
    })
}

fn run_bounded_test(mut command: Command) -> Output {
    use std::process::Stdio;

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = read_test_output(child.stdout.take().unwrap());
    let stderr = read_test_output(child.stderr.take().unwrap());
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if started.elapsed() >= Duration::from_secs(90) {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("XML/reload test exceeded 90 seconds");
        }
        thread::sleep(Duration::from_millis(10));
    };
    Output {
        status,
        stdout: stdout.join().unwrap(),
        stderr: stderr.join().unwrap(),
    }
}

fn successful_test_stderr(output: Output) -> String {
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "XML/reload behavior failed before cost assertion:\n{}\n{stderr}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed; 0 ignored;"),
        "exact child selector must execute one test"
    );
    stderr
}

#[test]
fn reload_xml_parent_marker_does_not_skip_compile_budget() {
    let mut command = exact_test_command(TEST_NAME);
    // Set only the subprocess environment; never mutate the parallel test runner.
    command.env("WOW_SIM_RELOAD_XML_COST_CHILD", "1");
    let stderr = successful_test_stderr(run_bounded_test(command));
    let observations: Vec<_> = stderr
        .lines()
        .filter(|line| line.starts_with("[reload-xml-cost] "))
        .collect();
    assert_eq!(
        observations.len(),
        RELOAD_COUNT,
        "parent must assert both reload budgets"
    );
    for (index, observation) in observations.iter().enumerate() {
        assert_eq!(
            *observation,
            format!(
                "[reload-xml-cost] reload={} source_compiles=0 budget=1",
                index + 1
            )
        );
    }
}

#[test]
fn reload_xml_layer_reconstruction_bounds_source_compiles() {
    // Fresh process: profiling/cache OnceLocks must not inherit other tests.
    // Fresh cache: unrelated previous runs must not hide ID-key churn.
    // Only an exact, distinct test selector enters reconstruction. No ambient
    // marker can select a branch that skips this parent's budget assertion.
    let cache = tempfile::tempdir().unwrap();
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever {
        // Preserve real input populated by `casc sync-forever-spells` while
        // isolating the bytecode cache. Do not fetch assets inside this test.
        let source = crate::blizzard_ui_sync::forever_spells::cache_path().unwrap();
        let destination = cache
            .path()
            .join("wow-ui-sim/spells/wowforever/spells.json");
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::copy(&source, &destination).unwrap_or_else(|error| {
            panic!(
                "copy Forever spell input {}: {error}; run `wow-cli casc sync-forever-spells` before this test",
                source.display()
            )
        });
        let mut permissions = std::fs::metadata(&destination).unwrap().permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&destination, permissions).unwrap();
    }
    let mut command = exact_test_command(CHILD_TEST_NAME);
    command
        .env("WOW_SIM_PROFILE_GENERATED_CHUNKS", "1")
        .env("WOW_SIM_ENABLE_BYTECODE_CACHE", "1")
        .env("XDG_CACHE_HOME", cache.path());
    let stderr = successful_test_stderr(run_bounded_test(command));
    assert_reload_compile_budget(&stderr);
}
