// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Megaton contributors

use std::collections::BTreeSet;
use std::path::Path;

use cu::pre::*;

/// Create a workspace Cargo.toml manifest that is a subset of the input
/// workspace, that only contains the requested packages
pub fn create_subset_workspace_manifest(
    manifest_path: &Path,
    packages: &[&str],
) -> cu::Result<String> {
    let workspace_manifest = cu::check!(
        cu::fs::read_string(manifest_path),
        "failed to read workspace Cargo.toml"
    )?;
    let mut workspace_manifest = toml::parse::<toml::Table>(&workspace_manifest)?;

    let workspace = cu::check!(
        workspace_manifest
            .get_mut("workspace")
            .and_then(|x| x.as_table_mut()),
        "workspace manifest should contain [workspace]"
    )?;

    // trace the dependencies of each member that is included
    let workspace_path = manifest_path.parent_abs()?;
    let mut included_packages = BTreeSet::new();
    let mut all_dependencies = BTreeSet::new();
    for p in packages {
        let package_manifest_path = cu::path!(&workspace_path / p / "Cargo.toml");
        let package_manifest = cu::check!(
            cu::fs::read_string(&package_manifest_path),
            "failed to read manifest for package '{p}' in the workspace"
        )?;
        let package_manifest = toml::parse::<toml::Table>(&package_manifest)?;
        let name = cu::check!(
            package_manifest
                .get("package")
                .and_then(|x| x.as_table())
                .and_then(|x| x.get("name"))
                .and_then(|x| x.as_str()),
            "failed to get name for workspace member '{p}'"
        )?;
        all_dependencies.insert(name.to_string());
        included_packages.insert(name.to_string());
        let dependencies = cu::check!(
            package_manifest
                .get("dependencies")
                .and_then(|x| x.as_table()),
            "failed to get dependencies for package '{p}'"
        )?;
        for (dep_name, dep_spec) in dependencies {
            if dep_spec.as_table().is_none_or(|x| {
                x.get("workspace")
                    .is_none_or(|x| !x.as_bool().unwrap_or_default())
            }) {
                cu::bail!(
                    "package '{name}' depends on '{dep_name}' that is not inherited from the workspace - all deps must specify workspace = true"
                );
            }
            all_dependencies.insert(dep_name.to_string());
        }
    }

    let workspace_dependencies = cu::check!(
        workspace
            .get_mut("dependencies")
            .and_then(|x| x.as_table_mut()),
        "workspace.dependencies should exist and be a table"
    )?;
    let mut new_workspace_dependencies = toml::Table::new();
    for (wksp_dep_name, dep_spec) in workspace_dependencies.iter() {
        if !all_dependencies.contains(wksp_dep_name) {
            continue;
        }
        if dep_spec.as_table().is_some_and(|x| x.get("path").is_some()) {
            // dependency is part of the workspace, so it must be included
            if !included_packages.contains(wksp_dep_name) {
                cu::bail!(
                    "workspace package '{wksp_dep_name}' is required by another package, but not included in the subset (add it to `packages` passed into create_subset_workspace_manifest)"
                );
            }
        }
        new_workspace_dependencies.insert(wksp_dep_name.to_string(), dep_spec.clone());
    }
    *workspace_dependencies = new_workspace_dependencies;

    let members = cu::check!(
        workspace.get_mut("members").and_then(|x| x.as_array_mut()),
        "workspace.members should exist and be an array"
    )?;
    let existing_members = members
        .iter()
        .map(|x| x.as_str().unwrap_or_default())
        .collect::<BTreeSet<_>>();
    let mut new_members = vec![];
    for p in packages {
        if !existing_members.contains(p) {
            cu::bail!("'{p}' is not a workspace member");
        }
        new_members.push(toml::Value::String(p.to_string()));
    }
    *members = new_members;

    cu::check!(
        toml::stringify(&workspace_manifest),
        "failed to serialize modified workspace manifest"
    )
}

