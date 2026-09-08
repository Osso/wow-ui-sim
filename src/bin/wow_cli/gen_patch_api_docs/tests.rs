use super::generate;
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

const DOCS: &str = "Interface/AddOns/Blizzard_APIDocumentationGenerated";

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn repository() -> TempDir {
    let repo = TempDir::new().unwrap();
    git(repo.path(), &["init", "-q"]);
    git(repo.path(), &["config", "user.name", "Fixture"]);
    git(
        repo.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    repo
}

fn snapshot(repo: &Path, version: &str, files: &[(&str, &str)]) -> String {
    let directory = repo.join(DOCS);
    if directory.exists() {
        fs::remove_dir_all(&directory).unwrap();
    }
    fs::create_dir_all(&directory).unwrap();
    fs::write(repo.join("version.txt"), format!("{version}\n")).unwrap();
    for (name, source) in files {
        fs::write(directory.join(name), source).unwrap();
    }
    git(repo, &["add", "--all"]);
    git(repo, &["commit", "-qm", version, "--allow-empty"]);
    git(repo, &["rev-parse", "HEAD"])
}

fn document(body: &str) -> String {
    format!(
        "local D = {{ Name='Fixture', Type='System', {body} }}; APIDocumentation:AddDocumentationTable(D)"
    )
}

fn register(repo: &Path, base: &str, target: &str) -> Value {
    serde_json::from_slice(&generate(repo, base, target, "12.1.5").unwrap()).unwrap()
}

#[test]
fn formatting_prose_relocation_and_checkout_state_do_not_change_contracts() {
    let repo = repository();
    let old = document(
        "Functions={{Name='DoThing',Type='Function',Documentation={'old'},Arguments={{Name='x',Type='number',Documentation={'old'}}}}}",
    );
    let new = document(
        "Functions = { { Arguments = {{ Type='number', Name='x', Documentation={'new'} }}, Type='Function', Name='DoThing', Documentation={'new'} } }",
    );
    let base = snapshot(
        repo.path(),
        "12.1.0.69587",
        &[("OldDocumentation.lua", &old)],
    );
    let target = snapshot(
        repo.path(),
        "12.1.5.69594",
        &[("NewDocumentation.lua", &new)],
    );
    fs::write(
        repo.path().join(DOCS).join("NewDocumentation.lua"),
        "not valid Lua",
    )
    .unwrap();
    let value = register(repo.path(), &base[..10], &target[..10]);
    assert_eq!(value["occurrences"], json!([]));
    assert_eq!(value["source"]["base"]["commit"], base);
    assert_eq!(value["source"]["target"]["commit"], target);
    assert_eq!(value["source"]["base"]["version"], "12.1.0");
    assert_eq!(value["source"]["base"]["build"], "69587");
    assert_eq!(value["source"]["target"]["version"], "12.1.5");
    assert_eq!(value["source"]["target"]["build"], "69594");
    assert_eq!(
        value["source"]["base"]["files"],
        json!([format!("{DOCS}/OldDocumentation.lua")])
    );
    assert_ne!(
        value["source"]["base"]["sha256"],
        value["source"]["target"]["sha256"]
    );
    assert_eq!(
        value["source"]["target"]["sha256"].as_str().unwrap().len(),
        64
    );
}

#[test]
fn argument_return_security_and_enum_metadata_changes_are_detected() {
    let repo = repository();
    let old = document(
        "Functions={{Name='Arg',Type='Function',Arguments={{Name='x',Type='number'},{Name='y',Type='string'}}},{Name='Ret',Type='Function',Returns={{Name='x',Type='number'}}},{Name='Secure',Type='Function',SecretArguments='AllowedWhenUntainted'}},Tables={{Name='Mode',Type='Enumeration',NumValues=1,MinValue=0,MaxValue=0,Fields={{Name='One',Type='Mode',EnumValue=0}}}}",
    );
    let new = old
        .replace(
            "{Name='x',Type='number'},{Name='y',Type='string'}",
            "{Name='y',Type='string'},{Name='x',Type='number'}",
        )
        .replace(
            "Returns={{Name='x',Type='number'}}",
            "Returns={{Name='x',Type='string',Default='x'}}",
        )
        .replace("AllowedWhenUntainted", "AllowedWhenTainted")
        .replace("MaxValue=0", "MaxValue=2");
    let base = snapshot(repo.path(), "12.1.0.69587", &[("ADocumentation.lua", &old)]);
    let target = snapshot(repo.path(), "12.1.5.69594", &[("ADocumentation.lua", &new)]);
    let value = register(repo.path(), &base, &target);
    let symbols: Vec<_> = value["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["symbol"].as_str().unwrap())
        .collect();
    assert_eq!(symbols, ["Arg", "Enum.Mode", "Ret", "Secure"]);
    assert_eq!(value["direction_counts"]["changed"], 4);
    assert_eq!(value["occurrences"][1]["before"]["MaxValue"], 0);
    assert_eq!(value["occurrences"][1]["after"]["MaxValue"], 2);
}

fn all_shapes() -> Vec<(&'static str, String)> {
    vec![
        ("SystemDocumentation.lua", document("Namespace='C_Fixture',Functions={{Name='Call',Type='Function',Arguments={{Name='mode',Type='Mode',Default=Enum.Mode.One}}}},Events={{Name='Fired',Type='Event',LiteralName='FIXTURE_FIRED'}},Tables={{Name='Data',Type='Structure',Fields={{Name='value',Type='number'}}},{Name='Mode',Type='Enumeration',NumValues=1,Fields={{Name='One',Type='Mode',EnumValue=1}}},{Name='Callback',Type='CallbackType',Arguments={{Name='x',Type='number'}}}},Predicates={{Name='IsReady',Type='Predicate',Arguments={{Name='x',Type='number'}}}}")),
        ("ObjectDocumentation.lua", "APIDocumentation:AddDocumentationTable({Name='FixtureAPI',Type='ScriptObject',ObjectType='Userdata',Parent='BaseAPI',Functions={{Name='Clear',Type='Function'}}})".into()),
    ]
}

#[test]
fn every_record_kind_adds_and_removes_with_stable_sorted_identity() {
    let repo = repository();
    let empty = document("");
    let base = snapshot(
        repo.path(),
        "12.1.0.69587",
        &[("EmptyDocumentation.lua", &empty)],
    );
    let shapes = all_shapes();
    let files: Vec<_> = shapes.iter().map(|(n, s)| (*n, s.as_str())).collect();
    let target = snapshot(repo.path(), "12.1.5.69594", &files);
    let added = register(repo.path(), &base, &target);
    let removed = register(repo.path(), &target, &base);
    assert_eq!(
        added["source"]["base"]["sha256"],
        "5c517361f35ed288d9422bb53e350d7451f7337e1a32610cded9ade2f87dc98f"
    );
    assert_eq!(
        added["source"]["target"]["files"],
        json!([
            format!("{DOCS}/ObjectDocumentation.lua"),
            format!("{DOCS}/SystemDocumentation.lua")
        ])
    );
    let expected = [
        "C_Fixture.Call",
        "C_Fixture.Callback",
        "C_Fixture.Data",
        "C_Fixture.Data.value",
        "C_Fixture.IsReady",
        "Enum.Mode",
        "Enum.Mode.One",
        "FIXTURE_FIRED",
        "Fixture",
        "Fixture.Clear",
    ];
    let rows = added["occurrences"].as_array().unwrap();
    assert_eq!(rows.len(), expected.len());
    for ((row, removal), symbol) in rows
        .iter()
        .zip(removed["occurrences"].as_array().unwrap())
        .zip(expected)
    {
        assert_eq!(row["symbol"], symbol);
        assert_eq!(row["direction"], "added");
        assert_eq!(row["before"], Value::Null);
        assert_eq!(removal["direction"], "removed");
        assert_eq!(removal["before"], row["after"]);
        assert_eq!(removal["after"], Value::Null);
    }
    assert_eq!(rows[0]["after"]["Arguments"][0]["Default"], "Enum.Mode.One");
    assert_eq!(rows[8]["after"]["Parent"], "BaseAPI");
    let first = generate(repo.path(), &base, &target, "12.1.5").unwrap();
    let second = generate(repo.path(), &base, &target, "12.1.5").unwrap();
    assert_eq!(first, second);
}

#[test]
fn enum_field_order_is_preserved_in_parent_record() {
    let repo = repository();
    let old = document(
        "Tables={{Name='Mode',Type='Enumeration',NumValues=2,Fields={{Name='A',Type='Mode',EnumValue=0},{Name='B',Type='Mode',EnumValue=1}}}}",
    );
    let new = old.replace(
        "{Name='A',Type='Mode',EnumValue=0},{Name='B',Type='Mode',EnumValue=1}",
        "{Name='B',Type='Mode',EnumValue=1},{Name='A',Type='Mode',EnumValue=0}",
    );
    let base = snapshot(
        repo.path(),
        "12.1.0.69587",
        &[("ModeDocumentation.lua", &old)],
    );
    let target = snapshot(
        repo.path(),
        "12.1.5.69594",
        &[("ModeDocumentation.lua", &new)],
    );
    let value = register(repo.path(), &base, &target);
    assert_eq!(value["occurrences"].as_array().unwrap().len(), 1);
    assert_eq!(value["occurrences"][0]["symbol"], "Enum.Mode");
    assert_eq!(value["occurrences"][0]["after"]["Fields"][0]["Name"], "B");
}

#[test]
fn moving_namespace_override_to_parent_is_not_a_contract_change() {
    let repo = repository();
    let old = document(
        "Functions={{Name='trim',Namespace='string',Type='Function',Arguments={{Name='value',Type='string'}}}}",
    );
    let new = document(
        "Namespace='string',Functions={{Name='trim',Type='Function',Arguments={{Name='value',Type='string'}}}}",
    );
    let base = snapshot(
        repo.path(),
        "12.1.0.69587",
        &[("OldDocumentation.lua", &old)],
    );
    let target = snapshot(
        repo.path(),
        "12.1.5.69594",
        &[("NewDocumentation.lua", &new)],
    );
    assert_eq!(
        register(repo.path(), &base, &target)["occurrences"],
        json!([])
    );
}

#[test]
fn cli_output_orders_directions_and_preserves_return_and_field_order() {
    use clap::Parser;
    let repo = repository();
    let old = document(
        "Functions={{Name='Gone',Type='Function'},{Name='Result',Type='Function',Returns={{Name='x',Type='number'},{Name='y',Type='string'}}}},Tables={{Name='Data',Type='Structure',Fields={{Name='x',Type='number'},{Name='y',Type='string'}}}}",
    );
    let new = old.replace("Name='Gone'", "Name='Added'").replace(
        "{Name='x',Type='number'},{Name='y',Type='string'}",
        "{Name='y',Type='string'},{Name='x',Type='number'}",
    );
    let base = snapshot(repo.path(), "12.1.0.69587", &[("ZDocumentation.lua", &old)]);
    let target = snapshot(repo.path(), "12.1.5.69594", &[("ADocumentation.lua", &new)]);
    let output = repo.path().join("register.json");
    let cli = crate::Cli::try_parse_from([
        "wow-cli",
        "generate",
        "patch-api-docs",
        "--repository",
        repo.path().to_str().unwrap(),
        "--base-commit",
        &base,
        "--target-commit",
        &target,
        "--patch",
        "12.1.5",
        "--output",
        output.to_str().unwrap(),
    ])
    .unwrap();
    let crate::Commands::Generate {
        what: crate::GenerateTarget::PatchApiDocs(options),
    } = cli.command
    else {
        panic!("wrong parsed command")
    };
    super::run(options).unwrap();
    let bytes = fs::read(output).unwrap();
    assert_eq!(
        bytes,
        generate(repo.path(), &base, &target, "12.1.5").unwrap()
    );
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let rows = value["occurrences"].as_array().unwrap();
    let identities: Vec<_> = rows
        .iter()
        .map(|r| {
            (
                r["direction"].as_str().unwrap(),
                r["symbol"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        identities,
        [
            ("added", "Added"),
            ("changed", "Data"),
            ("changed", "Result"),
            ("removed", "Gone")
        ]
    );
    assert_eq!(rows[1]["after"]["Fields"][0]["Name"], "y");
    assert_eq!(rows[2]["after"]["Returns"][0]["Name"], "y");
}

#[test]
fn constants_arithmetic_is_retained_symbolically_without_guessed_values() {
    let repo = repository();
    let base = snapshot(
        repo.path(),
        "12.1.0.69587",
        &[("EmptyDocumentation.lua", &document(""))],
    );
    let source = document(
        "Tables={{Name='Range',Type='Constants',Values={{Name='Count',Type='number',Value=Constants.Range.Last - Constants.Range.First + 1},{Name='Mask',Type='number',Value=Enum.Flags.A + Enum.Flags.B}}}}",
    );
    let target = snapshot(
        repo.path(),
        "12.1.5.69594",
        &[("RangeDocumentation.lua", &source)],
    );
    let value = register(repo.path(), &base, &target);
    assert_eq!(
        value["occurrences"][1]["after"]["Value"],
        "((Constants.Range.Last - Constants.Range.First) + 1)"
    );
    assert_eq!(
        value["occurrences"][2]["after"]["Value"],
        "(Enum.Flags.A + Enum.Flags.B)"
    );
}

#[test]
fn duplicate_and_unknown_declarations_fail_explicitly() {
    let repo = repository();
    let base = snapshot(
        repo.path(),
        "12.1.0.69587",
        &[("EmptyDocumentation.lua", &document(""))],
    );
    let cases = [
        (
            document("Functions={{Name='X',Type='Function'},{Name='X',Type='Function'}}"),
            "duplicate",
        ),
        (
            document("Tables={{Name='X',Type='UnknownShape'}}"),
            "unsupported",
        ),
        (
            "APIDocumentation:AddDocumentationTable({Name='X',Type='Mystery'})".into(),
            "unsupported",
        ),
        (
            "APIDocumentation:AddDocumentationTable({Name='X',Type=42})".into(),
            "unsupported",
        ),
    ];
    for (source, expected) in cases {
        let target = snapshot(
            repo.path(),
            "12.1.5.69594",
            &[("BadDocumentation.lua", &source)],
        );
        let error = generate(repo.path(), &base, &target, "12.1.5")
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
    }
}
