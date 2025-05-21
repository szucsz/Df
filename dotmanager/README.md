# DotManager - A Composable Dotfile Manager

DotManager is a simple, composable dotfile manager written in Rust, inspired by the concept of Nix Flakes. It helps you organize your dotfiles into reusable units and apply them with a flexible templating system.

## Features

*   **Composable Units:** Organize your dotfiles into self-contained "units" (e.g., for nvim, zsh, git).
*   **Templating:** Uses the Tera templating engine for dynamic configuration files. Define global and unit-specific variables.
*   **Simple CLI:** Easy-to-use commands to initialize, manage units, and apply configurations.

## Installation

Currently, you need to build DotManager from source.

1.  **Ensure Rust is installed:** If not, visit [rustup.rs](https://rustup.rs/).
2.  **Clone the repository:**
    ```bash
    # Replace with the actual repository URL once available
    # git clone <repository_url>
    # cd dotmanager
    ```
3.  **Build the project:**
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/dotmanager`. You can copy this to a directory in your PATH (e.g., `~/.local/bin`).

## Core Concepts

### `dotfiles.toml` (Global Configuration)

This file resides in the root of your dotfiles repository (e.g., `~/.dotfiles/dotfiles.toml`). It defines:

*   `units_base_dir`: The directory where all your units are stored (defaults to `"units"`).
*   `units`: A list of unit names (directory names within `units_base_dir`) to be applied.
*   `global_variables`: A TOML table of variables accessible to all templates.

**Example `dotfiles.toml`:**

```toml
units_base_dir = "units"

units = [
  "common",
  "nvim",
  "zsh"
]

[global_variables]
username = "your_user"
email = "user@example.com"
```

### `unit.toml` (Unit Configuration)

Each unit has its own `unit.toml` file in its directory (e.g., `units/nvim/unit.toml`). It defines:

*   `name`: An optional name for the unit.
*   `templates_dir`: Directory containing template files for this unit (defaults to `"templates"` relative to the unit's path).
*   `target_dir`: The base directory where files from this unit should be placed (e.g., `"~/.config/nvim"`). Tilde expansion (`~`) is supported.
*   `variables`: A TOML table of variables specific to this unit's templates. These override global variables with the same name.

**Example `units/nvim/unit.toml`:**

```toml
name = "neovim-config"
target_dir = "~/.config/nvim" # Files will be placed here

[variables]
theme = "gruvbox"
enable_lsp = true
```

### Templates

*   Templates are files ending with `.template` (e.g., `init.vim.template`).
*   They use Tera templating syntax (similar to Jinja2/Django).
*   When `dotmanager apply` is run, the `.template` suffix is removed from the target filename (e.g., `init.vim.template` becomes `init.vim`).
*   Variables from `global_variables` (in `dotfiles.toml`) are accessible under the `global_variables` key in templates (e.g., `{{ global_variables.username }}`).
*   Variables from a unit's `variables` (in `unit.toml`) are accessible directly at the root of the template context (e.g., `{{ theme }}`). Unit variables override global variables if there's a name collision at the root context level.

**Example `units/nvim/templates/init.vim.template`:**

```vim
" Neovim configuration for {{ global_variables.username }}
set termguicolors

colorscheme {{ theme }}

{% if enable_lsp %}
  lua require('lspconfig').pylsp.setup{}
{% endif %}
```

## CLI Usage

### `dotmanager init`

Initializes a new dotfiles repository in the current directory.

*   Creates a `dotfiles.toml` file.
*   Creates an example unit: `units/example/unit.toml` and `units/example/templates/example_config.txt.template`.

```bash
dotmanager init
```

### `dotmanager add-unit <unit_name>`

Creates a new, empty unit.

*   Creates a directory `units/<unit_name>`.
*   Creates `units/<unit_name>/unit.toml`.
*   Creates `units/<unit_name>/templates/<unit_name>.conf.template`.

```bash
dotmanager add-unit git
# Remember to add "git" to the `units` list in dotfiles.toml to apply it!
```

### `dotmanager validate`

Validates your `dotfiles.toml` and all referenced `unit.toml` files. Checks for:

*   Existence of specified unit directories and their `unit.toml` files.
*   Existence of `templates_dir` within each unit.

```bash
dotmanager validate
```

### `dotmanager apply`

Applies the dotfile configurations.

*   Reads `dotfiles.toml`.
*   For each unit in the `units` list:
    *   Loads its `unit.toml`.
    *   Discovers all `.template` files in its `templates_dir`.
    *   Renders each template using global and unit-specific variables.
    *   Writes the rendered output to the path constructed from the unit's `target_dir` and the template's relative path (with `.template` removed).
    *   Parent directories for target files are created automatically.

```bash
dotmanager apply
```

## Development

(Placeholder for future development notes, e.g., running tests)

## Contributing

(Placeholder for future contribution guidelines)

## License

This project is licensed under the MIT License. (Or choose Apache-2.0 if preferred - MIT is common for such tools)
