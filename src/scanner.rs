use crate::models::{Course, Lecture, Section};
use chrono::Utc;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "m4v", "webm", "mkv", "mov", "avi", "ts", "flv", "wmv",
];

/// Smart natural sort comparator for alphanumeric strings (e.g. "1", "2", "10", "01 - intro.mp4", "1. Arrays")
pub fn natural_compare(a: &str, b: &str) -> Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    while let (Some(&ca), Some(&cb)) = (a_chars.peek(), b_chars.peek()) {
        if ca.is_ascii_digit() && cb.is_ascii_digit() {
            let mut num_a = String::new();
            while let Some(&c) = a_chars.peek() {
                if c.is_ascii_digit() {
                    num_a.push(c);
                    a_chars.next();
                } else {
                    break;
                }
            }

            let mut num_b = String::new();
            while let Some(&c) = b_chars.peek() {
                if c.is_ascii_digit() {
                    num_b.push(c);
                    b_chars.next();
                } else {
                    break;
                }
            }

            let val_a = num_a.parse::<u64>().unwrap_or(0);
            let val_b = num_b.parse::<u64>().unwrap_or(0);

            match val_a.cmp(&val_b) {
                Ordering::Equal => {
                    let len_cmp = num_a.len().cmp(&num_b.len());
                    if len_cmp != Ordering::Equal {
                        return len_cmp;
                    }
                }
                non_eq => return non_eq,
            }
        } else {
            let ca_lower = ca.to_lowercase().next().unwrap_or(ca);
            let cb_lower = cb.to_lowercase().next().unwrap_or(cb);
            match ca_lower.cmp(&cb_lower) {
                Ordering::Equal => {
                    a_chars.next();
                    b_chars.next();
                }
                non_eq => return non_eq,
            }
        }
    }

    a.len().cmp(&b.len())
}

pub fn is_video_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}

pub fn clean_section_title(raw_folder_name: &str) -> String {
    let raw = raw_folder_name.trim();
    // Take the last folder component if path like "Nested/1. Arrays"
    let last_part = raw
        .split(&['/', '\\'][..])
        .filter(|s| !s.is_empty())
        .last()
        .unwrap_or(raw);

    let with_spaces = last_part.replace('_', " ");
    with_spaces.trim().to_string()
}

pub fn clean_lecture_title(file_name: &str) -> String {
    let mut title = file_name.trim();

    // Strip known video extensions only (.mp4, .mkv, .webm, etc.)
    for ext in VIDEO_EXTENSIONS {
        let suffix = format!(".{}", ext);
        if title.to_lowercase().ends_with(&suffix) {
            title = &title[..title.len() - suffix.len()];
            break;
        }
    }

    let with_spaces = title.replace('_', " ");
    with_spaces.trim().to_string()
}

pub fn probe_video_duration(path: &Path) -> u64 {
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;

    let mut cmd = std::process::Command::new("ffprobe");
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

    cmd.args([
        "-v",
        "error",
        "-show_entries",
        "format=duration",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
    ]);
    cmd.arg(path);

    if let Ok(output) = cmd.output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Ok(secs) = text.trim().parse::<f64>() {
                if secs > 0.0 {
                    return secs.round() as u64;
                }
            }
        }
    }
    0
}

