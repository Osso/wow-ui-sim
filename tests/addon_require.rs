//! Forever imports exercise the real TOC/XML loader, not a host package loader.

#[cfg(feature = "client-wowforever")]
mod forever {
    use std::path::PathBuf;
    use wow_ui_sim::loader::load_addon;
    use wow_ui_sim::lua_api::WowLuaEnv;

    struct Addons {
        root: tempfile::TempDir,
        env: WowLuaEnv,
    }

    impl Addons {
        fn new() -> Self {
            Self {
                root: tempfile::tempdir().unwrap(),
                env: WowLuaEnv::new().unwrap(),
            }
        }

        fn write(&self, name: &str, metadata: &str, files: &[(&str, &str)]) -> PathBuf {
            let dir = self.root.path().join("Interface/AddOns").join(name);
            std::fs::create_dir_all(&dir).unwrap();
            let mut toc = format!("## Title: {name}\n{metadata}\n");
            for (path, source) in files {
                let file = dir.join(path);
                std::fs::create_dir_all(file.parent().unwrap()).unwrap();
                std::fs::write(file, source).unwrap();
                toc.push_str(path);
                toc.push('\n');
            }
            let path = dir.join(format!("{name}.toc"));
            std::fs::write(&path, toc).unwrap();
            path
        }

        fn load(&self, name: &str, metadata: &str, files: &[(&str, &str)]) {
            let result =
                load_addon(&self.env.loader_env(), &self.write(name, metadata, files)).unwrap();
            assert!(result.warnings.is_empty(), "{name}: {:?}", result.warnings);
        }

        fn check(&self, code: &str) {
            self.env.exec(code).unwrap();
        }
    }

    #[test]
    fn family_definitions_load_before_camelot_overrides_and_bare_dependents() {
        let addons = Addons::new();
        let toc = addons.write("FamilyOrdering", "", &[
            ("Mainline/NineSliceLayouts.lua", "NineSliceLayouts = { border = 7 }; FamilyOrder = 'layouts'"),
            ("Camelot/NineSliceLayoutOverrides.lua", "assert(NineSliceLayouts.border == 7); NineSliceLayouts.border = 11; FamilyOrder = FamilyOrder .. ',override'"),
            ("Mainline/InputUtil.lua", "InputUtil = { ready = true }; FamilyOrder = FamilyOrder .. ',input'"),
            ("Mainline/SharedUIPanelTemplates.lua", "SidePanelTabButtonMixin = { ready = true }; FamilyOrder = FamilyOrder .. ',templates'"),
            ("Dependent.lua", "assert(NineSliceLayouts.border == 11 and InputUtil.ready and SidePanelTabButtonMixin.ready); FamilyOrder = FamilyOrder .. ',dependent'"),
        ]);
        std::fs::write(
            &toc,
            concat!(
                "## Title: FamilyOrdering\n",
                "[Family]\\NineSliceLayouts.lua\n",
                "[Game]\\NineSliceLayoutOverrides.lua\n",
                "[Family]\\InputUtil.lua\n",
                "[Family]\\SharedUIPanelTemplates.lua\n",
                "Dependent.lua\n",
            ),
        )
        .unwrap();
        let result = load_addon(&addons.env.loader_env(), &toc).unwrap();
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        addons.check("assert(FamilyOrder == 'layouts,override,input,templates,dependent')");
    }

    #[test]
    fn mainline_annotations_select_dependencies_and_camelot_overrides() {
        let addons = Addons::new();
        addons.load("MainlineDependency", "", &[("Value.lua", "return 17")]);
        let toc = addons.write("AnnotatedFamily", "", &[
            ("Mainline/Base.lua", "AnnotationValue = require('MainlineDependency.Value'); AnnotationOrder = 'base'"),
            ("Camelot/Override.lua", "assert(AnnotationValue == 17); AnnotationValue = 29; AnnotationOrder = AnnotationOrder .. ',camelot'"),
            ("Classic.lua", "error('classic file must not load')"),
            ("Standard.lua", "error('standard file must not load')"),
            ("Vanilla.lua", "error('vanilla file must not load')"),
            ("Excluded.lua", "error('excluded file must not load')"),
        ]);
        std::fs::write(
            &toc,
            concat!(
                "## Title: AnnotatedFamily\n",
                "## Dep: MainlineDependency [AllowLoadGameType mainline]\n",
                "## Dep: ClassicDependency [AllowLoadGameType classic]\n",
                "[Family]/Base.lua [AllowLoadGameType mainline]\n",
                "[Game]/Override.lua [AllowLoadGameType camelot]\n",
                "Classic.lua [AllowLoadGameType classic]\n",
                "Standard.lua [AllowLoadGameType standard]\n",
                "Vanilla.lua [AllowLoadGameType vanilla]\n",
                "Excluded.lua [ExcludeLoadGameType mainline]\n",
            ),
        )
        .unwrap();
        let parsed = wow_ui_sim::toc::TocFile::from_file(&toc).unwrap();
        assert_eq!(parsed.dependencies(), vec!["MainlineDependency"]);
        let result = load_addon(&addons.env.loader_env(), &toc).unwrap();
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        addons.check("assert(AnnotationValue == 29 and AnnotationOrder == 'base,camelot')");
    }

