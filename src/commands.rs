use crate::db::SharedDb;
use crate::models::{CourseDetail, CourseSummary, Note, UserSettings};
use crate::scanner::scan_course_directory;
use chrono::Utc;
use std::path::Path;
use tauri::State;
use uuid::Uuid;

pub struct StreamPort(pub u16);

#[tauri::command]
pub fn get_streaming_port(port: State<'_, StreamPort>) -> u16 {
    port.0
}

#[tauri::command]
pub async fn list_courses(db: State<'_, SharedDb>) -> Result<Vec<CourseSummary>, String> {
    let db_read = db.read().await;
    Ok(db_read.list_courses())
}

#[tauri::command]
pub async fn scan_course(
    app: tauri::AppHandle,
    db: State<'_, SharedDb>,
    path: String,
) -> Result<CourseDetail, String> {
    use tauri::Emitter;
    let path_obj = Path::new(&path);
    let app_handle = app.clone();
    let course = scan_course_directory(
        path_obj,
        Some(move |p: crate::models::ScanProgress| {
            let _ = app_handle.emit("scan-progress", p);
        }),
    )?;
    let mut db_write = db.write().await;
    db_write.add_course(course)
}

#[tauri::command]
pub async fn rescan_course(
    app: tauri::AppHandle,
    db: State<'_, SharedDb>,
    course_id: String,
) -> Result<CourseDetail, String> {
    use tauri::Emitter;
    let root_path = {
        let db_read = db.read().await;
        db_read
            .data
            .courses
            .get(&course_id)
            .map(|c| c.root_path.clone())
            .ok_or_else(|| "Course not found".to_string())?
    };

    let path_obj = Path::new(&root_path);
    let app_handle = app.clone();
    let mut scanned_course = scan_course_directory(
        path_obj,
        Some(move |p: crate::models::ScanProgress| {
            let _ = app_handle.emit("scan-progress", p);
        }),
    )?;
    scanned_course.id = course_id.clone();

    let mut db_write = db.write().await;
    db_write.add_course(scanned_course)
}

#[tauri::command]
pub async fn get_course(
    db: State<'_, SharedDb>,
    course_id: String,
) -> Result<CourseDetail, String> {
    let db_read = db.read().await;
    db_read
        .get_course(&course_id)
        .ok_or_else(|| "Course not found".to_string())
}

#[tauri::command]
pub async fn delete_course(db: State<'_, SharedDb>, course_id: String) -> Result<bool, String> {
    let mut db_write = db.write().await;
    db_write.delete_course(&course_id)
}

#[tauri::command]
pub async fn update_progress(
    db: State<'_, SharedDb>,
    course_id: String,
    lecture_id: String,
    completed: bool,
) -> Result<CourseDetail, String> {
    let mut db_write = db.write().await;
    db_write.set_lecture_completed(&course_id, &lecture_id, completed)
}

#[tauri::command]
pub async fn update_position(
    db: State<'_, SharedDb>,
    course_id: String,
    lecture_id: String,
    position_seconds: f64,
) -> Result<(), String> {
    let mut db_write = db.write().await;
    db_write.update_playback_position(&course_id, &lecture_id, position_seconds)
}

#[tauri::command]
pub async fn update_duration(
    db: State<'_, SharedDb>,
    course_id: String,
    lecture_id: String,
    duration_seconds: u64,
) -> Result<(), String> {
    let mut db_write = db.write().await;
    db_write.update_lecture_duration(&course_id, &lecture_id, duration_seconds)
}

#[tauri::command]
pub async fn add_note(
    db: State<'_, SharedDb>,
    course_id: String,
    lecture_id: String,
    lecture_title: String,
    timestamp_seconds: f64,
    text: String,
) -> Result<Vec<Note>, String> {
    let note = Note {
        id: Uuid::new_v4().to_string(),
        lecture_id,
        lecture_title,
        timestamp_seconds,
        text,
        created_at: Utc::now().to_rfc3339(),
    };

    let mut db_write = db.write().await;
    db_write.add_note(&course_id, note)
}

#[tauri::command]
pub async fn delete_note(
    db: State<'_, SharedDb>,
    course_id: String,
    note_id: String,
) -> Result<Vec<Note>, String> {
    let mut db_write = db.write().await;
    db_write.delete_note(&course_id, &note_id)
}

#[tauri::command]
pub async fn get_settings(db: State<'_, SharedDb>) -> Result<UserSettings, String> {
    let db_read = db.read().await;
    Ok(db_read.get_settings())
}

#[tauri::command]
pub async fn update_settings(
    db: State<'_, SharedDb>,
    settings: UserSettings,
) -> Result<UserSettings, String> {
    let mut db_write = db.write().await;
    db_write.update_settings(settings)
}

#[tauri::command]
pub async fn pick_folder() -> Result<Option<String>, String> {
    let picked = tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Select Course Folder")
            .pick_folder()
    })
    .await
    .map_err(|e| format!("Dialog error: {}", e))?;

    Ok(picked.map(|p| p.to_string_lossy().to_string()))
}
