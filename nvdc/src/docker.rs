use anyhow::{Context, Result};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::devcontainer::DevcontainerConfig;
use crate::nvim::NvimMountOptions;
use crate::RunOptions;

/// Generate a deterministic container name from the project dir and config name.
pub fn container_name(project_dir: &Path, config_name: &str) -> String {
    let project = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    format!("nvdc-{}-{}", sanitize(project), sanitize(config_name))
}

/// Generate a deterministic volume name for neovim data.
pub fn volume_name(project_dir: &Path) -> String {
    let project = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    format!("nvdc-nvim-data-{}", sanitize(project))
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c.to_ascii_lowercase() } else { '-' })
        .collect()
}

/// Check if a container is currently running.
pub fn is_running(name: &str, dry_run: bool) -> Result<bool> {
    if dry_run {
        return Ok(false);
    }

    let output = Command::new("docker")
        .args(["inspect", "-f", "{{.State.Running}}", name])
        .output()
        .context("Failed to run docker inspect")?;

    if !output.status.success() {
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim() == "true")
}

/// Start the container using the devcontainer CLI.
pub fn up(dc: &DevcontainerConfig, nvim_opts: &NvimMountOptions, opts: &RunOptions) -> Result<String> {
    eprintln!("Launching devcontainer using official CLI...");

    // 1. Create temporary override file
    let mut mounts = Vec::new();
    let user = dc.effective_user();
    
    // Neovim config
    mounts.push(json!({
        "source": nvim_opts.config_path().to_string_lossy(),
        "target": format!("/home/{}/.config/nvim", user),
        "type": "bind",
        "readonly": true
    }));

    // Neovim data (volume)
    if nvim_opts.mount_data {
        mounts.push(json!({
            "source": volume_name(&opts.project_dir),
            "target": format!("/home/{}/.local/share/nvim", user),
            "type": "volume"
        }));
    }

    // Tmux config
    if let Some(home) = dirs::home_dir() {
        let tmux_conf = home.join(".tmux.conf");
        if tmux_conf.exists() {
            mounts.push(json!({
                "source": tmux_conf.to_string_lossy(),
                "target": format!("/home/{}/.tmux.conf", user),
                "type": "bind",
                "readonly": true
            }));
        }

        // --- AI Tools Detection & Mounting ---
        let ai_configs = vec![
            (".config/gh", ".config/gh"),                   // GitHub CLI (Copilot)
            (".config/github-copilot", ".config/github-copilot"), // Copilot standalone
            (".config/claude-code", ".config/claude-code"), // Claude Code
            (".config/anthropic", ".config/anthropic"),     // Anthropic CLI
            (".gemini", ".gemini"),                         // Gemini
            (".config/google-cloud", ".config/google-cloud"), // GCloud (Gemini)
            (".config/supermaven", ".config/supermaven"),   // Supermaven
            (".config/openai", ".config/openai"),           // OpenAI
        ];

        for (host_rel, container_rel) in ai_configs {
            let host_path = home.join(host_rel);
            if host_path.exists() {
                if opts.verbose {
                    eprintln!("Detected AI config at {}, mounting...", host_path.display());
                }
                mounts.push(json!({
                    "source": host_path.to_string_lossy(),
                    "target": format!("/home/{}/{}", user, container_rel),
                    "type": "bind"
                }));
            }
        }
    }

    let override_json = json!({
        "mounts": mounts
    });

    let temp_dir = std::env::temp_dir();
    let override_path = temp_dir.join(format!("nvdc-override-{}.json", dc.name));
    std::fs::write(&override_path, serde_json::to_string_pretty(&override_json)?)
        .context("Failed to write temporary override file")?;

    // 2. Call devcontainer up
    let mut cmd_args = vec![
        "up".to_string(),
        "--workspace-folder".to_string(),
        opts.project_dir.to_string_lossy().to_string(),
        "--override-config".to_string(),
        override_path.to_string_lossy().to_string(),
        "--output".to_string(),
        "json".to_string(),
    ];

    if opts.rebuild {
        cmd_args.push("--prebuild".to_string());
    }

    // If using a sub-config, we need to specify it
    if dc.dir != opts.project_dir.join(".devcontainer") {
        cmd_args.push("--config".to_string());
        cmd_args.push(dc.dir.join("devcontainer.json").to_string_lossy().to_string());
    }

    if opts.verbose {
        eprintln!("$ devcontainer {}", cmd_args.join(" "));
    }

    if opts.dry_run {
        return Ok("dry-run-container".to_string());
    }

    let output = Command::new("devcontainer")
        .args(&cmd_args)
        .output()
        .context("Failed to execute devcontainer CLI. Is it installed? (npm install -g @devcontainers/cli)")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("devcontainer up failed: {}", stderr);
    }

    // 3. Parse output to get container ID
    let stdout = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&stdout)
        .context("Failed to parse devcontainer CLI output")?;
    
    let container_id = val.get("containerId")
        .and_then(|v| v.as_str())
        .context("Could not find containerId in CLI output")?
        .to_string();

    Ok(container_id)
}

