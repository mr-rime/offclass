use crate::db::SharedDb;
use crate::media_server::stream_video_file;
use axum::extract::{Path as AxPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
use std::path::Path;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;

pub async fn start_streaming_server(db: SharedDb) -> Result<u16, String> {
    let app = Router::new()
        .route("/stream/{course_id}/{lecture_id}", get(stream_video_handler))
        .route("/html/{course_id}/{lecture_id}", get(html_lecture_handler))
        .route("/content/{course_id}/{*file_path}", get(course_content_handler))
        .route("/subtitle/{course_id}/{lecture_id}", get(subtitle_handler))
        .layer(CorsLayer::permissive())
        .with_state(db);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind streaming server listener: {}", e))?;

    let addr: SocketAddr = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?;

    let port = addr.port();
    info!("OffClass video streaming server running on http://{}", addr);

    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    Ok(port)
}

async fn stream_video_handler(
    State(db): State<SharedDb>,
    AxPath((course_id, lecture_id)): AxPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let file_path = {
        let db_read = db.read().await;
        if let Some(course) = db_read.data.courses.get(&course_id) {
            let mut found = None;
            for sec in &course.sections {
                for lec in &sec.lectures {
                    if lec.id == lecture_id {
                        found = Some(lec.absolute_path.clone());
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            found
        } else {
            None
        }
    };

    match file_path {
        Some(path_str) => stream_video_file(Path::new(&path_str), &headers).await,
        None => (StatusCode::NOT_FOUND, "Lecture video file not found").into_response(),
    }
}

async fn subtitle_handler(
    State(db): State<SharedDb>,
    AxPath((course_id, lecture_id)): AxPath<(String, String)>,
) -> Response {
    let file_path = {
        let db_read = db.read().await;
        if let Some(course) = db_read.data.courses.get(&course_id) {
            let mut found = None;
            for sec in &course.sections {
                for lec in &sec.lectures {
                    if lec.id == lecture_id {
                        found = Some(lec.absolute_path.clone());
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            found
        } else {
            None
        }
    };

    let path_str = match file_path {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "Lecture not found").into_response(),
    };

    match crate::subtitles::get_or_extract_subtitle(&course_id, &lecture_id, Path::new(&path_str)) {
        Some(vtt_content) => (
            StatusCode::OK,
            [
                (axum::http::header::CONTENT_TYPE, "text/vtt; charset=utf-8"),
                (axum::http::header::CACHE_CONTROL, "public, max-age=3600"),
            ],
            vtt_content,
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "No subtitles available",
        )
            .into_response(),
    }
}

async fn html_lecture_handler(
    State(db): State<SharedDb>,
    AxPath((course_id, lecture_id)): AxPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let target_file_path = {
        let db_read = db.read().await;
        if let Some(course) = db_read.data.courses.get(&course_id) {
            let mut found = None;
            for sec in &course.sections {
                for lec in &sec.lectures {
                    if lec.id == lecture_id {
                        let abs = Path::new(&lec.absolute_path);
                        if abs.exists() {
                            found = Some(abs.to_path_buf());
                        } else {
                            let root = Path::new(&course.root_path);
                            let rel = root.join(&lec.relative_path);
                            if rel.exists() {
                                found = Some(rel);
                            } else {
                                found = Some(abs.to_path_buf());
                            }
                        }
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            found
        } else {
            None
        }
    };

    match target_file_path {
        Some(path_buf) => stream_video_file(&path_buf, &headers).await,
        None => (StatusCode::NOT_FOUND, "HTML document not found").into_response(),
    }
}

fn percent_decode_str(input: &str) -> String {
    let mut bytes = Vec::new();
    let chars = input.as_bytes();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == b'%' && i + 2 < chars.len() {
            if let Ok(byte_val) = u8::from_str_radix(
                std::str::from_utf8(&chars[i + 1..=i + 2]).unwrap_or(""),
                16,
            ) {
                bytes.push(byte_val);
                i += 3;
                continue;
            }
        }
        bytes.push(chars[i]);
        i += 1;
    }
    String::from_utf8_lossy(&bytes).to_string()
}

async fn course_content_handler(
    State(db): State<SharedDb>,
    AxPath((course_id, file_path)): AxPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let root_path_str = {
        let db_read = db.read().await;
        if let Some(course) = db_read.data.courses.get(&course_id) {
            course.root_path.clone()
        } else {
            return (StatusCode::NOT_FOUND, "Course not found").into_response();
        }
    };

    let decoded_path = percent_decode_str(&file_path);
    let base_dir = Path::new(&root_path_str);
    let clean_rel = decoded_path.trim_start_matches('/').replace('\\', "/");
    let target_path = base_dir.join(&clean_rel);

    if !target_path.exists() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    if target_path.is_dir() {
        let index_html = target_path.join("index.html");
        if index_html.is_file() {
            return stream_video_file(&index_html, &headers).await;
        }
        let index_htm = target_path.join("index.htm");
        if index_htm.is_file() {
            return stream_video_file(&index_htm, &headers).await;
        }
        return (StatusCode::NOT_FOUND, "Directory index not found").into_response();
    }

    if !target_path.is_file() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    stream_video_file(&target_path, &headers).await
}
