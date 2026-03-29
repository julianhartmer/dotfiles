# 🧛‍♂️ Terminal Workflow Cheat Sheet

Your customized, unified workflow for Tmux and Neovim.

## 🚀 General & Shell
| Command | Alias | Description |
| :--- | :--- | :--- |
| `ls -GF` | `ls` | Colored list (Dirs: Blue, Exec: Red) |
| `ls -lh` | `ll` | Detailed human-readable list |
| `tmux` | `t` | Start new Tmux session |
| `tmux attach` | `ta` | Reattach to last session |
| `nvim` | `v`, `n` | Open Neovim |

### 🌿 Git Prompt Status
| Symbol | Meaning |
| :--- | :--- |
| `[a1b2c3d]` | Current Commit Hash (Purple) |
| `#v1.0` | Current Tag (Pink) |
| `✗` | Unstaged changes |
| `+` | Staged changes |
| `[127: NOT FOUND]` | Last command failed (Exit code + Meaning) |

---

## 🖥️ Tmux (Prefix: `Ctrl + b`)
### Panes & Layouts (Unified!)
| Key | Action |
| :--- | :--- |
| `v` | Split Vertically (Side-by-side) |
| `x` | Split Horizontally (Top-to-bottom) |
| `q` | Close Current Pane |
| `h` / `j` / `k` / `l` | Move Focus Left / Down / Up / Right |
| `H` / `J` / `K` / `L` | Resize Pane (Left/Down/Up/Right) |
| `z` | Zoom (Toggle full screen) |
| `!` | Break pane into a **New Window** |
| `@` | Join window into **Current Pane** (Interactive) |
| `S` | Swap current pane with previous |

### Mouse & Clipboard
- **Resize**: Click and drag pane borders.
- **Select**: Drag mouse to select text (Auto-copies to system clipboard).
- **Native Select**: Hold **`Option`** while dragging to bypass Tmux.

---

## 📝 Neovim (Leader: `Space`)
### Window & Buffer Management
| Shortcut | Action |
| :--- | :--- |
| `<Space> + v` | Split Vertically |
| `<Space> + x` | Split Horizontally |
| `<Space> + q` | Close Current Split (Window) |
| `<Space> + c` | **Close Current File** (Delete Buffer) |
| `<Space> + l` / `h` | Next / Previous file in top bar |
| `<Space> + e` | Toggle File Explorer (Neo-tree) |
| `<Space> + w` | Save File |

### 🔍 Telescope (Fuzzy Finder)
| Shortcut | Action |
| :--- | :--- |
| `<Space> + ff` | Find Files |
| `<Space> + fb` | Find Open Buffers (Switch files) |
| `<Space> + fs` | Search Text (Grep) |
| `<Space> + fr` | Recent Files |
| **Inside View:** | `Ctrl + v` (v-split), `Ctrl + x` (x-split), `Ctrl + t` (tab) |

### 💻 Coding & LSP
| Shortcut | Action |
| :--- | :--- |
| `gd` | Go to Definition |
| `K` | Show Documentation |
| `gR` | Find References |
| `<Space> + ca` | Code Actions / Fixes |
| `<Space> + rn` | Rename Symbol |
| **Autocomplete:** | `Ctrl + j` / `k` (Navigate), `Enter` (Confirm) |
