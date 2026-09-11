# FiatLux DSL

The **FiatLux DSL** is a lightweight language for defining a project's initial structure and setup. A DSL file is called a **Blueprint**.

## Directories

The `directories` block defines the project's file and directory structure.

Curly braces `{}` represent directories, while entries without braces represent files.

```fiatlux
directories {
    src {
        main.js
        utils {
            helpers.js
        }
    }

    README.md
}
```

This produces:

```text
project/
├── src/
│   ├── main.js
│   └── utils/
│       └── helpers.js
└── README.md
```

Files are created empty by default.

### Extensionless Files

Files without an extension are assumed to be `.txt` files.

```fiatlux
directories {
    notes
    TODO
}
```

Produces:

```text
notes.txt
TODO.txt
```

---

## Scripts

The `scripts` block defines commands to run during project initialization.

```fiatlux
scripts {
    "npm install",
    "npm install react",
    "npm install -D vite"
}
```

Scripts are executed **in the order they are defined**.

> ⚠️ Scripts can execute arbitrary commands on your system. Only use Blueprints you trust.

---

## Comments

Comments begin with `#` and continue until the end of the line.

```fiatlux
# Project structure
directories {
    src {
        main.js # Entry point
    }
}
```

---

## Complete Example

```fiatlux
# Define project structure
directories {
    src {
        components {
            App.jsx
        }

        main.jsx
    }

    public {
        index.html
    }

    README.md
}

# Define setup commands
scripts {
    "npm install",
    "npm install react react-dom",
    "npm install -D vite"
}
```