// pub fn create_isolated_cargo_manifest(
//     original_manifest_path: &Path,
//     members: Option<&str>,
// ) -> cu::Result<String> {
//     create_isolated_cargo_manifest_with_deps_removed(
//         original_manifest_path,
//         members,
//         std::iter::empty::<String>(),
//     )
// }
//
// #[inline(always)]
// pub fn create_isolated_cargo_manifest_with_deps_removed<
//     I: IntoIterator<Item = impl Into<String>>,
// >(
//     original_manifest_path: &Path,
//     members: Option<&str>,
//     dep_to_remove: I,
// ) -> cu::Result<String> {
//     let dep_to_remove = dep_to_remove
//         .into_iter()
//         .map(|x| x.into())
//         .collect::<BTreeSet<_>>();
//     create_isolated_cargo_manifest_with_deps_removed_imp(
//         original_manifest_path,
//         members,
//         dep_to_remove,
//     )
// }
//
// fn create_isolated_cargo_manifest_with_deps_removed_imp(
//     original_manifest_path: &Path,
//     members: Option<&str>,
//     dep_to_remove: BTreeSet<String>,
// ) -> cu::Result<String> {
//     // load the workspace manifest
//     let mut workspace_manifest_path = original_manifest_path.parent_abs_times(3)?;
//     workspace_manifest_path.push("Cargo.toml");
//     let workspace_manifest = cu::check!(
//         cu::fs::read_string(&workspace_manifest_path),
//         "failed to read workspace Cargo.toml"
//     )?;
//     let workspace_manifest = toml::parse::<toml::Table>(&workspace_manifest)?;
//
//     // load and patch [workspace] in for the crate manifest
//     let mut manifest = cu::check!(
//         cu::fs::read_string(original_manifest_path),
//         "failed to read original crate Cargo.toml"
//     )?;
//     manifest.push_str("[workspace]\nresolver = \"2\"\n");
//     if let Some(members) = members {
//         manifest.push_str(&format!("members = {members}\n"));
//     }
//     let mut manifest = toml::parse::<toml::Table>(&manifest)?;
//
//     remove_dependencies("dependencies", &mut manifest, &dep_to_remove)?;
//     remove_dependencies("build-dependencies", &mut manifest, &dep_to_remove)?;
//
//     // collect dependencies that has .workspace = true
//     // currently only processing "dependencies" since that's all the use cases for us
//     let mut inherited_deps = vec![];
//     collect_inherited_dependencies("dependencies", &manifest, &mut inherited_deps)?;
//     collect_inherited_dependencies("build-dependencies", &manifest, &mut inherited_deps)?;
//
//     let workspace_deps = cu::check!(
//         workspace_manifest
//             .get("workspace")
//             .and_then(|x| x.as_table())
//             .and_then(|x| x.get("dependencies"))
//             .and_then(|x| x.as_table()),
//         "didn't find workspace dependencies or is not table"
//     )?;
//
//     let mut workspace_deps_to_add = toml::Table::new();
//     for dep_name in inherited_deps {
//         let workspace_dep_data = cu::check!(
//             workspace_deps.get(&dep_name),
//             "did not find dependency '{dep_name}' in workspace"
//         )?;
//         if let Some(data) = workspace_dep_data.as_table() {
//             if data.get("path").is_some() {
//                 cu::bail!(
//                     "workspace dep cannot be a path when creating isolated crate: {dep_name}"
//                 );
//             }
//         }
//         workspace_deps_to_add.insert(dep_name, workspace_dep_data.clone());
//     }
//     let workspace = cu::check!(
//         manifest["workspace"].as_table_mut(),
//         "unexpected: [workspace] not found in generated manifest"
//     )?;
//     workspace.insert("dependencies".to_string(), workspace_deps_to_add.into());
//
//     cu::check!(
//         toml::stringify_pretty(&manifest),
//         "failed to serialize generated manifest"
//     )
// }
// fn remove_dependencies(
//     dep_key: &str,
//     manifest: &mut toml::Table,
//     remove: &BTreeSet<String>,
// ) -> cu::Result<()> {
//     let Some(deps) = manifest.get_mut(dep_key) else {
//         return Ok(());
//     };
//     let deps = cu::check!(
//         deps.as_table_mut(),
//         "dependencies map must be a table (for key '{dep_key}')"
//     )?;
//     for r in remove {
//         let _ = deps.remove(r);
//     }
//     Ok(())
// }
//
// fn collect_inherited_dependencies(
//     dep_key: &str,
//     manifest: &toml::Table,
//     out: &mut Vec<String>,
// ) -> cu::Result<()> {
//     let Some(deps) = manifest.get(dep_key) else {
//         return Ok(());
//     };
//     let deps = cu::check!(
//         deps.as_table(),
//         "dependencies map must be a table (for key '{dep_key}')"
//     )?;
//     for (dep_name, dep_data) in deps {
//         let Some(dep_data) = dep_data.as_table() else {
//             cu::debug!("skipping dep '{dep_name}' since value is not a table");
//             continue;
//         };
//         let is_workspace = dep_data
//             .get("workspace")
//             .and_then(|x| x.as_bool())
//             .unwrap_or(false);
//         if is_workspace {
//             cu::debug!("adding dep '{dep_name}'");
//             out.push(dep_name.clone());
//         }
//     }
//     Ok(())
// }
