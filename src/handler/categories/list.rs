use axum::extract::Query;
use axum::Json;

use crate::app_state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Category {
    pub id: u64,
    // pub parent_id: u64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ListParams {
    parent_id: Option<i64>,
}

pub async fn handler(
    State(AppState { ref db }): State<AppState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let parent_id = params.parent_id.unwrap_or_default();

    let sql = r#"SELECT `id`, `name`, `description` FROM `categories` WHERE `parent_id` = ?"#;

    let rows = sqlx::query_as::<_, Category>(sql)
        .bind(parent_id)
        .fetch_all(db)
        .await
        .unwrap();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "ok",
            "data": rows
        })),
    )
}
