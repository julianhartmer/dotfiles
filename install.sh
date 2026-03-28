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

# 4. Optional: Update .bash_profile (ensure it sources .bashrc)
# This is a one-liner, we can just ensure it exists
if ! grep -q ".bashrc" "$HOME/.bash_profile" 2>/dev/null; then
    echo "Ensuring .bashrc is sourced in .bash_profile..."
    echo -e "\nif [ -f ~/.bashrc ]; then . ~/.bashrc; fi" >> "$HOME/.bash_profile"
fi
