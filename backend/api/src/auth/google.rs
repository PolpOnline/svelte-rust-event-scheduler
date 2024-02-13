use crate::auth::models::User;
use crate::auth::types::AuthContext;
use crate::auth::utils::normal_url_to_redirector;
use crate::Database;
use axum::extract::Query;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Extension, Router};
use axum_login::axum_sessions::extractors::{ReadableSession, WritableSession};
use axum_sessions::extractors::{ReadableSession, WritableSession};
use migration::DbErr;
use openidconnect::core::{CoreClient, CoreResponseType, CoreUserInfoClaims};
use openidconnect::reqwest::async_http_client;
use openidconnect::{AuthorizationCode, CsrfToken, Nonce, OAuth2TokenResponse, Scope};
use serde::Deserialize;
use svelte_rust_event_scheduler_service::{Mutation, UserToCreate};
use tracing::{info, trace};

pub fn google_router() -> Router {
    Router::new()
        .route("/login", get(google_login_handler))
        .route("/callback", get(google_oauth_callback_handler))
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
    state: CsrfToken,
}

pub(super) async fn google_oauth_callback_handler(
    mut auth: AuthContext,
    Query(query): Query<AuthRequest>,
    Extension(oauth_client): Extension<CoreClient>,
    session: ReadableSession,
) -> impl IntoResponse {
    info!("Running oauth callback {query:?}");
    // Compare the csrf state in the callback with the state generated before the
    // request
    let Some(original_csrf_state): Option<CsrfToken> = session.get("csrf_state") else {
        return Redirect::to("/auth/google/login");
    };
    let query_csrf_state = query.state.secret();
    let csrf_state_equal = original_csrf_state.secret() == query_csrf_state;

    drop(session);

    if !csrf_state_equal {
        info!("csrf state is invalid, cannot login",);

        //TODO: Return to some error
        return Redirect::to("/auth/login-success");
    }

    info!("Getting oauth token");
    // Get an auth token
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
        .unwrap();

    let profile: CoreUserInfoClaims = oauth_client
        .user_info(token.access_token().clone(), None)
        .unwrap()
        .request_async(async_http_client)
        .await
        .unwrap();

    let email = profile.email().unwrap().to_string();
    trace!("Got email: {}", email);

    oauth_client
        .revoke_token(token.access_token().into())
        .unwrap()
        .request_async(async_http_client)
        .await
        .expect("Failed to revoke token");

    info!("Getting db connection");

    info!("Getting user");
    let user = Mutation::add_user(
        &pool,
        UserToCreate {
            name: profile.name().unwrap().to_string(),
            email,
            section: 0,
            class: "".to_string(),
            admin: false,
        },
    )
    .await?;
    info!("Got user {:?}. Logging in.", user.name);
    auth.login(&user).await.unwrap();

    info!("Logged in the user: {user:?}");

    Redirect::to("/auth/login-success")
}

pub(super) async fn google_login_handler(
    Extension(client): Extension<CoreClient>,
    mut session: WritableSession,
) -> impl IntoResponse {
    let (authorize_url, csrf_state, _nonce) = client
        .authorize_url(
            openidconnect::AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .url();

    // Store the csrf_state in the session, so we can assert equality in the callback
    session.insert("csrf_state", csrf_state).unwrap();

    // Redirect to your oauth service
    Redirect::to(normal_url_to_redirector(authorize_url.as_str()).as_str())
}

fn normal_url_to_redirector(url: &str) -> String {
    todo!()
}
