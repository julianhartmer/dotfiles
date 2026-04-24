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

    /// Get raw workspace folder without substitution (to avoid recursion)
    fn workspace_folder_raw(&self) -> String {
        self.raw
            .get("workspaceFolder")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "/workspace".to_string())
    }
    /// Get the Dockerfile path relative to the devcontainer dir
    pub fn dockerfile(&self) -> Option<PathBuf> {
        self.raw
            .get("build")
            .and_then(|b| b.get("dockerfile"))
            .and_then(|v| v.as_str())
            .map(|s| self.dir.join(s))
            .or_else(|| {
                self.raw
                    .get("dockerFile")
                    .and_then(|v| v.as_str())
                    .map(|s| self.dir.join(s))
            })
    }

    /// Get the build context directory
    pub fn build_context(&self) -> PathBuf {
        self.raw
            .get("build")
            .and_then(|b| b.get("context"))
            .and_then(|v| v.as_str())
            .map(|s| self.dir.join(s))
            .unwrap_or_else(|| self.dir.clone())
    }

    /// Get build args as key-value pairs
    pub fn build_args(&self) -> Vec<(String, String)> {
        self.raw
            .get("build")
            .and_then(|b| b.get("args"))
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), self.substitute(s))))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get additional build options
    pub fn build_options(&self) -> Vec<String> {
        self.raw
            .get("build")
            .and_then(|b| b.get("options"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| self.substitute(s))).collect())
            .unwrap_or_default()
    }

    /// Get the base image (when no Dockerfile is used)
    pub fn image(&self) -> Option<String> {
        self.raw
            .get("image")
            .and_then(|v| v.as_str())
            .map(String::from)
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

    /// Get run arguments
    pub fn run_args(&self) -> Vec<String> {
        self.raw
            .get("runArgs")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| self.substitute(s))).collect())
            .unwrap_or_default()
    }

    /// Get the workspace folder inside the container
    pub fn workspace_folder(&self) -> String {
        self.substitute(&self.workspace_folder_raw())
    }

    /// Get the initializeCommand
    pub fn initialize_command(&self) -> Option<ShellCommand> {
        parse_command(self.raw.get("initializeCommand"))
            .map(|cmd| cmd.substitute(|s| self.substitute(s)))
    }

    /// Get the onCreateCommand
    pub fn on_create_command(&self) -> Option<ShellCommand> {
        parse_command(self.raw.get("onCreateCommand"))
            .map(|cmd| cmd.substitute(|s| self.substitute(s)))
    }

    /// Get additional mounts from devcontainer.json
    pub fn mounts(&self) -> Vec<String> {
        self.raw
            .get("mounts")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| match v {
                        Value::String(s) => Some(self.substitute(s)),
                        Value::Object(obj) => {
                            // Convert mount object to string format
                            let mount_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("bind");
                            let source = obj.get("source").and_then(|v| v.as_str())?;
                            let target = obj.get("target").and_then(|v| v.as_str())?;
                            Some(self.substitute(&format!("type={},source={},target={}", mount_type, source, target)))
                        }
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub enum ShellCommand {
    Simple(String),
    List(Vec<String>),
}

impl ShellCommand {
    pub fn to_shell_string(&self) -> String {
        match self {
            ShellCommand::Simple(s) => s.clone(),
            ShellCommand::List(parts) => parts.join(" && "),
        }
    }

    pub fn substitute<F: Fn(&str) -> String>(self, f: F) -> ShellCommand {
        match self {
            ShellCommand::Simple(s) => ShellCommand::Simple(f(&s)),
            ShellCommand::List(parts) => ShellCommand::List(parts.iter().map(|s| f(s)).collect()),
        }
    }
}

fn parse_command(value: Option<&Value>) -> Option<ShellCommand> {
    match value? {
        Value::String(s) => Some(ShellCommand::Simple(s.clone())),
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
            if parts.is_empty() {
                None
            } else {
                Some(ShellCommand::List(parts))
            }
        }
        _ => None,
    }
}

/// Strip JSON5/JSONC comments from a string so it can be parsed as standard JSON.
fn strip_json_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;

    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    out.push(next);
                    chars.next();
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            out.push(c);
            continue;
        }

        if c == '/' {
            match chars.peek() {
                Some(&'/') => {
                    // Line comment — consume until newline
                    for ch in chars.by_ref() {
                        if ch == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                }
                Some(&'*') => {
                    // Block comment — consume until */
                    chars.next(); // skip '*'
                    loop {
                        match chars.next() {
                            Some('*') if chars.peek() == Some(&'/') => {
                                chars.next();
                                break;
                            }
                            Some('\n') => out.push('\n'),
                            Some(_) => {}
                            None => break,
                        }
                    }
                }
                _ => out.push(c),
            }
        } else {
            out.push(c);
        }
    }

    out
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

    let stripped = strip_json_comments(&content);
    let stripped = strip_trailing_commas(&stripped);

    serde_json::from_str(&stripped)
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
