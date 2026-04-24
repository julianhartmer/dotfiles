use std::path::PathBuf;

use crate::RunOptions;

pub struct NvimMountOptions {
    /// Custom path to neovim config directory
    pub config_override: Option<PathBuf>,
    /// Whether to mount neovim data directory
    pub mount_data: bool,
}

impl NvimMountOptions {
    pub fn from_run_options(opts: &RunOptions) -> Self {
        Self {
            config_override: opts.nvim_config.clone(),
            mount_data: !opts.no_nvim_data,
        }
    }

    /// Resolve the neovim config directory path.
    pub fn config_path(&self) -> PathBuf {
        if let Some(ref p) = self.config_override {
            if p.starts_with("~") {
                expand_tilde(p)
            } else {
                p.clone()
            }
        } else {
            dirs::config_dir()
                .map(|d| d.join("nvim"))
                .unwrap_or_else(|| {
                    dirs::home_dir()
                        .map(|h| h.join(".config/nvim"))
                        .expect("Could not determine home directory")
                })
        }
    }
}

fn expand_tilde(path: &std::path::Path) -> PathBuf {
    if let Ok(stripped) = path.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    path.to_path_buf()
}
