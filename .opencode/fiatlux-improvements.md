# FiatLux — Review & Improvement Plan

Repo: local FiatLux checkout (all paths below are repo-relative)
Stack: Rust 2024 edition, clap, serde_yaml, anyhow, dirs
Scope: hardening, correctness, tests/CI, docs alignment, and a global OpenCode instructions file.

> Status: planning only. No source changes have been made. Switch to a build agent to execute.
>
> Line references are pinned to commit `b9cd72c` (source unchanged through `398252f`). Prefer
> function/symbol names when following this plan after further edits.

---

## 0. Non-goals

- No plugin system, remote blueprint registry, or daemon.
- No templating engine beyond simple `{{key}}` substitution.
- No change to the two-binary (`fiatlux`/`fl`) packaging model in this pass.

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

Current validation gaps:

- Project names: `.`, `..`, and path separators are rejected; Windows-invalid characters and
  reserved device names are not, so they fail later as raw OS errors.
- File entries: `name.contains('.')` keeps the name as-is; otherwise `.txt` is appended.
  `.gitignore` is preserved, but `v1.2`/`archive.tar.gz` get no special treatment.
- `import` copies raw bytes; a non-YAML or invalid blueprint is accepted.

---

## 2. Findings → work items

### P0 — correctness (small, high impact)

1. **Silent empty projects** (`src/blueprint.rs:6-12`)
   `BlueprintDef` uses `#[serde(default)]` without `deny_unknown_fields`, so a typo (`directory:`) parses into an empty struct and `init` creates nothing, exits 0.
   - Add `#[serde(deny_unknown_fields)]`.
   - After parse, error if both `directories` and `scripts` are empty: "blueprint defines no directories or scripts".
   - Acceptance: a blueprint with only `directorie:` exits non-zero.
2. **Non-deterministic tree** (`src/blueprint.rs:18`)
   `HashMap<String, Vec<FsEntry>>` randomizes creation order and can hide duplicate/conflicting keys.
   - Switch to `BTreeMap` (sorted, deterministic).
   - Detect conflicting duplicate paths (file vs dir, same path twice) and error with a clear message.
3. **Path traversal** (`src/cli.rs:60,138`, `src/blueprint.rs:34,40`)
   `blueprint`, `blueprint_name`, and `FsEntry` names are joined unvalidated; `../` escapes the config dir / project root.
   - Extract `validate_component(raw, label) -> anyhow::Result<()>`: exactly one `Component::Normal`, non-empty, not `.`/`..`, no `/` or `\`, no Windows-invalid chars (`<>:"|?*`), no trailing dot/space, not a DOS reserved name (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`, `CLOCK$`).
   - Apply to `project_name`, `blueprint`, `blueprint_name`, and every `FsEntry` name.
   - Validate the raw name before the `.txt` fallback; keep leading-dot names such as `.gitignore` valid.
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
   Parsing already happens before the dry-run branch, so YAML syntax errors already exit non-zero. The gap is semantic validation and reporting.
   - Run the same validation as a real init (empty blueprint, conflicting paths, component checks), then print: destination, every file, every directory, and the ordered scripts.
   - Do not create the destination or run scripts.
   - Exit non-zero if semantic validation fails.

### P1 — safety / UX

8. **Script execution gate** (`src/file_handler.rs:34-44`)
   - Print `Running [n/N]: <cmd>` before each script.
   - Prompt once before executing when `scripts` is non-empty and stdin is a TTY (`std::io::IsTerminal`); add `--yes` to skip and `--skip-scripts` to bypass entirely.
9. **Error output** (`src/main.rs:9`)
   - Use `{e:#}` instead of `{e:?}` to print the anyhow chain cleanly.
   - Exit code `2` for usage/validation errors, `1` for runtime failures. Clap already exits `2` for argument errors; validation errors need explicit mapping (e.g. a marker error type or a dedicated `usage_error` helper).
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
    - Preserve the extension rule: a file with content should not gain `.txt` unexpectedly.
14. **Premade blueprints are dead weight.** `premade_blueprints/react-express.yaml` is never referenced by the binary.
    - Embed with `include_str!`.
    - Add `fiatlux list --premade` and `fiatlux import react-express --premade`.
    - Optional: seed on first run.
