use std::fs;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use tokio::{fs::File, io::AsyncReadExt};

use crate::app_state::AppState;

pub async fn handler(State(AppState { ref db }): State<AppState>) -> impl IntoResponse {
    let mut f = File::open("N:\\Github\\hjklchan\\hjkl1-rsful\\src\\handler\\posts\\README.md")
        .await
        .unwrap();

    let mut result: String = String::with_capacity(100);


    f.read_to_string(&mut result).await.unwrap();
    drop(f);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "ok",
            "data": result
        })),
    )
}
