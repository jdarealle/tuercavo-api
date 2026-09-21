use sea_orm::{ConnectOptions, Database};
use std::{env, time::Duration};

fn required(name: &str) -> Result<String, std::io::Error> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty() && !value.starts_with("REEMPLAZAR_"))
        .ok_or_else(|| std::io::Error::other(format!("Configura {name}")))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(error) = dotenvy::dotenv()
        && !error.not_found()
    {
        return Err(std::io::Error::other("No se pudo cargar .env").into());
    }
    let tenant = required("ENTRA_TENANT_ID")?
        .parse()
        .map_err(|_| std::io::Error::other("ENTRA_TENANT_ID debe ser UUID"))?;
    let object = required("ADMIN_OBJECT_ID")?
        .parse()
        .map_err(|_| std::io::Error::other("ADMIN_OBJECT_ID debe ser UUID"))?;
    let email = required("ADMIN_EMAIL")?;
    let full_name = required("ADMIN_FULL_NAME")?;
    let mut options = ConnectOptions::new(required("DATABASE_URL")?);
    options
        .max_connections(1)
        .connect_timeout(Duration::from_secs(5))
        .sqlx_logging(false);
    let db = Database::connect(options)
        .await
        .map_err(|_| std::io::Error::other("No se pudo conectar a PostgreSQL"))?;
    let result = auth::bootstrap::first_admin(&db, tenant, object, email, full_name).await;
    db.close()
        .await
        .map_err(|_| std::io::Error::other("No se pudo cerrar el pool"))?;
    let id = result.map_err(std::io::Error::other)?;
    println!("Administrador disponible: {id}");
    Ok(())
}
