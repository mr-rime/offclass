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
