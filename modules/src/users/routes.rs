use super::handler;
use common::state::AppState;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handler::list))
        .routes(routes!(handler::get))
        .routes(routes!(handler::deactivate))
        .routes(routes!(handler::reactivate))
        .routes(routes!(handler::assign_role))
        .routes(routes!(handler::assign_department))
}
