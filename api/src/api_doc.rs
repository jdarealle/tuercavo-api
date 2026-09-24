use axum::{
    Json, Router,
    http::{HeaderValue, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use common::{error::AppError, state::AppState};
use scalar_api_reference::{get_asset_with_mime, scalar_html};
use serde_json::json;
use utoipa::openapi::{
    OpenApi,
    security::{ApiKey, ApiKeyValue, SecurityScheme},
};

pub fn router(mut spec: OpenApi, cookie_name: &str) -> Router<AppState> {
    spec.info.title = "Tuercavo API".into();
    spec.info.version = env!("CARGO_PKG_VERSION").into();
    spec.info.description = Some("Catálogo con autenticación Microsoft Entra y roles y permisos administrados en Tuercavo. Abre /api/auth/login en este mismo origen para iniciar sesión. El navegador envía la cookie HttpOnly automáticamente; las escrituras requieren Origin del mismo origen. PATCH conserva los campos omitidos.".into());
    spec.components
        .get_or_insert_with(Default::default)
        .add_security_scheme(
            "session",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(cookie_name))),
        );
    Router::new()
        .route("/api/openapi.json", get(move || async move { Json(spec) }))
        .route("/scalar", get(reference))
        .route("/scalar/scalar.js", get(script))
}
async fn reference() -> Response {
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let html = scalar_html(
        &json!({ "url": "/api/openapi.json", "agent": { "disabled": true }, "withDefaultFonts": false, "hideClientButton": true, "persistAuth": false }),
        Some("/scalar/scalar.js"),
    );
    let html = html.replace("<script", &format!("<script nonce=\"{nonce}\""));
    let mut response = Html(html).into_response();
    let csp = format!(
        "default-src 'self'; script-src 'self' 'nonce-{nonce}'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; worker-src 'self' blob:; frame-ancestors 'none'; base-uri 'none'; form-action 'self'"
    );
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_str(&csp).expect("generated CSP"),
    );
    response
}
async fn script() -> Result<Response, AppError> {
    let (mime, content) = get_asset_with_mime("scalar.js").ok_or(AppError::Internal)?;
    Ok(([(header::CONTENT_TYPE, mime)], content).into_response())
}
