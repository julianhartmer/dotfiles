# nvdc

A CLI tool that launches devcontainers with your local Neovim and tmux configuration mounted inside. Stop maintaining editor configs per-container — just bring your own.

## Features

- **Neovim-ready containers** — Mounts your `~/.config/nvim` and `~/.local/share/nvim` into the container so plugins, keybindings, and LSP configs just work.
- **Tmux integration** — Mounts your `~/.tmux.conf` and launches a tmux session with nvim, giving you full terminal multiplexing inside the container.
- **Auto-install** — Automatically installs Neovim (v0.12.2) and tmux inside the container if not present. No Dockerfile changes needed.
- **Devcontainer discovery** — Detects `.devcontainer/` subfolders and prompts you to choose one when multiple configurations exist.
- **Persistent preferences** — Remembers your last-used devcontainer config per project in a `.nvdc/` directory so you aren't prompted every time.
- **Docker-native** — Builds and runs containers using the Docker CLI directly, respecting `devcontainer.json` build args, Dockerfiles, and features.
- **Variable substitution** — Expands `${localWorkspaceFolder}`, `${localEnv:VAR}`, and other devcontainer variables.

## Installation

### Via dotfiles (recommended)

```sh
cd ~/gits/dotfiles
./install.sh
```

This installs Rust (if needed), builds nvdc, and places the binary in `~/.bin/`.

### From source

```sh
cargo build --release
cp target/release/nvdc ~/.bin/
```

## Usage

### Basic

Run `nvdc` inside any project that has a `.devcontainer/` directory:

```sh
cd ~/projects/my-app
nvdc
```

If there's a single devcontainer config, it builds/starts the container and opens a tmux session with nvim inside it. If there are multiple configs (subfolders), you'll be prompted:

```
Found multiple devcontainer configs:
  [1] vx7-app  — VxWorks7 - App build (RTP/DKM/SO) ENV
  [2] vx7-build — VxWorks7 - Build ENV
  [3] vx7-test — VxWorks7 - Test ENV
Select config [1-3] (last used: vx7-app):
```

Your selection is saved so next time it launches immediately.

### Commands

```
nvdc                  # Launch container (interactive select if needed)
nvdc up               # Start container without attaching
nvdc attach           # Attach to a running container
nvdc down             # Stop and remove the container
nvdc config           # Interactively change the default devcontainer config
nvdc config --reset   # Reset saved preferences for this project
nvdc list             # List available devcontainer configs
```

### Flags

```
-c, --config <name>     Use a specific devcontainer config by folder name
-n, --nvim-config <path> Override neovim config path (default: ~/.config/nvim)
    --no-nvim-data       Skip mounting ~/.local/share/nvim
    --rebuild            Force rebuild the container image
    --dry-run            Print the docker commands without executing
-v, --verbose            Verbose output
```

### Examples

```sh
# Use a specific config directly
nvdc -c vx7-app

# Force rebuild and launch
nvdc --rebuild

# See what docker commands would run
nvdc --dry-run

# Custom nvim config location
nvdc --nvim-config ~/dotfiles/nvim
```

## How It Works

1. **Discover** — Scans the current directory for `.devcontainer/devcontainer.json` or `.devcontainer/*/devcontainer.json`.
2. **Select** — If multiple configs exist, prompts the user (or uses the saved preference).
3. **Build** — Parses the selected `devcontainer.json`, constructs a `docker build` command with the specified Dockerfile, build args, and secrets.
4. **Run** — Starts the container with:
   - The project directory mounted as the workspace.
   - `~/.config/nvim` mounted read-only.
   - `~/.local/share/nvim` mounted for plugin data.
   - `~/.tmux.conf` mounted read-only.
   - Any `runArgs` from the devcontainer config (e.g., `--privileged`, `--network=host`).
   - Neovim v0.12.2 and tmux auto-installed if not already present.
5. **Attach** — Execs into the container, launching a tmux session with nvim at the workspace root.

## Preferences

Preferences are stored in `.nvdc/` at the project root:

```
.nvdc/
├── config.toml        # Saved preferences
└── state.toml         # Runtime state (running container ID, etc.)
```

Add `.nvdc/` to your global gitignore or the project's `.gitignore`.

## devcontainer.json Support

`nvdc` reads and respects the following fields from `devcontainer.json`:

| Field | Supported |
|---|---|
| `build.dockerfile` | ✅ |
| `build.context` | ✅ |
| `build.args` | ✅ |
| `build.options` | ✅ |
| `image` | ✅ |
| `remoteUser` | ✅ |
| `containerUser` | ✅ |
| `runArgs` | ✅ |
| `initializeCommand` | ✅ |
| `onCreateCommand` | ✅ |
| `features` | ⚠️ neovim+tmux auto-installed |
| `mounts` | ✅ |
| `${localEnv:VAR}` | ✅ |
| `${localWorkspaceFolder}` | ✅ |
| `forwardPorts` | ❌ planned |
| `customizations` | ❌ ignored (VS Code specific) |

## Project Structure

```
nvdc/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entrypoint and arg parsing
│   ├── config.rs         # Preferences and config management
│   ├── devcontainer.rs   # devcontainer.json parsing and discovery
│   ├── docker.rs         # Docker build/run/exec commands
│   ├── nvim.rs           # Neovim mount path resolution
│   └── prompt.rs         # Interactive selection UI
├── tests/
│   └── integration/
└── README.md
```

## Dependencies

- **Runtime**: Docker (or Podman with Docker CLI compatibility)
- **Build**: Rust 1.75+

## License

MIT
