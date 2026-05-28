use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Full pipeline configuration from rift.yml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub pipeline: PipelineMeta,
    pub source: SourceConfig,
    pub targets: Vec<TargetConfig>,
    #[serde(default)]
    pub rules: Vec<RuleConfig>,
    #[serde(default)]
    pub hooks: HooksConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Shell command to run before the pipeline starts
    #[serde(default)]
    pub pre_run: Option<String>,
    /// Shell command to run after the pipeline completes successfully
    #[serde(default)]
    pub post_run: Option<String>,
    /// Shell command to run if the pipeline encounters errors
    #[serde(default)]
    pub on_error: Option<String>,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            pre_run: None,
            post_run: None,
            on_error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMeta {
    pub name: String,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub root: PathBuf,
    #[serde(default = "default_true")]
    pub watch: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub engine: String,
    pub project: PathBuf,
    #[serde(default)]
    pub textures: TextureTargetConfig,
    #[serde(default)]
    pub audio: AudioTargetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureTargetConfig {
    #[serde(default = "default_texture_format")]
    pub format: String,
    #[serde(default = "default_max_dim")]
    pub max_width: u32,
    #[serde(default = "default_max_dim")]
    pub max_height: u32,
    #[serde(default)]
    pub generate_mipmaps: bool,
}

fn default_texture_format() -> String {
    "png".to_string()
}

fn default_max_dim() -> u32 {
    2048
}

impl Default for TextureTargetConfig {
    fn default() -> Self {
        Self {
            format: default_texture_format(),
            max_width: default_max_dim(),
            max_height: default_max_dim(),
            generate_mipmaps: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTargetConfig {
    #[serde(default = "default_audio_format")]
    pub format: String,
    #[serde(default = "default_bitrate")]
    pub bitrate: String,
}

fn default_audio_format() -> String {
    "ogg".to_string()
}

fn default_bitrate() -> String {
    "128k".to_string()
}

impl Default for AudioTargetConfig {
    fn default() -> Self {
        Self {
            format: default_audio_format(),
            bitrate: default_bitrate(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    pub pattern: String,
    pub convert: Option<String>,
    #[serde(default)]
    pub validate: bool,
    #[serde(default)]
    pub remove_original: bool,
    #[serde(default)]
    pub parameters: Option<HashMap<String, String>>,
}

impl PipelineConfig {
    /// Load from a YAML file path
    pub fn from_file(path: &std::path::Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::RiftError::Config(format!("Cannot read {}: {}", path.display(), e))
        })?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Find a rift.yml in the current directory or ancestors
    pub fn discover() -> crate::Result<Self> {
        let candidates = ["rift.yml", "rift.yaml", ".rift.yml"];
        for name in &candidates {
            let path = std::path::Path::new(name);
            if path.exists() {
                return Self::from_file(path);
            }
        }
        // Check parent directories
        if let Ok(cwd) = std::env::current_dir() {
            for parent in cwd.ancestors() {
                for name in &candidates {
                    let path = parent.join(name);
                    if path.exists() {
                        return Self::from_file(&path);
                    }
                }
            }
        }
        Err(crate::RiftError::Config(
            "No rift.yml found in current or parent directories".to_string(),
        ))
    }
}
