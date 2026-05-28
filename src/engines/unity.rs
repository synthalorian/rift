use crate::config::TargetConfig;
use std::path::Path;

/// Generate a Unity .meta file for a converted asset
pub fn generate_meta(output_path: &Path, _target: &TargetConfig) -> crate::Result<()> {
    let meta_path = output_path.with_extension(format!(
        "{}.meta",
        output_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
    ));

    let stub_guid = generate_stable_guid(output_path);
    let ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let importer_type = match ext.as_str() {
        "png" | "jpg" | "jpeg" | "tga" | "psd" | "tiff" | "bmp" | "webp" => "TextureImporter",
        "fbx" | "obj" | "blend" | "dae" | "3ds" | "dxf" => "ModelImporter",
        "wav" | "mp3" | "ogg" | "aiff" | "flac" => "AudioImporter",
        "mat" => "Material",
        "prefab" => "PrefabInstance",
        _ => "NativeFormatImporter",
    };

    let meta_content = format!(
        "fileFormatVersion: 2\nguid: {guid}\n{importer}:\n  externalObjects: {{}}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n",
        guid = stub_guid,
        importer = importer_type
    );

    std::fs::write(&meta_path, meta_content)?;

    Ok(())
}

/// Generate a deterministic (but unique) GUID from the file path
fn generate_stable_guid(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let path_str = path.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let hash = hex::encode(hasher.finalize());
    // Unity GUIDs are 32 hex chars
    hash[..32].to_string()
}
