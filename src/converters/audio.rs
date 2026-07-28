use crate::config::TargetConfig;
use crate::RiftError;
use std::path::{Path, PathBuf};

/// Convert audio: validate WAV, compress to OGG, output to engine project dir
pub fn convert(source: &Path, target: &TargetConfig) -> crate::Result<PathBuf> {
    let fmt = &target.audio.format;

    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // If it's already the target format and in bounds, just copy
    if ext == *fmt {
        let output_path = target.project.join(source.file_name().unwrap());
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(source, &output_path)?;
        return Ok(output_path);
    }

    // For WAV source, validate it
    if ext == "wav" {
        validate_wav(source)?;
    }

    let output_name = format!("{}.{}", stem, fmt);
    let output_path = target.project.join(&output_name);

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    match fmt.as_str() {
        "ogg" => {
            if ext == "wav" {
                // Real OGG Vorbis encoding via ffmpeg
                encode_wav_to_ogg(source, &output_path, &target.audio.bitrate)?;
            } else {
                std::fs::copy(source, &output_path)?;
            }
        }
        "wav" => {
            // Validate and copy
            validate_wav(source)?;
            std::fs::copy(source, &output_path)?;
        }
        "mp3" | "flac" => {
            // Copy as-is for formats we can't encode
            std::fs::copy(source, &output_path)?;
        }
        _ => {
            // Fallback: copy as-is
            std::fs::copy(source, &output_path)?;
        }
    }

    Ok(output_path)
}

