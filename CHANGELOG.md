# Changelog

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

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

[unreleased]: https://github.com/frozenlib/parse-display/compare/v0.8.2...HEAD
[0.8.2]: https://github.com/frozenlib/parse-display/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/frozenlib/parse-display/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/frozenlib/parse-display/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/frozenlib/parse-display/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/frozenlib/parse-display/compare/v0.5.5...v0.6.0
