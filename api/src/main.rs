#[cfg(all(feature = "scalar", not(debug_assertions)))]
compile_error!("La feature scalar es solo para desarrollo; compila release sin --features scalar");

#[cfg(feature = "scalar")]
mod api_doc;
mod app;

use common::{config::Config, state::AppState};
use sea_orm::{ConnectOptions, Database};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(error) = dotenvy::dotenv()
        && !error.not_found()
    {
        return Err(std::io::Error::other("No se pudo cargar el archivo .env").into());
    }
    common::telemetry::init();
    let config = Config::from_env().map_err(std::io::Error::other)?;
    let mut options = ConnectOptions::new(config.database_url.clone());
    options
        .max_connections(10)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(5))
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .sqlx_logging(false);
    let db = Database::connect(options)
        .await
        .map_err(|_| std::io::Error::other("No se pudo conectar a PostgreSQL"))?;
    let auth = auth::build_state(
        auth::AuthConfig::from_env().map_err(std::io::Error::other)?,
        db.clone(),
    )
    .await
    .map_err(std::io::Error::other)?;
    let state = AppState {
        db: db.clone(),
        auth: auth.clone(),
    };
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    let cleanup = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if auth.cleanup().await.is_err() {
                tracing::warn!("authentication cleanup failed; will retry");
            }
        }
    });
    tracing::info!(address = %config.bind, "API listening");
    let result = axum::serve(listener, app::router(state))
        .with_graceful_shutdown(shutdown())
        .await;
    cleanup.abort();
    let _ = cleanup.await;
    db.close()
        .await
        .map_err(|_| std::io::Error::other("No se pudo cerrar el pool"))?;
    result?;
    Ok(())
}
async fn shutdown() {
    let interrupt = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = interrupt => {}, _ = terminate => {} }
    tracing::info!("shutdown requested");
}
