# --- SHARED DOTFILE CFG START ---
# .bashrc

# Function to get detailed git status
get_git_info() {
  local branch=$(git branch --show-current 2>/dev/null)
  [ -z "$branch" ] && return

  local commit=$(git rev-parse --short HEAD 2>/dev/null)
  local tag=$(git describe --tags --exact-match 2>/dev/null)
  local status_info=""
  # Dirty: ✗, Staged: +, Untracked: ?
  if ! git diff --quiet 2>/dev/null; then status_info+=" ✗"; fi
  if ! git diff --cached --quiet 2>/dev/null; then status_info+=" +"; fi
  if [ -n "$(git ls-files --others --exclude-standard 2>/dev/null)" ]; then status_info+=" ?"; fi

  local git_string=" $branch [$commit]"
  [ -n "$tag" ] && git_string+=" ($tag)"

  # Dracula Pink for branch/commit, Yellow for status
  echo -e "\033[38;2;255;121;198m$git_string\033[38;2;241;250;140m$status_info\033[0m"
}

# Dracula Cyan for path, Purple for user, Green for prompt char
# Line 1: Path User Git
# Line 2: $ (or # for root)
PS1='\n\[\033[38;2;139;233;253m\]\w \[\033[38;2;189;147;249m\]\u@\h\[\033[0m\]$(get_git_info)\n\[\033[38;2;80;250;123m\]\$ \[\033[0m\]'

# Aliases
alias ls='ls -G' # -G for colors on macOS
alias ll='ls -lh'
alias la='ls -A'
alias t='tmux'
alias ta='tmux attach'
alias v='nvim'
# --- SHARED DOTFILE CFG END ---
