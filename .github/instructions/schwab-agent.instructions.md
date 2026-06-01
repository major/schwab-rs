---
applyTo: "src/bin/schwab-agent/**/*.rs,tests/cli_smoke.rs"
---

# schwab-agent review instructions

- Keep library boundaries intact: CLI modules may read config, environment variables, write process output, and return process exit codes; library modules must not.
- Preserve structured JSON for normal command results and command errors. The `completions` command is the only raw stdout exception because shells need a script.
- Keep mutable order actions behind the config guard, resolve account selectors to canonical Schwab hashes, and verify order state after placement, replacement, repeat placement, or cancel.
- Never print bearer tokens, account hashes beyond user-requested account output, raw HTTP error bodies, saved preview contents, or credential values in logs, errors, tests, or docs.
- Add or update compiled-binary smoke coverage in `tests/cli_smoke.rs` when command names, clap behavior, JSON error shape, dry-run order output, or completions behavior changes.
- Update `README.md`, root `AGENTS.md`, `src/AGENTS.md`, `src/models/AGENTS.md`, `src/bin/schwab-agent/AGENTS.md`, and `src/bin/schwab-agent/SKILL.md` when commands, args, output format, safety rules, or CLI dependencies change.
