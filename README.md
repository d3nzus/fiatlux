<img width="100%" height="100%" alt="image" src="docs\logo.png" />


<p align="center">
  <i>"Let there be light."</i>
</p>

**FiatLux** is a CLI for automatically setting up custom scaffolding for any project. It lets you define your preferred project setup once, then reproduce it whenever you start something new.

For developers working with languages or frameworks that lack a bootstrapper or who simply prefer not to use the default. FiatLux provides a way to define their own project setup. Customize your directory structure, boilerplate, toolchain, and general-purpose dependencies, then initialize them all with a single command.

## Features

* **Project initialization** — Quickly create a new project from your setup
* **User-defined Presets** — Define different setups for different projects with the ***FiatLux DSL***
* **Dependency Installation** — Automated dependency installation
* **Command-Line Scripts** — Automated running of setup CLS
---

## Custom DSL

FiatLux comes with its own lightweight domain-specific language (DSL) for defining how a project should be initialized.

Instead of manually creating directories, files, and running setup commands every time you start a project, you can describe your desired project structure and setup in a **Blueprint** denoted by the `.fl` file extension . FiatLux's parser interprets the Blueprint when initializing a project.

You can read more about the DSL [here](docs/DSL.md). 

## Installation

### Windows

1. Go to the [Releases page](https://github.com/d3nzus/fiatlux/releases) and download the latest `fiatlux.exe` and `fl.exe`.
2. Extract the zip to a permanent folder, e.g. `C:\Users\<you>\bin\`.
3. Add that folder to your PATH:
   - Press <kbd>Windows Key</kbd>, search **"Environment Variables"**, open **"Edit the system environment variables"**.
   - Click **Environment Variables** → under **User variables**, select **Path** → **Edit** → **New**.
   - Paste the folder path (e.g. `C:\Users\<you>\bin`) → **OK** on all dialogs.
4. Open a **new** terminal window and verify:
    ```bash
    fiatlux --version
    ```

### macOS / Linux 
TBA
### Requirements

## How to Use:

Create a project:

```bash
fiatlux init <template> <project-name>
```

## Project Structure

```text
~/.fiatlux/
├── premade_blueprints
│   └── react-express.fl
└── templates/
```


## 🏗️ Roadmap

* [ ] Project initialization
---

## 🤝 Contributing


---

## 📄 License
---



