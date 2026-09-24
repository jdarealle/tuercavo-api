use super::handler;
use common::state::AppState;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handler::list))
        .routes(routes!(handler::get))
        .routes(routes!(handler::permissions))
        .routes(routes!(handler::create))
        .routes(routes!(handler::update))
        .routes(routes!(handler::set_permissions))
}