15. **Blueprint metadata.** Allow `description`, `author`, `version`; show them in `list`/`--dry-run`.
    - Depends on item 1: add the fields to `BlueprintDef` before enabling `deny_unknown_fields`, or `deny_unknown_fields` will reject metadata.

### P3 — dependencies & packaging

16. **`serde_yaml` is archived/unmaintained.** `Cargo.lock` pins `serde_yaml 0.9`.
    - Migrate to a maintained drop-in (`serde_yaml_ng` preferred; evaluate `serde_yml`/`saphyr`).
    - Verify `untagged` enum behavior and duplicate-key handling are unchanged.
    - Treat this as a dependency swap with regression tests for both scalar and nested `FsEntry` forms.
17. **Cargo metadata** (`Cargo.toml`): add `description`, `license = "MIT"`, `repository`, `readme`, `rust-version = "1.85"`, `keywords`, `categories`.
    - `1.85` is the edition-2024 floor; confirm `LICENSE` is MIT and matches `license`.
18. **Two `[[bin]]` targets share `src/main.rs`** (intentional `fiatlux`/`fl` alias). Add a comment; confirm `cargo install` behavior is acceptable (both binaries are installed), or move to a thin `src/bin/fl.rs` shim.

---

## 3. Documentation alignment

19. **Extension contradiction.** README says `.fl`; code reads/writes `.yaml`.
    - Decision: make `.yaml` canonical (already implemented, standard, tooling-friendly).
    - Update `README.md:24,67`, the project tree, and all examples.
    - Optionally accept `.fl` as a read alias and warn.
20. **Config path and tree contradiction.** README (`README.md:65-69`) shows `~/.fiatlux/` with `premade_blueprints/` and `templates/`; actual is the OS config dir with only `blueprints/`.
    - Document per-OS paths, or explicitly move to `~/.fiatlux` if that was the intent (requires code change + migration note).
    - Replace the fabricated tree with the real layout; mention `premade_blueprints/` as a repo resource, not a config subdirectory.
21. **DSL docs are structurally obsolete.** `docs/DSL.md` still documents a brace-based `fiatlux` language (`docs/DSL.md:11,41,61,79,92`), but `parse_source` only accepts YAML. This is a correctness bug in the docs, not just a fence rename.
    - Rewrite all examples as YAML matching `BlueprintDef`/`FsEntry`.
    - Switch fences to `yaml`.
    - Add: scripts run from the project root, script failure aborts, path restrictions, `.txt` fallback rule, duplicate/conflict behavior, and a security warning about untrusted blueprints.
    - Do this early; it currently teaches syntax the binary rejects.
22. **README claims.** "Dependency Installation" (`README.md:16`) is only indirect (via scripts), and boilerplate customization (`README.md:10`) is not implemented as advertised; trim or mark as roadmap. Fix the "CLS" typo (`README.md:17`), align "Presets"/"DSL" terminology with "Blueprints + YAML", and fill or remove the empty Roadmap/Contributing/License sections (License should point at `LICENSE`, MIT).

---

## 4. Testing & CI

23. **No tests exist** (`tests/` is absent), although `assert_cmd`, `predicates`, and `tempfile` are dev-dependencies.
    - `tests/init.rs`: happy path (files + dirs), extensionless → `.txt`, `.gitignore` preserved, `v1.2` preserved, duplicate/conflicting entries rejected, existing dir errors, `--force`, traversal rejected, invalid YAML rejected, unknown field rejected, script failure leaves no partial tree (or keeps with flag), `--into` target, `--dry-run` creates nothing.
    - Unit tests in `blueprint.rs` for parse/`generate_fs`/validation; `file_handler.rs` for `run_shell` success/failure via a portable command.
    - Use `FIATLUX_CONFIG_DIR` + `tempfile` for isolation; pass the target dir explicitly so tests never touch the real CWD.
    - Prefer `#[cfg(test)] mod tests` for unit tests and `tests/` for CLI integration tests.
24. **CI** (`.github/workflows/ci.yml`) currently runs only `fmt` and `clippy` on Ubuntu, for `main`.
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

## 5. Sequencing & execution order

Sequencing constraints:

