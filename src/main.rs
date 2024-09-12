use std::time::Duration;

use axum::{extract::State, http::Method, response::IntoResponse, routing, Router};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    db: Pool<MySql>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok().unwrap();

    // Database
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| String::from("mysql://root:@127.0.0.1:3306/hjkl1db"));
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .unwrap();

    let cors_middleware = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_origin(Any);

    // Router
    let app = Router::new()
        .route("/posts", routing::get(posts::list))
        .route("/posts/:id", routing::get(posts::get))
        .route("/posts/:id", routing::put(posts::update))
        .route("/posts/:id", routing::delete(posts::delete))
        .route(
            "/posts/:id/soft_delete",
            routing::delete(posts::soft_delete),
        )
        .route("/posts/:id/recover", routing::patch(posts::recover))
        .route("/posts", routing::post(posts::create))
        // Categories
        .route("/categories", routing::post(categories::create))
        .route("/categories", routing::get(categories::list))
        .route("/categories/tree", routing::get(categories::tree))
        .route("/categories/:id", routing::get(categories::get))
        .route("/categories/:id", routing::put(categories::update))
        .route("/categories/:id", routing::delete(categories::delete))
        .route(
            "/categories/:id/soft_delete",
            routing::delete(categories::soft_delete),
        )
        .layer(cors_middleware)
        .with_state(AppState { db: pool });

    // Tcp listener
    let tcp_listener = TcpListener::bind("0.0.0.0:9000").await.unwrap();

    axum::serve(tcp_listener, app).await.unwrap();
}

mod categories {
    use axum::extract::Path;
    use axum::extract::Query;
    use axum::Json;
    use sqlx::MySql;

    use crate::AppState;

    use super::IntoResponse;
    use super::State;
    use axum::http::StatusCode;

    #[derive(Debug, serde::Deserialize)]
    pub struct CreateCategory {
        pub parent_id: Option<i64>,
        pub name: String,
    }

    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    pub struct Category {
        pub id: u64,
        // pub parent_id: u64,
        pub name: String,
        pub description: Option<String>,
    }

    #[derive(Debug, sqlx::FromRow)]
    struct CategoryRow {
        pub id: u64,
        pub parent_id: u64,
        pub name: String,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct ListParams {
        parent_id: Option<i64>,
    }

    // Category list
    pub async fn list(
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

    pub fn build_tree(categories: Vec<CategoryRow>) {
        // TODO
    }

    pub async fn tree(
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

    pub async fn get(State(AppState { ref db }): State<AppState>) -> impl IntoResponse {
        _ = db;
        "Get category"
    }

    // Create category
    pub async fn create(
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

        Json(serde_json::json!({
            "new_id": category.last_insert_id()
        }))
    }

    pub async fn update(State(AppState { ref db }): State<AppState>) -> impl IntoResponse {
        "Update category"
    }

    pub async fn delete(
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

    pub async fn soft_delete(State(AppState { ref db }): State<AppState>) -> impl IntoResponse {
        // The table structure does not support soft deletion
        "Soft delete category"
    }
}

mod posts {
    use axum::{
        extract::{Path, Query, State},
        http::StatusCode,
        Json,
    };
    use chrono;
    use sqlx::QueryBuilder;

    use crate::AppState;

    use super::IntoResponse;

    #[derive(Debug, serde::Deserialize)]
    pub struct CreatePost {
        category_id: u64,
        title: String,
        description: Option<String>,
        body: Option<String>,
    }

    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    pub struct Post {
        id: u64,
        category_id: u64,
        category_name: String,
        title: String,
        description: Option<String>,
        created_at: Option<chrono::DateTime<chrono::Local>>,
        updated_at: Option<chrono::DateTime<chrono::Local>>,
    }

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

    #[derive(Debug, serde::Deserialize)]
    pub struct ListParams {
        category_id: Option<u64>,
    }

    pub async fn list(
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

    pub async fn get(
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

    pub async fn create(
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

    pub async fn update() -> impl IntoResponse {
        "Update post"
    }

    pub async fn delete(
        State(AppState { ref db }): State<AppState>,
        Path(id): Path<u64>,
    ) -> impl IntoResponse {
        let sql = "DELETE FROM `posts` WHERE `id` = ? AND `deleted_at` IS NOT NULL";

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
                    "message": "文章不存在或已经被删除",
                })),
            );
        }
    }

    pub async fn soft_delete(
        State(AppState { ref db }): State<AppState>,
        Path(id): Path<u64>,
    ) -> impl IntoResponse {
        let sql = "UPDATE `posts` SET `deleted_at` = NOW() WHERE `id` = ? AND `deleted_at` IS NULL";

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
                    "message": "文章不存在或已经被删除",
                })),
            );
        }
    }

    pub async fn recover(
        State(AppState { ref db }): State<AppState>,
        Path(id): Path<u64>,
    ) -> impl IntoResponse {
        let sql =
            "UPDATE `posts` SET `deleted_at` = NULL WHERE `id` = ? AND `deleted_at` IS NOT NULL";

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
}