/// Ensure tools (neovim, tmux, gh, claude, gemini) are available inside.
fn ensure_tools(name: &str, opts: &RunOptions) -> Result<()> {
    if opts.dry_run {
        eprintln!("$ docker exec {} ... # (check/install environment tools)", name);
        return Ok(());
    }

    // Check what we need
    let check_cmd = "which nvim tmux gh claude gemini node npm >/dev/null 2>&1";
    let status = Command::new("docker")
        .args(["exec", name, "sh", "-c", check_cmd])
        .status()?;

    if status.success() {
        // Everything seems to be there
        return Ok(());
    }

    eprintln!("Some tools missing in container, performing magic injection...");

    // 1. Install base dependencies and GH CLI via package manager
    let install_base = r#"
if command -v apt-get >/dev/null 2>&1; then
    apt-get update -qq
    apt-get install -y -qq curl tar tmux git 2>/dev/null
    # Install GitHub CLI
    if ! command -v gh >/dev/null 2>&1; then
        curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
        chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg
        echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null
        apt-get update -qq && apt-get install -y -qq gh 2>/dev/null
    fi
elif command -v apk >/dev/null 2>&1; then
    apk add --no-cache curl tar tmux git github-cli
elif command -v dnf >/dev/null 2>&1; then
    dnf install -y curl tar tmux git github-cli
else
    echo "WARNING: Unsupported package manager. Skipping base tool install." >&2
fi
"#;
    let _ = Command::new("docker")
        .args(["exec", "--user", "root", name, "sh", "-c", install_base])
        .status();

    // 2. Install Neovim (v0.12.2)
    let nvim_check = Command::new("docker").args(["exec", name, "which", "nvim"]).status()?;
    if !nvim_check.success() {
        eprintln!("Installing Neovim...");
        let install_nvim = r#"
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  NVIM_ARCHIVE="nvim-linux-x86_64.tar.gz" ;;
    aarch64) NVIM_ARCHIVE="nvim-linux-arm64.tar.gz" ;;
    *)       exit 1 ;;