pub fn scan_course_directory<F>(
    dir_path: &Path,
    on_progress: Option<F>,
) -> Result<Course, String>
where
    F: Fn(crate::models::ScanProgress) + Send + Sync + 'static,
{
    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", dir_path.display()));
    }
    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {}", dir_path.display()));
    }

    let root_path_str = dir_path
        .to_str()
        .ok_or_else(|| "Invalid UTF-8 in directory path".to_string())?
        .to_string();

    let course_title = dir_path
        .file_name()
        .and_then(|f| f.to_str())
        .map(clean_section_title)
        .unwrap_or_else(|| "Imported Course".to_string());

    // Group videos by section folder
    // Map: Section relative path -> Vec of video files
    let mut sections_map: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();

    for entry in WalkDir::new(dir_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_video_file(path) {
            let rel_path = path.strip_prefix(dir_path).unwrap_or(path);
            let parent = rel_path.parent();

            let section_key = match parent {
                Some(p) if p.as_os_str().is_empty() => "General / Root".to_string(),
                Some(p) => p.to_string_lossy().replace('\\', "/"),
                None => "General / Root".to_string(),
            };

            sections_map
                .entry(section_key)
                .or_default()
                .push(path.to_path_buf());
        }
    }

    if sections_map.is_empty() {
        return Err(format!(
            "No video files found in directory: {}. Supported formats: {}",
            dir_path.display(),
            VIDEO_EXTENSIONS.join(", ")
        ));
    }

    let total_videos: usize = sections_map.values().map(|v| v.len()).sum();

    if let Some(ref cb) = on_progress {
        cb(crate::models::ScanProgress {
            current: 0,
            total: total_videos,
            current_file: "Discovered video files. Analyzing video metadata...".to_string(),
            phase: "discovering".to_string(),
        });
    }

    // Sort sections naturally
    let mut section_keys: Vec<String> = sections_map.keys().cloned().collect();
    section_keys.sort_by(|a, b| {
        // Keep root/general at the start if present
        if a == "General / Root" {
            Ordering::Less
        } else if b == "General / Root" {
            Ordering::Greater
        } else {
            natural_compare(a, b)
        }
    });

    let course_id = Uuid::new_v4().to_string();
    let mut final_sections = Vec::new();
    let mut current_video_idx = 0;

    for (sec_idx, sec_key) in section_keys.into_iter().enumerate() {
        let mut video_paths = sections_map.remove(&sec_key).unwrap_or_default();

        // Sort videos naturally by filename
        video_paths.sort_by(|a, b| {
            let name_a = a.file_name().and_then(|f| f.to_str()).unwrap_or("");
            let name_b = b.file_name().and_then(|f| f.to_str()).unwrap_or("");
            natural_compare(name_a, name_b)
        });

        let sec_title = if sec_key == "General / Root" {
            "Introduction".to_string()
        } else {
            clean_section_title(&sec_key)
        };

        let mut lectures = Vec::new();
        for (lec_idx, vid_path) in video_paths.into_iter().enumerate() {
            current_video_idx += 1;
            let file_name = vid_path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("lecture.mp4")
                .to_string();

            if let Some(ref cb) = on_progress {
                cb(crate::models::ScanProgress {
                    current: current_video_idx,
                    total: total_videos,
                    current_file: file_name.clone(),
                    phase: "indexing".to_string(),
                });
            }

            let rel_path = vid_path
                .strip_prefix(dir_path)
                .unwrap_or(&vid_path)
                .to_string_lossy()
                .replace('\\', "/");

            let abs_path = vid_path.to_string_lossy().to_string();
            let title = clean_lecture_title(&file_name);
            let duration = probe_video_duration(&vid_path);

            // Generate deterministic ID from relative path
            let lec_id = format!("{}_{}_{}", sec_idx + 1, lec_idx + 1, Uuid::new_v4().simple());

            lectures.push(Lecture {
                id: lec_id,
                title,
                file_name,
                relative_path: rel_path,
                absolute_path: abs_path,
                duration_seconds: duration,
                order: lec_idx + 1,
            });
        }

        let sec_duration: u64 = lectures.iter().map(|l| l.duration_seconds).sum();

        final_sections.push(Section {
            id: format!("sec_{}", sec_idx + 1),
            title: sec_title,
            order: sec_idx + 1,
            lectures,
            duration_seconds: sec_duration,
        });
    }

    if let Some(ref cb) = on_progress {
        cb(crate::models::ScanProgress {
            current: total_videos,
            total: total_videos,
            current_file: "Finalizing course indexing...".to_string(),
            phase: "finishing".to_string(),
        });
    }

    let now = Utc::now().to_rfc3339();

    Ok(Course {
        id: course_id,
        title: course_title,
        root_path: root_path_str,
        sections: final_sections,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_compare() {
        let mut list = vec![
            "10. Lesson 10.mp4",
            "1. Lesson 1.mp4",
            "2. Lesson 2.mp4",
            "20. Lesson 20.mp4",
            "03. Lesson 3.mp4",
        ];

        list.sort_by(|a, b| natural_compare(a, b));

        assert_eq!(
            list,
            vec![
                "1. Lesson 1.mp4",
                "2. Lesson 2.mp4",
                "03. Lesson 3.mp4",
                "10. Lesson 10.mp4",
                "20. Lesson 20.mp4",
            ]
        );
    }

    #[test]
    fn test_clean_section_title() {
        assert_eq!(clean_section_title("1. Arrays"), "1. Arrays");
        assert_eq!(clean_section_title("02. Linked Lists"), "02. Linked Lists");
        assert_eq!(clean_section_title("Part 1/3. Dynamic Programming"), "3. Dynamic Programming");
        assert_eq!(clean_section_title("Section_04_Trees_And_Graphs"), "Section 04 Trees And Graphs");
    }

    #[test]
    fn test_clean_lecture_title() {
        assert_eq!(clean_lecture_title("1. Arrays.mp4"), "1. Arrays");
        assert_eq!(clean_lecture_title("01. Introduction to Rust.mp4"), "01. Introduction to Rust");
        assert_eq!(clean_lecture_title("02_Memory_Management.mkv"), "02 Memory Management");
    }
}
