use crate::RiftError;
use std::path::Path;

/// Validate a 3D model file for common issues
/// Since we can't parse FBX/glTF binary without heavy deps,
/// we do file-level sanity checks and size validation.
pub fn validate(path: &Path) -> crate::Result<()> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "fbx" => validate_fbx(path),
        "gltf" | "glb" => validate_gltf(path),
        "obj" => validate_obj(path),
        "blend" => Ok(()), // Too complex to validate without Blender API
        _ => Ok(()),       // Unknown format, skip validation
    }
}

fn validate_fbx(path: &Path) -> crate::Result<()> {
    let metadata = std::fs::metadata(path)?;
    let size = metadata.len();

    if size == 0 {
        return Err(RiftError::Validation(format!(
            "FBX file is empty: {}",
            path.display()
        )));
    }

    // Read first few bytes to check ASCII FBX header
    let mut buf = [0u8; 20];
    let mut file = std::fs::File::open(path)?;
    use std::io::Read;
    let n = file.read(&mut buf)?;

    if n > 0 {
        let header = String::from_utf8_lossy(&buf[..n.min(20)]);
        if !header.starts_with("; FBX") && !header.starts_with("Kaydara FBX") {
            // Binary FBX starts with [0x4B, 0x61, 0x79, 0x64, 0x61, 0x72, 0x61] = "Kaydara"
            if buf[0] != 0x4B || buf[1] != 0x61 {
                return Err(RiftError::Validation(format!(
                    "FBX file has invalid header: {}",
                    path.display()
                )));
            }
        }
    }

    if size > 500_000_000 {
        // 500MB
        return Err(RiftError::Validation(format!(
            "FBX file is very large ({}MB). Consider simplifying: {}",
            size / 1_000_000,
            path.display()
        )));
    }

    Ok(())
}

fn validate_gltf(path: &Path) -> crate::Result<()> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() == 0 {
        return Err(RiftError::Validation(format!(
            "GLTF file is empty: {}",
            path.display()
        )));
    }

    // For .gltf (text/JSON), try to parse minimal structure
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "gltf" {
        let content = std::fs::read_to_string(path)?;
        if !content.contains("\"asset\"") {
            return Err(RiftError::Validation(format!(
                "GLTF file missing required 'asset' field: {}",
                path.display()
            )));
        }
    }

    Ok(())
}

fn validate_obj(path: &Path) -> crate::Result<()> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() == 0 {
        return Err(RiftError::Validation(format!(
            "OBJ file is empty: {}",
            path.display()
        )));
    }

    // Quick scan for vertex data
    let content = std::fs::read_to_string(path)?;
    let has_vertices = content.lines().any(|l| l.starts_with("v "));
    if !has_vertices {
        return Err(RiftError::Validation(format!(
            "OBJ file has no vertex data: {}",
            path.display()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_fbx_valid_ascii() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.fbx");
        std::fs::write(&path, "; FBX 1.0 format test file\n").unwrap();
        assert!(validate_fbx(&path).is_ok());
    }

    #[test]
    fn test_validate_fbx_valid_binary() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.fbx");
        let header = b"Kaydara FBX Binary  ";
        std::fs::write(&path, header).unwrap();
        assert!(validate_fbx(&path).is_ok());
    }

    #[test]
    fn test_validate_fbx_bad_header() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.fbx");
        std::fs::write(&path, b"NOT A VALID FBX").unwrap();
        let result = validate_fbx(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("header"));
    }

    #[test]
    fn test_validate_gltf_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model.gltf");
        std::fs::write(&path, r#"{"asset": {"version": "2.0"}}"#).unwrap();
        assert!(validate_gltf(&path).is_ok());
    }

    #[test]
    fn test_validate_gltf_missing_asset() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.gltf");
        std::fs::write(&path, r#"{"meshes": []}"#).unwrap();
        let result = validate_gltf(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("asset"));
    }

    #[test]
    fn test_validate_gltf_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.gltf");
        std::fs::write(&path, b"").unwrap();
        let result = validate_gltf(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_obj_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model.obj");
        std::fs::write(
            &path,
            "# Simple cube\nv 0.0 0.0 0.0\nv 1.0 0.0 0.0\nf 1 2 3\n",
        )
        .unwrap();
        assert!(validate_obj(&path).is_ok());
    }

    #[test]
    fn test_validate_obj_no_vertices() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.obj");
        std::fs::write(&path, "# Just a comment\n").unwrap();
        let result = validate_obj(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("vertex"));
    }

    #[test]
    fn test_validate_obj_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.obj");
        std::fs::write(&path, b"").unwrap();
        let result = validate_obj(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_blend_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model.blend");
        std::fs::write(&path, b"BLENDER").unwrap();
        assert!(validate(&path).is_ok());
    }
}
