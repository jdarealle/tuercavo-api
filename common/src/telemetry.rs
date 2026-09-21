use uuid::Uuid;
tokio::task_local! { pub static REQUEST_ID: Uuid; }
pub fn request_id() -> String {
    REQUEST_ID
        .try_with(ToString::to_string)
        .unwrap_or_else(|_| Uuid::new_v4().to_string())
}
pub fn init() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,sqlx=warn,sea_orm=warn".into());
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
