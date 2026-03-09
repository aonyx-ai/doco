<!-- markdownlint-disable-file MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to [Semantic
Versioning].

## [Unreleased]

### Added

- Headless browser mode via `Doco::builder().headless(true)`. Defaults to
  auto-detection based on the `CI` environment variable.

## [0.2.0] - 2026-03-08

### Changed

- Migrated WebDriver client from [fantoccini] to [thirtyfour]. The `Client` type
  now derefs to `thirtyfour::WebDriver` instead of `fantoccini::Client`. Users
  who imported fantoccini types directly will need to switch to the thirtyfour
  equivalents.
- Switched from a custom test loop to [libtest-mimic]. Tests now support
  standard runner flags like `--list`, `--skip`, `--ignored`, `--exact`,
  `--test-threads`, and name filtering out of the box.
- Updated `testcontainers` to 0.27, `reqwest` to 0.13, `typed-builder` to 0.23.

### Internal

- Migrated build system from Earthly to Just + Flox.

[fantoccini]: https://crates.io/crates/fantoccini
[thirtyfour]: https://crates.io/crates/thirtyfour
[libtest-mimic]: https://crates.io/crates/libtest-mimic

## [0.1.0] - 2024-10-27

Initial release of the `doco` and `doco-derive` crates

[0.2.0]: https://github.com/otterbuild/doco/releases/tag/v0.2.0
[0.1.0]: https://github.com/otterbuild/doco/releases/tag/v0.1.0
[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
