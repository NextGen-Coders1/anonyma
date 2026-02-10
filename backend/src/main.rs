use axum::{routing::get, Extension, Router};
use dotenvy::dotenv;
use std::{env, sync::Arc};
use tower_cookies::CookieManagerLayer;

mod auth;
mod db;

use db::init_db;

// Authkestra imports from facade
use authkestra::axum::AuthkestraAxumExt;
use authkestra::flow::{Authkestra, OAuth2Flow};
use authkestra::providers::github::GithubProvider;
use authkestra::session::memory::MemoryStore;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let pool = init_db().await;

    // Setup Authkestra
    let client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");
    let client_secret = env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set");
    let redirect_uri = "http://localhost:3000/auth/github/callback".to_string();

    let github_provider = GithubProvider::new(client_id, client_secret, redirect_uri);
    let github_flow = OAuth2Flow::new(github_provider);
    let session_store = Arc::new(MemoryStore::default());

    // Create Authkestra instance
    let authkestra = Authkestra::builder()
        .session_store(session_store.clone())
        .provider(github_flow)
        .build();

    // AuthkestraState wraps the Authkestra instance
    let authkestra_state = authkestra::axum::AuthkestraState { authkestra };

    // Build auth router with AuthkestraState
    let auth_router = authkestra_state
        .authkestra
        .axum_router()
        .with_state(authkestra_state.clone());

    // Build app router with AuthkestraState and PgPool via Extension
    let app_router = Router::new()
        .route("/logout", get(auth::logout_handler))
        .route("/me", get(auth::me_handler))
        .layer(Extension(Arc::new(pool)))
        .with_state(authkestra_state);

    // Merge the two routers
    let app = auth_router
        .merge(app_router)
        .layer(CookieManagerLayer::new());

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    println!("🚀 Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
