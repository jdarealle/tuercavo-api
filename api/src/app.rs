use axum::{
    Router,
    extract::{DefaultBodyLimit, MatchedPath, Request},
    http::HeaderValue,
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use common::{error::AppError, state::AppState, telemetry::REQUEST_ID};
use std::time::Duration;
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

pub fn api_routes() -> OpenApiRouter<AppState> {
    let api = OpenApiRouter::new()
        .merge(modules::category::router())
        .merge(modules::supplier::router())
        .merge(modules::product::router())
        .merge(modules::users::router())
        .merge(modules::roles::router())
        .merge(auth::router())
        .merge(modules::health::router());
    OpenApiRouter::new().nest("/api", api)
}

pub fn router(state: AppState) -> Router {
    let (router, _openapi) = api_routes().split_for_parts();
    #[cfg(feature = "scalar")]
    let router = router.merge(crate::api_doc::router(
        _openapi,
        state.auth.session_cookie_name(),
    ));
    router
        .fallback(|| async { AppError::NotFound })
        .method_not_allowed_fallback(|| async { AppError::MethodNotAllowed })
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(middleware::from_fn(request_context))
        .with_state(state)
}
async fn request_context(mut req: Request, next: Next) -> Response {
    let id = Uuid::new_v4();
    req.headers_mut().remove("x-request-id");
    let method = req.method().clone();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|| "unmatched".into());
    REQUEST_ID.scope(id, async move {
        let start = std::time::Instant::now();
        let mut response = match tokio::time::timeout(Duration::from_secs(30), next.run(req)).await {
            Ok(response) => response, Err(_) => AppError::Timeout.into_response(),
        };
        let headers = response.headers_mut();
        headers.insert("x-request-id", HeaderValue::from_str(&id.to_string()).expect("UUID header"));
        for (name, value) in [("cache-control", "no-store"), ("x-content-type-options", "nosniff"), ("x-frame-options", "DENY"), ("referrer-policy", "no-referrer"), ("content-security-policy", "default-src 'none'; frame-ancestors 'none'"), ("permissions-policy", "geolocation=(), microphone=(), camera=()")] {
            headers.entry(name).or_insert(HeaderValue::from_static(value));
        }
        tracing::info!(request_id = %id, %method, %route, status = response.status().as_u16(), elapsed_ms = start.elapsed().as_millis(), "request completed");
        response
    }).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn authorization_routes_are_mounted_and_documented() {
        let (_, document) = super::api_routes().split_for_parts();
        let paths = &document.paths.paths;
        assert!(paths["/api/roles"].get.is_some());
        assert!(paths["/api/roles"].post.is_some());
        assert!(paths["/api/roles/{code}"].patch.is_some());
        assert!(paths["/api/roles/{code}/permissions"].put.is_some());
        assert!(paths["/api/users/{public_id}/role"].put.is_some());
        assert!(paths["/api/permissions"].get.is_some());
    }
}
