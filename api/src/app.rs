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
        .merge(modules::departments::router())
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
    use std::collections::BTreeSet;

    #[test]
    fn authorization_routes_are_mounted_and_documented() {
        let (_, document) = super::api_routes().split_for_parts();
        let paths = &document.paths.paths;
        assert!(paths["/api/roles"].get.is_some());
        assert!(paths["/api/roles"].post.is_some());
        assert!(paths["/api/roles/{code}"].patch.is_some());
        assert!(paths["/api/roles/{code}/permissions"].put.is_some());
        assert!(paths["/api/users/{public_id}/role"].put.is_some());
        assert!(paths["/api/users/{public_id}/department"].put.is_some());
        assert!(paths["/api/departments"].get.is_some());
        assert!(paths["/api/departments"].post.is_some());
        assert!(paths["/api/departments/me"].get.is_some());
        assert!(paths["/api/departments/{public_id}"].get.is_some());
        assert!(paths["/api/permissions"].get.is_some());

        for (path, item) in paths {
            if ![
                "/api/products",
                "/api/categories",
                "/api/suppliers",
                "/api/departments",
                "/api/users",
                "/api/roles",
                "/api/permissions",
            ]
            .iter()
            .any(|prefix| path == prefix || path.starts_with(&format!("{prefix}/")))
            {
                continue;
            }
            for operation in [&item.get, &item.put, &item.post, &item.delete, &item.patch]
                .into_iter()
                .flatten()
            {
                assert!(
                    operation
                        .security
                        .as_ref()
                        .is_some_and(|requirements| !requirements.is_empty()),
                    "missing session security in {path}"
                );
            }
        }
    }

    #[test]
    fn runtime_permissions_match_database_catalog() {
        let migration = include_str!("../../db/migration/src/m20260917_000008_reference_data.rs");
        let entries = migration
            .split_once("const PERMISSIONS:")
            .expect("reference migration permission catalog")
            .1
            .split_once("const CAPTURISTA:")
            .expect("reference migration role grants")
            .0;
        let quoted: Vec<_> = entries.split('"').skip(1).step_by(2).collect();
        assert_eq!(quoted.len() % 2, 0, "permission code/description pairs");
        let migrated: BTreeSet<_> = quoted.chunks_exact(2).map(|pair| pair[0]).collect();
        let runtime: BTreeSet<_> = auth::permission::ALL.iter().copied().collect();
        assert_eq!(
            migrated.len(),
            quoted.len() / 2,
            "duplicate migration codes"
        );
        assert_eq!(
            runtime.len(),
            auth::permission::ALL.len(),
            "duplicate runtime codes"
        );
        assert_eq!(migrated, runtime);

        let reference = include_str!("../../db/reference/ddl/tables.sql");
        let rows = reference
            .split_once("INSERT INTO permissions (code, description) VALUES")
            .expect("reference SQL permission catalog")
            .1
            .split_once("INSERT INTO role_permissions")
            .expect("reference SQL role grants")
            .0;
        let ddl: BTreeSet<_> = rows
            .lines()
            .filter_map(|line| line.trim().strip_prefix("('"))
            .map(|row| row.split_once('\'').expect("permission code").0)
            .collect();
        assert_eq!(ddl, runtime);
    }
}
