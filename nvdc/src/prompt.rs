use anyhow::{Context, Result};
use dialoguer::Select;

use crate::devcontainer::DevcontainerConfig;

/// Prompt the user to select a devcontainer config from a list.
pub fn select_config(
    configs: &[DevcontainerConfig],
    last_used: Option<String>,
) -> Result<DevcontainerConfig> {
    eprintln!("Found multiple devcontainer configs:");

    let items: Vec<String> = configs
        .iter()
        .map(|c| {
            let desc = c
                .raw
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| format!(" — {}", s))
                .unwrap_or_default();
            format!("{}{}", c.name, desc)
        })
        .collect();

    let default_idx = last_used
        .as_ref()
        .and_then(|name| configs.iter().position(|c| &c.name == name))
        .unwrap_or(0);

    let prompt_msg = if let Some(ref name) = last_used {
        format!("Select config (last used: {})", name)
    } else {
        "Select config".to_string()
    };

    let selection = Select::new()
        .with_prompt(&prompt_msg)
        .items(&items)
        .default(default_idx)
        .interact()
        .context("Failed to read selection")?;

    Ok(configs[selection].clone())
}
