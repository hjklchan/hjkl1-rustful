use axum::Json;

use crate::app_state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};

#[derive(Debug, serde::Deserialize)]
pub struct CreateCategory {
    pub parent_id: Option<i64>,
    pub name: String,
}

pub async fn handler(
    State(AppState { ref db }): State<AppState>,
    Json(req): Json<CreateCategory>,
) -> impl IntoResponse {
    let parent_id = req.parent_id.unwrap_or_default();

    let category = sqlx::query(
        r#"
                INSERT INTO `categories` (`parent_id`, `name`) VALUES (?, ?)
            "#,
    )
    .bind(parent_id)
    .bind(req.name)
    .execute(db)
    .await
    .unwrap();

    (StatusCode::OK, Json(serde_json::json!({
        "new_id": category.last_insert_id()
    })))
}
