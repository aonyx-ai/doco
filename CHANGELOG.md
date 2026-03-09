<!-- markdownlint-disable-file MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to [Semantic
Versioning].

## [Unreleased]

## [0.2.3] - 2026-03-09

### Added

- Filesystem mount support on `Server` and `Service` via `.mount()`. Enables
  mounting host directories into containers — for example, a pre-built dist
  directory into a lightweight web server image.
- Command override support on `Server` and `Service` via `.cmd_arg()`. Allows
  customizing the container entrypoint arguments.
- Re-exported `Mount` and `AccessMode` types from testcontainers for
  convenience.

## [0.2.2] - 2026-03-09

### Added

- `Doco::connect()` method that returns a long-lived `Session` with a
  ready-to-use `Client`. Unlike the test runner which creates a fresh
  environment per test, `connect()` creates a single session that can be reused
  across many operations — useful for visual regression testing and custom test
  harnesses.
- `Session` type that holds a `Client` and keeps all containers alive via
  ownership. Derefs to `Client` for ergonomic access to WebDriver methods.

### Changed

- Refactored `TestRunner` to use `Session` internally, removing duplicated
  container startup logic.

## [0.2.1] - 2026-03-09

### Added

- Headless browser mode via `Doco::builder().headless(true)`. Defaults to
  auto-detection based on the `CI` environment variable.
- Viewport configuration via `Doco::builder().viewport(Viewport::new(w, h))`.
  When set, the browser window is resized before each test runs.

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

[0.2.3]: https://github.com/otterbuild/doco/releases/tag/0.2.3
[0.2.2]: https://github.com/otterbuild/doco/releases/tag/0.2.2
[0.2.1]: https://github.com/otterbuild/doco/releases/tag/0.2.1
[0.2.0]: https://github.com/otterbuild/doco/releases/tag/0.2.0
[0.1.0]: https://github.com/otterbuild/doco/releases/tag/0.1.0
[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
