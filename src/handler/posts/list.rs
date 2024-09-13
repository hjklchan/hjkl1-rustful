use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono;
use sqlx::QueryBuilder;

use crate::app_state::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct ListParams {
    category_id: Option<u64>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Post {
    pub id: u64,
    pub category_id: u64,
    pub category_name: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Local>>,
    pub updated_at: Option<chrono::DateTime<chrono::Local>>,
}

pub async fn handler(
    State(AppState { ref db }): State<AppState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let sql = r#"
            SELECT
                `p`.`id`,
                `p`.`category_id`,
                `c`.`name` AS `category_name`,
                `p`.`title`,
                `p`.`description`, 
                `p`.`created_at`, 
                `p`.`updated_at`
            FROM `posts` AS `p`
            JOIN `categories` AS `c` ON `c`.`id` = `p`.`category_id`
            WHERE `p`.`deleted_at` IS NULL
        "#;

    let mut query_builder = QueryBuilder::new(sql);
    // Filters
    if let Some(category_id) = params.category_id {
        query_builder
            .push(" AND `p`.`category_id` = ")
            .push_bind(category_id);
    }

    let rows = query_builder
        .build_query_as::<Post>()
        .fetch_all(db)
        .await
        .unwrap();

    return (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "ok",
            "data": rows,
        })),
    );
}
