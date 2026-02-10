use axum::{extract::FromRef, routing::get, Router};
use dotenvy::dotenv;
use std::{env, sync::Arc};
use tower_cookies::CookieManagerLayer;

mod auth;
mod db;

use db::init_db;

// Authkestra imports
use authkestra::axum::AuthkestraAxumExt;
use authkestra::flow::{Authkestra, OAuth2Flow};
use authkestra::providers::github::GithubProvider;
use authkestra::session::memory::MemoryStore;

mod api;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Configure structured logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = init_db(&database_url)
        .await
        .expect("Failed to initialize database");

    // Setup Authkestra
    let client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");
    let client_secret =
        env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set");
    let redirect_uri = "http://localhost:3000/auth/github/callback".to_string();

    let github_provider = GithubProvider::new(client_id, client_secret, redirect_uri);
    let github_flow = OAuth2Flow::new(github_provider);
    let session_store = Arc::new(MemoryStore::default());

    // Create Authkestra instance
    let authkestra = Authkestra::builder()
        .session_store(session_store.clone())
        .provider(github_flow)
        .build();

    // Create custom app state
    let state = AppState {
        authkestra: authkestra.clone(),
        db_pool: Arc::new(pool),
    };

    // Build app with routes and merge Authkestra router
    let app = Router::new()
        .route("/logout", get(auth::logout_handler))
        //.route("/me", get(auth::me_handler)) // Moved to api_router
        .nest("/api", api::api_router())
        .merge(authkestra.axum_router())
        .layer(CookieManagerLayer::new())
        .with_state(state);

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    tracing::info!("Server starting on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
