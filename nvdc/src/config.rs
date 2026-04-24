use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const NVDC_DIR: &str = ".nvdc";
const CONFIG_FILE: &str = "config.toml";
const STATE_FILE: &str = "state.toml";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NvdcConfig {
    pub preferences: Preferences,
    #[serde(default)]
    pub container: ContainerConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Preferences {
    pub default_config: Option<String>,
    pub nvim_config_path: Option<String>,
    pub nvim_data_path: Option<String>,
    pub mount_nvim_data: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ContainerConfig {
    #[serde(default)]
    pub extra_mounts: Vec<String>,
    #[serde(default)]
    pub extra_env: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NvdcState {
    pub container_name: Option<String>,
}

fn nvdc_dir(project_dir: &Path) -> std::path::PathBuf {
    project_dir.join(NVDC_DIR)
}

fn ensure_nvdc_dir(project_dir: &Path) -> Result<std::path::PathBuf> {
    let dir = nvdc_dir(project_dir);
    if !dir.exists() {
        fs::create_dir_all(&dir).context("Failed to create .nvdc directory")?;
    }
    Ok(dir)
}

pub fn load_preferences(project_dir: &Path) -> Result<NvdcConfig> {
    let path = nvdc_dir(project_dir).join(CONFIG_FILE);
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    toml::from_str(&content).context("Failed to parse config.toml")
}

pub fn save_default_config(project_dir: &Path, config_name: &str) -> Result<()> {
    let dir = ensure_nvdc_dir(project_dir)?;
    let path = dir.join(CONFIG_FILE);

    let mut cfg = load_preferences(project_dir).unwrap_or_default();
    cfg.preferences.default_config = Some(config_name.to_string());

    let content = toml::to_string_pretty(&cfg).context("Failed to serialize config")?;
    fs::write(&path, content).context("Failed to write config.toml")
}

pub fn reset_preferences(project_dir: &Path) -> Result<()> {
    let path = nvdc_dir(project_dir).join(CONFIG_FILE);
    if path.exists() {
        fs::remove_file(&path).context("Failed to remove config.toml")?;
    }
    Ok(())
}

pub fn load_state(project_dir: &Path) -> Result<NvdcState> {
    let path = nvdc_dir(project_dir).join(STATE_FILE);
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    toml::from_str(&content).context("Failed to parse state.toml")
}

pub fn save_state(project_dir: &Path, container_name: &str) -> Result<()> {
    let dir = ensure_nvdc_dir(project_dir)?;
    let path = dir.join(STATE_FILE);

    let state = NvdcState {
        container_name: Some(container_name.to_string()),
    };

    let content = toml::to_string_pretty(&state).context("Failed to serialize state")?;
    fs::write(&path, content).context("Failed to write state.toml")
}

pub fn clear_state(project_dir: &Path) -> Result<()> {
    let path = nvdc_dir(project_dir).join(STATE_FILE);
    if path.exists() {
        fs::remove_file(&path).context("Failed to remove state.toml")?;
    }
    Ok(())
}
