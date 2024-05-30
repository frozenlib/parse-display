# Changelog

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Performance

### Security

## [0.9.1] - 2024-05-31

### Changed

- Set `rust-version` to 1.70.0. [#42](https://github.com/frozenlib/parse-display/issues/42)

### Fixed

- Ensure `Pointer` format is formatted correctly.

### Performance

- Optimizing runtime performance for the literal string case. [#39](https://github.com/frozenlib/parse-display/issues/39)

## [0.9.0] - 2024-02-04

### Added

- Support `#[display(with = ...)]`. [#36](https://github.com/frozenlib/parse-display/issues/36)
- Support for use of format traits other than `Display` for self. [#35](https://github.com/frozenlib/parse-display/issues/35)
- Allow DST fields with `#[derive(Display)]`.

### Changed

- Use [`std::sync::OnceLock`] in generated code and remove [`once_cell`] dependency.

[`std::sync::OnceLock`]: https://doc.rust-lang.org/std/sync/struct.OnceLock.html
[`once_cell`]: https://crates.io/crates/once_cell

## [0.8.2] - 2023-07-16

### Added

- Enabled `(?<name>.*)` usage in regex alongside `(?P<name>.*)`.

### Changed

- Update to `regex-syntax` 0.7.

### Fixed

- Fix handling of regex that resemble, but aren't, captures (e.g. `(\(?<a>.*)`)

## [0.8.1] - 2023-06-10

### Added

- Support `#[display(crate = ...)]`.

### Changed

- Update to `syn` 2.0.

## [0.8.0] - 2022-12-21

### Fixed

- Fixed a problem where strings containing newlines could not be parsed [#27](https://github.com/frozenlib/parse-display/issues/27)

## [0.7.0] - 2022-12-05

### Fixed

- Use result with full path in the generated code [#26](https://github.com/frozenlib/parse-display/pull/26)

## [0.6.0] - 2022-09-01

### Added

- Support `#[from_str(ignore)]` for variant.

[unreleased]: https://github.com/frozenlib/parse-display/compare/v0.9.1...HEAD
[0.9.1]: https://github.com/frozenlib/parse-display/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/frozenlib/parse-display/compare/v0.8.2...v0.9.0
[0.8.2]: https://github.com/frozenlib/parse-display/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/frozenlib/parse-display/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/frozenlib/parse-display/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/frozenlib/parse-display/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/frozenlib/parse-display/compare/v0.5.5...v0.6.0