esac
curl -fsSL "https://github.com/neovim/neovim/releases/download/v0.12.2/${NVIM_ARCHIVE}" -o /tmp/nvim.tar.gz
tar -C /opt -xzf /tmp/nvim.tar.gz
ln -sf /opt/nvim-linux-*/bin/nvim /usr/local/bin/nvim
"#;
        let _ = Command::new("docker")
            .args(["exec", "--user", "root", name, "sh", "-c", install_nvim])
            .status();
    }

    // 3. Install Node.js if missing (needed for Claude/Gemini)
    let node_check = Command::new("docker").args(["exec", name, "which", "node"]).status()?;
    if !node_check.success() {
        eprintln!("Node.js missing, installing minimal version...");
        let install_node = r#"
if command -v apt-get >/dev/null 2>&1; then
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
    apt-get install -y nodejs -qq
elif command -v apk >/dev/null 2>&1; then
    apk add --no-cache nodejs npm
fi
"#;
        let _ = Command::new("docker")
            .args(["exec", "--user", "root", name, "sh", "-c", install_node])
            .status();
    }

    // 4. Install Claude and Gemini CLIs
    let npm_check = Command::new("docker").args(["exec", name, "which", "npm"]).status()?;
    if npm_check.success() {
        eprintln!("Installing Claude and Gemini CLIs...");
        let install_ai = "npm install -g @anthropic-ai/claude-code @google/gemini-cli";
        let _ = Command::new("docker")
            .args(["exec", "--user", "root", name, "sh", "-c", install_ai])
            .status();
    }

    Ok(())
}

/// Exec into the container running nvim.
pub fn exec_nvim(name: &str, dc: &DevcontainerConfig, opts: &RunOptions) -> Result<()> {
    ensure_tools(name, opts)?;

    let user = dc.remote_user().unwrap_or_else(|| dc.effective_user());
    let workspace = dc.workspace_folder();

    // Ensure the user's home directories exist and are writable
    if !opts.dry_run {
        let setup_dirs = format!(
            "mkdir -p /home/{user}/.local/state /home/{user}/.local/share /home/{user}/.cache && chown -R $(id -u {user}):$(id -g {user}) /home/{user}/.local /home/{user}/.cache 2>/dev/null || true",
            user = user,
        );
        let _ = Command::new("docker")
            .args(["exec", "--user", "root", name, "sh", "-c", &setup_dirs])
            .status();
    }

    // Write a temp init script that launches tmux.
    if !opts.dry_run {
        let init_script = format!(
            "export PATH=\"/usr/local/bin:/opt/nvim-linux-x86_64/bin:$PATH\"; tmux new-session -A -s nvdc -c {} 'bash'",
            workspace
        );
        let _ = Command::new("docker")
            .args(["exec", "--user", &user, name, "sh", "-c",
                   &format!("echo '{}' > /tmp/.nvdc-init", init_script)])
            .status();
    }

    let cmd_args = vec![
        "exec".to_string(),
        "-it".to_string(),
        "-e".to_string(),
        "PATH=/usr/local/bin:/usr/bin:/bin:/opt/nvim-linux-x86_64/bin".to_string(),
        "--user".to_string(),
        user,
        "-w".to_string(),
        workspace,
        name.to_string(),
        "bash".to_string(),
        "--rcfile".to_string(),
        "/tmp/.nvdc-init".to_string(),
    ];

    print_cmd("docker", &cmd_args, opts);

    if !opts.dry_run {
        // Disable suspend key on host so Ctrl+Z passes through to the container's bash
        let _ = Command::new("stty").args(["susp", "undef"]).status();

        let status = Command::new("docker")
            .args(&cmd_args)
            .status()
            .context("Failed to exec into container");

        // Restore suspend key on host
        let _ = Command::new("stty").args(["susp", "^Z"]).status();

        let status = status?;
        if !status.success() {
            anyhow::bail!("nvim exited with code {:?}", status.code());
        }
    }

    Ok(())
}

/// Stop and remove a container.
pub fn stop_and_remove(name: &str, opts: &RunOptions) -> Result<()> {
    let stop_args = vec!["stop".to_string(), name.to_string()];
    let rm_args = vec!["rm".to_string(), name.to_string()];

    print_cmd("docker", &stop_args, opts);
    print_cmd("docker", &rm_args, opts);

    if !opts.dry_run {
        let _ = Command::new("docker")
            .args(&stop_args)
            .status();

        let _ = Command::new("docker")
            .args(&rm_args)
            .status();
    }

    Ok(())
}

fn print_cmd(program: &str, args: &[String], opts: &RunOptions) {
    if opts.dry_run || opts.verbose {
        eprintln!("$ {} {}", program, args.join(" "));
    }
}
