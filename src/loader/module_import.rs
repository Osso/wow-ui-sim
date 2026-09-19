//! Pure module-name resolution for Forever's already-loaded module imports.
//!
//! Paths contain addon-relative components, including the module's file stem.
//! Registry existence, file loading, and return-value storage belong to the caller.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ModuleIdentity {
    pub addon: String,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImportError {
    NoModule,
    RelativeOutsideAddon,
    MissingDirectDependency,
}

impl fmt::Display for ImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NoModule => "Invalid import: No module with that name exists",
            Self::RelativeOutsideAddon => {
                "Invalid import: Relative imports may only be used within the same addon"
            }
            Self::MissingDirectDependency => {
                "Invalid import: Modules from other addons may only be imported if the calling addon has a direct dependency on the addon being imported"
            }
        })
    }
}

impl std::error::Error for ImportError {}

/// Resolve a logical import; `direct_deps` contains required and optional names.
///
/// Dynamic callers have no addon identity and may import absolute names without
/// a dependency. Relative imports without an origin use the relative-import
/// error: this is explicit simulator policy for a case unspecified by the Wiki.
pub(crate) fn resolve_import(
    request: &str,
    caller: Option<&ModuleIdentity>,
    direct_deps: &[String],
) -> Result<ModuleIdentity, ImportError> {
    let suffix = request.trim_start_matches('.');
    let leading_dots = request.len() - suffix.len();
    let components = split_components(suffix)?;
    if leading_dots == 0 {
        return resolve_absolute(&components, caller, direct_deps);
    }
    resolve_relative(&components, caller, leading_dots - 1)
}

fn split_components(path: &str) -> Result<Vec<&str>, ImportError> {
    let components: Vec<_> = path.split('.').collect();
    let malformed = components.iter().any(|component| {
        component.is_empty()
            || component
                .chars()
                .any(|character| character.is_control() || matches!(character, '/' | '\\'))
    });
    if malformed {
        return Err(ImportError::NoModule);
    }
    Ok(components)
}

fn resolve_absolute(
    components: &[&str],
    caller: Option<&ModuleIdentity>,
    direct_deps: &[String],
) -> Result<ModuleIdentity, ImportError> {
    let (addon, path) = components.split_first().ok_or(ImportError::NoModule)?;
    if path.is_empty() {
        return Err(ImportError::NoModule);
    }
    if let Some(caller) = caller {
        let same_addon = caller.addon.eq_ignore_ascii_case(addon);
        let declared = direct_deps
            .iter()
            .any(|dep| dep.eq_ignore_ascii_case(addon));
        if !same_addon && !declared {
            return Err(ImportError::MissingDirectDependency);
        }
    }
    Ok(ModuleIdentity {
        addon: (*addon).to_owned(),
        path: path.iter().map(|part| (*part).to_owned()).collect(),
    })
}

fn resolve_relative(
    components: &[&str],
    caller: Option<&ModuleIdentity>,
    parent_steps: usize,
) -> Result<ModuleIdentity, ImportError> {
    let caller = caller.ok_or(ImportError::RelativeOutsideAddon)?;
    let directory_len = caller
        .path
        .len()
        .checked_sub(1)
        .ok_or(ImportError::RelativeOutsideAddon)?;
    let retained_len = directory_len
        .checked_sub(parent_steps)
        .ok_or(ImportError::RelativeOutsideAddon)?;
    let path = caller.path[..retained_len]
        .iter()
        .cloned()
        .chain(components.iter().map(|part| (*part).to_owned()))
        .collect();
    Ok(ModuleIdentity {
        addon: caller.addon.clone(),
        path,
    })
}

#[cfg(test)]
mod tests {
    use super::{ImportError, ModuleIdentity, resolve_import};

    fn module(addon: &str, path: &[&str]) -> ModuleIdentity {
        ModuleIdentity {
            addon: addon.to_owned(),
            path: path.iter().map(|part| (*part).to_owned()).collect(),
        }
    }

    #[test]
    fn absolute_import_preserves_addon_and_module_spelling() {
        let caller = module("ExampleAddon", &["Core", "Startup"]);
        assert_eq!(
            resolve_import("ExampleAddon.UI.Panel", Some(&caller), &[]),
            Ok(module("ExampleAddon", &["UI", "Panel"]))
        );
    }

    #[test]
    fn relative_import_uses_callers_directory() {
        let caller = module("ExampleAddon", &["Core", "Logging"]);
        assert_eq!(
            resolve_import(".Startup", Some(&caller), &[]),
            Ok(module("ExampleAddon", &["Core", "Startup"]))
        );
    }

    #[test]
    fn additional_leading_dot_ascends_one_directory() {
        let caller = module("ExampleAddon", &["Core", "Logging"]);
        assert_eq!(
            resolve_import("..UI.Panel", Some(&caller), &[]),
            Ok(module("ExampleAddon", &["UI", "Panel"]))
        );
    }

    #[test]
    fn deep_relative_import_can_reach_addon_root() {
        let caller = module("ExampleAddon", &["UI", "Widgets", "Panel"]);
        assert_eq!(
            resolve_import("...Data.Constants", Some(&caller), &[]),
            Ok(module("ExampleAddon", &["Data", "Constants"]))
        );
    }