- Item 1 (`deny_unknown_fields`) before item 15 (metadata), otherwise metadata fields are rejected.
- Items 3 (validation) and 10 (config override) before integration tests.
- Item 4's cleanup and `--force` must land together; `--force` alone can mask partial trees.
- Item 21 (DSL rewrite) should precede or accompany item 19, otherwise docs teach the wrong format.

| Phase | Items | Outcome |
|---|---|---|
| 1 | P0 1-4 | No silent empties, no traversal, deterministic, clean failure |
| 2 | P0 5-7 + P1 8-11 | Safe import, good dry-run, isolated/configurable, clear errors |
| 3 | P3 16-18 | Maintained deps + package metadata |
| 4 | Testing/CI 23-24 | Tests + Windows CI green |
| 5 | Docs 19-22 (21 first) | Docs match binary and actual YAML schema |
| 6 | P2 13-15 | Templating, premade blueprints, metadata |

Acceptance criteria:

- `cargo test` passes on Windows and Linux.
- `init` on a typo'd blueprint exits non-zero.
- `import ../../x` is rejected.
- No partial tree after a failed script unless `--keep-on-error`.
- `--dry-run` on an empty/invalid blueprint exits non-zero and creates nothing.
- README/DSL paths, extension, and schema match observed behavior.

---

## 6. Global instructions file

Two distinct files, easy to confuse:

- **Global** (all projects, all sessions): `~/.config/opencode/AGENTS.md` — auto-loaded.
- **Project** (FiatLux only): `.opencode/AGENTS.md` — committed to the repo and auto-loaded for this
  project (verified).

The project file already exists. To also use it globally:

```bash
mkdir -p "$HOME/.config/opencode"
cp .opencode/AGENTS.md "$HOME/.config/opencode/AGENTS.md"
```

The conventional alternative is the repo root (`AGENTS.md`); `.opencode/AGENTS.md` is valid. If you
ever use a non-standard filename or path, register it explicitly:

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
- Blueprint schema lives in `src/blueprint.rs`; every user-supplied name must go through `validate_component` once it exists.
- Treat blueprint scripts as untrusted: require confirmation, never run them in `--dry-run`.
- Blueprints are YAML (`.yaml`). Do not reintroduce the `.fl` extension in docs or code.
- Prefer `BTreeMap` over `HashMap` wherever iteration order affects output.
- Add an integration test in `tests/` for every new subcommand or flag.
```

> Current state: `validate_component` does not exist yet and `blueprint.rs` still uses `HashMap`;
> the last two bullets are target-state rules, not descriptions of the current code.

---

## 7. Decisions (proposed defaults)

| # | Decision | Proposed default | Status |
|---|---|---|---|
| 1 | Canonical blueprint extension | `.yaml`; optional `.fl` read alias with a warning | Open |
| 2 | Config location | Keep OS config dir; document per-OS; add `FIATLUX_CONFIG_DIR` override | Open |
| 3 | Script confirmation | Prompt on TTY, `--yes` to skip, `--skip-scripts` to bypass; never in `--dry-run` | Open |
| 4 | File-content syntax | Inline scalar (`- main.js: "..."`); bare `- name` stays empty; no separate `content:` key yet | Open |
| 5 | Templating variables | Defer to a later release; document as roadmap | Open |

---

## 8. Rust rule mapping (rust-skills)

| Area | Rules |
|---|---|
| Errors / context | `err-anyhow-app`, `err-context-chain`, `err-no-unwrap-prod`, `err-expect-bugs-only`, `err-lowercase-msg` |
| Validation / types | `api-parse-dont-validate`, `type-newtype-validated`, `type-no-stringly`, `num-cast-try-from` |
| Containers | `coll-map-choice`, `own-slice-over-vec` |
| Serde | `serde-deny-unknown-fields`, `serde-try-from-validate`, `serde-rename-all` |
| Testing | `test-cfg-test-module`, `test-integration-dir`, `test-fixture-raii`, `test-descriptive-names` |
| Structure / lint | `proj-lib-main-split`, `proj-pub-crate-internal`, `lint-deny-correctness`, `lint-rustfmt-check` |
| Docs | `doc-all-public`, `doc-errors-section`, `doc-panics-section` |