    #[test]
    fn completed_file_values_preserve_identity_false_and_nil_without_reexecution() {
        let addons = Addons::new();
        addons.load(
            "ExampleAddon",
            "",
            &[
                ("Empty.lua", ""),
                ("False.lua", "return false"),
                ("Number.lua", "return 37"),
                ("Text.lua", "return 'module value'"),
                (
                    "Object.lua",
                    "ModuleRuns = (ModuleRuns or 0) + 1; return { value = 19 }",
                ),
                ("Function.lua", "return function() return 41 end"),
                (
                    "Check.lua",
                    r#"
                assert(require('ExampleAddon.Empty') == nil)
                assert(require('ExampleAddon.False') == false)
                assert(require('ExampleAddon.Number') == 37)
                assert(require('ExampleAddon.Text') == 'module value')
                local first = require('ExampleAddon.Object')
                first.value = 23
                assert(rawequal(first, require('ExampleAddon.Object')))
                assert(require('ExampleAddon.Object').value == 23)
                assert(require('ExampleAddon.Function')() == 41)
                assert(ModuleRuns == 1)
                ModuleValuesChecked = true
            "#,
                ),
            ],
        );
        addons.check("assert(ModuleValuesChecked)");
        addons
            .check("collectgarbage('collect'); assert(require('ExampleAddon.Object').value == 23)");
    }