    #[test]
    fn relative_import_cannot_escape_addon_even_with_dependency() {
        let caller = module("ExampleAddon", &["Core", "Startup"]);
        assert_eq!(
            resolve_import(
                "...ExampleLibrary.Formatting.Text",
                Some(&caller),
                &["ExampleLibrary".to_owned()],
            ),
            Err(ImportError::RelativeOutsideAddon)
        );
    }

    #[test]
    fn root_module_can_import_sibling_but_not_ascend() {
        let caller = module("ExampleAddon", &["Startup"]);
        assert_eq!(
            resolve_import(".Logging", Some(&caller), &[]),
            Ok(module("ExampleAddon", &["Logging"]))
        );
        assert_eq!(
            resolve_import("..Logging", Some(&caller), &[]),
            Err(ImportError::RelativeOutsideAddon)
        );
    }

    #[test]
    fn direct_required_or_optional_dependency_grants_cross_addon_import() {
        let caller = module("ExampleAddon", &["UI", "Panel"]);
        let direct_deps = vec!["OtherRequired".to_owned(), "ExampleLibrary".to_owned()];
        assert_eq!(
            resolve_import(
                "ExampleLibrary.Formatting.Text",
                Some(&caller),
                &direct_deps,
            ),
            Ok(module("ExampleLibrary", &["Formatting", "Text"]))
        );
    }

    #[test]
    fn dependency_authorization_is_case_insensitive_without_rewriting_identity() {
        let caller = module("ExampleAddon", &["UI", "Panel"]);
        assert_eq!(
            resolve_import(
                "EXAMPLElibrary.Formatting.TeXt",
                Some(&caller),
                &["exampleLIBRARY".to_owned()],
            ),
            Ok(module("EXAMPLElibrary", &["Formatting", "TeXt"]))
        );
    }

    #[test]
    fn same_addon_case_difference_needs_no_dependency() {
        let caller = module("ExampleAddon", &["Core", "Startup"]);
        assert_eq!(
            resolve_import("exampleaddon.Core.Logging", Some(&caller), &[]),
            Ok(module("exampleaddon", &["Core", "Logging"]))
        );
    }

    #[test]
    fn missing_direct_dependency_is_not_granted_by_intermediary_or_prefix() {
        let caller = module("ExampleAddon", &["Core", "Startup"]);
        for deps in [
            vec![],
            vec!["Intermediary".to_owned()],
            vec!["ExampleLibraryExtra".to_owned()],
        ] {
            assert_eq!(
                resolve_import("ExampleLibrary.Text", Some(&caller), &deps),
                Err(ImportError::MissingDirectDependency)
            );
        }
    }

    #[test]
    fn dynamic_absolute_import_is_exempt_but_relative_import_has_no_origin() {
        assert_eq!(
            resolve_import("ExampleLibrary.Formatting.Text", None, &[]),
            Ok(module("ExampleLibrary", &["Formatting", "Text"]))
        );
        assert_eq!(
            resolve_import(".Text", None, &[]),
            Err(ImportError::RelativeOutsideAddon)
        );
    }

    #[test]
    fn resolver_does_not_check_existence_or_block_self_import() {
        let caller = module("ExampleAddon", &["Core", "Logging"]);
        assert_eq!(
            resolve_import(".Logging", Some(&caller), &[]),
            Ok(caller.clone())
        );
        assert_eq!(
            resolve_import(".NotLoaded", Some(&caller), &[]),
            Ok(module("ExampleAddon", &["Core", "NotLoaded"]))
        );
    }

    #[test]
    fn malformed_dot_names_and_filesystem_separators_are_not_aliases() {
        let caller = module("ExampleAddon", &["Core", "Logging"]);
        for request in [
            "",
            "ExampleAddon",
            "ExampleAddon.",
            "ExampleAddon..Core",
            ".",
            "..",
            "...",
            ".Core..Text",
            ".Core.",
            "ExampleAddon/Core/Text",
            "ExampleAddon\\Core\\Text",
            "ExampleAddon.Core/Text",
            "ExampleAddon.Core\\Text",
            ".../Text",
            "ExampleAddon.Core\0Text",
            "ExampleAddon.Core\nText",
        ] {
            assert_eq!(
                resolve_import(request, Some(&caller), &[]),
                Err(ImportError::NoModule),
                "request {request:?}"
            );
        }
    }

    #[test]
    fn logical_components_do_not_require_lua_identifier_syntax() {
        assert_eq!(
            resolve_import("Example-Addon.Core_Widgets.2D-Panel", None, &[]),
            Ok(module("Example-Addon", &["Core_Widgets", "2D-Panel"]))
        );
    }

    #[test]
    fn empty_caller_path_has_no_relative_origin() {
        let caller = module("ExampleAddon", &[]);
        assert_eq!(
            resolve_import(".Text", Some(&caller), &[]),
            Err(ImportError::RelativeOutsideAddon)
        );
    }

    #[test]
    fn errors_have_exact_import_messages() {
        assert_eq!(
            ImportError::NoModule.to_string(),
            "Invalid import: No module with that name exists"
        );
        assert_eq!(
            ImportError::RelativeOutsideAddon.to_string(),
            "Invalid import: Relative imports may only be used within the same addon"
        );
        assert_eq!(
            ImportError::MissingDirectDependency.to_string(),
            "Invalid import: Modules from other addons may only be imported if the calling addon has a direct dependency on the addon being imported"
        );
    }
}
