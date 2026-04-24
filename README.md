# dotfiles

My personal configuration files for various tools and shells.

## Configuration Files

This repository includes basic configurations for:
- Bash (`.bashrc`, `.bash_profile`)
- Zsh (`.zshrc`) - Features the Dracula theme and high-performance Git status.
- Tmux (`.tmux.conf`) - Features Vim-style pane navigation (`h`, `j`, `k`, `l`), resizing (`H`, `J`, `K`, `L`), splitting (`v`, `x`), breaking/joining/swapping (`!`, `@`, `S`), closing (`q`), mouse resizing, and the Dracula theme.
- Neovim (`nvim/`) - A modern Lua-based config with:
    - **Dracula Theme**: Native Lua implementation.
    - **Plugin Manager**: `lazy.nvim` for fast loading.
    - **LSP**: Intelligent code completion and diagnostics (via Mason).
    - **Syntax**: `nvim-treesitter` for advanced highlighting.
    - **Navigation**: `Telescope` (fuzzy finder) and `Neo-tree` (explorer).
    - **UI**: `Lualine` (status bar) and `Bufferline` (tabs).
- nvdc (`nvdc/`) - A Rust CLI tool that launches devcontainers with your local Neovim and tmux config mounted inside. See [`nvdc/README.md`](nvdc/README.md) for details.

## Aliases

The following aliases are included in both Bash and Zsh:
- `ls`: `ls -G` (colors on macOS)
- `ll`: `ls -lh` (human-readable list)
- `la`: `ls -A` (all files except . and ..)
- `t`: `tmux`
- `ta`: `tmux attach`
- `v`: `nvim`
- `n`: `nvim`

## Installation

To install or update the configurations, run:

```bash
./install.sh
```

### How it works:
- **Tmux**: Creates a symlink from `~/.tmux.conf` to this repository's `.tmux.conf`.
- **Bash/Zsh**: Appends the configuration block (defined between `# --- SHARED DOTFILE CFG START ---` and `# --- SHARED DOTFILE CFG END ---`) to your existing `~/.bashrc` and `~/.zshrc` files. If the block already exists, it will be updated in place, preserving your other settings.
- **Neovim**: Creates a symlink from `~/.config/nvim` to this repository's `nvim/` directory.
- **nvdc**: Installs Rust via rustup (if not present), builds nvdc from source, and places the binary at `~/.bin/nvdc`. The shell configs add `~/.bin` to your PATH.

For macOS, use `./install_mac.sh` instead.

## CI

A GitHub Actions pipeline (`.github/workflows/nvdc.yml`) runs on pushes and PRs that touch `nvdc/`. It builds the project and runs `clippy` to catch warnings.
