use crate::app_state::AppState;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};

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

    // TODO tree()
}

fn build_tree(categories: Vec<CategoryRow>) {
    // TODO
}
