mod config;
mod devcontainer;
mod docker;
mod nvim;
mod prompt;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nvdc", about = "Launch devcontainers with your local Neovim config")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Use a specific devcontainer config by folder name
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Override neovim config path (default: ~/.config/nvim)
    #[arg(short = 'n', long = "nvim-config", global = true)]
    nvim_config: Option<PathBuf>,

    /// Skip mounting ~/.local/share/nvim
    #[arg(long, global = true)]
    no_nvim_data: bool,

    /// Force rebuild the container image
    #[arg(long, global = true)]
    rebuild: bool,

    /// Print the docker commands without executing
    #[arg(long, global = true)]
    dry_run: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Start container without attaching
    Up,
    /// Attach to a running container
    Attach,
    /// Stop and remove the container
    Down,
    /// Show or reset current preferences
    Config {
        /// Reset saved preferences for this project
        #[arg(long)]
        reset: bool,
    },
    /// List available devcontainer configs
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let project_dir = std::env::current_dir().context("Failed to determine current directory")?;

    let opts = RunOptions {
        project_dir: project_dir.clone(),
        config_name: cli.config,
        nvim_config: cli.nvim_config,
        no_nvim_data: cli.no_nvim_data,
        rebuild: cli.rebuild,
        dry_run: cli.dry_run,
        verbose: cli.verbose,
    };

    match cli.command {
        None => cmd_launch(&opts),
        Some(Command::Up) => cmd_up(&opts),
        Some(Command::Attach) => cmd_attach(&opts),
        Some(Command::Down) => cmd_down(&opts),
        Some(Command::Config { reset }) => cmd_config(&opts, reset),
        Some(Command::List) => cmd_list(&opts),
    }
}

pub struct RunOptions {
    pub project_dir: PathBuf,
    pub config_name: Option<String>,
    pub nvim_config: Option<PathBuf>,
    pub no_nvim_data: bool,
    pub rebuild: bool,
    pub dry_run: bool,
    pub verbose: bool,
}

fn resolve_config(opts: &RunOptions) -> Result<devcontainer::DevcontainerConfig> {
    let configs = devcontainer::discover(&opts.project_dir)?;

    if configs.is_empty() {
        anyhow::bail!(
            "No devcontainer configuration found in {}",
            opts.project_dir.display()
        );
    }

    // If a specific config was requested via --config flag
    if let Some(ref name) = opts.config_name {
        return configs
            .into_iter()
            .find(|c| c.name == *name)
            .with_context(|| format!("Devcontainer config '{}' not found", name));
    }

    // If there's only one config, use it
    if configs.len() == 1 {
        return Ok(configs.into_iter().next().unwrap());
    }

    // Check for saved preference
    let prefs = config::load_preferences(&opts.project_dir);
    if let Ok(ref prefs) = prefs {
        if let Some(ref default) = prefs.preferences.default_config {
            if let Some(cfg) = configs.iter().find(|c| c.name == *default) {
                eprintln!("Using config: {}", default);
                return Ok(cfg.clone());
            }
        }
    }

    // Prompt the user
    let selected = prompt::select_config(&configs, prefs.ok().and_then(|p| p.preferences.default_config))?;

    // Save the preference
    config::save_default_config(&opts.project_dir, &selected.name)?;

    Ok(selected)
}

fn cmd_launch(opts: &RunOptions) -> Result<()> {
    let dc = resolve_config(opts)?;
    if opts.verbose {
        eprintln!("Using devcontainer config: {}", dc.name);
    }

    let nvim_opts = nvim::NvimMountOptions::from_run_options(opts);
    let container_name = docker::container_name(&opts.project_dir, &dc.name);

    // Check if container is already running
    if docker::is_running(&container_name, opts.dry_run)? {
        eprintln!("Container '{}' is already running, attaching...", container_name);
        return docker::exec_nvim(&container_name, &dc, opts);
    }

    // Build
    docker::build(&dc, opts)?;

    // Run
    docker::run(&dc, &nvim_opts, opts)?;

    // Save state
    config::save_state(&opts.project_dir, &container_name)?;

    // Attach with nvim
    docker::exec_nvim(&container_name, &dc, opts)
}

fn cmd_up(opts: &RunOptions) -> Result<()> {
    let dc = resolve_config(opts)?;
    let nvim_opts = nvim::NvimMountOptions::from_run_options(opts);
    let container_name = docker::container_name(&opts.project_dir, &dc.name);

    if docker::is_running(&container_name, opts.dry_run)? {
        eprintln!("Container '{}' is already running.", container_name);
        return Ok(());
    }

    docker::build(&dc, opts)?;
    docker::run(&dc, &nvim_opts, opts)?;
    config::save_state(&opts.project_dir, &container_name)?;

    eprintln!("Container '{}' started.", container_name);
    Ok(())
}

fn cmd_attach(opts: &RunOptions) -> Result<()> {
    let dc = resolve_config(opts)?;
    let container_name = docker::container_name(&opts.project_dir, &dc.name);

    if !docker::is_running(&container_name, opts.dry_run)? {
        anyhow::bail!("Container '{}' is not running. Use `nvdc` or `nvdc up` first.", container_name);
    }

    docker::exec_nvim(&container_name, &dc, opts)
}

fn cmd_down(opts: &RunOptions) -> Result<()> {
    let state = config::load_state(&opts.project_dir);
    let container_name = if let Ok(ref state) = state {
        state.container_name.clone()
    } else {
        // Try to figure out the container name from config
        let dc = resolve_config(opts)?;
        Some(docker::container_name(&opts.project_dir, &dc.name))
    };

    if let Some(ref name) = container_name {
        docker::stop_and_remove(name, opts)?;
        config::clear_state(&opts.project_dir)?;
        eprintln!("Container '{}' stopped and removed.", name);
    } else {
        eprintln!("No running container found for this project.");
    }

    Ok(())
}

fn cmd_config(opts: &RunOptions, reset: bool) -> Result<()> {
    if reset {
        config::reset_preferences(&opts.project_dir)?;
        eprintln!("Preferences reset for this project.");
        return Ok(());
    }

    let configs = devcontainer::discover(&opts.project_dir)?;
    let prefs = config::load_preferences(&opts.project_dir).ok();
    let current_default = prefs.and_then(|p| p.preferences.default_config);

    if configs.is_empty() {
        eprintln!("No devcontainer configurations found in this project.");
        return Ok(());
    }

    // Show current config
    if let Some(ref name) = current_default {
        eprintln!("Current default config: {}", name);
    } else {
        eprintln!("No default config set.");
    }
    eprintln!();

    // Let the user pick a new default
    let selected = prompt::select_config(&configs, current_default)?;
    config::save_default_config(&opts.project_dir, &selected.name)?;
    eprintln!("Default config set to: {}", selected.name);

    Ok(())
}

fn cmd_list(opts: &RunOptions) -> Result<()> {
    let configs = devcontainer::discover(&opts.project_dir)?;

    if configs.is_empty() {
        eprintln!("No devcontainer configurations found.");
        return Ok(());
    }

    let prefs = config::load_preferences(&opts.project_dir).ok();
    let default_name = prefs.and_then(|p| p.preferences.default_config);

    println!("Available devcontainer configs:");
    for (i, cfg) in configs.iter().enumerate() {
        let marker = if default_name.as_deref() == Some(&cfg.name) {
            " (default)"
        } else {
            ""
        };
        let desc = cfg
            .raw
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| format!(" — {}", s))
            .unwrap_or_default();
        println!("  [{}] {}{}{}", i + 1, cfg.name, desc, marker);
    }

    Ok(())
}
