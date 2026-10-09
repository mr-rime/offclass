use std::fs;
use std::path::{Path, PathBuf};

/// Convert standard SubRip (.srt) or similar text subtitles to WebVTT (.vtt) format
pub fn srt_to_vtt(srt_content: &str) -> String {
    let mut vtt = String::from("WEBVTT\n\n");
    for line in srt_content.lines() {
        if line.contains("-->") {
            // Replace commas with dots in timestamps: 00:00:01,500 --> 00:00:04,200 => 00:00:01.500 --> 00:00:04.200
            let fixed = line.replace(',', ".");
            vtt.push_str(&fixed);
            vtt.push('\n');
        } else {
            vtt.push_str(line);
            vtt.push('\n');
        }
    }
    vtt
}

/// Look for external subtitle files in the same directory as the video file
pub fn find_external_subtitle_file(video_path: &Path) -> Option<PathBuf> {
    let parent = video_path.parent()?;
    let video_stem = video_path.file_stem()?.to_string_lossy().to_lowercase();

    let extensions = ["vtt", "srt", "sub", "sbv", "ass"];

    // 1. Check exact match: <stem>.<ext> or <stem>.en.<ext> or <stem>_en.<ext>
    if let Some(stem_str) = video_path.file_stem().and_then(|s| s.to_str()) {
        for ext in &extensions {
            let candidate = parent.join(format!("{}.{}", stem_str, ext));
            if candidate.is_file() {
                return Some(candidate);
            }
            let candidate_en = parent.join(format!("{}.en.{}", stem_str, ext));
            if candidate_en.is_file() {
                return Some(candidate_en);
            }
            let candidate_en2 = parent.join(format!("{}_en.{}", stem_str, ext));
            if candidate_en2.is_file() {
                return Some(candidate_en2);
            }
        }
    }

    // 2. Scan directory for any subtitle file whose stem matches the video stem
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext.to_lowercase().as_str()) {
                        let sub_stem = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_lowercase())
                            .unwrap_or_default();
                        if sub_stem.starts_with(&video_stem) || video_stem.starts_with(&sub_stem) {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Extract embedded subtitle stream using ffmpeg into WebVTT format
pub fn extract_embedded_subtitles(video_path: &Path) -> Option<String> {
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;

    let mut cmd = std::process::Command::new("ffmpeg");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    cmd.args([
        "-v",
        "error",
        "-i",
    ]);
    cmd.arg(video_path);
    cmd.args([
        "-map",
        "0:s:0?",
        "-f",
        "webvtt",
        "-",
    ]);

    let output = cmd.output().ok()?;
    if output.status.success() {
        let vtt = String::from_utf8_lossy(&output.stdout).to_string();
        if vtt.trim().starts_with("WEBVTT") && vtt.trim().len() > 10 {
            return Some(vtt);
        }
    }

    None
}

/// Get subtitle content for a lecture, extracting and caching if needed
pub fn get_or_extract_subtitle(
    course_id: &str,
    lecture_id: &str,
    video_path: &Path,
) -> Option<String> {
    // 1. Check cache in data/captions/<course_id>/<lecture_id>.vtt
    let cache_dir = Path::new("data").join("captions").join(course_id);
    let cache_file = cache_dir.join(format!("{}.vtt", lecture_id));

    if cache_file.is_file() {
        if let Ok(content) = fs::read_to_string(&cache_file) {
            if !content.trim().is_empty() {
                return Some(content);
            }
        }
    }

    // 2. Check external subtitle file next to the video
    if let Some(sub_path) = find_external_subtitle_file(video_path) {
        if let Ok(content) = fs::read_to_string(&sub_path) {
            let is_vtt = sub_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase() == "vtt")
                .unwrap_or(false);

            let vtt = if is_vtt {
                content
            } else {
                srt_to_vtt(&content)
            };

            // Save to cache
            let _ = fs::create_dir_all(&cache_dir);
            let _ = fs::write(&cache_file, &vtt);
            return Some(vtt);
        }
    }

    // 3. Try extracting embedded subtitle stream via ffmpeg
    if let Some(vtt) = extract_embedded_subtitles(video_path) {
        let _ = fs::create_dir_all(&cache_dir);
        let _ = fs::write(&cache_file, &vtt);
        return Some(vtt);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srt_to_vtt() {
        let srt = "1\n00:00:01,234 --> 00:00:04,567\nHello World\n\n2\n00:00:05,000 --> 00:00:08,000\nRust is great!";
        let vtt = srt_to_vtt(srt);

        assert!(vtt.starts_with("WEBVTT\n\n"));
        assert!(vtt.contains("00:00:01.234 --> 00:00:04.567"));
        assert!(vtt.contains("Hello World"));
        assert!(vtt.contains("00:00:05.000 --> 00:00:08.000"));
        assert!(vtt.contains("Rust is great!"));
    }
}
