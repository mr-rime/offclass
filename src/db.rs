use crate::models::{
    Course, CourseDetail, CourseProgress, CourseStats, CourseSummary, Note, UserSettings,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseSchema {
    pub courses: HashMap<String, Course>,
    pub progress: HashMap<String, CourseProgress>,
    pub settings: UserSettings,
}

#[derive(Debug, Clone)]
pub struct AppDatabase {
    pub file_path: PathBuf,
    pub data: DatabaseSchema,
}

pub type SharedDb = Arc<RwLock<AppDatabase>>;

impl AppDatabase {
    pub fn init() -> Self {
        let db_path = Self::get_db_path();
        if let Some(parent) = db_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut schema = if db_path.exists() {
            match fs::read_to_string(&db_path) {
                Ok(content) => serde_json::from_str::<DatabaseSchema>(&content).unwrap_or_else(|e| {
                    error!("Failed to parse database file, initializing default: {}", e);
                    DatabaseSchema::default()
                }),
                Err(e) => {
                    error!("Failed to read database file: {}", e);
                    DatabaseSchema::default()
                }
            }
        } else {
            DatabaseSchema::default()
        };

        // Auto-repair section titles and probe missing lecture durations
        let mut repaired = false;
        for course in schema.courses.values_mut() {
            for sec in &mut course.sections {
                if let Some(first_lec) = sec.lectures.first() {
                    let rel_path = Path::new(&first_lec.relative_path);
                    if let Some(parent) = rel_path.parent() {
                        let parent_str = parent.to_string_lossy();
                        if !parent_str.is_empty() {
                            let correct_title = crate::scanner::clean_section_title(&parent_str);
                            if sec.title != correct_title && !correct_title.is_empty() {
                                sec.title = correct_title;
                                repaired = true;
                            }
                        }
                    }
                }

                for lec in &mut sec.lectures {
                    if lec.duration_seconds == 0 {
                        let path = Path::new(&lec.absolute_path);
                        if path.exists() {
                            let dur = crate::scanner::probe_video_duration(path);
                            if dur > 0 {
                                lec.duration_seconds = dur;
                                repaired = true;
                            }
                        }
                    }
                }

                let sec_dur: u64 = sec.lectures.iter().map(|l| l.duration_seconds).sum();
                if sec.duration_seconds != sec_dur {
                    sec.duration_seconds = sec_dur;
                    repaired = true;
                }
            }
        }

        let db = Self {
            file_path: db_path,
            data: schema,
        };

        if repaired {
            let _ = db.save();
        }

        db
    }

    fn get_db_path() -> PathBuf {
        // Use local data directory first
        let local_path = Path::new("data").join("db.json");
        local_path
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.data)
            .map_err(|e| format!("Serialization error: {}", e))?;

        if let Some(parent) = self.file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let tmp_path = self.file_path.with_extension("tmp");
        fs::write(&tmp_path, json).map_err(|e| format!("Write error: {}", e))?;
        fs::rename(&tmp_path, &self.file_path).map_err(|e| format!("Rename error: {}", e))?;
        Ok(())
    }

    pub fn compute_stats(&self, course: &Course, progress: &CourseProgress) -> CourseStats {
        let total_sections = course.sections.len();
        let mut total_lectures = 0;
        let mut total_duration_seconds = 0;

        for sec in &course.sections {
            total_lectures += sec.lectures.len();
            for lec in &sec.lectures {
                total_duration_seconds += lec.duration_seconds;
            }
        }

        let completed_lectures = progress.completed_lecture_ids.len();
        let progress_percent = if total_lectures > 0 {
            (completed_lectures as f32 / total_lectures as f32) * 100.0
        } else {
            0.0
        };

        CourseStats {
            total_sections,
            total_lectures,
            total_duration_seconds,
            completed_lectures,
            progress_percent,
        }
    }

    pub fn add_course(&mut self, course: Course) -> Result<CourseDetail, String> {
        let course_id = course.id.clone();
        self.data.settings.active_course_id = Some(course_id.clone());

        let progress = self
            .data
            .progress
            .entry(course_id.clone())
            .or_insert_with(|| CourseProgress {
                course_id: course_id.clone(),
                ..Default::default()
            })
            .clone();

        let stats = self.compute_stats(&course, &progress);
        self.data.courses.insert(course_id.clone(), course.clone());
        self.save()?;

        info!("Added/Updated course: {} ({})", course.title, course_id);

        Ok(CourseDetail {
            course,
            progress,
            stats,
        })
    }

    pub fn get_course(&self, course_id: &str) -> Option<CourseDetail> {
        let course = self.data.courses.get(course_id)?;
        let progress = self
            .data
            .progress
            .get(course_id)
            .cloned()
            .unwrap_or_else(|| CourseProgress {
                course_id: course_id.to_string(),
                ..Default::default()
            });
        let stats = self.compute_stats(course, &progress);

        Some(CourseDetail {
            course: course.clone(),
            progress,
            stats,
        })
    }

    pub fn list_courses(&self) -> Vec<CourseSummary> {
        let mut summaries = Vec::new();
        for (id, course) in &self.data.courses {
            let progress = self
                .data
                .progress
                .get(id)
                .cloned()
                .unwrap_or_else(|| CourseProgress {
                    course_id: id.clone(),
                    ..Default::default()
                });
            let stats = self.compute_stats(course, &progress);
            summaries.push(CourseSummary {
                id: id.clone(),
                title: course.title.clone(),
                root_path: course.root_path.clone(),
                stats,
                last_played_lecture_id: progress.last_played_lecture_id,
                updated_at: course.updated_at.clone(),
            });
        }
        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        summaries
    }

    pub fn delete_course(&mut self, course_id: &str) -> Result<bool, String> {
        let removed = self.data.courses.remove(course_id).is_some();
        self.data.progress.remove(course_id);

        if self.data.settings.active_course_id.as_deref() == Some(course_id) {
            self.data.settings.active_course_id = self.data.courses.keys().next().cloned();
        }

        self.save()?;
        Ok(removed)
    }

    pub fn set_lecture_completed(
        &mut self,
        course_id: &str,
        lecture_id: &str,
        completed: bool,
    ) -> Result<CourseDetail, String> {
        let progress = self
            .data
            .progress
            .entry(course_id.to_string())
            .or_insert_with(|| CourseProgress {
                course_id: course_id.to_string(),
                ..Default::default()
            });

        if completed {
            progress.completed_lecture_ids.insert(lecture_id.to_string());
        } else {
            progress.completed_lecture_ids.remove(lecture_id);
        }

        self.save()?;
        self.get_course(course_id)
            .ok_or_else(|| "Course not found".to_string())
    }

    pub fn update_playback_position(
        &mut self,
        course_id: &str,
        lecture_id: &str,
        position_seconds: f64,
    ) -> Result<(), String> {
        let progress = self
            .data
            .progress
            .entry(course_id.to_string())
            .or_insert_with(|| CourseProgress {
                course_id: course_id.to_string(),
                ..Default::default()
            });

        progress.last_played_lecture_id = Some(lecture_id.to_string());
        progress
            .playback_positions
            .insert(lecture_id.to_string(), position_seconds);

        self.save()
    }

    pub fn update_lecture_duration(
        &mut self,
        course_id: &str,
        lecture_id: &str,
        duration_seconds: u64,
    ) -> Result<(), String> {
        if let Some(course) = self.data.courses.get_mut(course_id) {
            for sec in &mut course.sections {
                for lec in &mut sec.lectures {
                    if lec.id == lecture_id {
                        lec.duration_seconds = duration_seconds;
                        sec.duration_seconds =
                            sec.lectures.iter().map(|l| l.duration_seconds).sum();
                        return self.save();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn add_note(&mut self, course_id: &str, note: Note) -> Result<Vec<Note>, String> {
        let notes = {
            let progress = self
                .data
                .progress
                .entry(course_id.to_string())
                .or_insert_with(|| CourseProgress {
                    course_id: course_id.to_string(),
                    ..Default::default()
                });

            progress.notes.push(note);
            progress
                .notes
                .sort_by(|a, b| a.timestamp_seconds.partial_cmp(&b.timestamp_seconds).unwrap());
            progress.notes.clone()
        };

        self.save()?;
        Ok(notes)
    }

    pub fn delete_note(&mut self, course_id: &str, note_id: &str) -> Result<Vec<Note>, String> {
        let notes = {
            let progress = self
                .data
                .progress
                .entry(course_id.to_string())
                .or_insert_with(|| CourseProgress {
                    course_id: course_id.to_string(),
                    ..Default::default()
                });

            progress.notes.retain(|n| n.id != note_id);
            progress.notes.clone()
        };

        self.save()?;
        Ok(notes)
    }

    pub fn get_settings(&self) -> UserSettings {
        self.data.settings.clone()
    }

    pub fn update_settings(&mut self, settings: UserSettings) -> Result<UserSettings, String> {
        self.data.settings = settings;
        self.save()?;
        Ok(self.data.settings.clone())
    }
}
