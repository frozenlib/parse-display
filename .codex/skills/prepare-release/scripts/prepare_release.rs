#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"
---

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CATEGORIES: &[&str] = &[
    "Added",
    "Changed",
    "Deprecated",
    "Removed",
    "Fixed",
    "Performance",
    "Security",
];

#[derive(Clone, Copy)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut dry_run = false;
    let mut version = None;

    for arg in env::args().skip(1) {
        if arg == "--dry-run" {
            dry_run = true;
        } else if version.is_none() {
            version = Some(arg);
        } else {
            return Err("usage: prepare_release.rs [--dry-run] <version>".to_owned());
        }
    }

    let root = find_root(&env::current_dir().map_err(|err| err.to_string())?)?;
    let parse_display_manifest = root.join("parse-display").join("Cargo.toml");
    let current_version = package_version(&parse_display_manifest)?;

    let Some(version) = version else {
        print_version_candidates(&root, &current_version)?;
        return Ok(());
    };
    if !is_version(&version) {
        return Err("version must be explicit x.y.z, for example 0.11.0".to_owned());
    }

    let derive_manifest = root.join("parse-display-derive").join("Cargo.toml");
    let lockfile = root.join("Cargo.lock");

    let previous_version = previous_changelog_version(&root.join("CHANGELOG.md"))?;
    let derive_changed = derive_changed_since_last_version(&root)?;

    let mut changed = Vec::new();
    set_package_version(&parse_display_manifest, &version, dry_run, &mut changed)?;
    set_lock_package_version(&lockfile, "parse-display", &version, dry_run, &mut changed)?;
    update_docs(&root, &current_version, &version, dry_run, &mut changed)?;

    if derive_changed {
        set_package_version(&derive_manifest, &version, dry_run, &mut changed)?;
        set_parse_display_derive_dependency(
            &parse_display_manifest,
            &version,
            dry_run,
            &mut changed,
        )?;
        set_lock_package_version(
            &lockfile,
            "parse-display-derive",
            &version,
            dry_run,
            &mut changed,
        )?;
    }

    update_changelog(
        &root.join("CHANGELOG.md"),
        &previous_version,
        &version,
        dry_run,
        &mut changed,
    )?;

    changed.sort();
    changed.dedup();

    let mode = if dry_run { "Would update" } else { "Updated" };
    println!("{mode} release files for parse-display {version}.");
    println!("parse-display-derive changed since previous version commit: {derive_changed}");
    for path in changed {
        println!("{}", relative_slash(&root, &path));
    }

    Ok(())
}

fn is_version(value: &str) -> bool {
    parse_version(value).is_some()
}

fn parse_version(value: &str) -> Option<Version> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || !parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
    {
        return None;
    }

    Some(Version {
        major: parts[0].parse().ok()?,
        minor: parts[1].parse().ok()?,
        patch: parts[2].parse().ok()?,
    })
}

fn print_version_candidates(root: &Path, current_version: &str) -> Result<(), String> {
    let version = parse_version(current_version)
        .ok_or_else(|| format!("current parse-display version is invalid: {current_version}"))?;
    let categories = unreleased_categories(&root.join("CHANGELOG.md"))?;
    let has_minor_changes = categories.iter().any(|(name, body)| {
        matches!(
            name.as_str(),
            "Added" | "Changed" | "Deprecated" | "Removed"
        ) && body.iter().any(|line| !line.trim().is_empty())
    });
    let has_patch_changes = categories.iter().any(|(name, body)| {
        matches!(name.as_str(), "Fixed" | "Performance" | "Security")
            && body.iter().any(|line| !line.trim().is_empty())
    });

    let patch = format!("{}.{}.{}", version.major, version.minor, version.patch + 1);
    let minor = format!("{}.{}.0", version.major, version.minor + 1);
    let major = format!("{}.0.0", version.major + 1);
    let recommended = if has_minor_changes {
        &minor
    } else if has_patch_changes {
        &patch
    } else {
        &minor
    };

    println!("No target version was provided. No files were changed.");
    println!("Current parse-display version: {current_version}");
    println!("Next version candidates:");
    println!("- patch: {patch}");
    println!("- minor: {minor}");
    println!("- major: {major}");
    if has_minor_changes || has_patch_changes {
        println!("Recommended from CHANGELOG.md Unreleased entries: {recommended}");
    } else {
        println!("Recommended default when Unreleased is empty: {recommended}");
    }
    println!("Run again with an explicit version, for example: {recommended}");
    Ok(())
}

