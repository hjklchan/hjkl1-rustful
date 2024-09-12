use crate::app_state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono;

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct PostDetail {
    id: u64,
    category_id: u64,
    title: String,
    description: Option<String>,
    body: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Local>>,
    updated_at: Option<chrono::DateTime<chrono::Local>>,
}

pub async fn handler(
    State(AppState { ref db }): State<AppState>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    let sql = r#"
            SELECT
                `p`.`id`,
                `p`.`category_id`,
                `c`.`category_name`,
                `p`.`title`,
                `p`.`description`,
                `p`.`body`,
                `p`.`created_at`,
                `p`.`updated_at`
            FROM `posts` AS `p`
            JOIN `categories` AS `c` ON `c`.`id` = `p`.`category_id`
            WHERE
                `id` = ?
                AND `deleted_at` IS NULL
            LIMIT 1
        "#;

    let res = sqlx::query_as::<_, PostDetail>(sql)
        .bind(id)
        .fetch_optional(db)
        .await
        .unwrap();

    match res {
        Some(post) => {
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "message": "ok",
                    "data": post,
                })),
            );
        }
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "message": "文章不存在或已经被删除",
                })),
            );
        }
    }
}
