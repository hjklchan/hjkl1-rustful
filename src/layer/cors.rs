use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

pub fn cors_middleware() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_origin(Any)
}
