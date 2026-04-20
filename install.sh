#!/bin/bash

# Get the directory where this script is located
DOTFILES_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
HEADER="# --- SHARED DOTFILE CFG START ---"
FOOTER="# --- SHARED DOTFILE CFG END ---"

# Function to install/update a configuration block in a target file
install_config_block() {
    local source_file="$1"
    local target_file="$2"
    local label="$3"

    echo "Updating $label configuration in $target_file..."

    if [ ! -f "$source_file" ]; then
        echo "Error: Source file $source_file not found."
        return 1
    fi

    # Create target file if it doesn't exist
    touch "$target_file"

    if grep -qF "$HEADER" "$target_file"; then
        echo "Found existing dotfiles block. Updating..."
        # Replace content between markers
        # We use a temporary file to safely replace
        local tmp_file=$(mktemp)
        
        # Write everything before the header
        sed -n "1,/$HEADER/p" "$target_file" | head -n -1 > "$tmp_file"
        # Append the new source content
        cat "$source_file" >> "$tmp_file"
        # Append everything after the footer
        sed -n "/$FOOTER/,\$p" "$target_file" | tail -n +2 >> "$tmp_file"
        
        mv "$tmp_file" "$target_file"
    else
        echo "No dotfiles block found. Appending to $target_file..."
        echo "" >> "$target_file"
        cat "$source_file" >> "$target_file"
    fi

    echo "Successfully updated $target_file"
}

# 0. Install system dependencies
echo "Installing system dependencies..."
if command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y kitty gnome-shell-extension-blur-my-shell
elif command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update && sudo apt-get install -y kitty gnome-shell-extension-blur-my-shell
else
    echo "Warning: No supported package manager found (dnf/apt). Please install kitty and extensions manually."
fi

# 1. Install tmux (standard symlink approach)
echo "Installing tmux configuration..."
TMUX_SOURCE="$DOTFILES_DIR/.tmux.conf"
TMUX_TARGET="$HOME/.tmux.conf"

if [ -L "$TMUX_TARGET" ]; then
    rm "$TMUX_TARGET"
elif [ -f "$TMUX_TARGET" ]; then
    mv "$TMUX_TARGET" "$TMUX_TARGET.bak"
fi
ln -s "$TMUX_SOURCE" "$TMUX_TARGET"
echo "Successfully linked $TMUX_TARGET -> $TMUX_SOURCE"

# 2. Update .bashrc
install_config_block "$DOTFILES_DIR/.bashrc" "$HOME/.bashrc" "Bash"

# 3. Update .zshrc
install_config_block "$DOTFILES_DIR/.zshrc" "$HOME/.zshrc" "Zsh"

# 4. Install Neovim configuration
echo "Installing Neovim configuration..."
NVIM_SOURCE="$DOTFILES_DIR/nvim"
NVIM_TARGET="$HOME/.config/nvim"

mkdir -p "$HOME/.config"

# Overwrite: Remove existing file or directory before linking
if [ -e "$NVIM_TARGET" ] || [ -L "$NVIM_TARGET" ]; then
    echo "Removing existing nvim configuration..."
    rm -rf "$NVIM_TARGET"
fi

ln -s "$NVIM_SOURCE" "$NVIM_TARGET"
echo "Successfully linked $NVIM_TARGET -> $NVIM_SOURCE"

# 5. Install WezTerm configuration
echo "Installing WezTerm configuration..."
WEZTERM_SOURCE="$DOTFILES_DIR/wezterm"
WEZTERM_TARGET="$HOME/.config/wezterm"

mkdir -p "$HOME/.config"

# Overwrite: Remove existing file or directory before linking
if [ -e "$WEZTERM_TARGET" ] || [ -L "$WEZTERM_TARGET" ]; then
    echo "Removing existing wezterm configuration..."
    rm -rf "$WEZTERM_TARGET"
fi

ln -s "$WEZTERM_SOURCE" "$WEZTERM_TARGET"
echo "Successfully linked $WEZTERM_TARGET -> $WEZTERM_SOURCE"

# 6. Install Kitty configuration
echo "Installing Kitty configuration..."
KITTY_SOURCE="$DOTFILES_DIR/kitty"
KITTY_TARGET="$HOME/.config/kitty"

mkdir -p "$HOME/.config"

# Overwrite: Remove existing file or directory before linking
if [ -e "$KITTY_TARGET" ] || [ -L "$KITTY_TARGET" ]; then
    echo "Removing existing kitty configuration..."
    rm -rf "$KITTY_TARGET"
fi

ln -s "$KITTY_SOURCE" "$KITTY_TARGET"
echo "Successfully linked $KITTY_TARGET -> $KITTY_SOURCE"

# 7. Optional: Update .bash_profile (ensure it sources .bashrc)
# This is a one-liner, we can just ensure it exists
if ! grep -q ".bashrc" "$HOME/.bash_profile" 2>/dev/null; then
    echo "Ensuring .bashrc is sourced in .bash_profile..."
    echo -e "\nif [ -f ~/.bashrc ]; then . ~/.bashrc; fi" >> "$HOME/.bash_profile"
fi

# 8. Set window button layout to the left (for Kitty/GNOME/Wayland)
echo "Setting window button layout to the left..."
if command -v gsettings >/dev/null 2>&1; then
    # 'close,maximize,minimize:' puts them on the left
    gsettings set org.gnome.desktop.wm.preferences button-layout 'close,maximize,minimize:'
    echo "Successfully set window button layout to the left via gsettings."
else
    echo "Warning: gsettings not found. Skipping window button layout configuration."
fi

# 9. Load GNOME extension settings
echo "Loading GNOME extension settings..."
DCONF_FILE="$DOTFILES_DIR/gnome_extensions_settings.dconf"
if [ -f "$DCONF_FILE" ] && command -v dconf >/dev/null 2>&1; then
    dconf load /org/gnome/shell/extensions/ < "$DCONF_FILE"
    echo "Successfully loaded GNOME extension settings."
    
    # Enable extensions
    echo "Enabling GNOME extensions..."
    gnome-extensions enable blur-my-shell@aunetx 2>/dev/null
    gnome-extensions enable background-logo@fedorahosted.org 2>/dev/null
else
    echo "Warning: dconf file not found or dconf command missing. Skipping extension settings."
fi
