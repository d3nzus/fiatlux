<p align="center">
  <img width="100%" alt="FiatLux" src="docs/logo.png" />
</p>

<h1 align="center">FiatLux</h1>

<p align="center">
  <i>"Let there be light."</i>
</p>

<p align="center">
  Scaffold any project from a reusable YAML blueprint.
</p>

<p align="center">
  <a href="https://github.com/d3nzus/fiatlux/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/d3nzus/fiatlux/actions/workflows/ci.yml/badge.svg" /></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg" /></a>
</p>

**FiatLux** is a command-line tool for reproducing your preferred project setup on demand. Define a directory tree and setup commands once in a **blueprint**, then create the same starting point for every new project with a single command.

It is aimed at languages and frameworks that have no bootstrapper, or whose defaults you would rather replace.

## Features

- **Reusable blueprints** — describe nested directories and files in a small YAML schema.
- **One-command scaffolding** — `fiatlux init <blueprint> <project-name>` builds the tree for you.
- **Setup scripts** — run commands such as `npm install` or `cargo init` in order, from the project root.
- **Blueprint management** — import blueprints from files and list what is available.
- **Dry runs** — preview where a project would be created without touching the filesystem.
- **Cross-platform** — Windows, macOS, and Linux, with a short `fl` alias for `fiatlux`.

> [!WARNING]
> Blueprints can run arbitrary commands on your machine. Only initialize projects from blueprints you trust, and review the `scripts` section before running one. `--dry-run` never executes scripts.

## Quick start

FiatLux reads blueprints from a per-user config directory. Create a file named `hello.yaml` anywhere:

```yaml
directories:
  - src:
      - main.js
  - public:
      - index.html
  - README.md
  - .gitignore
scripts:
  - npm init -y
  - npm install express
```

Import it, check that it is listed, preview, then scaffold:

```bash
fiatlux import hello hello.yaml
fiatlux list
fiatlux init hello my-app --dry-run
fiatlux init hello my-app
```

From the parent directory of `my-app`:

```text
my-app/
├── src/
│   └── main.js
├── public/
│   └── index.html
├── README.md
└── .gitignore
```

Files are created empty; fill them in or let your scripts do it. The scripts run from inside `my-app`, so `npm init -y` applies to the new project.

## Installation

### Windows (prebuilt)

