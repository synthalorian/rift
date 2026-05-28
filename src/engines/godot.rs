use crate::config::TargetConfig;
use std::path::Path;

/// Generate a Godot .import file for a converted asset
pub fn generate_import(output_path: &Path, _target: &TargetConfig) -> crate::Result<()> {
    let ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let import_path = output_path.with_extension(format!("{}.import", ext));

    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "tga" => {
            let import = format!(
                r#"#[remap]
importer="texture"
type="CompressedTexture2D"
uid="uid://{uid}"
path="res://{path}"

[deps]

source_file="res://{path}"

[params]

compress/mode=0
compress/high_quality=true
detect_3d=true
svg/scale=1.0
"#,
                uid = generate_uid(output_path, 6),
                path = output_path.file_name().unwrap().to_string_lossy()
            );
            std::fs::write(&import_path, import)?;
        }
        "wav" | "ogg" | "mp3" => {
            let import = format!(
                r#"#[remap]
importer="wav"
type="AudioStreamWAV"
uid="uid://{uid}"
path="res://{path}"

[deps]

source_file="res://{path}"

[params]

force/8_bit=false
force/mono=false
force/max_rate=false
force/max_rate_hz=44100
edit/trim=false
edit/normalize=true
"#,
                uid = generate_uid(output_path, 6),
                path = output_path.file_name().unwrap().to_string_lossy()
            );
            std::fs::write(&import_path, import)?;
        }
        "fbx" | "gltf" | "glb" | "obj" => {
            let import = format!(
                r#"#[remap]
importer="scene"
type="PackedScene"
uid="uid://{uid}"
path="res://{path}"

[deps]

source_file="res://{path}"

[params]

import/import_as=SINGLE_SCENE
"#,
                uid = generate_uid(output_path, 6),
                path = output_path.file_name().unwrap().to_string_lossy()
            );
            std::fs::write(&import_path, import)?;
        }
        _ => {
            // Godot doesn't need .import for every type
        }
    }

    Ok(())
}

/// Generate a Godot-compatible UID (shorter than Unity's)
fn generate_uid(path: &Path, length: usize) -> String {
    use sha2::{Digest, Sha256};
    let path_str = path.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let hash = hex::encode(hasher.finalize());
    // Use alphanumeric subset
    hash[..length].to_string()
}
