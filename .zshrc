# --- SHARED DOTFILE CFG START ---
# .zshrc

# Load efficient modules
zmodload zsh/datetime

# Basic shell completion
autoload -U compinit && compinit

# Fast Git Status (Optimized, synchronous to prevent multiline redrawing bugs)
get_git_info() {
  # Fast porcelain call
  local status_out
  status_out=$(git status --porcelain=v2 --branch --untracked-files=no 2>/dev/null) || return

  local branch="" hash="" status_m="" tag=""
  while IFS= read -r line; do
    if [[ $line == "# branch.head "* ]]; then
      branch="${line#*head }"
    elif [[ $line == "# branch.oid "* ]]; then
      hash="${line:13:7}"
    elif [[ $line == "1 "* ]] || [[ $line == "2 "* ]]; then
      status_m="%F{#f1fa8c} ✗%f"
    fi
  done <<< "$status_out"

  if [[ -n "$hash" ]]; then
    tag=$(git describe --tags --exact-match 2>/dev/null)
    [[ -n "$tag" ]] && tag=" %F{#ff79c6}#$tag%f"
  fi

  echo -n "%K{#282a36} %F{#ff79c6}$branch%f %F{#bd93f9}[$hash]%f$tag$status_m %k"
}

# Function to resolve exit codes to meanings
get_error_meaning() {
  local code=$1
  case $code in
    1) echo "ERROR" ;;
    2) echo "MISUSE" ;;
    126) echo "CANT EXEC" ;;
    127) echo "NOT FOUND" ;;
    128) echo "INVALID EXIT" ;;
    130) echo "CANCELLED" ;;
    137) echo "KILLED" ;;
    139) echo "SEGFAULT" ;;
    141) echo "SIGPIPE" ;;
    *) echo "FAIL" ;;
  esac
}

set_prompt() {
    local exit_status=$?
    local git_info=$(get_git_info)
    local time_str="%F{#8be9fd}%D{%H:%M:%S}%f"
    local pipe_color="%F{#6272a4}"
    
    # Resolve error meaning
    local ret_val=""
    local plain_ret=""
    if [[ $exit_status -ne 0 ]]; then
        local meaning=$(get_error_meaning $exit_status)
        ret_val=" %F{#ff5555}[$exit_status: $meaning]%f"
        plain_ret=" [$exit_status: $meaning]"
    fi

    # Calculate exact padding for line 1
    # Account for "╭ " (2 chars) and exit code if any
    local raw_left_info="${(%):- ╭  %~ %n@%m $plain_ret }"
    local left_len=${#raw_left_info}
    local term_width=$COLUMNS
    local padding_len=$((term_width - left_len - 9)) # -9 for HH:MM:SS
    local padding=""
    if [ $padding_len -gt 0 ]; then
        printf -v padding '%*s' $padding_len ""
    fi

    local left_info="${pipe_color}╭%f%K{#282a36} %F{#8be9fd}%~ %F{#bd93f9}%n@%m%f$ret_val "

    # Line 1: [Path User [Exit: MEANING] ... Time]
    # Line 2: [$ ... Git]
    PROMPT="${left_info}${padding}${time_str} %k
${pipe_color}╰%f %F{#50fa7b}%(!.#.$)%f "
    
    # RPROMPT ensures Git info stays safely on the right without manual padding logic
    RPROMPT="$git_info"
}

# Ensure prompt variables are updated before drawing
setopt prompt_subst
precmd_functions=(set_prompt)

# Aliases (shared with bash)
# ex: directory=blue, symlink=magenta, executable=red, etc.
export LSCOLORS="Gxfxcxdxbxegedabagacad"
alias ls='ls -GF'
alias ll='ls -lh'
alias la='ls -A'
alias t='tmux'
alias ta='tmux attach'
alias v='nvim'
alias n='nvim'

# Make completions use the same colors
zstyle ':completion:*' list-colors "${(s.:.)LS_COLORS}"
# --- SHARED DOTFILE CFG END ---
