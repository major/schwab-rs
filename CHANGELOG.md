# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1](https://github.com/major/schwab-rs/compare/0.3.0..0.3.1) - 2026-06-01

### Features

- Add schwab-agent binary - ([878dbaa](https://github.com/major/schwab-rs/commit/878dbaa9ab4e2db2b0d9aa1fa52d5ff470722667))

### Bug Fixes

- Address schwab-agent review findings - ([2c3bac5](https://github.com/major/schwab-rs/commit/2c3bac5a44d5efcc47afcf3b1e60d6d7cc6e4bd1))

### Documentation

- Clarify coverage threshold - ([b86964b](https://github.com/major/schwab-rs/commit/b86964bafa0d233ec4e855b7f6d26212f585b34e))
- Update workflow documentation - ([424516f](https://github.com/major/schwab-rs/commit/424516f529406df3df2c4fecd31bc9054eadabb0))

### Miscellaneous Tasks

- Harden coverage tooling - ([43f8204](https://github.com/major/schwab-rs/commit/43f820481888f219ec90cb59522cf56590913cdb))
- Ignore coverage report in reviews - ([cbb9b6b](https://github.com/major/schwab-rs/commit/cbb9b6b470dc3c73ec4689123fdce605c385b856))
- Add local workflow checks - ([9ce7e33](https://github.com/major/schwab-rs/commit/9ce7e333ddfed937f7117143008274513dd0c9da))
- Add coverage and machete jobs - ([6eebc9e](https://github.com/major/schwab-rs/commit/6eebc9e1fed42e585a246ac9ec9e56920738c26e))
- Harden release-plz config - ([284eef8](https://github.com/major/schwab-rs/commit/284eef8fe4b91569eb323ad67354a9d331022e5a))
- Bump MSRV to Rust 1.96 - ([7f710f1](https://github.com/major/schwab-rs/commit/7f710f149ba37468393737b68256abf986cf9cd8))


## [0.3.0](https://github.com/major/schwab-rs/compare/0.2.3..0.3.0) - 2026-05-19

### Features

- Add repeat order conversion - ([d722f2b](https://github.com/major/schwab-rs/commit/d722f2b9439e44b83cfc24375b01ef066b246777))


## [0.2.3](https://github.com/major/schwab-rs/compare/0.2.2..0.2.3) - 2026-05-19


## [0.2.2](https://github.com/major/schwab-rs/compare/0.2.1..0.2.2) - 2026-05-18

### Bug Fixes

- Tolerate unknown order statuses - ([e1cfc6e](https://github.com/major/schwab-rs/commit/e1cfc6e553d5588b19c416901e4830a9f04f880d))

### Miscellaneous Tasks

- Switch from dtolnay/rust-toolchain to actions-rust-lang/setup-rust-toolchain - ([d0a836e](https://github.com/major/schwab-rs/commit/d0a836e79dd32ad57d39653340f6cf887c038c0a))


## [0.2.1](https://github.com/major/schwab-rs/compare/0.2.0..0.2.1) - 2026-05-17


## [0.2.0](https://github.com/major/schwab-rs/compare/v0.1.7...v0.2.0) - 2026-05-17

### Added

- *(streaming)* add ACCT_ACTIVITY service support
- *(streaming)* add SCREENER_EQUITY and SCREENER_OPTION streaming services
- *(streaming)* add CHART_EQUITY and CHART_FUTURES streaming services
- *(streaming)* add Wave 2 - field enums, data structs, protocol builders, transport impl
- *(streaming)* add module scaffold and dependencies

### Fixed

- *(streaming)* address Copilot review feedback
- *(streaming)* redact bearer token in Debug, add non_exhaustive to field enums, fix build_view credentials
- *(streaming)* resolve make check failures

### Other

- *(streaming)* centralize subscription field serialization
- Add schwab streaming docs
- add rustdoc examples to all public streaming types and methods
- improve lint suppression comments and parse_num consistency
- update AGENTS.md and README.md for streaming API
- *(streaming)* add unit tests with mock WebSocket transport

## [0.1.7](https://github.com/major/schwab-rs/compare/v0.1.6...v0.1.7) - 2026-05-15

### Other

- clarify MSRV is independent of Edition 2024 requirement
- bump MSRV to 1.95 and adopt new language features

## [0.1.6](https://github.com/major/schwab-rs/compare/v0.1.5...v0.1.6) - 2026-05-15

### Other

- update MSRV references to 1.88 and add doc-freshness notice

## [0.1.5](https://github.com/major/schwab-rs/compare/v0.1.4...v0.1.5) - 2026-05-15

### Fixed

- use PAT for release-plz PRs to trigger CI checks

## [0.1.4](https://github.com/major/schwab-rs/compare/v0.1.3...v0.1.4) - 2026-05-15

### Added

- derive Serialize on order-related response types

## [0.1.3](https://github.com/major/schwab-rs/compare/v0.1.2...v0.1.3) - 2026-05-14

### Other

- adopt standard release-plz workflow with automated release PRs
- update AGENTS.md with release process and trusted publishing

## [0.1.2] - 2026-05-14

### Fixed

- Derive `Serialize` on `CandleList` and `Candle` so consumers can serialize market history responses.

## [0.1.1] - 2026-05-14

### Changed

- Enable crates.io publishing.
- Allow direct release publishing via CI workflow.

## [0.1.0] - 2026-05-13

Initial release.
