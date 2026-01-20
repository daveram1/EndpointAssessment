use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use common::AdminUser;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::db::users;

const SESSION_COOKIE_NAME: &str = "session";

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: Uuid,
    pub username: String,
}

impl Session {
    pub fn new(user: &AdminUser) -> Self {
        Self {
            user_id: user.id,
            username: user.username.clone(),
        }
    }

    pub fn to_cookie_value(&self) -> String {
        // In production, this should be encrypted/signed
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_cookie_value(value: &str) -> Option<Self> {
        serde_json::from_str(value).ok()
    }
}

pub struct AuthenticatedUser {
    pub session: Session,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let session_cookie = jar
            .get(SESSION_COOKIE_NAME)
            .ok_or_else(|| Redirect::to("/login").into_response())?;

        let session = Session::from_cookie_value(session_cookie.value())
            .ok_or_else(|| Redirect::to("/login").into_response())?;

        // Verify user still exists
        let user = users::get_user_by_id(&state.pool, session.user_id)
            .await
            .ok()
            .flatten()
            .ok_or_else(|| Redirect::to("/login").into_response())?;

        Ok(AuthenticatedUser {
            session: Session::new(&user),
        })
    }
}

pub fn create_session_cookie(session: &Session) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE_NAME, session.to_cookie_value()))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build()
}

pub fn clear_session_cookie() -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build()
}
