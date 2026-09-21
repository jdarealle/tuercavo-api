use std::{env, net::IpAddr};
use url::{Host, Url};
use uuid::Uuid;

pub const CALLBACK_PATH: &str = "/api/auth/callback";
const DEFAULT_POST_LOGIN_REDIRECT_PATH: &str = "/api/auth/me";
pub const LOGIN_FLOW_TTL_SECS: i64 = 300;
const MAX_SESSION_TTL_SECS: i64 = 604_800;

// Deliberately omit Debug: configuration contains the client secret.
pub struct AuthConfig {
    pub tenant_id: Uuid,
    pub client_id: Uuid,
    pub client_secret: String,
    pub redirect_uri: Url,
    pub post_login_redirect_path: String,
    pub session_ttl_secs: i64,
    pub session_idle_ttl_secs: i64,
}

impl AuthConfig {
    pub fn from_env() -> Result<Self, String> {
        let config = Self {
            tenant_id: required("ENTRA_TENANT_ID")?
                .parse()
                .map_err(|_| "ENTRA_TENANT_ID debe ser un UUID")?,
            client_id: required("ENTRA_CLIENT_ID")?
                .parse()
                .map_err(|_| "ENTRA_CLIENT_ID debe ser un UUID")?,
            client_secret: required("ENTRA_CLIENT_SECRET")?,
            redirect_uri: Url::parse(&required("OIDC_REDIRECT_URI")?)
                .map_err(|_| "OIDC_REDIRECT_URI debe ser una URL absoluta válida")?,
            post_login_redirect_path: match env::var("POST_LOGIN_REDIRECT_PATH") {
                Ok(path) => path,
                Err(env::VarError::NotPresent) => DEFAULT_POST_LOGIN_REDIRECT_PATH.into(),
                Err(_) => return Err("POST_LOGIN_REDIRECT_PATH inválida".into()),
            },
            session_ttl_secs: ttl("SESSION_TTL_SECS", 28_800)?,
            session_idle_ttl_secs: ttl("SESSION_IDLE_TTL_SECS", 1_800)?,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.tenant_id.is_nil() || self.client_id.is_nil() {
            return Err("Los UUID del tenant y de la aplicación no pueden ser nil".into());
        }
        if self.client_secret.trim().is_empty() {
            return Err("ENTRA_CLIENT_SECRET no puede estar vacío".into());
        }
        for (name, seconds) in [
            ("SESSION_TTL_SECS", self.session_ttl_secs),
            ("SESSION_IDLE_TTL_SECS", self.session_idle_ttl_secs),
        ] {
            if !(1..=MAX_SESSION_TTL_SECS).contains(&seconds) {
                return Err(format!("{name} debe estar entre 1 y 604800 segundos"));
            }
        }
        let redirect = &self.redirect_uri;
        if redirect.host().is_none()
            || !redirect.username().is_empty()
            || redirect.password().is_some()
            || redirect.path() != CALLBACK_PATH
            || redirect.query().is_some()
            || redirect.fragment().is_some()
        {
            return Err(format!(
                "OIDC_REDIRECT_URI debe tener host y ruta {CALLBACK_PATH}, sin credenciales, query ni fragmento"
            ));
        }
        let loopback = match redirect.host() {
            Some(Host::Domain("localhost")) => true,
            Some(Host::Ipv4(ip)) => IpAddr::V4(ip).is_loopback(),
            Some(Host::Ipv6(ip)) => IpAddr::V6(ip).is_loopback(),
            _ => false,
        };
        if redirect.scheme() != "https" && !(redirect.scheme() == "http" && loopback) {
            return Err("El callback requiere HTTPS, salvo HTTP local para desarrollo".into());
        }
        let path = self.post_login_redirect_path.as_bytes();
        if !path.starts_with(b"/")
            || path.starts_with(b"//")
            || !path
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || b"/-._~".contains(byte))
        {
            return Err("POST_LOGIN_REDIRECT_PATH debe ser una ruta local absoluta sin query ni fragmento, por ejemplo /app".into());
        }
        Ok(())
    }

    pub fn issuer(&self) -> Url {
        Url::parse(&format!(
            "https://login.microsoftonline.com/{}/v2.0",
            self.tenant_id
        ))
        .expect("UUID interpolated into a fixed HTTPS issuer URL")
    }

    pub fn public_origin(&self) -> String {
        self.redirect_uri.origin().ascii_serialization()
    }
}

fn required(name: &str) -> Result<String, String> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty() && !value.starts_with("REEMPLAZAR_"))
        .ok_or_else(|| format!("Configura {name}"))
}

fn ttl(name: &str, default: i64) -> Result<i64, String> {
    match env::var(name) {
        Ok(value) => value
            .parse()
            .map_err(|_| format!("{name} debe ser un entero")),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(_) => Err(format!("{name} inválido")),
    }
}
