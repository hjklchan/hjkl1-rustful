use crate::app_state::AppState;
use axum::{extract::State, response::IntoResponse};

pub async fn handler(State(AppState { ref db }): State<AppState>) -> impl IntoResponse {
    _ = db;
    "Get category"
}
