use std::collections::VecDeque;

use crate::app_state::AppState;
use axum::{
    extract::{Query, State}, http::StatusCode, response::IntoResponse, Json
};

use super::list::Category;

#[derive(Debug, serde::Deserialize)]
pub struct TreeParams {
    parent_id: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct TreeNode {
    id: u64,
    name: String,
    children: Vec<TreeNode>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CategoryRow {
    pub id: u64,
    pub parent_id: u64,
    pub name: String,
}

pub async fn handler(
    State(AppState { ref db }): State<AppState>,
    Query(params): Query<TreeParams>,
) -> impl IntoResponse {
    let sql = "SELECT `id`, `name`, `parent_id` FROM `categories`";

    let rows = sqlx::query_as::<_, CategoryRow>(sql)
        .fetch_all(db)
        .await
        .unwrap();

    let tree_data = build_tree(&rows, 0);

    (StatusCode::OK, Json(serde_json::json!({
        "message": "ok",
        "data": tree_data
    })))
}

fn build_tree(categories: &Vec<CategoryRow>, parent_id: u64) -> Vec<TreeNode> {
    let mut result = Vec::<TreeNode>::with_capacity(8);

    categories.iter().for_each(|category| {
        if category.parent_id == parent_id {
            let children: Vec<TreeNode> = build_tree(categories, category.id);
            result.push(TreeNode {
                id: category.id,
                name: category.name.clone(),
                children: children,
            })
        }
    });

    return result;
}
