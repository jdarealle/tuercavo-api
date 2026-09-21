use common::error::AppError;
use sea_orm::DatabaseConnection;
pub async fn ready(db: &DatabaseConnection) -> Result<(), AppError> {
    tokio::time::timeout(std::time::Duration::from_secs(2), db.ping())
        .await
        .map_err(|_| AppError::Unavailable)?
        .map_err(|_| AppError::Unavailable)
}
