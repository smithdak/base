use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use semver::Version;
use serde::Serialize;

use crate::base_home;
use crate::cli::AdoptArgs;
use crate::config::{Config, PackRecord};
use crate::lock::{LockMode, RepositoryLock};
use crate::pack;

use super::print_json;

#[derive(Debug, Serialize)]
struct AdoptReport {
    pack: String,
    version: String,
    action: &'static str,
    root: String,
    files: Vec<String>,
    follow_ups: Vec<String>,
}

pub fn run(project_root: &Path, args: AdoptArgs, json: bool) -> Result<()> {
    let mut config = Config::load(project_root)?;
    let home = base_home()?;
    let _global_lock = RepositoryLock::global(&home, LockMode::Shared)?;
    let packs_root = home.join("canon").join("packs");
    let source_root = pack::library_root(&home, &args.pack);
    if !source_root.is_dir() {
        bail!(
            "no pack `{}` in {}; {}",
            args.pack,
            packs_root.display(),
            available_packs(&packs_root)
        );
    }

    let incoming = pack::build_record(&source_root)?;
    if incoming.id != args.pack {
        bail!(
            "pack folder `{}` contains manifest id `{}`",
            args.pack,
            incoming.id
        );
    }
    let files = pack::collect_files(&source_root)?;
    let installed_index = config.packs.iter().position(|item| item.id == args.pack);
    let (action, follow_ups) = match (installed_index, args.upgrade) {
        (None, false) => {
            install_new(project_root, &mut config, &incoming, &files)?;
            (
                "adopted",
                vec![
                    "run `base check && base sync`".to_owned(),
                    "review pack policy and verifier commands before enabling generated hooks"
                        .to_owned(),
                    "put project-specific changes in .base/canon/ overrides, not .base/packs/"
                        .to_owned(),
                    "commit the pack, config, and regenerated surfaces together".to_owned(),
                ],
            )
        }
        (Some(_), false) => bail!(
            "pack `{}` is already adopted; use `base adopt {} --upgrade` for a newer version",
            args.pack,
            args.pack
        ),
        (None, true) => bail!(
            "pack `{}` is not adopted; omit --upgrade for the initial adoption",
            args.pack
        ),
        (Some(index), true) => {
            let installed = config.packs[index].clone();
            pack::verify_installed(project_root, &installed)?;
            let old = Version::parse(&installed.version)?;
            let new = Version::parse(&incoming.version)?;
            if new < old {
                bail!(
                    "refusing to downgrade pack `{}` from {} to {}",
                    args.pack,
                    old,
                    new
                );
            }
            if new == old {
                if incoming.files != installed.files {
                    bail!(
                        "pack `{}` version {} changed content; versions are immutable — publish a newer version",
                        args.pack,
                        new
                    );
                }
                (
                    "unchanged",
                    vec!["installed pack already matches the library version".to_owned()],
                )
            } else {
                replace(project_root, &mut config, index, &incoming, &files)?;
                (
                    "upgraded",
                    vec![
                        "run `base check && base sync`".to_owned(),
                        "review changed policy and verifier commands before enabling generated hooks"
                            .to_owned(),
                        "review the pack diff and project overrides before committing".to_owned(),
                        "commit the upgraded pack, config, and regenerated surfaces together"
                            .to_owned(),
                    ],
                )
            }
        }
    };

    let report = AdoptReport {
        pack: incoming.id,
        version: incoming.version,
        action,
        root: format!(".base/packs/{}", args.pack),
        files: incoming.files.keys().cloned().collect(),
        follow_ups,
    };
    if json {
        print_json(&report)
    } else {
        println!(
            "{} pack `{}` version {} ({} files at {})",
            report.action,
            report.pack,
            report.version,
            report.files.len(),
            report.root
        );
        for follow_up in &report.follow_ups {
            println!("  - {follow_up}");
        }
        Ok(())
    }
}

fn install_new(
    project_root: &Path,
    config: &mut Config,
    record: &PackRecord,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    let destination = pack::installed_root(project_root, &record.id);
    if destination.exists() {
        bail!(
            "unmanaged pack directory already exists at {}; move it or add a matching config record deliberately",
            destination.display()
        );
    }
    let staged = stage(project_root, &record.id, files)?;
    fs::rename(&staged, &destination).with_context(|| {
        format!(
            "cannot install staged pack {} at {}",
            staged.display(),
            destination.display()
        )
    })?;
    config.packs.push(record.clone());
    if let Err(error) = config.save(project_root) {
        let _ = fs::rename(&destination, &staged);
        let _ = fs::remove_dir_all(&staged);
        config.packs.pop();
        return Err(error).context("pack files were rolled back after config save failed");
    }
    Ok(())
}

fn replace(
    project_root: &Path,
    config: &mut Config,
    index: usize,
    record: &PackRecord,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    let destination = pack::installed_root(project_root, &record.id);
    let staged = stage(project_root, &record.id, files)?;
    let backup = temporary_path(project_root, &format!("backup-{}", record.id));
    fs::rename(&destination, &backup)
        .with_context(|| format!("cannot stage current pack at {}", backup.display()))?;
    if let Err(error) = fs::rename(&staged, &destination) {
        let _ = fs::rename(&backup, &destination);
        let _ = fs::remove_dir_all(&staged);
        return Err(error).context("cannot activate upgraded pack; current pack was restored");
    }
    let previous = std::mem::replace(&mut config.packs[index], record.clone());
    if let Err(error) = config.save(project_root) {
        let _ = fs::remove_dir_all(&destination);
        let _ = fs::rename(&backup, &destination);
        config.packs[index] = previous;
        return Err(error).context("upgraded pack was rolled back after config save failed");
    }
    fs::remove_dir_all(&backup)
        .with_context(|| format!("cannot remove pack upgrade backup {}", backup.display()))?;
    Ok(())
}

fn stage(project_root: &Path, id: &str, files: &BTreeMap<String, Vec<u8>>) -> Result<PathBuf> {
    let root = temporary_path(project_root, &format!("stage-{id}"));
    fs::create_dir_all(&root)
        .with_context(|| format!("cannot create staged pack {}", root.display()))?;
    let result = (|| {
        for (relative, content) in files {
            let destination = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("cannot create {}", parent.display()))?;
            }
            fs::write(&destination, content)
                .with_context(|| format!("cannot write {}", destination.display()))?;
        }
        Ok::<(), anyhow::Error>(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&root);
        return Err(error);
    }
    Ok(root)
}

fn temporary_path(project_root: &Path, label: &str) -> PathBuf {
    project_root.join(".base").join("packs").join(format!(
        ".base-{label}-{}-{}",
        std::process::id(),
        Utc::now().timestamp_millis()
    ))
}

fn available_packs(packs_root: &Path) -> String {
    let mut packs: Vec<String> = fs::read_dir(packs_root)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    packs.sort();
    if packs.is_empty() {
        "no packs installed".to_owned()
    } else {
        format!("available packs: {}", packs.join(", "))
    }
}
