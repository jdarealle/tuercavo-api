use openidconnect::{CsrfToken, Nonce, PkceCodeVerifier};
use sea_orm::DatabaseConnection;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    AuthConfig, AuthError, CookiePolicy, config::LOGIN_FLOW_TTL_SECS, oidc::OidcProvider,
    session::SessionStore,
};

const MAX_PENDING_FLOWS: usize = 1024;

pub(crate) struct LoginFlow {
    pub state: CsrfToken,
    pub nonce: Nonce,
    pub verifier: PkceCodeVerifier,
    pub started: Instant,
}

pub(crate) struct Inner {
    pub provider: OidcProvider,
    pub sessions: SessionStore,
    pub cookies: CookiePolicy,
    pub public_origin: String,
    tenant_id: uuid::Uuid,
    flows: Mutex<HashMap<[u8; 32], LoginFlow>>,
}

#[derive(Clone)]
pub struct AuthState(pub(crate) Arc<Inner>);

pub async fn build_state(config: AuthConfig, db: DatabaseConnection) -> Result<AuthState, String> {
    config.validate()?;
    let cookies = CookiePolicy::from_config(&config)?;
    let sessions = SessionStore::new(
        db,
        config.tenant_id,
        config.issuer().to_string(),
        config.session_ttl_secs,
        config.session_idle_ttl_secs,
    )
    .await
    .map_err(|_| "No se pudo acceder a las tablas de autenticación; comprueba las migraciones")?;
    let provider = OidcProvider::discover(&config)
        .await
        .map_err(|_| "No se pudo descubrir el proveedor OIDC de Entra")?;
    Ok(AuthState(Arc::new(Inner {
        provider,
        sessions,
        cookies,
        public_origin: config.public_origin(),
        tenant_id: config.tenant_id,
        flows: Mutex::new(HashMap::new()),
    })))
}

impl AuthState {
    pub fn tenant_id(&self) -> uuid::Uuid {
        self.0.tenant_id
    }
    pub fn session_cookie_name(&self) -> &'static str {
        self.0.cookies.session_name()
    }

    pub async fn cleanup(&self) -> Result<(), AuthError> {
        self.purge_flows()?;
        self.0.sessions.cleanup().await
    }

    fn purge_flows(&self) -> Result<(), AuthError> {
        self.0
            .flows
            .lock()
            .map_err(|_| AuthError::Internal)?
            .retain(|_, flow| {
                flow.started.elapsed() < Duration::from_secs(LOGIN_FLOW_TTL_SECS as u64)
            });
        Ok(())
    }

    pub(crate) fn insert_flow(
        &self,
        hash: [u8; 32],
        old: Option<[u8; 32]>,
        flow: LoginFlow,
    ) -> Result<(), AuthError> {
        self.purge_flows()?;
        let mut flows = self.0.flows.lock().map_err(|_| AuthError::Internal)?;
        if let Some(old) = old {
            flows.remove(&old);
        }
        if flows.len() >= MAX_PENDING_FLOWS {
            return Err(AuthError::Unavailable);
        }
        flows.insert(hash, flow);
        Ok(())
    }

    pub(crate) fn take_flow(&self, hash: [u8; 32]) -> Result<LoginFlow, AuthError> {
        let flow = self
            .0
            .flows
            .lock()
            .map_err(|_| AuthError::Internal)?
            .remove(&hash)
            .ok_or(AuthError::InvalidLogin)?;
        if flow.started.elapsed() >= Duration::from_secs(LOGIN_FLOW_TTL_SECS as u64) {
            return Err(AuthError::InvalidLogin);
        }
        Ok(flow)
    }
}
