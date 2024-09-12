use std::time::Duration;

use axum::{http::Method, routing, Router};
use hjkl1_rsful::{app_state::AppState, layer::cors::cors_middleware, handler::{categories, posts}};
use sqlx::mysql::MySqlPoolOptions;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

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
        // Posts
        .route("/posts", routing::get(posts::list::handler))
        .route("/posts/:id", routing::get(posts::get::handler))
        .route("/posts/:id", routing::put(posts::update::handler))
        .route("/posts/:id", routing::delete(posts::delete::handler))
        .route(
            "/posts/:id/soft_delete",
            routing::delete(posts::soft_delete::handler),
        )
        .route("/posts/:id/recover", routing::patch(posts::recover::handler))
        .route("/posts", routing::post(posts::create::handler))
        // Categories
        .route("/categories", routing::post(categories::create::handler))
        .route("/categories", routing::get(categories::list::handler))
        .route("/categories/tree", routing::get(categories::tree::handler))
        .route("/categories/:id", routing::get(categories::get::handler))
        .route("/categories/:id", routing::put(categories::update::handler))
        .route("/categories/:id", routing::delete(categories::delete::handler))
        .route(
            "/categories/:id/soft_delete",
            routing::delete(categories::soft_delete::handler),
        )
        .layer(cors_middleware())
        .with_state(AppState { db: pool });

    // Tcp listener
    let tcp_listener = TcpListener::bind("0.0.0.0:9000").await.unwrap();

    axum::serve(tcp_listener, app).await.unwrap();
}
