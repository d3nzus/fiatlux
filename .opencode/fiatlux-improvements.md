# FiatLux — Review & Improvement Plan

Repo: `C:\Users\d3nzu\Documents\fiatlux`
Stack: Rust 2024 edition, clap, serde_yaml, anyhow, dirs
Scope: hardening, correctness, tests/CI, docs alignment, and a global OpenCode instructions file.

> Status: planning only. No source changes have been made. Switch to a build agent to execute.

---

## 1. Current behavior (verified)

| Command | Behavior |
|---|---|
| `init <blueprint> <name>` | Reads `<config>/fiatlux/blueprints/<blueprint>.yaml`, rejects multi-component project names, creates `<cwd>/<name>`, writes empty files/dirs, then runs scripts with `cmd /C` (Windows) or `sh -c`. |
| `list` | Sorts `*.yaml` stems from the blueprints dir. |
| `import <name> <path>` | Copies any file to `<config>/fiatlux/blueprints/<name>.yaml`, overwriting silently. |

Global config dir resolves via `dirs::config_dir()`:
`%APPDATA%\fiatlux\blueprints` (Windows), `~/.config/fiatlux/blueprints` (Linux),
`~/Library/Application Support/fiatlux/blueprints` (macOS).

---

## 2. Findings → work items

### P0 — correctness (small, high impact)

1. **Silent empty projects** (`src/blueprint.rs:6-12`)
   `BlueprintDef` uses `#[serde(default)]` without `deny_unknown_fields`, so a typo (`directory:`) parses into an empty struct and `init` creates nothing, exits 0.
   - Add `#[serde(deny_unknown_fields)]`.
   - After parse, error if both `directories` and `scripts` are empty: "blueprint defines no directories or scripts".
2. **Non-deterministic tree** (`src/blueprint.rs:18`)
   `HashMap<String, Vec<FsEntry>>` randomizes creation order and makes file-vs-dir collisions a coin flip.
   - Switch to `BTreeMap` (sorted, deterministic).
   - Detect conflicting duplicate paths and error with a clear message.
3. **Path traversal** (`src/cli.rs:60,138`, `src/blueprint.rs:34,40`)
   `blueprint`, `blueprint_name`, and FsEntry names are joined unvalidated; `../` escapes the config dir / project root.
   - Extract `validate_component(raw, label) -> anyhow::Result<()>`: exactly one `Component::Normal`, non-empty, not `.`/`..`, no `/` or `\`, no Windows-invalid chars (`<>:"|?*`), no trailing dot/space, not a DOS reserved name (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`, `CLOCK$`).
   - Apply to `project_name`, `blueprint`, `blueprint_name`, and every `FsEntry` name.
4. **Partial project on failure** (`src/cli.rs:85-104`)
   A failed file write or script leaves a half-built directory; re-running then fails because the destination exists.
   - On `generate_fs` failure, remove the directory tree created by this run (track it and only delete if we created it).
   - Add `--force`: allow an existing directory; never delete pre-existing content.
   - Add `--keep-on-error` for debugging.
5. **`import` accepts garbage / clobbers** (`src/cli.rs:133-152`)
   - Parse-validate with `parse_source` before copying; reject non-YAML and invalid blueprints.
   - Refuse to overwrite unless `--force`; print a hint.
   - Reject directories and non-regular files.
6. **`list` swallows errors** (`src/cli.rs:112-118`)
   - Replace `filter_map(|e| e.ok())` with explicit error propagation.
   - Accept `.yaml`/`.yml` case-insensitively; keep one canonical extension for writes.
7. **Shallow `--dry-run`** (`src/cli.rs:77-83`)
   - Parse the blueprint, then print: destination, every file, every directory, and the ordered scripts.
   - Exit non-zero if the blueprint fails validation.

### P1 — safety / UX

8. **Script execution gate** (`src/file_handler.rs:34-44`)
   - Print `Running [n/N]: <cmd>` before each script.
   - Prompt once before executing when `scripts` is non-empty and stdin is a TTY; add `--yes` to skip and `--skip-scripts` to bypass entirely.
9. **Error output** (`src/main.rs:9`)
   - Use `{e:#}` instead of `{e:?}` to print the anyhow chain cleanly.
   - Exit code `2` for usage/validation errors, `1` for runtime failures.
10. **Test isolation hook** (`src/file_handler.rs:5-12`)
    - Honor `FIATLUX_CONFIG_DIR` (or `--config-dir`) so integration tests and CI never touch the real user config.
11. **Output path**
    - Add `init --into <dir>` (default: CWD) so scaffolding is not tied to the process directory.
12. **Progress**
    - Print a summary at the end: `N files, M directories, K scripts`; add `--quiet`.

### P2 — features (post-hardening)

13. **File contents / templating.** Files are always empty (`src/blueprint.rs:35`), yet the README advertises boilerplate customization.
    - Support `- src/main.js: "console.log('hi')"` (scalar value = content) and/or a `content` key.
    - Substitution: `{{project_name}}`, `{{blueprint}}`, plus `--set key=value`.
    - Keep bare `- name` as an empty file for backward compatibility.
14. **Premade blueprints are dead weight.** `premade_blueprints/react-express.yaml` is never referenced by the binary.
    - Embed with `include_str!`.
    - Add `fiatlux list --premade` and `fiatlux import react-express --premade`.
    - Optional: seed on first run.
15. **Blueprint metadata.** Allow `description`, `author`, `version`; show them in `list`/`--dry-run`.

### P3 — dependencies & packaging

