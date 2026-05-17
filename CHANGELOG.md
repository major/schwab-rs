# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
