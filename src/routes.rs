use crate::{AppState, MAX_SIZE, MAX_TEXT_SIZE, errors::AppError};
use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderValue, header},
    response::{Html, IntoResponse, Response},
};
use tokio::{fs, io};
use uuid::Uuid;

pub async fn serve_homepage() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

pub async fn create_paste(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let mut content = Vec::new();
    let mut content_type = String::from("text/plain");

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "content" => {
                let text = field.text().await?;
                if !text.is_empty() {
                    if text.len() > MAX_TEXT_SIZE {
                        return Err(AppError::BadRequest("Text too large (max 1MB)".to_string()));
                    }
                    content = text.into_bytes();
                }
            }
            "file" => {
                let filename = field.file_name().unwrap_or("").to_string();
                if !filename.is_empty() {
                    content_type = mime_guess::from_path(&filename)
                        .first_or_octet_stream()
                        .to_string();

                    let data = field.bytes().await?;
                    content = data.to_vec();
                }
            }
            _ => {}
        }
    }

    if content.is_empty() {
        return Err(AppError::BadRequest("Content cannot be empty".to_string()));
    }

    if content.len() > MAX_SIZE {
        return Err(AppError::BadRequest(
            "Content too large (max 10MB)".to_string(),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let size = content.len() as i64;

    let content_path = state.cache_dir.join(&id);
    fs::write(&content_path, &content)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to save file: {}", e)))?;

    if let Err(e) = state.db.create_paste(&id, &content_type, size).await {
        let _ = fs::remove_file(&content_path).await;
        return Err(e.into());
    }

    let success_html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Paste Created</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <div class="container">
        <h1>distrust</h1>
        <div class="success">
            <p>✓ Paste created successfully!</p>
            <p><strong>Link:</strong> <a href="/paste/{0}">/paste/{0}</a></p>
            <p><strong>Raw:</strong> <a href="/raw/{0}">/raw/{0}</a></p>
            <p><a href="/">← Create another</a></p>
        </div>
    </div>
</body>
</html>"#,
        id
    );

    Ok(Html(success_html).into_response())
}

pub async fn get_paste(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    match state.db.get_paste(&id).await? {
        Some(paste) => {
            let content_path = state.cache_dir.join(&id);
            let content = match fs::read(&content_path).await {
                Ok(content) => content,
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    let _ = state.db.delete_paste(&id).await;
                    return Err(AppError::NotFound);
                }
                Err(_) => return Err(AppError::NotFound),
            };

            state.db.increment_views(&id).await?;

            let is_text = is_text_content(&paste.content_type);
            let html = if is_text {
                let content_str = String::from_utf8_lossy(&content);
                format!(
                    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Paste {}</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <div class="container">
        <div class="meta">
            ID: {} | Type: {} | Views: {} | <a href="/raw/{}">raw</a> | <a href="/">new paste</a>
        </div>
        <pre>{}</pre>
    </div>
</body>
</html>"#,
                    id,
                    id,
                    paste.content_type,
                    paste.view_count,
                    id,
                    html_escape(&content_str)
                )
            } else {
                format!(
                    r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Paste {}</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <div class="container">
        <div class="meta">
            ID: {} | Type: {} | Views: {} | <a href="/raw/{}">download</a> | <a href="/">new paste</a>
        </div>
        <div class="binary-message">
            <p>This is a binary file (type: {}) and cannot be displayed.</p>
            <p><a href="/raw/{}"> Download</a> to view it.</p>
        </div>
    </div>
</body>
</html>"#,
                    id, id, paste.content_type, paste.view_count, id, paste.content_type, id
                )
            };
            Ok(Html(html).into_response())
        }
        None => Err(AppError::NotFound),
    }
}

pub async fn get_paste_raw(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    match state.db.get_paste(&id).await? {
        Some(paste) => {
            let content_path = state.cache_dir.join(&id);
            let content = match fs::read(&content_path).await {
                Ok(content) => content,
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    let _ = state.db.delete_paste(&id).await;
                    return Err(AppError::NotFound);
                }
                Err(_) => return Err(AppError::NotFound),
            };

            state.db.increment_views(&id).await?;

            let is_text = is_text_content(&paste.content_type);

            let mut resp = content.into_response();
            let headers = resp.headers_mut();

            if is_text {
                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                );
            } else {
                headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(&paste.content_type)
                        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
                );
                headers.insert(
                    header::CONTENT_DISPOSITION,
                    HeaderValue::from_static("attachment"),
                );
            }

            headers.insert(
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            );

            Ok(resp)
        }
        None => Err(AppError::NotFound),
    }
}

pub async fn serve_css() -> impl IntoResponse {
    let css = include_str!("../static/style.css");
    ([(header::CONTENT_TYPE, "text/css")], css)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn is_text_content(content_type: &str) -> bool {
    content_type.starts_with("text/")
        || [
            "application/json",
            "application/xml",
            "application/javascript",
            "application/ecmascript",
            "application/x-sh",
            "application/x-www-form-urlencoded",
        ]
        .contains(&content_type)
        || ["script", "json", "xml", "yaml", "toml", "csv"]
            .iter()
            .any(|s| content_type.contains(s))
}