16. **`serde_yaml` is archived/unmaintained.** Migrate to a maintained drop-in (`serde_yaml_ng` preferred; evaluate `serde_yml`/`saphyr`). Verify `untagged` enum behavior is unchanged.
17. **Cargo metadata** (`Cargo.toml`): add `description`, `license = "MIT"`, `repository`, `readme`, `rust-version = "1.85"`, `keywords`, `categories`.
18. **Two `[[bin]]` targets share `src/main.rs`** (intentional `fiatlux`/`fl` alias). Add a comment; confirm `cargo install` behavior is acceptable, or move to a thin `src/bin/fl.rs` shim.

---

## 3. Documentation alignment

19. **Extension contradiction.** README says `.fl`; code reads/writes `.yaml`.
    - Decision: make `.yaml` canonical (already implemented, standard, tooling-friendly).
    - Update `README.md:24,67`, project tree, and all examples.
    - Optionally accept `.fl` as a read alias and warn.
20. **Config path contradiction.** README shows `~/.fiatlux/`; actual is OS config dir.
    - Document per-OS paths, or explicitly move to `~/.fiatlux` if that was the intent (requires code change + migration note).
21. **DSL docs.** `docs/DSL.md` uses ```` ```fiatlux ```` fences; switch to `yaml`. Add: scripts run from the project root, script failure aborts, path restrictions, `.txt` fallback rule, and a security warning about untrusted blueprints.
22. **README claims.** "Dependency Installation" and boilerplate customization are not implemented as advertised; trim or mark as roadmap. Fix "CLS" typo, empty Roadmap/Contributing/License sections.

---

## 4. Testing & CI

23. **No tests exist**, although `assert_cmd`, `predicates`, and `tempfile` are dev-dependencies.
    - `tests/init.rs`: happy path (files + dirs), extensionless → `.txt`, `.gitignore` preserved, existing dir errors, `--force`, traversal rejected, invalid YAML rejected, unknown field rejected, script failure leaves no partial tree (or keeps with flag).
    - Unit tests in `blueprint.rs` for parse/`generate_fs`/validation; `file_handler.rs` for `run_shell` success/failure via a portable command.
    - Use `FIATLUX_CONFIG_DIR` + `tempfile` for isolation; pass the target dir explicitly so tests never touch the real CWD.
24. **CI** (`.github/workflows/ci.yml`) currently runs only `fmt` and `clippy`.
    - Add a `test` job with a matrix `[ubuntu-latest, windows-latest]` running `cargo test --all-targets`.
    - Add `cargo build --release` (Windows artifact smoke test).
    - Keep `fmt`/`clippy`; `clippy -- -D warnings` is already present.
    - Optional MSRV job pinned to `rust-version`.

**Verification commands**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

---

## 5. Suggested execution order

| Phase | Items | Outcome |
|---|---|---|
| 1 | P0 1-4 | No silent empties, no traversal, deterministic, clean failure |
| 2 | P0 5-7 + P1 8-10 | Safe import, good dry-run, isolated/configurable, clear errors |
| 3 | P3 16-18 | Maintained deps + package metadata |
| 4 | P4 23-24 | Tests + Windows CI green |
| 5 | Docs 19-22 | Docs match binary |
| 6 | P2 13-15 | Templating, premade blueprints, metadata |

Acceptance criteria: `cargo test` passes on Windows and Linux; `init` on a typo'd blueprint exits non-zero; `import ../../x` is rejected; no partial tree after a failed script unless `--keep-on-error`; README/DSL paths and extensions match observed behavior.

---

## 6. Global instructions file

Two distinct files, easy to confuse:

- **Global** (all projects, all sessions): `%USERPROFILE%\.config\opencode\AGENTS.md` — on this machine `C:\Users\d3nzu\.config\opencode\AGENTS.md`. Auto-loaded.
- **Project** (FiatLux only): `C:\Users\d3nzu\Documents\fiatlux\AGENTS.md` — committed to the repo.

The draft lives next to this plan at `Documents\fiatlux\.opencode\AGENTS.md`.

To use it as the **global** file, copy it to the path above:

```bash
mkdir -p "$HOME/.config/opencode"
cp "$HOME/Documents/fiatlux/.opencode/AGENTS.md" "$HOME/.config/opencode/AGENTS.md"
```

This repo keeps it at `Documents\fiatlux\.opencode\AGENTS.md`, which OpenCode loads for this
project. The conventional alternative is the repo root (`Documents\fiatlux\AGENTS.md`); both
work. If you ever use a non-standard filename or path, register it explicitly:

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "instructions": [".opencode/AGENTS.md"]
}
```

### FiatLux-specific additions

The repo already ships baseline instructions at `.opencode/AGENTS.md` (general engineering
rules). Project-specific rules worth adding there or at the repo root:

```md
- Binary crate; edition 2024. Build: `cargo build`. Verify: `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test`.
- Keep the CLI surface documented in `README.md` and `docs/DSL.md` in sync with `src/cli.rs`.
- Blueprint schema lives in `src/blueprint.rs`; every user-supplied name goes through `validate_component`.
- Treat blueprint scripts as untrusted: require confirmation, never run them in `--dry-run`.
- Blueprints are YAML (`.yaml`). Do not reintroduce the `.fl` extension in docs or code.
- Prefer `BTreeMap` over `HashMap` wherever iteration order affects output.
- Add an integration test in `tests/` for every new subcommand or flag.
```

---

## 7. Open decisions (need your call)

1. Canonical blueprint extension: `.yaml` (recommended) vs `.fl` vs accept both.
2. Config location: keep OS config dir (recommended) vs migrate to `~/.fiatlux`.
3. Script confirmation default: prompt on TTY + `--yes` (recommended) vs always run (current) vs always require `--yes`.
4. File-content syntax: inline scalar (`- main.js: "..."`) vs `content:` block vs separate `.tmpl` files.
5. Whether to support templating variables now or defer to a later release.
