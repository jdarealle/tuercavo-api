use axum_extra::extract::cookie::{Cookie, SameSite};

use crate::{AuthConfig, config::LOGIN_FLOW_TTL_SECS};

#[derive(Clone)]
pub struct CookiePolicy {
    secure: bool,
    session_ttl_secs: i64,
}

impl CookiePolicy {
    pub fn from_config(config: &AuthConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self {
            secure: config.redirect_uri.scheme() == "https",
            session_ttl_secs: config.session_ttl_secs,
        })
    }

    pub fn session_name(&self) -> &'static str {
        if self.secure {
            "__Host-session"
        } else {
            "session"
        }
    }

    pub fn flow_name(&self) -> &'static str {
        if self.secure {
            "__Host-oidc-flow"
        } else {
            "oidc-flow"
        }
    }

    pub fn session(&self, value: String) -> Cookie<'static> {
        self.build(self.session_name(), value, self.session_ttl_secs)
    }

    pub fn login_flow(&self, value: String) -> Cookie<'static> {
        self.build(self.flow_name(), value, LOGIN_FLOW_TTL_SECS)
    }

    pub fn clear_session(&self) -> Cookie<'static> {
        let mut cookie = self.session(String::new());
        cookie.make_removal();
        cookie
    }

    pub fn clear_login_flow(&self) -> Cookie<'static> {
        let mut cookie = self.login_flow(String::new());
        cookie.make_removal();
        cookie
    }

    fn build(&self, name: &'static str, value: String, ttl: i64) -> Cookie<'static> {
        Cookie::build((name, value))
            .path("/")
            .http_only(true)
            .secure(self.secure)
            .same_site(SameSite::Lax)
            .max_age(time::Duration::seconds(ttl))
            .build()
    }
}
