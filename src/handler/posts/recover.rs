use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse
};

use crate::app_state::AppState;


pub async fn handler(
    State(AppState { ref db }): State<AppState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    let sql = "UPDATE `posts` SET `deleted_at` = NULL WHERE `id` = ? AND `deleted_at` IS NOT NULL";

    let res = sqlx::query(sql).bind(id).execute(db).await.unwrap();

    if res.rows_affected() > 0 {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "ok",
            })),
        );
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "message": "文章已经被恢复",
            })),
        );
    }
}
