#!/bin/bash

# Get the directory where this script is located
DOTFILES_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo "🚀 Starting macOS Dotfiles Installation..."

# 1. Install Homebrew if not present
if ! command -v brew >/dev/null 2>&1; then
    echo "🍺 Homebrew not found. Installing..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Initialize brew for the current session
    if [[ -f /opt/homebrew/bin/brew ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi
else
    echo "✅ Homebrew already installed."
fi

# 2. Install dependencies
echo "📦 Installing dependencies via Homebrew..."
brew install kitty tmux neovim coreutils font-symbols-only-nerd-font

# 3. Create necessary directories
mkdir -p "$HOME/.config"

# 4. Symlink configurations
echo "🔗 Creating symlinks..."

# Tmux
ln -sf "$DOTFILES_DIR/.tmux.conf" "$HOME/.tmux.conf"

# Neovim
ln -sfn "$DOTFILES_DIR/nvim" "$HOME/.config/nvim"

# Kitty
ln -sfn "$DOTFILES_DIR/kitty" "$HOME/.config/kitty"

# Zsh - Link the mac-specific one as the primary .zshrc
# We also link the shared one so .zshrc_mac can source it
ln -sf "$DOTFILES_DIR/.zshrc_mac" "$HOME/.zshrc"
ln -sf "$DOTFILES_DIR/.zshrc" "$HOME/.zshrc_shared"

echo "✨ Installation complete! Please restart your terminal or run 'source ~/.zshrc'"
