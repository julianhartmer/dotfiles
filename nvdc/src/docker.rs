use anyhow::{Context, Result};
use std::path::Path;
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

/// Generate the image tag for a devcontainer build.
fn image_tag(project_dir: &Path, config_name: &str) -> String {
    let project = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    format!("nvdc/{}-{}:latest", sanitize(project), sanitize(config_name))
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

/// Build the container image.
pub fn build(dc: &DevcontainerConfig, opts: &RunOptions) -> Result<()> {
    // Run initializeCommand if present
    if let Some(ref cmd) = dc.initialize_command() {
        let shell_str = cmd.to_shell_string();
        eprintln!("Running initializeCommand: {}", shell_str);

        if !opts.dry_run {
            let status = Command::new("sh")
                .args(["-c", &shell_str])
                .current_dir(&opts.project_dir)
                .status()
                .context("Failed to run initializeCommand")?;

            if !status.success() {
                anyhow::bail!("initializeCommand failed with exit code {:?}", status.code());
            }
        }
    }

    let tag = image_tag(&opts.project_dir, &dc.name);

    if let Some(dockerfile) = dc.dockerfile() {
        let context = dc.build_context();
        let mut cmd_args = vec![
            "build".to_string(),
            "-t".to_string(),
            tag.clone(),
            "-f".to_string(),
            dockerfile.to_string_lossy().to_string(),
        ];

        for (key, value) in dc.build_args() {
            cmd_args.push("--build-arg".to_string());
            cmd_args.push(format!("{}={}", key, value));
        }

        for opt in dc.build_options() {
            cmd_args.push(opt);
        }

        if opts.rebuild {
            cmd_args.push("--no-cache".to_string());
        }

        cmd_args.push(context.to_string_lossy().to_string());

        print_cmd("docker", &cmd_args, opts);

        if !opts.dry_run {
            let status = Command::new("docker")
                .args(&cmd_args)
                .status()
                .context("Failed to run docker build")?;

            if !status.success() {
                anyhow::bail!("docker build failed with exit code {:?}", status.code());
            }
        }
    } else if let Some(ref image) = dc.image() {
        // Pull the image if using image-based config
        let cmd_args = vec!["pull".to_string(), image.clone()];
        print_cmd("docker", &cmd_args, opts);

        if !opts.dry_run {
            let status = Command::new("docker")
                .args(&cmd_args)
                .status()
                .context("Failed to pull image")?;

            if !status.success() {
                anyhow::bail!("docker pull failed with exit code {:?}", status.code());
            }

            // Tag it so we have a consistent name
            let status = Command::new("docker")
                .args(["tag", image, &tag])
                .status()
                .context("Failed to tag image")?;

            if !status.success() {
                anyhow::bail!("docker tag failed");
            }
        }
    } else {
        anyhow::bail!("devcontainer.json has neither a Dockerfile nor an image specified");
    }

    eprintln!("Image built: {}", tag);
    Ok(())
}

/// Run the container.
pub fn run(dc: &DevcontainerConfig, nvim_opts: &NvimMountOptions, opts: &RunOptions) -> Result<()> {
    let tag = image_tag(&opts.project_dir, &dc.name);
    let name = container_name(&opts.project_dir, &dc.name);
    let workspace_folder = dc.workspace_folder();

    let mut cmd_args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--name".to_string(),
        name.clone(),
    ];

    // Mount project directory as workspace
    cmd_args.push("-v".to_string());
    cmd_args.push(format!(
        "{}:{}",
        opts.project_dir.to_string_lossy(),
        workspace_folder,
    ));

    // Mount neovim config (read-only)
    let nvim_config = nvim_opts.config_path();
    cmd_args.push("-v".to_string());
    cmd_args.push(format!(
        "{}:/home/{}/.config/nvim:ro",
        nvim_config.to_string_lossy(),
        dc.effective_user(),
    ));

    // Mount neovim data
    if nvim_opts.mount_data {
        let nvim_data = nvim_opts.data_path();
        cmd_args.push("-v".to_string());
        cmd_args.push(format!(
            "{}:/home/{}/.local/share/nvim",
            nvim_data.to_string_lossy(),
            dc.effective_user(),
        ));
    }

    // Mount tmux config (read-only)
    if let Some(home) = dirs::home_dir() {
        let tmux_conf = home.join(".tmux.conf");
        if tmux_conf.exists() {
            cmd_args.push("-v".to_string());
            cmd_args.push(format!(
                "{}:/home/{}/.tmux.conf:ro",
                tmux_conf.to_string_lossy(),
                dc.effective_user(),
            ));
        }
    }

    // Add devcontainer mounts
    for mount in dc.mounts() {
        cmd_args.push("--mount".to_string());
        cmd_args.push(mount);
    }

    // Add runArgs
    for arg in dc.run_args() {
        cmd_args.push(arg);
    }

    // Set working directory
    cmd_args.push("-w".to_string());
    cmd_args.push(workspace_folder.clone());

    // Set user
    if let Some(user) = dc.container_user() {
        cmd_args.push("--user".to_string());
        cmd_args.push(user);
    }

    // Image
    cmd_args.push(tag);

    // Keep container running with a long sleep
    cmd_args.push("sleep".to_string());
    cmd_args.push("infinity".to_string());

    print_cmd("docker", &cmd_args, opts);

    if !opts.dry_run {
        let output = Command::new("docker")
            .args(&cmd_args)
            .output()
            .context("Failed to run docker run")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("docker run failed: {}", stderr.trim());
        }

        // Run onCreateCommand if present
        if let Some(ref cmd) = dc.on_create_command() {
            let shell_str = cmd.to_shell_string();
            eprintln!("Running onCreateCommand: {}", shell_str);

            let status = Command::new("docker")
                .args(["exec", &name, "sh", "-c", &shell_str])
                .status()
                .context("Failed to run onCreateCommand")?;

            if !status.success() {
                eprintln!("Warning: onCreateCommand failed with exit code {:?}", status.code());
            }
        }
    }

    Ok(())
}

