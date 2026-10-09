use axum::body::Body;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use std::io::SeekFrom;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use tracing::error;

const CHUNK_SIZE: u64 = 2 * 1024 * 1024; // 2MB streaming chunks for fast buffering

pub async fn stream_video_file(file_path: &Path, headers: &HeaderMap) -> Response {
    if !file_path.exists() {
        return (StatusCode::NOT_FOUND, "Video file not found").into_response();
    }

    let mut file = match File::open(file_path).await {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open video file {}: {}", file_path.display(), e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Cannot open file").into_response();
        }
    };

    let metadata = match file.metadata().await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to read metadata for {}: {}", file_path.display(), e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Cannot read metadata").into_response();
        }
    };

    let total_size = metadata.len();
    let mime_type = mime_guess::from_path(file_path)
        .first_or_octet_stream()
        .to_string();

    let range_header = headers.get(RANGE).and_then(|h| h.to_str().ok());

    if let Some(range_str) = range_header {
        if let Some((start, end)) = parse_range(range_str, total_size) {
            let length = end - start + 1;

            if let Err(e) = file.seek(SeekFrom::Start(start)).await {
                error!("Seek failed: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Seek failed").into_response();
            }

            let stream = ReaderStream::new(file.take(length));
            let body = Body::from_stream(stream);

            let content_range = format!("bytes {}-{}/{}", start, end, total_size);

            return Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(CONTENT_TYPE, mime_type)
                .header(ACCEPT_RANGES, "bytes")
                .header(CONTENT_RANGE, content_range)
                .header(CONTENT_LENGTH, length.to_string())
                .body(body)
                .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Response error").into_response());
        }
    }

    // Default full file streaming / fallback
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, mime_type)
        .header(ACCEPT_RANGES, "bytes")
        .header(CONTENT_LENGTH, total_size.to_string())
        .body(body)
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Response error").into_response())
}

fn parse_range(range_str: &str, total_size: u64) -> Option<(u64, u64)> {
    if !range_str.starts_with("bytes=") {
        return None;
    }

    let range = &range_str[6..];
    let parts: Vec<&str> = range.split('-').collect();

    if parts.is_empty() {
        return None;
    }

    let start_str = parts[0].trim();
    let end_str = parts.get(1).map(|s| s.trim()).unwrap_or("");

    if start_str.is_empty() {
        // Suffix byte range: "-500" means last 500 bytes
        let length: u64 = end_str.parse().ok()?;
        if length >= total_size {
            Some((0, total_size.saturating_sub(1)))
        } else {
            Some((total_size - length, total_size - 1))
        }
    } else {
        let start: u64 = start_str.parse().ok()?;
        if start >= total_size {
            return None;
        }

        let end: u64 = if end_str.is_empty() {
            // Stream up to CHUNK_SIZE or end of file
            (start + CHUNK_SIZE - 1).min(total_size - 1)
        } else {
            let parsed_end: u64 = end_str.parse().ok()?;
            parsed_end.min(total_size - 1)
        };

        Some((start, end))
    }
}
