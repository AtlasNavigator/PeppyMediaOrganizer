use std::path::Path;
use std::process::Command;
use serde_json::Value;

use crate::db::models::MediaStreamInfo;

/// Tries to find an available ffprobe executable.
pub fn detect_ffprobe_binary(configured: Option<&str>) -> Option<String> {
    if let Some(custom) = configured {
        let trimmed = custom.trim();
        if !trimmed.is_empty() && Path::new(trimmed).exists() {
            return Some(trimmed.to_string());
        }
    }

    // Check system PATH
    if let Ok(output) = Command::new("ffprobe").arg("-version").output() {
        if output.status.success() {
            return Some("ffprobe".to_string());
        }
    }

    // Common standard Linux / Unix paths
    for p in &["/usr/bin/ffprobe", "/usr/local/bin/ffprobe", "/opt/homebrew/bin/ffprobe"] {
        if Path::new(p).exists() {
            return Some(p.to_string());
        }
    }

    None
}

/// Executes ffprobe on a media file and parses its JSON stream data into a `MediaStreamInfo`.
pub fn run_ffprobe(
    binary: &str,
    file_path: &Path,
    media_type: &str,
    media_id: i64,
) -> Result<MediaStreamInfo, String> {
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", file_path.display()));
    }

    if let Ok(meta) = file_path.metadata() {
        if meta.len() == 0 {
            return Err(format!("File is 0 bytes (cannot be probed by ffprobe): {}", file_path.display()));
        }
    }

    let output = Command::new(binary)
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(file_path)
        .output()
        .map_err(|e| format!("Failed to spawn ffprobe: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffprobe exited with status {:?}: {}", output.status.code(), err_msg));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    parse_ffprobe_json(&json_str, &file_path.to_string_lossy(), media_type, media_id)
}

