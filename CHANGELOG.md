# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0](https://github.com/major/schwab-rs/compare/0.5.1..0.6.0) - 2026-06-05

### Bug Fixes

- *(cli)* Report margin cash balance - ([beef699](https://github.com/major/schwab-rs/commit/beef699c4a9ce75fa37f4db1572c42ba57f64124))
- *(models)* Accept NASDAQ exchange values - ([ecb1a90](https://github.com/major/schwab-rs/commit/ecb1a900718ee48345ea4ee66df48970d6287050))

### Documentation

- *(cli)* Clarify true cash guidance - ([bb5f493](https://github.com/major/schwab-rs/commit/bb5f493d5c1eebb30c00115dc66700715a4e7bd7))


## [0.5.1](https://github.com/major/schwab-rs/compare/0.5.0..0.5.1) - 2026-06-05

### Bug Fixes

- *(cli)* Expose analyze derived price basis - ([7bd441b](https://github.com/major/schwab-rs/commit/7bd441b6fc19651a12ad0052233d7d9573b1a64c))


## [0.5.0](https://github.com/major/schwab-rs/compare/0.4.0..0.5.0) - 2026-06-05

### Bug Fixes

- *(auth)* Classify invalid refresh tokens - ([7368631](https://github.com/major/schwab-rs/commit/73686314e3f97b9a76a7493b9e8f6fe9713165a1))

### Documentation

- *(cli)* Clarify auth recovery commands - ([d505c4d](https://github.com/major/schwab-rs/commit/d505c4d6f242ed2a623c86ded1dc0b1c6d9f3c18))

### Testing

- *(auth)* Cover text invalid grant response - ([b50a5b4](https://github.com/major/schwab-rs/commit/b50a5b40ef7996f88784819184dadc5e0d327949))


## [0.4.0](https://github.com/major/schwab-rs/compare/0.3.3..0.4.0) - 2026-06-04

### Bug Fixes

- *(cli)* Expose true cash balance status - ([abc6c7d](https://github.com/major/schwab-rs/commit/abc6c7d9b9e7394089a8d43040a7bbd7035065fc))


## [0.3.3](https://github.com/major/schwab-rs/compare/0.3.2..0.3.3) - 2026-06-02

### Bug Fixes

- *(models)* Accept canceled execution activity - ([6681df4](https://github.com/major/schwab-rs/commit/6681df4b3287fedf35409086656f474da801b433))


## [0.3.2](https://github.com/major/schwab-rs/compare/0.3.1..0.3.2) - 2026-06-02

### Features

- *(cli)* Add completion command alias - ([6e14c33](https://github.com/major/schwab-rs/commit/6e14c33e23d20ba13c247a261c09afebac877ccb))
- *(cli)* Standardize order id flags - ([b01eb22](https://github.com/major/schwab-rs/commit/b01eb22a3d69eab9941ad2eee502f4d26a465400))
- *(cli)* Add high-value command aliases - ([2704df4](https://github.com/major/schwab-rs/commit/2704df4e4c151fc7ea5f0f4a98bad9495b83449a))
- *(cli)* Add agent discovery commands - ([a7ae323](https://github.com/major/schwab-rs/commit/a7ae3235d5d826477885e66c12fdb049988b09b0))
- *(cli)* Add explicit order draft flags - ([06e89f4](https://github.com/major/schwab-rs/commit/06e89f4c09f8ba923a870b9ba12e014647bf29f8))
- *(cli)* Add json usage errors - ([ea60709](https://github.com/major/schwab-rs/commit/ea60709805b84e9cce5dd0befccece880c80d62f))
- *(cli)* Add sanitized config status - ([e730145](https://github.com/major/schwab-rs/commit/e73014537cc78ea47e6ffc023c006c8694ccbad0))
- *(cli)* Add command help examples - ([7872d3a](https://github.com/major/schwab-rs/commit/7872d3ae4d3c6f4b7dc2ca0452fab033aededc9e))
- Add schwab-agent completions - ([eb0a18c](https://github.com/major/schwab-rs/commit/eb0a18cbac74e8a0cc55eaa5ea0086df4b7f45eb))

### Bug Fixes

- *(cli)* Accept readable market history dates - ([a4201f7](https://github.com/major/schwab-rs/commit/a4201f7dc45397a1290470602d716f1cf8d02313))
- Remove redundant renovate config - ([2e766ce](https://github.com/major/schwab-rs/commit/2e766ce00ae3857cce86b044c169ba3e272a39b0))

### Documentation

- Document renovate validation - ([3794e26](https://github.com/major/schwab-rs/commit/3794e26594da7f4cfdea041740b526f51cd9a351))
- Clarify order symbol filter workflow - ([13694c5](https://github.com/major/schwab-rs/commit/13694c54273b401697a3b2ce3dfa737ec7566e01))
- Expose schwab-agent guidance - ([282b598](https://github.com/major/schwab-rs/commit/282b59860fabd264f1f0f9985bda862ea17a0f3b))

### Testing

- Harden schwab-agent smoke tests - ([9239063](https://github.com/major/schwab-rs/commit/92390634bb7a9a1c393a1d641006e37ae44286fc))
- Add schwab-agent smoke tests - ([a2f2c73](https://github.com/major/schwab-rs/commit/a2f2c7316b0d9531596ad59fcb36903ad675ae5a))

### Miscellaneous Tasks

- Remove hidden unicode characters - ([8206833](https://github.com/major/schwab-rs/commit/820683398b55a01e792026f1a513b163e988b764))
- Gate schwab-agent behind cli feature - ([bbeec63](https://github.com/major/schwab-rs/commit/bbeec638fea99a36b7755404a3a82f40c09674b5))


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
