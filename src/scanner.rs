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

const HTML_EXTENSIONS: &[&str] = &["html", "htm"];
const PDF_EXTENSIONS: &[&str] = &["pdf"];

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

pub fn is_html_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        HTML_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}

pub fn is_pdf_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        PDF_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    } else {
        false
    }
}

pub fn is_supported_course_file(path: &Path) -> bool {
    is_video_file(path) || is_html_file(path) || is_pdf_file(path)
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

    // Strip known video, HTML & PDF extensions (.mp4, .mkv, .webm, .html, .htm, .pdf, etc.)
    for ext in VIDEO_EXTENSIONS.iter().chain(HTML_EXTENSIONS.iter()).chain(PDF_EXTENSIONS.iter()) {
        let suffix = format!(".{}", ext);
        if title.to_lowercase().ends_with(&suffix) {
            title = &title[..title.len() - suffix.len()];
            break;
        }
    }

    let with_spaces = title.replace('_', " ");
    with_spaces.trim().to_string()
}

pub fn normalize_path(path: &Path) -> String {
    let path_buf = if let Ok(canon) = path.canonicalize() {
        canon
    } else {
        path.to_path_buf()
    };
    let s = path_buf.to_string_lossy().to_string();
    let s_clean = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s_clean
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase()
}

pub fn normalize_rel_path(rel_path: &str) -> String {
    rel_path
        .replace('\\', "/")
        .trim_matches('/')
        .to_lowercase()
}

pub fn generate_deterministic_course_id(root_path: &Path) -> String {
    let norm = normalize_path(root_path);
    Uuid::new_v5(&Uuid::NAMESPACE_URL, norm.as_bytes()).to_string()
}

