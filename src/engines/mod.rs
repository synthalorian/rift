pub mod unity;
pub mod godot;
pub mod unreal;

use crate::config::TargetConfig;
use std::path::Path;

/// Generate appropriate engine metadata for a converted asset
pub fn generate_meta(output_path: &Path, target: &TargetConfig) -> crate::Result<()> {
    match target.engine.as_str() {
        "unity" => unity::generate_meta(output_path, target),
        "godot" => godot::generate_import(output_path, target),
        "unreal" => unreal::generate_import_settings(output_path, target),
        _ => Ok(()), // Unknown engine, skip
    }
}