/// Parses ffprobe JSON output into `MediaStreamInfo`.
pub fn parse_ffprobe_json(
    json_str: &str,
    file_path: &str,
    media_type: &str,
    media_id: i64,
) -> Result<MediaStreamInfo, String> {
    let root: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON from ffprobe: {}", e))?;

    let format_obj = root.get("format");
    let container = format_obj
        .and_then(|f| f.get("format_name"))
        .and_then(|v| v.as_str())
        .map(|s| {
            // Clean common multi-container names like "matroska,webm" -> "mkv"
            if s.contains("matroska") {
                "mkv".to_string()
            } else if s.contains("mp4") {
                "mp4".to_string()
            } else if s.contains("avi") {
                "avi".to_string()
            } else {
                s.to_string()
            }
        });

    let duration_sec = format_obj
        .and_then(|f| f.get("duration"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    let overall_bitrate = format_obj
        .and_then(|f| f.get("bit_rate"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .map(|b| (b / 1000) as i32);

    let empty_vec = Vec::new();
    let streams = root
        .get("streams")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_vec);

    let mut video_codec: Option<String> = None;
    let mut width: Option<i32> = None;
    let mut height: Option<i32> = None;
    let mut aspect_ratio: Option<String> = None;
    let mut hdr_format: Option<String> = None;

    let mut audio_codec: Option<String> = None;
    let mut audio_channels: Option<i32> = None;
    let mut audio_layout: Option<String> = None;
    let mut audio_languages: Vec<String> = Vec::new();
    let mut subtitle_languages: Vec<String> = Vec::new();

    for stream in streams {
        let codec_type = stream.get("codec_type").and_then(|v| v.as_str()).unwrap_or("");

        if codec_type == "video" && video_codec.is_none() {
            let codec = stream.get("codec_name").and_then(|v| v.as_str()).unwrap_or("");
            video_codec = Some(normalize_video_codec(codec));

            width = stream.get("width").and_then(|v| v.as_i64()).map(|w| w as i32);
            height = stream.get("height").and_then(|v| v.as_i64()).map(|h| h as i32);

            if let Some(dar) = stream.get("display_aspect_ratio").and_then(|v| v.as_str()) {
                if !dar.is_empty() && dar != "0:1" {
                    aspect_ratio = Some(dar.to_string());
                }
            }

            // Detect HDR / Dynamic Range
            let color_transfer = stream.get("color_transfer").and_then(|v| v.as_str()).unwrap_or("");
            let color_primaries = stream.get("color_primaries").and_then(|v| v.as_str()).unwrap_or("");
            let side_data = stream.get("side_data_list").and_then(|v| v.as_array());

            let mut has_dovi = false;
            if let Some(sides) = side_data {
                for side in sides {
                    if let Some(side_type) = side.get("side_data_type").and_then(|v| v.as_str()) {
                        if side_type.to_lowercase().contains("dovi") || side_type.to_lowercase().contains("dolby vision") {
                            has_dovi = true;
                            break;
                        }
                    }
                }
            }

            if has_dovi {
                hdr_format = Some("Dolby Vision".to_string());
            } else if color_transfer == "smpte2084" || color_primaries == "bt2020" {
                hdr_format = Some("HDR10".to_string());
            } else if color_transfer == "arib-std-b67" {
                hdr_format = Some("HLG".to_string());
            } else {
                hdr_format = Some("SDR".to_string());
            }
        } else if codec_type == "audio" {
            if audio_codec.is_none() {
                let codec = stream.get("codec_name").and_then(|v| v.as_str()).unwrap_or("");
                audio_codec = Some(normalize_audio_codec(codec));
                audio_channels = stream.get("channels").and_then(|v| v.as_i64()).map(|c| c as i32);
                audio_layout = stream.get("channel_layout").and_then(|v| v.as_str()).map(|s| s.to_string());
            }

            if let Some(tags) = stream.get("tags") {
                if let Some(lang) = tags.get("language").and_then(|v| v.as_str()) {
                    if !audio_languages.contains(&lang.to_string()) {
                        audio_languages.push(lang.to_string());
                    }
                }
            }
        } else if codec_type == "subtitle" {
            if let Some(tags) = stream.get("tags") {
                if let Some(lang) = tags.get("language").and_then(|v| v.as_str()) {
                    if !subtitle_languages.contains(&lang.to_string()) {
                        subtitle_languages.push(lang.to_string());
                    }
                }
            }
        }
    }

    Ok(MediaStreamInfo {
        id: 0,
        media_type: media_type.to_string(),
        media_id,
        file_path: file_path.to_string(),
        container,
        video_codec,
        width,
        height,
        aspect_ratio,
        hdr_format,
        audio_codec,
        audio_channels,
        audio_layout,
        audio_languages: if audio_languages.is_empty() { None } else { Some(audio_languages.join(",")) },
        subtitle_languages: if subtitle_languages.is_empty() { None } else { Some(subtitle_languages.join(",")) },
        bitrate_kbps: overall_bitrate,
        duration_sec,
        probe_status: "probed".to_string(),
        probed_at: None,
    })
}

fn normalize_video_codec(raw: &str) -> String {
    let lower = raw.to_lowercase();
    match lower.as_str() {
        "h264" | "avc" => "H.264".to_string(),
        "hevc" | "h265" => "HEVC (H.265)".to_string(),
        "av01" | "av1" => "AV1".to_string(),
        "vp9" => "VP9".to_string(),
        "mpeg4" => "MPEG-4".to_string(),
        "vc1" => "VC-1".to_string(),
        _ => raw.to_uppercase(),
    }
}

fn normalize_audio_codec(raw: &str) -> String {
    let lower = raw.to_lowercase();
    match lower.as_str() {
        "dts" => "DTS".to_string(),
        "eac3" => "Dolby Digital Plus (EAC3)".to_string(),
        "ac3" => "Dolby Digital (AC3)".to_string(),
        "truehd" => "Dolby TrueHD".to_string(),
        "flac" => "FLAC".to_string(),
        "aac" => "AAC".to_string(),
        "opus" => "Opus".to_string(),
        "mp3" => "MP3".to_string(),
        _ => raw.to_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ffprobe_sample_json() {
        let sample = r#"{
            "streams": [
                {
                    "codec_name": "hevc",
                    "codec_type": "video",
                    "width": 3840,
                    "height": 2160,
                    "display_aspect_ratio": "16:9",
                    "color_transfer": "smpte2084",
                    "color_primaries": "bt2020",
                    "side_data_list": [
                        { "side_data_type": "DOVI configuration record" }
                    ]
                },
                {
                    "codec_name": "truehd",
                    "codec_type": "audio",
                    "channels": 8,
                    "channel_layout": "7.1",
                    "tags": { "language": "eng" }
                },
                {
                    "codec_name": "subrip",
                    "codec_type": "subtitle",
                    "tags": { "language": "eng" }
                }
            ],
            "format": {
                "format_name": "matroska,webm",
                "duration": "7200.5",
                "bit_rate": "25000000"
            }
        }"#;

        let info = parse_ffprobe_json(sample, "/movies/Film.mkv", "movie", 42).unwrap();
        assert_eq!(info.video_codec.as_deref(), Some("HEVC (H.265)"));
        assert_eq!(info.width, Some(3840));
        assert_eq!(info.height, Some(2160));
        assert_eq!(info.hdr_format.as_deref(), Some("Dolby Vision"));
        assert_eq!(info.audio_codec.as_deref(), Some("Dolby TrueHD"));
        assert_eq!(info.audio_channels, Some(8));
        assert_eq!(info.audio_layout.as_deref(), Some("7.1"));
        assert_eq!(info.audio_languages.as_deref(), Some("eng"));
        assert_eq!(info.subtitle_languages.as_deref(), Some("eng"));
        assert_eq!(info.bitrate_kbps, Some(25000));
        assert_eq!(info.container.as_deref(), Some("mkv"));
    }
}
