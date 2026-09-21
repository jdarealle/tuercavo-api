use std::{
    env,
    net::{IpAddr, SocketAddr},
};
use url::Url;

pub struct Config {
    pub database_url: String,
    pub bind: SocketAddr,
}
impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL")
            .ok()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "Configura DATABASE_URL".to_owned())?;
        let db = Url::parse(&database_url).map_err(|_| "DATABASE_URL inválida")?;
        if !["postgres", "postgresql"].contains(&db.scheme()) {
            return Err("DATABASE_URL debe usar PostgreSQL".into());
        }
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse()
            .map_err(|_| "PORT inválido")?;
        if port == 0 {
            return Err("PORT no puede ser 0".into());
        }
        let host: IpAddr = env::var("HOST")
            .unwrap_or_else(|_| "127.0.0.1".into())
            .parse()
            .map_err(|_| "HOST debe ser una dirección IP válida")?;
        Ok(Self {
            database_url,
            bind: SocketAddr::from((host, port)),
        })
    }
}
