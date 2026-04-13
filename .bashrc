# --- SHARED DOTFILE CFG START ---
# .bashrc

# Faster Git Status (Optimized for 2026)
get_git_info() {
  local status_out
  status_out=$(git status --porcelain=v2 --branch --untracked-files=no 2>/dev/null) || return

  local branch="" tag="" status_info=""
  while IFS= read -r line; do
    if [[ $line == "# branch.head "* ]]; then
      branch="${line#*head }"
    elif [[ $line == "1 "* ]] || [[ $line == "2 "* ]]; then
      status_info=" ✗"
    fi
  done <<< "$status_out"

  if [[ -n "$branch" ]]; then
    tag=$(git describe --tags --exact-match 2>/dev/null)
    # MUST wrap colors in \[ and \] for Bash readline!
    [[ -n "$tag" ]] && tag=" \[\033[38;2;255;121;198m\]#$tag"
  fi

  local git_string=" \[\033[38;2;98;114;164m\][\[\033[38;2;255;121;198m\]$branch$tag\[\033[38;2;241;250;140m\]$status_info\[\033[38;2;98;114;164m\]]"
  echo -e "$git_string"
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
    130) echo "CANCELLED" ;; # SIGINT
    137) echo "KILLED" ;;    # SIGKILL
    139) echo "SEGFAULT" ;;  # SIGSEGV
    141) echo "SIGPIPE" ;;
    *) echo "FAIL" ;;
  esac
}

set_ps1() {
  local exit_status=$?
  local cols=$COLUMNS
  # Use built-in printf for time (Zero subshells)
  local time_str
  printf -v time_str '%(%H:%M:%S)T' -1
  
  local git_info=$(get_git_info)
  # Strip ALL ANSI codes and \[ \] for length calculation
  local plain_git=$(echo -e "$git_info" | sed -E 's/\\\[//g; s/\\\]//g; s/\x1b\[[0-9;]*m//g')
  local pipe_c="\[\033[38;2;98;114;164m\]"
  
  # --- Return Value ---
  local ret_val=""
  local plain_ret=""
  if [[ $exit_status -ne 0 ]]; then
    local meaning=$(get_error_meaning $exit_status)
    ret_val=" \[\033[38;2;210;105;70m\][$exit_status: $meaning]"
    plain_ret=" [$exit_status: $meaning]"
  fi

  # Calculate exact lengths to prevent line-wrapping bugs
  local path_text="$(dirs)"
  local host_short="${HOSTNAME%%.*}"
  # Strip ANSI codes for accurate length calculation
  local plain_path=$(echo -e "$path_text" | sed -E 's/\x1b\[[0-9;]*m//g')
  # Account for "╭ " (2 chars) + user@host first, then path
  local left_text="╭  ${USER}@${host_short} ${plain_path}${plain_ret} "

  # Line1: path + user@host + error + git + spacing + time
  # Account for extra spacing (4 spaces) between git and time
  local line1_pad=$((cols - ${#left_text} - ${#plain_git} - ${#time_str} - 6))
  local pad1=""
  [[ $line1_pad -gt 0 ]] && printf -v pad1 "%${line1_pad}s" " "

  # High-resolution Dracula purple-to-pink gradient background
  # Smooth transition with 8 steps
  local bg1="\[\033[48;2;75;70;95m\]"     # Step 1: Desaturated purple
  local bg2="\[\033[48;2;79;70;93m\]"     # Step 2
  local bg3="\[\033[48;2;83;70;92m\]"     # Step 3
  local bg4="\[\033[48;2;87;70;91m\]"     # Step 4
  local bg5="\[\033[48;2;91;70;90m\]"     # Step 5
  local bg6="\[\033[48;2;95;70;90m\]"     # Step 6
  local bg7="\[\033[48;2;98;70;90m\]"     # Step 7
  local bg8="\[\033[48;2;100;70;90m\]"    # Step 8: Desaturated pink
  local fg_white="\[\033[38;2;248;248;242m\]"  # Dracula foreground - high contrast

  # Different Dracula accent colors for each element (similar variety as git info)
  local fg_path="\[\033[38;2;189;147;249m\]"    # Dracula purple - path
  local fg_user="\[\033[38;2;139;233;253m\]"    # Dracula cyan - username
  local fg_at="\[\033[38;2;160;160;160m\]"       # Light gray separator - @
  local fg_host="\[\033[38;2;255;121;198m\]"    # Dracula pink - hostname
  local fg_time="\[\033[38;2;180;200;255m\]"    # Light blue - timestamp

  # Split padding for gradient distribution across 3 sections
  local pad_per_step=$((line1_pad / 3))
  local pad_remainder=$((line1_pad % 3))
  local pad5="" pad6="" pad7=""
  [[ $pad_per_step -gt 0 ]] && printf -v pad5 "%${pad_per_step}s" " "
  [[ $pad_per_step -gt 0 ]] && printf -v pad6 "%${pad_per_step}s" " "
  [[ $((pad_per_step + pad_remainder)) -gt 0 ]] && printf -v pad7 "%$((pad_per_step + pad_remainder))s" " "

  local line1="${pipe_c}╭ ${bg1}${fg_user}${USER}${bg2}${fg_at}@${bg3}${fg_host}${host_short} ${bg4}${fg_path}${path_text}${ret_val}${bg5}${pad5}${bg6}${pad6}${bg7}${pad7}${git_info}    ${bg8}${fg_time}$time_str \[\033[0m\]"
  local line2="${pipe_c}╰\[\033[0m\] \[\033[38;2;80;250;123m\]\$\[\033[0m\] "
  
  PS1="${line1}\n${line2}"
}
PROMPT_COMMAND=set_ps1

# Aliases
# ex: directory=blue, symlink=magenta, executable=red, etc.
export LSCOLORS="Gxfxcxdxbxegedabagacad"
alias ls='ls -GF' # -G for colors, -F for file type indicators
alias ll='ls -lh'
alias la='ls -A'
alias t='tmux'
alias ta='tmux attach'
alias v='nvim'
alias n='nvim'
# --- SHARED DOTFILE CFG END ---
