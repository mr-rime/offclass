use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lecture {
    pub id: String,
    pub title: String,
    pub file_name: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub duration_seconds: u64,
    pub order: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub title: String,
    pub order: usize,
    pub lectures: Vec<Lecture>,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseStats {
    pub total_sections: usize,
    pub total_lectures: usize,
    pub total_duration_seconds: u64,
    pub completed_lectures: usize,
    pub progress_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub lecture_id: String,
    pub lecture_title: String,
    pub timestamp_seconds: f64,
    pub text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CourseProgress {
    pub course_id: String,
    pub completed_lecture_ids: HashSet<String>,
    pub last_played_lecture_id: Option<String>,
    pub playback_positions: HashMap<String, f64>,
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: String,
    pub title: String,
    pub root_path: String,
    pub sections: Vec<Section>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseDetail {
    pub course: Course,
    pub progress: CourseProgress,
    pub stats: CourseStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseSummary {
    pub id: String,
    pub title: String,
    pub root_path: String,
    pub stats: CourseStats,
    pub last_played_lecture_id: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub theme: String, // "dark" or "light"
    pub auto_play_next: bool,
    pub playback_speed: f32,
    pub volume: f32,
    pub active_course_id: Option<String>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            auto_play_next: true,
            playback_speed: 1.0,
            volume: 1.0,
            active_course_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub phase: String,
}

