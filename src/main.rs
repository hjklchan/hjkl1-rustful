use std::time::Duration;

use axum::{extract::State, response::IntoResponse, routing, Router};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use tokio::net::TcpListener;

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
        .route("/posts", routing::post(posts::create))
        // Categories
        .route("/categories", routing::post(categories::create))
        .route("/categories", routing::get(categories::list))
        .route("/categories/:id", routing::get(categories::get))
        .route("/categories/:id", routing::put(categories::update))
        .route("/categories/:id", routing::delete(categories::delete))
        .route(
            "/categories/:id/soft_delete",
            routing::delete(categories::soft_delete),
        )
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

        println!("{rows:#?}");
        "Categories"
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
    use super::IntoResponse;

    pub async fn list() -> impl IntoResponse {
        "Posts"
    }

    pub async fn get() -> impl IntoResponse {
        "Get post"
    }

    pub async fn create() -> impl IntoResponse {
        "Create Post"
    }

    pub async fn update() -> impl IntoResponse {
        "Update post"
    }

    pub async fn delete() -> impl IntoResponse {
        "Delete post"
    }

    pub async fn soft_delete() -> impl IntoResponse {
        "Soft delete post"
    }
}
