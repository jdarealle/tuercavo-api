mod bootstrap;

use bootstrap::bootstrap_admin;
use sea_orm::{ConnectOptions, Database};
use std::{env, error::Error, io};
use uuid::Uuid;

fn required(name: &str) -> Result<String, io::Error> {
    env::var(name)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| io::Error::other(format!("Configura {name}")))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [] => {}
        [flag, file] if flag == "--env-file" => {
            dotenvy::from_path(file)
                .map_err(|_| io::Error::other("No se pudo cargar el archivo de entorno"))?;
        }
        [flag] if flag == "--help" || flag == "-h" => {
            println!(
                "bootstrap-admin [--env-file RUTA]\nVariables: DATABASE_URL, ENTRA_TENANT_ID, BOOTSTRAP_ADMIN_OBJECT_ID"
            );
            return Ok(());
        }
        _ => return Err(io::Error::other("Uso: bootstrap-admin [--env-file RUTA]").into()),
    }
    let tenant: Uuid = required("ENTRA_TENANT_ID")?
        .parse()
        .map_err(|_| io::Error::other("ENTRA_TENANT_ID debe ser UUID"))?;
    let object: Uuid = required("BOOTSTRAP_ADMIN_OBJECT_ID")?
        .parse()
        .map_err(|_| io::Error::other("BOOTSTRAP_ADMIN_OBJECT_ID debe ser UUID"))?;
    let mut options = ConnectOptions::new(required("DATABASE_URL")?);
    options.sqlx_logging(false).max_connections(2);
    let db = Database::connect(options)
        .await
        .map_err(|_| io::Error::other("No se pudo conectar a PostgreSQL"))?;
    let user = bootstrap_admin(&db, tenant, object).await?;
    println!("Administrador inicial configurado: {user}. Debe iniciar sesión de nuevo.");
    db.close()
        .await
        .map_err(|_| io::Error::other("No se pudo cerrar la conexión"))?;
    Ok(())
}
