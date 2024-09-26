use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono;
use sqlx::QueryBuilder;

use crate::app_state::AppState;
use crate::utils::pagination;

#[derive(Debug, serde::Deserialize)]
pub struct ListParams {
    category_id: Option<u64>,
    page: Option<u64>,
    page_size: Option<u64>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct PostCount {
    count: i32,
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

#[derive(Debug, serde::Serialize)]
pub struct ListResponse {
    items: Vec<Post>,
    total: u64,
    page_size: u64,
    has_prev: bool,
    has_next: bool,
}

pub async fn handler(
    State(AppState { ref db }): State<AppState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let sql = r#"
        SELECT
            COUNT(`p`.`id`) AS `count`
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

    let result = query_builder.build_query_as::<PostCount>().fetch_one(db).await.unwrap();
    let count = result.count;

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

    // Pagination
    let page = params.page.unwrap_or_else(|| 1);
    let page_size = params.page_size.unwrap_or_else(|| 25);
    // Compute the offset and limit for the pagination
    let (offset, limit) = pagination::compute(page as u32, page_size as u32);
    // Prepare OFFSET and LIMIT
    query_builder.push(" LIMIT ").push_bind(limit);
    query_builder.push(" OFFSET ").push_bind(offset);

    let rows = query_builder
        .build_query_as::<Post>()
        .fetch_all(db)
        .await
        .unwrap();

    // FIXME Miscalculation
    let total_pages = (count as u64 + page_size - 1) / page_size;

    let has_next = page < total_pages;
    let has_prev = page != 1;

    let list_response = ListResponse {
        items: rows,
        total: count as u64,
        page_size,
        has_prev,
        has_next,
    };

    return (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "ok",
            "data": list_response,
        })),
    );
}
