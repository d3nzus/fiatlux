# Instructions

These rules apply to this repository. They are written as portable defaults, so the same
file can also serve as a global `~/.config/opencode/AGENTS.md`. More specific instruction
files may add rules, but must not contradict these.

## Working style

- Read the relevant code before editing; never guess at a symbol, path, or API.
- State assumptions and open questions early, especially when requirements are ambiguous.
- Keep changes focused on the task. Do not reformat, rename, or delete unrelated code.
- Prefer the smallest change that fully solves the problem; call out broader refactors instead of doing them silently.
- When a task spans multiple steps, outline the plan before executing and report what changed.
- Never report work as complete without running the relevant verification commands.

## Verification

- Rust: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets`.
- TypeScript/Node: `pnpm typecheck`, `pnpm lint`, `pnpm test` (or the repo's documented equivalents).
- Python: run the configured linter/formatter plus the test suite.
- If a check cannot run (missing toolchain, network, credentials), say so explicitly and describe the residual risk.
- New behavior requires a test. Bug fixes require a regression test.

## Code conventions

- Rust: use `anyhow` at binary/application boundaries and `thiserror` for library errors; avoid `unwrap`/`expect` in non-test code; propagate context with `.with_context(...)`.
- Prefer deterministic containers (`BTreeMap`, sorted vectors) wherever iteration order affects output.
- Validate and normalize every path or identifier that originates from user input or a config file. Reject `..`, absolute paths, and platform-reserved names when a single component is expected.
- Keep modules small and single-purpose; mirror the existing file layout and naming instead of inventing new patterns.
- Document public behavior where users will look for it (README, CLI help, DSL/schema docs) in the same change.

## Safety

- Never commit secrets, tokens, credentials, or machine-specific absolute paths. Use environment variables or config indirection.
- Treat generated scripts, imported configs, and other user-supplied content as untrusted: confirm before executing, and never execute during a dry run.
- Do not run destructive git operations (`push --force`, `reset --hard`, history rewrites, branch deletion) without explicit confirmation.
- Do not overwrite or delete user files that are unrelated to the task; investigate unfamiliar changes before touching them.

## Commits and branches

- Use Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`, `ci:`.
- Keep commits scoped; do not mix refactors with behavior changes.
- Branch names: `feature/<topic>`, `fix/<topic>`, `docs/<topic>`.
- Never amend or rewrite history that has been pushed to a shared branch unless asked.

## Environment notes

- Shell is Git Bash on Windows. Quote paths containing spaces or backslashes.
- Scripts that must run cross-platform should use `cfg(target_os = ...)` branches rather than assuming a shell.
- Prefer repo-relative paths in output; avoid leaking the local username or home directory into committed files.

## Communication

- Be concise and concrete. Reference files as `path:line` so the user can navigate directly.
- Summarize findings by severity, then recommend a prioritized next step.
- Ask before making irreversible changes; otherwise proceed and report.
