pub mod handlers;
pub mod models;
pub mod print;

use axum::{
    routing::{get, post},
    Router,
};
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer};

use crate::{
    handlers::frontend_handlers::{html_handler, javascript_handler},
    handlers::server_handlers::{
        disable_otp_handler, enable_otp_handler, register_handler, signin_handler, signout_handler,
        verify_otp_handler,
    },
    models::{app_state::AppState, user::User},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("itsuki.sid")
        .with_http_only(true)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(1)));

    let state = AppState::default();

    // some dummy data for testing
    let dummy = User {
        email: "email@example.com".to_owned(),
        password: "password".to_owned(),
        otp_secret: None,
        otp_verified: None,
    };

    state.db.lock().await.push(dummy);

    let otp_router = Router::new()
        .route("/enable", get(enable_otp_handler))
        .route("/disable", get(disable_otp_handler))
        .route("/verify", post(verify_otp_handler));

    let auth_router = Router::new()
        .route("/register", post(register_handler))
        .route("/signin", post(signin_handler))
        .route("/signout", get(signout_handler))
        .nest("/otp", otp_router);

    let app = Router::new()
        .route("/index.html", get(html_handler))
        .route("/index.js", get(javascript_handler))
        .nest("/auth", auth_router)
        .layer(session_layer) // layer to store user's session
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
