use std::path::Path;
use std::path::PathBuf;

use crate::errors::CargoResult;
use cargo_edit::{shell_status, shell_warn, LocalManifest};
use cargo_metadata::Package;
use clap::Args;

/// Change a package's registry in the local manifest file (i.e. Cargo.toml).
#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("reg").multiple(false))]
pub struct SetRegistryArgs {
    /// Registry to update dependency to
    #[arg(group = "reg")]
    registry: String,

    /// Path to the manifest to upgrade
    #[arg(long, value_name = "PATH")]
    manifest_path: Option<PathBuf>,

    /// Package id of the crate to change the registry of
    #[arg(long = "package", short = 'p', value_name = "PKGID")]
    pkgids: Vec<String>,

    /// Print changes to be made without making them.
    #[arg(long)]
    dry_run: bool,

    /// Exclude a crate from the modification.
    #[arg(long)]
    exclude: Vec<String>,

    /// Require `Cargo.toml` to be up to date
    #[arg(long)]
    locked: bool,
}

impl SetRegistryArgs {
    pub fn exec(self) -> CargoResult<()> {
        exec(self)
    }
}

/// Main processing function. Allows us to return a `Result` so that `main` can print pretty error
/// messages.
fn exec(args: SetRegistryArgs) -> CargoResult<()> {
    let SetRegistryArgs {
        registry,
        manifest_path,
        pkgids,
        dry_run,
        exclude,
        locked,
    } = args;

    let ws_metadata = resolve_ws(manifest_path.as_deref(), locked)?;
    let workspace_members = find_ws_members(&ws_metadata);

    let selected = workspace_members
        .iter()
        .filter(|p| !exclude.contains(&p.name))
        .collect::<Vec<_>>();

    for package in selected {
        update_member(&pkgids, &registry, package, dry_run)?
    }

    resolve_ws(manifest_path.as_deref(), locked)?;
    if dry_run {
        shell_warn("aborting set-registry due to dry run")?;
    }

    Ok(())
}

fn update_member(
    pkgids: &[String],
    new_registry: &str,
    member: &Package,
    dry_run: bool,
) -> CargoResult<()> {
    let manifest_path = member.manifest_path.as_std_path();
    let name = member.name.as_str();
    let mut dep_manifest = LocalManifest::try_new(manifest_path)?;
    let mut changed = false;

    let deps = dep_manifest.get_dependency_tables_mut().flat_map(|t| {
        t.iter_mut().filter_map(|(k, d)| {
            let key_repr = k.to_string();

            let pkg_is_selected_for_update = pkgids
                .iter()
                .any(|pkgid| pkgid.as_str().trim() == key_repr.as_str().trim());

            if pkg_is_selected_for_update {
                d.as_table_like_mut()
            } else {
                None
            }
        })
    });

    for dep in deps {
        let old_reg = dep.get("registry").and_then(|item| item.as_str());
        match old_reg {
            Some(old_reg_value) if old_reg_value == new_registry => (),
            Some(old_reg_value) => {
                shell_status(
                    "Updating",
                    &format!(
                        "{}'s dependency from registry \"{}\" to \"{}\"",
                        name, old_reg_value, new_registry
                    ),
                )?;
                dep.insert("registry", toml_edit::value(new_registry));
                changed = true;
            }
            None => {
                shell_status(
                    "Updating",
                    &format!("{}'s dependency to add registry \"{}\"", name, new_registry),
                )?;
                dep.insert("registry", toml_edit::value(new_registry));
                changed = true;
            }
        }
    }
    if changed && !dry_run {
        dep_manifest.write()?;
    }

    Ok(())
}

fn resolve_ws(manifest_path: Option<&Path>, locked: bool) -> CargoResult<cargo_metadata::Metadata> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    if let Some(manifest_path) = manifest_path {
        cmd.manifest_path(manifest_path);
    }
    cmd.features(cargo_metadata::CargoOpt::AllFeatures);
    let mut other = Vec::new();
    if locked {
        other.push("--locked".to_owned());
    }
    other.push("--offline".to_owned());
    cmd.other_options(other);

    let ws = cmd.exec().or_else(|_| {
        cmd.no_deps();
        cmd.exec()
    })?;
    Ok(ws)
}

fn find_ws_members(ws: &cargo_metadata::Metadata) -> Vec<cargo_metadata::Package> {
    let workspace_members: std::collections::HashSet<_> = ws.workspace_members.iter().collect();
    ws.packages
        .iter()
        .filter(|p| workspace_members.contains(&p.id))
        .cloned()
        .collect()
}
