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
      status_m=" %F{#f1fa8c}✗%f"
    fi
  done <<< "$status_out"

  if [[ -n "$hash" ]]; then
    tag=$(git describe --tags --exact-match 2>/dev/null)
    [[ -n "$tag" ]] && tag=" %F{#ff79c6}#$tag"
  fi

  # Blocky git info with brackets
  echo -n " %F{#6272a4}[%F{#ff79c6}$branch %F{#bd93f9}$hash$tag$status_m%F{#6272a4}]%f"
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
    local time_str="%D{%H:%M:%S}"
    local pipe_color="%F{#6272a4}"

    # Resolve error meaning with brown-red color
    local ret_val=""
    local plain_ret=""
    if [[ $exit_status -ne 0 ]]; then
        local meaning=$(get_error_meaning $exit_status)
        ret_val=" %F{#d26946}[$exit_status: $meaning]%f"
        plain_ret=" [$exit_status: $meaning]"
    fi

    # Strip color codes for length calculation
    local plain_git=$(echo -n "$git_info" | sed -E 's/%[FK]\{[^}]+\}//g; s/%[fk]//g')

    # Calculate exact padding for line 1
    # Account for "╭ " (2 chars) and spacing between git and time (4 spaces)
    local raw_left_info="${(%):- ╭  %~ %n@%m $plain_ret }"
    local left_len=${#raw_left_info}
    local term_width=$COLUMNS
    local padding_len=$((term_width - left_len - ${#plain_git} - 9 - 6)) # -9 for HH:MM:SS, -6 for spacing

    # Split padding for gradient distribution across 3 sections
    local pad_per_step=$((padding_len / 3))
    local pad_remainder=$((padding_len % 3))
    local pad5="" pad6="" pad7=""
    [[ $pad_per_step -gt 0 ]] && printf -v pad5 "%${pad_per_step}s" " "
    [[ $pad_per_step -gt 0 ]] && printf -v pad6 "%${pad_per_step}s" " "
    [[ $((pad_per_step + pad_remainder)) -gt 0 ]] && printf -v pad7 "%$((pad_per_step + pad_remainder))s" " "

    # High-resolution Dracula purple-to-pink gradient background (8 steps)
    local bg1="%K{#4b465f}"  # Step 1: Desaturated purple
    local bg2="%K{#4f465d}"  # Step 2
    local bg3="%K{#53465c}"  # Step 3
    local bg4="%K{#57465b}"  # Step 4
    local bg5="%K{#5b465a}"  # Step 5
    local bg6="%K{#5f465a}"  # Step 6
    local bg7="%K{#62465a}"  # Step 7
    local bg8="%K{#64465a}"  # Step 8: Desaturated pink

    # Different Dracula accent colors for each element
    local fg_path="%F{#bd93f9}"    # Dracula purple - path
    local fg_user="%F{#8be9fd}"    # Dracula cyan - username
    local fg_at="%F{#a0a0a0}"      # Light gray separator - @
    local fg_host="%F{#ff79c6}"    # Dracula pink - hostname
    local fg_time="%F{#b4c8ff}"    # Light blue - timestamp

    # Line 1: gradient background with colorful text
    PROMPT="${pipe_color}╭%f ${bg1}${fg_path}%~%f ${bg2}${fg_user}%n${bg3}${fg_at}@${bg4}${fg_host}%m%f${ret_val}${bg5}${pad5}${bg6}${pad6}${bg7}${pad7}${git_info}    ${bg8}${fg_time}${time_str} %k%f
${pipe_color}╰%f %F{#50fa7b}%(!.#.$)%f "

    RPROMPT=""
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
