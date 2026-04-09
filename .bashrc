# --- SHARED DOTFILE CFG START ---
# .bashrc

# Faster Git Status (Optimized for 2026)
get_git_info() {
  local status_out
  status_out=$(git status --porcelain=v2 --branch --untracked-files=no 2>/dev/null) || return

  local branch="" hash="" tag="" status_info=""
  while IFS= read -r line; do
    if [[ $line == "# branch.head "* ]]; then
      branch="${line#*head }"
    elif [[ $line == "# branch.oid "* ]]; then
      hash="${line:13:7}"
    elif [[ $line == "1 "* ]] || [[ $line == "2 "* ]]; then
      status_info=" ✗"
    fi
  done <<< "$status_out"

  if [[ -n "$hash" ]]; then
    tag=$(git describe --tags --exact-match 2>/dev/null)
    # MUST wrap colors in \[ and \] for Bash readline!
    [[ -n "$tag" ]] && tag=" \[\033[38;2;255;121;198m\]#$tag"
  fi

  local git_string=" \[\033[38;2;98;114;164m\][\[\033[38;2;255;121;198m\]$branch \[\033[38;2;189;147;249m\]$hash$tag\[\033[38;2;241;250;140m\]$status_info\[\033[38;2;98;114;164m\]]"
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
    ret_val=" \[\033[38;2;255;85;85m\][$exit_status: $meaning]"
    plain_ret=" [$exit_status: $meaning]"
  fi

  # Calculate exact lengths to prevent line-wrapping bugs
  local path_text="$(dirs)"
  local host_short="${HOSTNAME%%.*}"
  # Strip ANSI codes for accurate length calculation
  local plain_path=$(echo -e "$path_text" | sed -E 's/\x1b\[[0-9;]*m//g')
  # Account for "╭ " (2 chars) + space after path + space before user
  local left_text="╭  ${plain_path} ${USER}@${host_short}${plain_ret} "

  # Line1: path + user@host + error + git + spacing + time
  # Account for extra spacing (4 spaces) between git and time
  local line1_pad=$((cols - ${#left_text} - ${#plain_git} - ${#time_str} - 6))
  local pad1=""
  [[ $line1_pad -gt 0 ]] && printf -v pad1 "%${line1_pad}s" " "

  local line1="${pipe_c}╭ \[\033[0m\]\[\033[48;2;40;42;54m\]${path_text} ${USER}@${host_short}${ret_val}${pad1}${git_info}    \[\033[38;2;139;233;253m\]$time_str \[\033[0m\]"
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
