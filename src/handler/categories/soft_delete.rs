use crate::app_state::AppState;
use axum::{
    extract::State,
    response::IntoResponse,
};

pub async fn handler(State(AppState { ref db }): State<AppState>) -> impl IntoResponse {
    // The table structure does not support soft deletion
    "Soft delete category"
}