1. Download the latest `fiatlux.exe` and `fl.exe` from the [Releases page](https://github.com/d3nzus/fiatlux/releases).
2. Extract them to a permanent folder, for example `C:\Users\<you>\bin\`.
3. Add that folder to your `PATH`:
   - Press <kbd>Windows</kbd>, search for **"Environment Variables"**, and open **"Edit the system environment variables"**.
   - Click **Environment Variables**, then under **User variables** select **Path** and click **Edit**.
   - Choose **New**, paste the folder path, and confirm with **OK**.
4. Open a new terminal and verify:

   ```bash
   fiatlux --version
   fl --version
   ```

### macOS / Linux (from source)

FiatLux is a Rust crate. With a Rust toolchain installed:

```bash
git clone https://github.com/d3nzus/fiatlux
cd fiatlux
cargo install --path .
```

This installs both `fiatlux` and `fl` into `~/.cargo/bin`. Make sure that directory is on your `PATH`.

### Requirements

- **Prebuilt binaries:** no runtime dependencies.
- **Building from source:** Rust 1.85 or newer (the crate uses the 2024 edition).

## Usage

```text
fiatlux <command> [arguments]
fl      <command> [arguments]
```

| Command | Description |
|---|---|
| `fiatlux init <blueprint> <project-name>` | Create `<project-name>` in the current directory from an imported blueprint. |
| `fiatlux init <blueprint> <project-name> --dry-run` | Print the destination that would be used, without creating files or running scripts. |
| `fiatlux list` | List the blueprint names available locally. |
| `fiatlux import <name> <path>` | Copy a blueprint file into the local blueprint directory as `<name>.yaml`. |

Notes:

- `<project-name>` must be a single folder name; path separators are rejected.
- `init` writes into the current working directory. Run it from the parent folder of the project you want to create.
- `import` copies the source file as-is and currently overwrites an existing blueprint with the same name without prompting.
- `list` shows blueprints with a lowercase `.yaml` extension.

## Blueprint format

A blueprint is a YAML file with two optional top-level keys: `directories` and `scripts`.

```yaml
directories:
  - src:
      - components:
          - Button.jsx
      - main.jsx
  - public:
      - index.html
  - README.md
  - .gitignore
scripts:
  - npm init -y
  - npm install react react-dom
```

### `directories`

`directories` is a nested list. A bare string is a file; a mapping is a directory whose value is another list of entries.

```yaml
directories:
  - src:
      - main.js       # src/main.js
      - utils:
          - helpers.js # src/utils/helpers.js
  - TODO              # TODO.txt (see below)
  - .gitignore        # .gitignore
```

File names containing a `.` are kept exactly as written, including dotfiles and names such as `v1.2` or `archive.tar.gz`. A file name without a `.` gets a `.txt` extension (`TODO` becomes `TODO.txt`). Files are always created empty.

### `scripts`

`scripts` is an ordered list of shell commands. They run from the generated project root using `cmd /C` on Windows and `sh -c` elsewhere. Execution stops at the first command that exits non-zero, and FiatLux reports which script failed.

```yaml
scripts:
  - npm init -y
  - npm install express
```

Unknown top-level keys are currently ignored, so typos such as `directorie:` will not be reported. Double-check your blueprint before running it.

> [!NOTE]
> `docs/DSL.md` is being rewritten to describe this YAML format; until then, the schema above is authoritative.

## Where blueprints live

FiatLux resolves the OS config directory and stores blueprints in `fiatlux/blueprints` inside it. The directory is created automatically on first use.

| OS | Path |
|---|---|
| Windows | `%APPDATA%\fiatlux\blueprints` |
| Linux | `${XDG_CONFIG_HOME:-~/.config}/fiatlux/blueprints` |
| macOS | `~/Library/Application Support/fiatlux/blueprints` |

FiatLux does not use a `~/.fiatlux` directory and does not install blueprints automatically.

### Premade blueprints

The repository ships an example at [`premade_blueprints/react-express.yaml`](premade_blueprints/react-express.yaml). It is not read by the binary; import it manually from a clone:

```bash
fiatlux import react-express premade_blueprints/react-express.yaml
```

## Repository layout

```text
fiatlux/
├── .github/workflows/ci.yml   # formatting and lint checks
├── docs/
│   ├── DSL.md                 # blueprint documentation (being updated)
│   └── logo.png
├── premade_blueprints/
│   └── react-express.yaml     # example blueprint, imported manually
├── src/
│   ├── blueprint.rs           # schema parsing and file-tree generation
│   ├── cli.rs                 # command definitions and dispatch
│   ├── file_handler.rs        # config directory and script execution
│   └── main.rs
├── Cargo.toml
├── CONTRIBUTING.md
├── LICENSE
└── README.md
```

## Development

```bash
cargo build
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

CI currently runs the formatting and lint checks on pushes and pull requests to `main`. See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow.

## Roadmap

Planned improvements, in rough priority order:

- [ ] Validate project names, file entries, and blueprint files before writing anything to disk.
- [ ] Roll back partially created projects when a write or script fails; add `--force`, `--into`, and `--keep-on-error`.
- [ ] Require confirmation before running blueprint scripts, with `--yes` and `--skip-scripts` escape hatches.
- [ ] Validate blueprints on `import` and refuse to overwrite existing ones without `--force`.
- [ ] Support file contents and `{{project_name}}`-style substitution.
- [ ] Blueprint metadata such as `description`, `author`, and `version`.
- [ ] First-class support for the bundled premade blueprints.
- [ ] Integration tests and a Windows test job in CI.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, branch naming, and pull request guidelines.

## License

Released under the [MIT License](LICENSE). Copyright (c) 2026 Renz Frederick Banas.
