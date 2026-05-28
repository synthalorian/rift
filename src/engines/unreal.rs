use crate::config::TargetConfig;
use std::path::Path;

/// Generate Unreal Engine import settings (JSON-based)
/// Unreal stores import settings in .json files alongside assets
pub fn generate_import_settings(output_path: &Path, target: &TargetConfig) -> crate::Result<()> {
    let ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let settings_path = output_path.with_extension(format!("{}.json", ext));

    let settings = match ext.as_str() {
        "png" | "jpg" | "jpeg" | "tga" | "bmp" | "webp" => {
            serde_json::json!({
                "AssetImportSettings": {
                    "TextureSettings": {
                        "CompressionSettings": "Default",
                        "MaxTextureSize": target.textures.max_width.max(target.textures.max_height),
                        "TextureGroup": "TEXTUREGROUP_World",
                        "sRGB": true,
                        "MipGenSettings": if target.textures.generate_mipmaps { "TMGS_FromTextureGroup" } else { "TMGS_NoMipmaps" }
                    }
                }
            })
        }
        "fbx" | "obj" => {
            serde_json::json!({
                "AssetImportSettings": {
                    "ModelSettings": {
                        "ImportContentType": "All",
                        "GenerateLightmapUVs": true,
                        "AutoComputeLODs": true,
                        "NumberOfLODs": 4
                    }
                }
            })
        }
        "wav" | "ogg" | "flac" => {
            serde_json::json!({
                "AssetImportSettings": {
                    "SoundSettings": {
                        "LoadingBehavior": "LoadOnDemand",
                        "CompressionQuality": 80
                    }
                }
            })
        }
        _ => {
            serde_json::json!({})
        }
    };

    let content = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&settings_path, content)?;

    Ok(())
}