fn find_root(start: &Path) -> Result<PathBuf, String> {
    for candidate in start.ancestors() {
        if candidate.join("Cargo.toml").is_file()
            && candidate.join("CHANGELOG.md").is_file()
            && candidate.join("parse-display").join("Cargo.toml").is_file()
            && candidate
                .join("parse-display-derive")
                .join("Cargo.toml")
                .is_file()
        {
            return Ok(candidate.to_path_buf());
        }
    }
    Err("could not find the parse-display repository root".to_owned())
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path.display()))
}

fn write_if_changed(
    path: &Path,
    text: String,
    dry_run: bool,
    changed: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if read(path)? == text {
        return Ok(());
    }

    changed.push(path.to_path_buf());
    if !dry_run {
        fs::write(path, text)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    }
    Ok(())
}

fn package_version(manifest: &Path) -> Result<String, String> {
    for line in read(manifest)?.lines() {
        if let Some(version) = parse_quoted_value(line, "version") {
            return Ok(version.to_owned());
        }
    }
    Err(format!(
        "could not find package version in {}",
        manifest.display()
    ))
}

fn parse_quoted_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let line = line.trim();
    let prefix = format!("{key} = \"");
    let rest = line.strip_prefix(&prefix)?;
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn set_package_version(
    manifest: &Path,
    version: &str,
    dry_run: bool,
    changed: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut replaced = false;
    let text = read(manifest)?;
    let lines = text
        .lines()
        .map(|line| {
            if !replaced && parse_quoted_value(line, "version").is_some() {
                replaced = true;
                format!("version = \"{version}\"")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>();

    if !replaced {
        return Err(format!(
            "could not update package version in {}",
            manifest.display()
        ));
    }

    write_if_changed(manifest, finish(lines), dry_run, changed)
}

fn set_parse_display_derive_dependency(
    manifest: &Path,
    version: &str,
    dry_run: bool,
    changed: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let text = read(manifest)?;
    let old_line = text
        .lines()
        .find(|line| line.trim_start().starts_with("parse-display-derive = {"))
        .ok_or_else(|| {
            format!(
                "could not find parse-display-derive dependency in {}",
                manifest.display()
            )
        })?;
    let new_line = replace_dependency_version(old_line, &format!("={version}"))?;
    write_if_changed(
        manifest,
        text.replacen(old_line, &new_line, 1),
        dry_run,
        changed,
    )
}

fn replace_dependency_version(line: &str, version: &str) -> Result<String, String> {
    let marker = "version = \"";
    let start = line
        .find(marker)
        .ok_or_else(|| "dependency line has no version".to_owned())?
        + marker.len();
    let end = line[start..]
        .find('"')
        .map(|offset| start + offset)
        .ok_or_else(|| "dependency line has unterminated version".to_owned())?;
    Ok(format!("{}{}{}", &line[..start], version, &line[end..]))
}

fn set_lock_package_version(
    lockfile: &Path,
    package: &str,
    version: &str,
    dry_run: bool,
    changed: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut in_target_package = false;
    let mut replaced = false;
    let lines = read(lockfile)?
        .lines()
        .map(|line| {
            if line == "[[package]]" {
                in_target_package = false;
            } else if line == format!("name = \"{package}\"") {
                in_target_package = true;
            } else if in_target_package && line.starts_with("version = \"") {
                replaced = true;
                return format!("version = \"{version}\"");
            }
            line.to_owned()
        })
        .collect::<Vec<_>>();

    if !replaced {
        return Err(format!(
            "could not update {package} in {}",
            lockfile.display()
        ));
    }

    write_if_changed(lockfile, finish(lines), dry_run, changed)
}

fn update_docs(
    root: &Path,
    old_version: &str,
    version: &str,
    dry_run: bool,
    changed: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for path in doc_files(root)? {
        let text = read(&path)?;
        let new_text = update_doc_text(&text, old_version, version);
        write_if_changed(&path, new_text, dry_run, changed)?;
    }
    Ok(())
}

fn update_doc_text(text: &str, old_version: &str, version: &str) -> String {
    let dependency_before = format!("parse-display = \"{old_version}\"");
    let dependency_after = format!("parse-display = \"{version}\"");
    let docs_before = format!("docs.rs/parse-display/{old_version}/");
    let docs_after = format!("docs.rs/parse-display/{version}/");
    let mut in_deprecated_section = false;

    let lines = text
        .lines()
        .map(|line| {
            let doc_line = markdown_text(line);
            if doc_line.starts_with("## ") {
                in_deprecated_section =
                    doc_line.contains("Deprecated") || doc_line.contains("非推奨");
            }

            let mut line = line.replace(&dependency_before, &dependency_after);
            if !in_deprecated_section {
                line = line.replace(&docs_before, &docs_after);
            }
            line
        })
        .collect::<Vec<_>>();

    finish(lines)
}

fn markdown_text(line: &str) -> &str {
    line.trim_start()
        .strip_prefix("///")
        .or_else(|| line.trim_start().strip_prefix("//!"))
        .unwrap_or(line)
        .trim_start()
}

fn doc_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_doc_files(root, root, &mut files)?;
    Ok(files)
}

fn collect_doc_files(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|err| format!("failed to read {}: {err}", dir.display()))?
    {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        let name = entry.file_name();

        if path.is_dir() {
            if matches!(name.to_str(), Some(".git" | "target" | ".codex")) {
                continue;
            }
            collect_doc_files(root, &path, files)?;
            continue;
        }

        if path.is_file()
            && matches!(path.extension().and_then(OsStr::to_str), Some("md" | "rs"))
            && !matches!(
                path.file_name().and_then(OsStr::to_str),
                Some("Cargo.toml" | "Cargo.lock" | "CHANGELOG.md")
            )
            && path.starts_with(root)
        {
            files.push(path);
        }
    }
    Ok(())
}

fn derive_changed_since_last_version(root: &Path) -> Result<bool, String> {
    let commit = git_output(
        root,
        &[
            "log",
            "--format=%H",
            "--extended-regexp",
            "--grep",
            "^Version [0-9]+\\.[0-9]+\\.[0-9]+\\.$",
            "-n",
            "1",
        ],
    )?;
    let commit = commit.trim();
    if commit.is_empty() {
        return Ok(true);
    }

    let committed = Command::new("git")
        .args([
            "diff",
            "--quiet",
            &format!("{commit}..HEAD"),
            "--",
            "parse-display-derive",
        ])
        .current_dir(root)
        .status()
        .map_err(|err| format!("failed to run git diff: {err}"))?;
    if committed.code() == Some(1) {
        return Ok(true);
    }
    if !committed.success() {
        return Err("git diff failed while checking parse-display-derive".to_owned());
    }

    Ok(!git_output(
        root,
        &["status", "--porcelain", "--", "parse-display-derive"],
    )?
    .trim()
    .is_empty())
}

fn git_output(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|err| format!("failed to run git: {err}"))?;
    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|err| err.to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

fn previous_changelog_version(changelog: &Path) -> Result<String, String> {
    for line in read(changelog)?.lines() {
        if let Some(rest) = line.strip_prefix("## [") {
            let Some(end) = rest.find(']') else {
                continue;
            };
            let version = &rest[..end];
            if version != "Unreleased" {
                return Ok(version.to_owned());
            }
        }
    }
    Err("could not find previous changelog version".to_owned())
}

fn unreleased_categories(changelog: &Path) -> Result<Vec<(String, Vec<String>)>, String> {
    let text = read(changelog)?;
    let lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    let unreleased = lines
        .iter()
        .position(|line| line == "## [Unreleased]")
        .ok_or_else(|| "could not find ## [Unreleased] in CHANGELOG.md".to_owned())?;
    let next_release = lines[unreleased + 1..]
        .iter()
        .position(|line| line.starts_with("## ["))
        .map(|index| unreleased + 1 + index)
        .ok_or_else(|| "could not find previous release section in CHANGELOG.md".to_owned())?;

    split_categories(&lines[unreleased + 1..next_release])
}

fn update_changelog(
    changelog: &Path,
    previous_version: &str,
    version: &str,
    dry_run: bool,
    changed: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let text = read(changelog)?;
    let lines = text.lines().map(str::to_owned).collect::<Vec<_>>();

    if lines.iter().any(|line| {
        line == &format!("## [{version}]") || line.starts_with(&format!("## [{version}] - "))
    }) {
        return Err(format!(
            "CHANGELOG.md already contains a section for {version}"
        ));
    }

    let unreleased = lines
        .iter()
        .position(|line| line == "## [Unreleased]")
        .ok_or_else(|| "could not find ## [Unreleased] in CHANGELOG.md".to_owned())?;
    let next_release = lines[unreleased + 1..]
        .iter()
        .position(|line| line.starts_with("## ["))
        .map(|index| unreleased + 1 + index)
        .ok_or_else(|| "could not find previous release section in CHANGELOG.md".to_owned())?;

    let mut categories = split_categories(&lines[unreleased + 1..next_release])?;
    for category in CATEGORIES {
        if !categories.iter().any(|(name, _)| name == category) {
            categories.push(((*category).to_owned(), Vec::new()));
        }
    }

    let non_empty = categories
        .iter()
        .filter(|(_, body)| body.iter().any(|line| !line.trim().is_empty()))
        .cloned()
        .collect::<Vec<_>>();

    let mut rebuilt = Vec::new();
    rebuilt.extend_from_slice(&lines[..unreleased]);
    rebuilt.push("## [Unreleased]".to_owned());
    rebuilt.push(String::new());
    for (name, _) in &categories {
        rebuilt.push(format!("### {name}"));
        rebuilt.push(String::new());
    }

    rebuilt.push(format!("## [{version}] - {}", today()?));
    rebuilt.push(String::new());
    for (name, body) in non_empty {
        rebuilt.push(format!("### {name}"));
        rebuilt.push(String::new());
        rebuilt.extend(body);
        rebuilt.push(String::new());
    }
    rebuilt.extend_from_slice(&lines[next_release..]);

    let new_text = update_changelog_links(&finish(rebuilt), previous_version, version);
    write_if_changed(changelog, new_text, dry_run, changed)
}

fn split_categories(lines: &[String]) -> Result<Vec<(String, Vec<String>)>, String> {
    let mut categories = Vec::new();
    let mut current_name = None;
    let mut current_body = Vec::new();

    for line in trim_blank_edges(lines) {
        if let Some(name) = line.strip_prefix("### ") {
            if let Some(name) = current_name.replace(name.trim().to_owned()) {
                categories.push((name, trim_blank_edges(&current_body).to_vec()));
                current_body.clear();
            }
        } else if current_name.is_some() {
            current_body.push(line.to_owned());
        } else if !line.trim().is_empty() {
            return Err(
                "found non-category content under Unreleased; move it under a category first"
                    .to_owned(),
            );
        }
    }

    if let Some(name) = current_name {
        categories.push((name, trim_blank_edges(&current_body).to_vec()));
    }
    Ok(categories)
}

fn update_changelog_links(text: &str, previous_version: &str, version: &str) -> String {
    let mut lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    let base = "https://github.com/frozenlib/parse-display/compare/";
    let unreleased = lines
        .iter()
        .position(|line| line.to_ascii_lowercase().starts_with("[unreleased]:"));

    if let Some(index) = unreleased {
        lines[index] = format!("[unreleased]: {base}v{version}...HEAD");
        if !lines
            .iter()
            .any(|line| line.starts_with(&format!("[{version}]:")))
        {
            lines.insert(
                index + 1,
                format!("[{version}]: {base}v{previous_version}...v{version}"),
            );
        }
    } else {
        lines.push(format!("[unreleased]: {base}v{version}...HEAD"));
        lines.push(format!(
            "[{version}]: {base}v{previous_version}...v{version}"
        ));
    }

    finish(lines)
}

fn trim_blank_edges(lines: &[String]) -> &[String] {
    let mut start = 0;
    let mut end = lines.len();
    while start < end && lines[start].trim().is_empty() {
        start += 1;
    }
    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    &lines[start..end]
}

fn today() -> Result<String, String> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", "Get-Date -Format yyyy-MM-dd"])
        .output()
        .map_err(|err| format!("failed to get date: {err}"))?;
    if output.status.success() {
        let date = String::from_utf8(output.stdout).map_err(|err| err.to_string())?;
        Ok(date.trim().to_owned())
    } else {
        Err("failed to get current date".to_owned())
    }
}

fn finish(lines: Vec<String>) -> String {
    let mut text = lines.join("\n");
    while text.ends_with("\n\n") {
        text.pop();
    }
    text.push('\n');
    text
}

fn relative_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