    #[test]
    fn unfinished_and_absent_files_do_not_load_on_require() {
        let addons = Addons::new();
        addons.load("Ordering", "", &[
            ("First.lua", r#"
                for _, name in ipairs({'Ordering.First', 'Ordering.Later', 'Ordering.Absent'}) do
                    local ok, err = pcall(require, name)
                    assert(not ok and string.find(err, 'Invalid import: No module with that name exists', 1, true))
                end
                assert(LaterRuns == nil)
                OrderChecked = true
            "#),
            ("Later.lua", "LaterRuns = (LaterRuns or 0) + 1; return 13"),
        ]);
        addons.check("assert(OrderChecked and LaterRuns == 1 and require('Ordering.Later') == 13)");
    }

    #[test]
    fn relative_imports_keep_the_defining_file_after_loading() {
        let addons = Addons::new();
        addons.load("ExampleAddon", "", &[
            ("Core/Startup.lua", "return 'startup'"),
            ("UI/Panel.lua", "return 'panel'"),
            ("Core/Logging.lua", r#"
                function DelayedModuleImports()
                    assert(require('.Startup') == 'startup')
                    assert(require('..UI.Panel') == 'panel')
                    local ok, err = pcall(require, '...ExampleLibrary.Formatting.Text')
                    assert(not ok and string.find(err, 'Invalid import: Relative imports may only be used within the same addon', 1, true))
                    return true
                end
            "#),
        ]);
        addons.load(
            "Invoker",
            "",
            &[("Run.lua", "assert(DelayedModuleImports())")],
        );
        addons.check("assert(DelayedModuleImports())");
    }

    #[test]
    fn direct_required_and_optional_dependencies_allow_imports_case_insensitively() {
        let addons = Addons::new();
        addons.load(
            "ExampleLibrary",
            "",
            &[("Formatting/Text.lua", "return { prefix = 'ok' }")],
        );
        for (name, declaration) in [
            ("RequiredClient", "## Dep: eXaMpLeLiBrArY"),
            ("OptionalClient", "## OptionalDeps: EXAMPLELIBRARY"),
        ] {
            addons.load(
                name,
                declaration,
                &[(
                    "Read.lua",
                    r#"
                local value = require('ExampleLibrary.Formatting.Text')
                assert(value.prefix == 'ok')
            "#,
                )],
            );
        }
    }

    #[test]
    fn transitive_dependencies_do_not_authorize_delayed_imports() {
        let addons = Addons::new();
        addons.load("Library", "", &[("Value.lua", "return 29")]);
        addons.load(
            "Middle",
            "## Dep: Library",
            &[(
                "Read.lua",
                r#"
            function MiddleImport() return require('Library.Value') end
        "#,
            )],
        );
        addons.load("Outer", "## Dep: Middle", &[("Read.lua", r#"
            function OuterImport()
                local ok, err = pcall(require, 'Library.Value')
                assert(not ok and string.find(err, 'Invalid import: Modules from other addons may only be imported if the calling addon has a direct dependency on the addon being imported', 1, true))
                assert(MiddleImport() == 29)
            end
        "#)]);
        addons.check("OuterImport()");
    }

    #[test]
    fn dynamic_chunks_are_exempt_even_when_their_display_name_looks_like_an_addon_file() {
        let addons = Addons::new();
        addons.load("Library", "", &[("Value.lua", "return 43")]);
        addons.load("NoDependency", "", &[("Dynamic.lua", r#"
            function DynamicModuleImports()
                local normal = assert(loadstring("return require('Library.Value')"))
                local renamed = assert(loadstring("return require('Library.Value')", '@Interface/AddOns/NoDependency/Dynamic.lua'))
                assert(normal() == 43 and renamed() == 43)
            end
        "#)]);
        addons.check("DynamicModuleImports(); assert(require('Library.Value') == 43)");
    }

    #[test]
    fn xml_script_files_use_the_same_completion_registry() {
        let addons = Addons::new();
        let toc = addons.write(
            "XmlModules",
            "",
            &[(
                "Load.xml",
                r#"<Ui><Script file="Data.lua"/><Script file="Read.lua"/></Ui>"#,
            )],
        );
        std::fs::write(
            toc.parent().unwrap().join("Data.lua"),
            "return { answer = 47 }",
        )
        .unwrap();
        std::fs::write(
            toc.parent().unwrap().join("Read.lua"),
            "assert(require('.Data').answer == 47); XmlImportChecked = true",
        )
        .unwrap();
        load_addon(&addons.env.loader_env(), &toc).unwrap();
        addons.check("assert(XmlImportChecked)");
    }

    #[test]
    fn module_execution_keeps_addon_varargs_and_delayed_closure_provenance() {
        let addons = Addons::new();
        addons.load("Ownership", "", &[
            ("Core/Value.lua", r#"
                local name, private = ...
                private.answer = 53
                return { owner = name, private = private, insecure = not issecure() }
            "#),
            ("Core/Export.lua", r#"
                local name, private = ...
                local value = require('.Value')
                assert(value.owner == name, 'module addon name changed')
                assert(value.private == private, 'module private table changed')
                assert(value.insecure == not issecure(), 'module taint differs from peer file')
                function CoroutineImport()
                    local thread = coroutine.create(function() return require('.Value').private.answer end)
                    local ok, answer = coroutine.resume(thread)
                    assert(ok and answer == 53)
                end
            "#),
        ]);
        addons.check("CoroutineImport(); collectgarbage('collect'); CoroutineImport()");
    }

    #[test]
    fn secure_addon_files_share_the_profile_module_registry() {
        let addons = Addons::new();
        addons.load(
            "Blizzard_ModuleFixture",
            "## UseSecureEnvironment: 1",
            &[
                ("Value.lua", "return { answer = 59 }"),
                (
                    "Read.lua",
                    "assert(require('.Value').answer == 59); return require('.Value')",
                ),
            ],
        );
        addons.check("assert(require('Blizzard_ModuleFixture.Value').answer == 59)");
    }

    #[test]
    fn failed_module_does_not_publish_a_value() {
        let addons = Addons::new();
        let toc = addons.write("Broken", "", &[("Bad.lua", "error('module failure')")]);
        let _ = load_addon(&addons.env.loader_env(), &toc);
        addons.check(r#"
            local ok, err = pcall(require, 'Broken.Bad')
            assert(not ok and string.find(err, 'Invalid import: No module with that name exists', 1, true))
        "#);
    }
}
