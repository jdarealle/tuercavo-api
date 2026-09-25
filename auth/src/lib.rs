//! OIDC authentication with local sessions and opaque cookies.
//!
//! The host supplies a SeaORM pool and exposes AuthState through Axum's FromRef.
//! Provider tokens only live during the callback; PostgreSQL stores session hashes.

pub mod authorization;
pub mod config;
pub mod cookie;
pub mod error;
mod extractor;
pub mod identity;
mod oidc;
pub mod permission;
mod routes;
pub mod session;
mod state;
pub mod token;

pub use config::AuthConfig;
pub use cookie::CookiePolicy;
pub use error::{AuthError, AuthErrorResponse};
pub use extractor::{AuthUser, Permission, Require, SameOrigin};
pub use identity::Principal;
pub use routes::router;
pub use state::{AuthState, build_state};
pub use token::SessionToken;
