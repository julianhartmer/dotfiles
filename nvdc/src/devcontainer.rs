use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DevcontainerConfig {
    /// Folder name used as identifier (e.g. "vx7-app")
    pub name: String,
    /// Path to the directory containing devcontainer.json
    pub dir: PathBuf,
    /// Path to devcontainer.json
    #[allow(dead_code)]
    pub json_path: PathBuf,
    /// Project root directory
    pub project_dir: PathBuf,
    /// Parsed JSON content
    pub raw: Value,
}

impl DevcontainerConfig {
    /// Get raw workspace folder without substitution (to avoid recursion)
    fn workspace_folder_raw(&self) -> String {
        self.raw
            .get("workspaceFolder")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "/workspace".to_string())
    }

    /// Substitute devcontainer variables in a string.
    /// Supports: ${localWorkspaceFolder}, ${containerWorkspaceFolder},
    /// ${localWorkspaceFolderBasename}, ${devcontainerId}, ${localEnv:VAR}
    fn substitute(&self, s: &str) -> String {
        let workspace_folder = self.workspace_folder_raw();
        let result = s
            .replace("${localWorkspaceFolder}", &self.project_dir.to_string_lossy())
            .replace("${containerWorkspaceFolder}", &workspace_folder)
            .replace(
                "${localWorkspaceFolderBasename}",
                self.project_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(""),
            )
            .replace("${devcontainerId}", &format!("nvdc-{}", self.name));

        // Expand ${localEnv:VAR} and ${localEnv:VAR:default}
        let re = regex_lite::Regex::new(r"\$\{localEnv:([^}:]+)(?::([^}]*))?\}").unwrap();
        re.replace_all(&result, |caps: &regex_lite::Captures| {
            let var_name = caps.get(1).unwrap().as_str();
            let default = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            std::env::var(var_name).unwrap_or_else(|_| default.to_string())
        })
        .into_owned()
    }

    /// Get the remote user
    pub fn remote_user(&self) -> Option<String> {
        self.raw
            .get("remoteUser")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    /// Get the container user
    pub fn container_user(&self) -> Option<String> {
        self.raw
            .get("containerUser")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    /// Get the effective user (containerUser takes priority for runtime, remoteUser for exec)
    pub fn effective_user(&self) -> String {
        self.container_user()
            .or_else(|| self.remote_user())
            .unwrap_or_else(|| "root".to_string())
    }

    /// Get the workspace folder inside the container
    pub fn workspace_folder(&self) -> String {
        self.substitute(&self.workspace_folder_raw())
    }
}

/// Remove trailing commas before } and ] in JSON-like content.
fn strip_trailing_commas(input: &str) -> String {
    let re_obj = regex_lite::Regex::new(r",(\s*[}\]])").unwrap();
    re_obj.replace_all(input, "$1").into_owned()
}

/// Parse a devcontainer.json file (supports JSONC comments and trailing commas)
fn parse_devcontainer_json(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let stripped = strip_trailing_commas(&content);
    let mut parser = json_comments::CommentSettings::c_style().strip_comments(stripped.as_bytes());

    serde_json::from_reader(&mut parser)
        .with_context(|| format!("Failed to parse {}", path.display()))
}

/// Discover devcontainer configurations in a project directory.
///
/// Looks for:
/// 1. `.devcontainer/devcontainer.json` (single config, named after project dir)
/// 2. `.devcontainer/*/devcontainer.json` (multiple configs, named after subfolder)
pub fn discover(project_dir: &Path) -> Result<Vec<DevcontainerConfig>> {
    let devcontainer_dir = project_dir.join(".devcontainer");

    if !devcontainer_dir.exists() {
        return Ok(vec![]);
    }

    let mut configs = Vec::new();

    // Check for root-level devcontainer.json
    let root_json = devcontainer_dir.join("devcontainer.json");
    if root_json.exists() {
        let raw = parse_devcontainer_json(&root_json)?;
        let name = project_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default")
            .to_string();

        configs.push(DevcontainerConfig {
            name,
            dir: devcontainer_dir.clone(),
            json_path: root_json,
            project_dir: project_dir.to_path_buf(),
            raw,
        });
    }

    // Check for subdirectory configs
    if let Ok(entries) = fs::read_dir(&devcontainer_dir) {
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                let json_path = path.join("devcontainer.json");
                if json_path.exists() {
                    let raw = parse_devcontainer_json(&json_path)?;
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    configs.push(DevcontainerConfig {
                        name,
                        dir: path,
                        json_path,
                        project_dir: project_dir.to_path_buf(),
                        raw,
                    });
                }
            }
        }
    }

    Ok(configs)
}
