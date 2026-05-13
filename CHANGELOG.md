# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/major/schwab-rs/releases/tag/v0.1.1) - 2026-05-13

### Fixed

- *(deps)* update rust crate rcgen to 0.14.0 ([#6](https://github.com/major/schwab-rs/pull/6))
- *(deps)* update rust crate rand to 0.10.0 ([#5](https://github.com/major/schwab-rs/pull/5))
- *(renovate)* disable cargo dependency pinning for library crate
- collapse nested if blocks in TokenData::with_expires_at
- set accepted stream to blocking before TLS handshake
- bump MSRV from 1.85 to 1.88

### Other

- allow direct release publishing
- release 0.1.1
- enable crates.io publishing
- Merge pull request #4 from major/renovate/rust-ci-actions
- *(deps)* lock file maintenance
- *(deps)* update reqwest to 0.13.3 ([#9](https://github.com/major/schwab-rs/pull/9))
- *(deps)* update actions/checkout action to v6
- trigger release-plz on push to main instead of manual-only
- Merge branch 'docs/add-missing-docstrings'
- add coverage gate and raise line coverage to 97%
- add hierarchical AGENTS.md for AI agent context
- add CodeRabbit, Renovate, and Copilot review configs
- clarify project affiliation
- use Makefile targets in GitHub Actions for consistent checks
- add rust check makefile
- add security guidance and MSRV CI
- Initial import
