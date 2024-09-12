use crate::app_state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

#[derive(Debug, serde::Deserialize)]
pub struct CreatePost {
    category_id: u64,
    title: String,
    description: Option<String>,
    body: Option<String>,
}

pub async fn handler(
    State(AppState { ref db }): State<AppState>,
    Json(req): Json<CreatePost>,
) -> impl IntoResponse {
    let sql = r#"
        INSERT INTO `posts` (`category_id`, `title`, `description`, `body`, `created_at`, `updated_at`)
        VALUES (?, ?, ?, NOW(), NOW())
    "#;

    let res = sqlx::query(sql)
        .bind(req.category_id)
        .bind(req.title)
        .bind(req.description)
        .bind(req.body)
        .execute(db)
        .await
        .unwrap();

    let new_id = res.last_insert_id();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "ok",
            "data": {
                "new_id": new_id
            }
        })),
    )
}
