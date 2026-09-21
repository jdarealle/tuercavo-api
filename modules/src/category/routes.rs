use super::handler;
use common::state::AppState;
use utoipa_axum::{router::OpenApiRouter, routes};
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handler::list, handler::create))
        .routes(routes!(handler::get, handler::update, handler::delete))
}