/// Ensure neovim and tmux are available inside the container, installing them if needed.
fn ensure_nvim(name: &str, opts: &RunOptions) -> Result<()> {
    if opts.dry_run {
        eprintln!("$ docker exec {} which nvim tmux  # (check/install neovim+tmux)", name);
        return Ok(());
    }

    let nvim_check = Command::new("docker")
        .args(["exec", name, "which", "nvim"])
        .output()
        .context("Failed to check for nvim in container")?;

    let tmux_check = Command::new("docker")
        .args(["exec", name, "which", "tmux"])
        .output()
        .context("Failed to check for tmux in container")?;

    let need_nvim = !nvim_check.status.success();
    let need_tmux = !tmux_check.status.success();

    if !need_nvim && !need_tmux {
        return Ok(());
    }

    // Install tmux via package manager if needed
    if need_tmux {
        eprintln!("tmux not found in container, installing...");
        let install_tmux = r#"
if command -v apt-get >/dev/null 2>&1; then
    apt-get update -qq && apt-get install -y -qq tmux 2>/dev/null
elif command -v apk >/dev/null 2>&1; then
    apk add --no-cache tmux
elif command -v dnf >/dev/null 2>&1; then
    dnf install -y tmux
elif command -v yum >/dev/null 2>&1; then
    yum install -y tmux
elif command -v pacman >/dev/null 2>&1; then
    pacman -Sy --noconfirm tmux
else
    echo "WARNING: Could not install tmux. No supported package manager found." >&2
fi
"#;
        let status = Command::new("docker")
            .args(["exec", "--user", "root", name, "sh", "-c", install_tmux])
            .status()
            .context("Failed to install tmux in container")?;

        if status.success() {
            eprintln!("tmux installed successfully.");
        } else {
            eprintln!("Warning: tmux installation failed. Falling back to nvim without tmux.");
        }
    }

    if need_nvim {
        eprintln!("Neovim not found in container, installing...");

        // Download neovim v0.12.2 from GitHub releases
        let install_script = r#"
set -e
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  NVIM_ARCHIVE="nvim-linux-x86_64.tar.gz" ;;
    aarch64) NVIM_ARCHIVE="nvim-linux-arm64.tar.gz" ;;
    *)       echo "ERROR: Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac
curl -fsSL "https://github.com/neovim/neovim/releases/download/v0.12.2/${NVIM_ARCHIVE}" -o /tmp/nvim.tar.gz
rm -rf /opt/nvim-linux
tar -C /opt -xzf /tmp/nvim.tar.gz
rm /tmp/nvim.tar.gz
# Symlink so nvim is on PATH
ln -sf /opt/nvim-linux-*/bin/nvim /usr/local/bin/nvim
nvim --version | head -1
"#;

        let status = Command::new("docker")
            .args(["exec", "--user", "root", name, "sh", "-c", install_script])
            .status()
            .context("Failed to install neovim in container")?;

        if !status.success() {
            anyhow::bail!("Failed to install neovim in container. Install it in your Dockerfile or image.");
        }

        eprintln!("Neovim installed successfully.");
    }

    Ok(())
}

/// Exec into the container running nvim.
pub fn exec_nvim(name: &str, dc: &DevcontainerConfig, opts: &RunOptions) -> Result<()> {
    ensure_nvim(name, opts)?;

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

    // Write a temp init script that launches tmux with nvim.
    if !opts.dry_run {
        let init_script = r#"export PATH="/usr/local/bin:/opt/nvim-linux-x86_64/bin:$PATH"; tmux new-session -A -s nvdc 'nvim .'"#;
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
