# --- SHARED DOTFILE CFG START ---
# .zshrc

# Basic shell completion
autoload -U compinit && compinit

# Set prompt for Zsh
# Dracula Colors:
# Cyan:   #8be9fd (Path)
# Purple: #bd93f9 (User)
# Pink:   #ff79c6 (Git info)
# Yellow: #f1fa8c (Status)
# Green:  #50fa7b (Prompt Char)
setopt prompt_subst
autoload -Uz vcs_info

# Function to get tag if it exists on current commit
zsh_git_tag() {
    local tag=$(git describe --tags --exact-match 2>/dev/null)
    [ -n "$tag" ] && echo -n " (%F{#ff79c6}$tag%f)"
}

zstyle ':vcs_info:*' enable git
zstyle ':vcs_info:*' check-for-changes true
zstyle ':vcs_info:*' unstagedstr '%F{#f1fa8c} ✗%f'
zstyle ':vcs_info:*' stagedstr '%F{#f1fa8c} +%f'
zstyle ':vcs_info:*' max-exports 2
# %b: branch, %i: 7-char hash
zstyle ':vcs_info:git:*' formats ' %F{#ff79c6}%b [%i]%f%u%c'
zstyle ':vcs_info:git:*' actionformats ' %F{#ff79c6}%b|%a [%i]%f%u%c'

precmd() { 
    vcs_info 
}

# Line 1: Path User + Git Info (Branch [Hash] (Tag) Status)
# Line 2: $ (switches to # for root)
PROMPT='
%F{#8be9fd}%~ %F{#bd93f9}%n@%m%f${vcs_info_msg_0_}$(zsh_git_tag)
%F{#50fa7b}%(#.#.$)%f '

# Aliases (shared with bash)
alias ls='ls -G'
alias ll='ls -lh'
alias la='ls -A'
alias t='tmux'
alias ta='tmux attach'
alias v='nvim'
# --- SHARED DOTFILE CFG END ---