pub fn generate_deterministic_lecture_id(rel_path: &str) -> String {
    let norm = normalize_rel_path(rel_path);
    format!("lec_{}", Uuid::new_v5(&Uuid::NAMESPACE_OID, norm.as_bytes()).simple())
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
    existing_course: Option<&Course>,
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

    let course_title = if let Some(ec) = existing_course {
        ec.title.clone()
    } else {
        dir_path
            .file_name()
            .and_then(|f| f.to_str())
            .map(clean_section_title)
            .unwrap_or_else(|| "Imported Course".to_string())
    };

    // Group media & HTML files by top-level section folder
    let mut sections_map: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let mut seen_paths: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    for entry in WalkDir::new(dir_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_supported_course_file(path) {
            let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            if !seen_paths.insert(canon) {
                continue; // Avoid duplicate files from symlinks/junctions
            }

            let rel_path = path.strip_prefix(dir_path).unwrap_or(path);
            let section_key = match rel_path.components().next() {
                Some(std::path::Component::Normal(c)) if rel_path.components().count() > 1 => {
                    c.to_string_lossy().replace('\\', "/")
                }
                _ => "General / Root".to_string(),
            };

            sections_map
                .entry(section_key)
                .or_default()
                .push(path.to_path_buf());
        }
    }

    if sections_map.is_empty() {
        return Err(format!(
            "No video, HTML, or PDF files found in directory: {}. Supported formats: {}, {}, {}",
            dir_path.display(),
            VIDEO_EXTENSIONS.join(", "),
            HTML_EXTENSIONS.join(", "),
            PDF_EXTENSIONS.join(", ")
        ));
    }

    let total_items: usize = sections_map.values().map(|v| v.len()).sum();

    if let Some(ref cb) = on_progress {
        cb(crate::models::ScanProgress {
            current: 0,
            total: total_items,
            current_file: "Discovered media, HTML, and PDF files. Analyzing metadata...".to_string(),
            phase: "discovering".to_string(),
        });
    }

    let course_id = existing_course
        .map(|c| c.id.clone())
        .unwrap_or_else(|| generate_deterministic_course_id(dir_path));

    let mut current_item_idx = 0;
    let mut final_sections: Vec<Section> = Vec::new();

    if let Some(prev) = existing_course {
        // SMART MERGE:
        // 1. Walk existing sections, keeping all valid existing lectures and their user progress.
        // 2. Append only NEW lectures discovered on disk to their matching section.
        // 3. Append any completely NEW sections discovered on disk.
        let mut matched_section_keys: std::collections::HashSet<String> = std::collections::HashSet::new();

        for prev_sec in &prev.sections {
            let matching_key = sections_map.keys().find(|k| {
                let clean_k = if *k == "General / Root" {
                    "Introduction".to_string()
                } else {
                    clean_section_title(k)
                };
                clean_k.eq_ignore_ascii_case(&prev_sec.title)
                    || prev_sec.lectures.iter().any(|l| {
                        let norm_l = normalize_rel_path(&l.relative_path);
                        let norm_k = normalize_rel_path(k);
                        norm_l.starts_with(&norm_k)
                    })
            }).cloned();

            let mut merged_lectures: Vec<Lecture> = Vec::new();
            let mut existing_norm_rels: std::collections::HashSet<String> = std::collections::HashSet::new();

            // Retain all existing lectures in this section
            for lec in &prev_sec.lectures {
                let norm = normalize_rel_path(&lec.relative_path);
                existing_norm_rels.insert(norm.clone());

                // Retain as long as file still exists on disk or relative path exists
                let abs_p = Path::new(&lec.absolute_path);
                if abs_p.exists() || dir_path.join(&lec.relative_path).exists() {
                    merged_lectures.push(lec.clone());
                }
            }

            // If files on disk exist for this section, add only the NEW lectures
            if let Some(ref sec_key) = matching_key {
                matched_section_keys.insert(sec_key.clone());
                if let Some(disk_files) = sections_map.get(sec_key) {
                    let mut sorted_disk_files = disk_files.clone();
                    sorted_disk_files.sort_by(|a, b| {
                        let name_a = a.file_name().and_then(|f| f.to_str()).unwrap_or("");
                        let name_b = b.file_name().and_then(|f| f.to_str()).unwrap_or("");
                        natural_compare(name_a, name_b)
                    });

                    for file_path in sorted_disk_files {
                        current_item_idx += 1;
                        let rel_path = file_path
                            .strip_prefix(dir_path)
                            .unwrap_or(&file_path)
                            .to_string_lossy()
                            .replace('\\', "/");
                        let norm_rel = normalize_rel_path(&rel_path);

                        if existing_norm_rels.contains(&norm_rel) {
                            continue; // Already exists, never duplicate!
                        }

                        let file_name = file_path
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("lecture")
                            .to_string();

                        if let Some(ref cb) = on_progress {
                            cb(crate::models::ScanProgress {
                                current: current_item_idx,
                                total: total_items,
                                current_file: file_name.clone(),
                                phase: "indexing".to_string(),
                            });
                        }

                        let is_pdf = is_pdf_file(&file_path);
                        let is_html = is_html_file(&file_path);
                        let item_type = if is_pdf {
                            "pdf"
                        } else if is_html {
                            "html"
                        } else {
                            "video"
                        };

                        let duration = if is_pdf || is_html {
                            0
                        } else {
                            probe_video_duration(&file_path)
                        };

                        let new_lec = Lecture {
                            id: generate_deterministic_lecture_id(&rel_path),
                            title: clean_lecture_title(&file_name),
                            file_name,
                            relative_path: rel_path,
                            absolute_path: file_path.to_string_lossy().to_string(),
                            duration_seconds: duration,
                            order: merged_lectures.len() + 1,
                            item_type: item_type.to_string(),
                        };
                        merged_lectures.push(new_lec);
                        existing_norm_rels.insert(norm_rel);
                    }
                }
            }

            // Ensure orders in section are sequential
            for (idx, l) in merged_lectures.iter_mut().enumerate() {
                l.order = idx + 1;
            }

            let sec_dur: u64 = merged_lectures.iter().map(|l| l.duration_seconds).sum();
            final_sections.push(Section {
                id: prev_sec.id.clone(),
                title: prev_sec.title.clone(),
                order: prev_sec.order,
                lectures: merged_lectures,
                duration_seconds: sec_dur,
            });
        }

        // Handle BRAND NEW sections on disk
        let mut unmatched_keys: Vec<String> = sections_map
            .keys()
            .filter(|k| !matched_section_keys.contains(*k))
            .cloned()
            .collect();

        unmatched_keys.sort_by(|a, b| natural_compare(a, b));

        let mut next_sec_order = final_sections.iter().map(|s| s.order).max().unwrap_or(0);

        for new_key in unmatched_keys {
            next_sec_order += 1;
            let mut file_paths = sections_map.remove(&new_key).unwrap_or_default();
            file_paths.sort_by(|a, b| {
                let name_a = a.file_name().and_then(|f| f.to_str()).unwrap_or("");
                let name_b = b.file_name().and_then(|f| f.to_str()).unwrap_or("");
                natural_compare(name_a, name_b)
            });

            let sec_title = if new_key == "General / Root" {
                "Introduction".to_string()
            } else {
                clean_section_title(&new_key)
            };

            let mut new_lectures = Vec::new();
            for (lec_idx, file_path) in file_paths.into_iter().enumerate() {
                current_item_idx += 1;
                let file_name = file_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("lecture")
                    .to_string();

                if let Some(ref cb) = on_progress {
                    cb(crate::models::ScanProgress {
                        current: current_item_idx,
                        total: total_items,
                        current_file: file_name.clone(),
                        phase: "indexing".to_string(),
                    });
                }

                let is_pdf = is_pdf_file(&file_path);
                let is_html = is_html_file(&file_path);
                let item_type = if is_pdf {
                    "pdf"
                } else if is_html {
                    "html"
                } else {
                    "video"
                };

                let rel_path = file_path
                    .strip_prefix(dir_path)
                    .unwrap_or(&file_path)
                    .to_string_lossy()
                    .replace('\\', "/");

                let duration = if is_pdf || is_html {
                    0
                } else {
                    probe_video_duration(&file_path)
                };

                new_lectures.push(Lecture {
                    id: generate_deterministic_lecture_id(&rel_path),
                    title: clean_lecture_title(&file_name),
                    file_name,
                    relative_path: rel_path,
                    absolute_path: file_path.to_string_lossy().to_string(),
                    duration_seconds: duration,
                    order: lec_idx + 1,
                    item_type: item_type.to_string(),
                });
            }

            let sec_dur: u64 = new_lectures.iter().map(|l| l.duration_seconds).sum();
            final_sections.push(Section {
                id: format!("sec_{}", next_sec_order),
                title: sec_title,
                order: next_sec_order,
                lectures: new_lectures,
                duration_seconds: sec_dur,
            });
        }
    } else {
        // FIRST-TIME SCAN:
        let mut section_keys: Vec<String> = sections_map.keys().cloned().collect();
        section_keys.sort_by(|a, b| {
            if a == "General / Root" {
                Ordering::Less
            } else if b == "General / Root" {
                Ordering::Greater
            } else {
                natural_compare(a, b)
            }
        });

        for (sec_idx, sec_key) in section_keys.into_iter().enumerate() {
            let mut file_paths = sections_map.remove(&sec_key).unwrap_or_default();
            file_paths.sort_by(|a, b| {
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
            for (lec_idx, file_path) in file_paths.into_iter().enumerate() {
                current_item_idx += 1;
                let file_name = file_path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("lecture")
                    .to_string();

                if let Some(ref cb) = on_progress {
                    cb(crate::models::ScanProgress {
                        current: current_item_idx,
                        total: total_items,
                        current_file: file_name.clone(),
                        phase: "indexing".to_string(),
                    });
                }

                let is_pdf = is_pdf_file(&file_path);
                let is_html = is_html_file(&file_path);
                let item_type = if is_pdf {
                    "pdf"
                } else if is_html {
                    "html"
                } else {
                    "video"
                };

                let rel_path = file_path
                    .strip_prefix(dir_path)
                    .unwrap_or(&file_path)
                    .to_string_lossy()
                    .replace('\\', "/");

                let duration = if is_pdf || is_html {
                    0
                } else {
                    probe_video_duration(&file_path)
                };

                let lec_id = generate_deterministic_lecture_id(&rel_path);

                lectures.push(Lecture {
                    id: lec_id,
                    title: clean_lecture_title(&file_name),
                    file_name,
                    relative_path: rel_path,
                    absolute_path: file_path.to_string_lossy().to_string(),
                    duration_seconds: duration,
                    order: lec_idx + 1,
                    item_type: item_type.to_string(),
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
    }

    if let Some(ref cb) = on_progress {
        cb(crate::models::ScanProgress {
            current: total_items,
            total: total_items,
            current_file: "Finalizing course indexing...".to_string(),
            phase: "finishing".to_string(),
        });
    }

    let now = Utc::now().to_rfc3339();
    let created_at = existing_course.map(|c| c.created_at.clone()).unwrap_or_else(|| now.clone());

    Ok(Course {
        id: course_id,
        title: course_title,
        root_path: root_path_str,
        sections: final_sections,
        created_at,
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
            "04. Quiz.html",
            "05. Cheatsheet.pdf",
        ];

        list.sort_by(|a, b| natural_compare(a, b));

        assert_eq!(
            list,
            vec![
                "1. Lesson 1.mp4",
                "2. Lesson 2.mp4",
                "03. Lesson 3.mp4",
                "04. Quiz.html",
                "05. Cheatsheet.pdf",
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
        assert_eq!(clean_lecture_title("03_Cheat_Sheet.html"), "03 Cheat Sheet");
        assert_eq!(clean_lecture_title("index.htm"), "index");
        assert_eq!(clean_lecture_title("04_Slides_Notes.pdf"), "04 Slides Notes");
    }

    #[test]
    fn test_is_html_file() {
        assert!(is_html_file(Path::new("lesson.html")));
        assert!(is_html_file(Path::new("index.htm")));
        assert!(!is_html_file(Path::new("video.mp4")));
        assert!(!is_html_file(Path::new("document.pdf")));
    }

    #[test]
    fn test_is_pdf_file() {
        assert!(is_pdf_file(Path::new("document.pdf")));
        assert!(is_pdf_file(Path::new("notes.PDF")));
        assert!(!is_pdf_file(Path::new("lesson.html")));
        assert!(!is_pdf_file(Path::new("video.mp4")));
    }

    #[test]
    fn test_normalize_rel_path() {
        assert_eq!(
            normalize_rel_path(r"1 - Intro\01. Overview.mp4"),
            "1 - intro/01. overview.mp4"
        );
        assert_eq!(
            normalize_rel_path("1 - Intro/01. Overview.mp4"),
            "1 - intro/01. overview.mp4"
        );
    }

    #[test]
    fn test_deterministic_lecture_id_is_stable() {
        let id1 = generate_deterministic_lecture_id(r"Section 1\Lesson 1.mp4");
        let id2 = generate_deterministic_lecture_id("Section 1/Lesson 1.mp4");
        let id3 = generate_deterministic_lecture_id("section 1/lesson 1.mp4");
        assert_eq!(id1, id2);
        assert_eq!(id1, id3);
        assert!(id1.starts_with("lec_"));
    }

    #[test]
    fn test_deterministic_course_id_is_stable() {
        let id1 = generate_deterministic_course_id(Path::new("D:/courses/my_course"));
        let id2 = generate_deterministic_course_id(Path::new(r"d:\courses\my_course\"));
        assert_eq!(id1, id2);
    }
}
