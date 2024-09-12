use crate::app_state::AppState;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

pub async fn handler(
    State(AppState { ref db }): State<AppState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    let sql = "DELETE FROM `categories` WHERE `id` = ?";

    let res = sqlx::query(sql).bind(id).execute(db).await.unwrap();
    if res.rows_affected() == 1 {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "ok"
            })),
        );
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "message": "record does not exist"
            })),
        );
    }
}