fn encode_wav_to_ogg(source: &Path, output: &Path, bitrate: &str) -> crate::Result<()> {
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let quality = bitrate_to_vorbis_quality(bitrate);

    let status = std::process::Command::new("ffmpeg")
        .args([
            "-i",
            &source.to_string_lossy(),
            "-codec:a",
            "libvorbis",
            "-q:a",
            &quality.to_string(),
            "-y", // overwrite output
            &output.to_string_lossy(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => {
            let msg = format!("ffmpeg exited with code {} for: {}", s, source.display());
            Err(RiftError::Conversion(msg))
        }
        Err(e) => {
            let msg = format!("ffmpeg not found or failed ({}): {}", e, source.display());
            Err(RiftError::Conversion(msg))
        }
    }
}

/// Map a bitrate string like "128k" to a Vorbis quality level (0.0–10.0).
/// Vorbis quality is logarithmic, so we map roughly:
///   64k → 2, 96k → 3, 128k → 4, 160k → 5, 192k → 6, 256k → 7, 320k → 8
fn bitrate_to_vorbis_quality(bitrate: &str) -> f64 {
    let trimmed = bitrate.trim().to_lowercase();
    let numeric: u32 = trimmed
        .trim_end_matches('k')
        .trim_end_matches("bps")
        .parse()
        .unwrap_or(128);

    match numeric {
        0..=48 => 1.0,
        49..=72 => 2.0,
        73..=96 => 3.0,
        97..=128 => 4.0,
        129..=160 => 5.0,
        161..=192 => 6.0,
        193..=256 => 7.0,
        257..=320 => 8.0,
        _ => 6.0,
    }
}

fn validate_wav(path: &Path) -> crate::Result<()> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() < 44 {
        return Err(RiftError::Conversion(format!(
            "WAV file too small ({} bytes) — not a valid WAV: {}",
            metadata.len(),
            path.display()
        )));
    }

    let reader = hound::WavReader::open(path)
        .map_err(|e| RiftError::Conversion(format!("Cannot parse WAV: {}", e)))?;

    let spec = reader.spec();
    let channels = spec.channels;
    let sample_rate = spec.sample_rate;

    if channels == 0 || channels > 8 {
        return Err(RiftError::Conversion(format!(
            "Unsupported channel count ({}): {}",
            channels,
            path.display()
        )));
    }

    if !(4000..=384000).contains(&sample_rate) {
        return Err(RiftError::Conversion(format!(
            "Unsupported sample rate ({} Hz): {}",
            sample_rate,
            path.display()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AudioTargetConfig, TargetConfig, TextureTargetConfig};
    use std::path::PathBuf;

    fn test_target() -> TargetConfig {
        TargetConfig {
            engine: "unity".into(),
            project: PathBuf::from("/tmp/rift-test-out"),
            textures: TextureTargetConfig::default(),
            audio: AudioTargetConfig::default(),
        }
    }

    #[test]
    fn test_validate_wav_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for _ in 0..44100 {
            writer.write_sample(0i16).unwrap();
        }
        writer.finalize().unwrap();
        assert!(validate_wav(&path).is_ok());
    }

    #[test]
    fn test_validate_wav_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.wav");
        std::fs::write(&path, b"").unwrap();
        let result = validate_wav(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));
    }

    #[test]
    fn test_validate_wav_bad_channels() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.wav");
        let spec = hound::WavSpec {
            channels: 99,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for _ in 0..99 {
            writer.write_sample(0i16).unwrap();
        }
        writer.finalize().unwrap();
        let result = validate_wav(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("channel"));
    }

    #[test]
    fn test_convert_audio_copy_already_ogg() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("test.ogg");
        std::fs::write(&source, b"fake ogg").unwrap();
        let out_dir = dir.path().join("out");
        let mut target = test_target();
        target.project = out_dir.clone();
        target.audio.format = "ogg".into();
        let result = convert(&source, &target).unwrap();
        assert_eq!(result.extension().unwrap(), "ogg");
    }

    #[test]
    fn test_convert_audio_wav_to_wav() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("test.wav");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&source, spec).unwrap();
        for _ in 0..48000 {
            writer.write_sample(0i16).unwrap();
        }
        writer.finalize().unwrap();
        let out_dir = dir.path().join("out");
        let mut target = test_target();
        target.project = out_dir.clone();
        target.audio.format = "wav".into();
        let result = convert(&source, &target).unwrap();
        assert_eq!(result.extension().unwrap(), "wav");
        assert!(result.exists());
    }

    #[test]
    fn test_encode_wav_to_ogg_via_ffmpeg() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("test.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&source, spec).unwrap();
        for _ in 0..44100 {
            writer.write_sample(0i16).unwrap();
        }
        writer.finalize().unwrap();

        let output = dir.path().join("out").join("test.ogg");
        let result = encode_wav_to_ogg(&source, &output, "128k");
        assert!(result.is_ok(), "ffmpeg encoding failed: {:?}", result.err());
        assert!(output.exists(), "OGG output file should exist");
        assert_eq!(output.extension().unwrap(), "ogg");

        // Verify real OGG header
        let header = std::fs::read(&output).unwrap();
        assert_eq!(&header[0..4], b"OggS", "OGG should have valid magic header");
    }

    #[test]
    fn test_encode_wav_to_ogg_via_convert() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("sound.wav");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&source, spec).unwrap();
        for _ in 0..44100 {
            writer.write_sample(0i16).unwrap();
        }
        writer.finalize().unwrap();

        let out_dir = dir.path().join("out");
        let mut target = test_target();
        target.project = out_dir.clone();
        target.audio.format = "ogg".into();
        target.audio.bitrate = "96k".into();

        let result = convert(&source, &target).unwrap();
        assert_eq!(result.extension().unwrap(), "ogg");
        assert!(result.exists());

        // Verify it's actually OGG, not a renamed WAV
        let header = std::fs::read(&result).unwrap();
        assert_eq!(&header[0..4], b"OggS", "Should be valid OGG container");
    }

    #[test]
    fn test_bitrate_to_quality() {
        assert_eq!(bitrate_to_vorbis_quality("64k"), 2.0);
        assert_eq!(bitrate_to_vorbis_quality("128k"), 4.0);
        assert_eq!(bitrate_to_vorbis_quality("192k"), 6.0);
        assert_eq!(bitrate_to_vorbis_quality("320k"), 8.0);
        assert_eq!(bitrate_to_vorbis_quality("96"), 3.0);
        assert_eq!(bitrate_to_vorbis_quality(""), 4.0); // default
    }
}
