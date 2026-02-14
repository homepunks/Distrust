use axum::{
    extract::{Multipart, Path, State},
    response::{Html, IntoResponse, Response},
    http::header,
};
use uuid::Uuid;
use crate::{data, errors::AppError, AppState};

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

    if content.len() > 10 * 1024 * 1024 {
        return Err(AppError::BadRequest("Content too large (max 10MB)".to_string()));
    }

    let id = Uuid::new_v4().to_string();

    let new_paste = data::NewPaste {
        content,
        content_type,
    };

    state.db.create_paste(&id, new_paste).await?;

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
            let is_text = paste.content_type.starts_with("text/")
                || paste.content_type == "application/json"
                || paste.content_type == "application/xml"
                || paste.content_type == "application/javascript"
                || paste.content_type == "application/ecmascript"
                || paste.content_type == "application/x-sh"
                || paste.content_type == "application/x-www-form-urlencoded"
                || paste.content_type.contains("script")
                || paste.content_type.contains("json")
                || paste.content_type.contains("xml")
                || paste.content_type.contains("yaml")
                || paste.content_type.contains("toml")
                || paste.content_type.contains("csv");

            let html = if is_text {
                let content_str = String::from_utf8_lossy(&paste.content);
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
                    id,
                    id,
                    paste.content_type,
                    paste.view_count,
                    id,
                    paste.content_type,
                    id
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
            let mut resp = paste.content.into_response();
            resp.headers_mut().insert(
                header::CONTENT_TYPE,
                paste.content_type.parse().unwrap(),
            );

            Ok(resp)
        }
        None => Err(AppError::NotFound),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
