---
name: prepare-release
description: Prepare parse-display release changes before publishing with a Rust Cargo script when nightly Cargo is available. Use when Codex is asked to prepare a parse-display release, bump parse-display to a requested version, move CHANGELOG.md Unreleased entries into a release section, update parse-display-derive only when it changed since the previous version commit, commit the release prep, and create the release git tag.
---

# Prepare Release

## Workflow

1. Extract the target version from the user's request.
   - Accept only an explicit version such as `0.11.0`.
   - If the user did not provide a version, do not modify files. Ask the user to provide the target version and end the turn.

2. Confirm the repository root is the parse-display workspace.
   - Work from the directory containing `Cargo.toml`, `CHANGELOG.md`, `parse-display/Cargo.toml`, and `parse-display-derive/Cargo.toml`.
   - Inspect `git status --short` before editing. Do not include unrelated user changes in the release commit.
   - Do not use Python scripts for this workflow.

3. Prefer the bundled Cargo script when nightly Cargo supports `-Zscript`.

```powershell
cargo +nightly -Zscript .codex\skills\prepare-release\scripts\prepare_release.rs --dry-run <version>
cargo +nightly -Zscript .codex\skills\prepare-release\scripts\prepare_release.rs <version>
```

The script updates release files only. It never commits and never creates tags.

In sandboxed Codex environments, `cargo +nightly -Zscript` may need approval because Cargo writes build artifacts under Cargo's cache directory.

4. If `cargo +nightly -Zscript` is unavailable, make the same edits manually.
   - Set `parse-display/Cargo.toml` package `version` to `<version>`.
   - Update `Cargo.lock`'s `parse-display` package version to `<version>`.
   - Search docs and Rust doc comments for the previous parse-display version and update only active parse-display version references, such as dependency snippets and current-version docs links.
   - Do not update docs.rs links for deprecated features. Deprecated feature documentation must keep linking to the older version where that deprecated feature is still documented.

```powershell
rg -n '<old-version>|parse-display = "' README.md parse-display parse-display-with
```

5. Decide whether to update `parse-display-derive`.
   - Find the previous version commit:

```powershell
git log --format=%H --extended-regexp --grep "^Version [0-9]+\.[0-9]+\.[0-9]+\.$" -n 1
```

   - Check whether `parse-display-derive` changed since that commit:

```powershell
git diff --name-only <previous-version-commit>..HEAD -- parse-display-derive
git status --porcelain -- parse-display-derive
```

   - If either command shows changes, set `parse-display-derive/Cargo.toml` package `version` to `<version>`, set `parse-display/Cargo.toml`'s `parse-display-derive` dependency version to `=<version>`, and update `Cargo.lock`'s `parse-display-derive` package version to `<version>`.
   - If there are no changes, leave `parse-display-derive/Cargo.toml`, the dependency version, and the lockfile's `parse-display-derive` version unchanged.

6. Update `CHANGELOG.md` manually only if the Cargo script was not used.
   - Insert `## [<version>] - YYYY-MM-DD` immediately after the `Unreleased` block, using the current local date.
   - Move every non-empty category body from `Unreleased` into the same category under the new version.
   - Add only categories with at least one item under the new version.
   - Leave `Unreleased` with category headings only. Preserve these headings unless the file already uses a different category set:
     `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Performance`, `Security`.
   - Update the compare links at the bottom:
     - Change `[unreleased]` to compare `v<version>...HEAD`.
     - Add `[<version>]: https://github.com/frozenlib/parse-display/compare/v<previous-version>...v<version>` immediately after `[unreleased]`.

7. Verify the result.
   - Inspect `git diff -- CHANGELOG.md parse-display/Cargo.toml parse-display-derive/Cargo.toml Cargo.lock README.md parse-display/src parse-display-with/src`.
   - Confirm `CHANGELOG.md` has a non-empty section for the new version and only empty category headings under `Unreleased`.
   - Run the repository's normal Rust validation when practical, at minimum `cargo check --workspace`.

8. Commit only the release-prep files.
   - Stage the files changed by the release workflow.
   - Commit with exactly `Version <version>.`
   - Do not add unrelated worktree changes to this commit.

```powershell
git commit -m "Version <version>."
```

9. Create the release tag after the commit succeeds.

```powershell
git tag "v<version>"
```

## Notes

- Do not create the tag before the release commit exists.
- If the tag already exists, stop and report the conflict.
- If changelog `Unreleased` has no items, still create the version section only if the user explicitly wants that release; otherwise report that there are no changelog entries to release.
