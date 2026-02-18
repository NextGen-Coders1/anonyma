use axum::{
    response::Redirect,
    routing::get,
    Router,
};
use tower_http::trace::TraceLayer;
use dotenvy::dotenv;
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;
use config::Config;

mod auth;
mod db;
mod config;

use db::init_db;

// Authkestra imports
use authkestra::axum::AuthkestraAxumExt;
use authkestra::flow::{Authkestra, OAuth2Flow};
use authkestra::providers::github::GithubProvider;
use authkestra::session::memory::MemoryStore;
use authkestra::session::SessionConfig;

mod api;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Configure structured logging
    Config::setup_tracing();

    // initialize configurations
    let config = Config::init();
    tracing::info!("Configured Redirect URI: {}", config.redirect_uri);
    tracing::info!("Configured Client ID: {}", config.client_id);

    let pool = init_db(&config.database_url)
        .await
        .expect("Failed to initialize database");

    // Setup Authkestra

    let github_provider = GithubProvider::new(config.client_id, config.client_secret, config.redirect_uri);
    let github_flow = OAuth2Flow::new(github_provider)
        .with_scopes(vec!["read:user".to_string(), "user:email".to_string()]);
    let session_store = Arc::new(MemoryStore::default());

    // Create Authkestra instance
    let authkestra = Authkestra::builder()
        .session_store(session_store.clone())
        .provider(github_flow)
        .session_config(SessionConfig {
            secure: false, // Must be false for HTTP localhost
            ..SessionConfig::default()
        })
        .build();

    // Create custom app state
    let state = AppState {
        authkestra: authkestra.clone(),
        db_pool: Arc::new(pool),
    };

    // CORS configuration
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(vec![
            "http://localhost:8080".parse::<axum::http::HeaderValue>().unwrap(),
        ])
        .allow_methods(vec![
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(vec![
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::COOKIE,
        ])
        .allow_credentials(true);

    // Build app with routes and merge Authkestra router
    let app = Router::new()
        .route("/", get(root_redirect_handler))
        .route("/auth/login", axum::routing::post(auth::login_handler))
        .route("/auth/register", axum::routing::post(auth::register_handler))
        .route("/logout", get(auth::logout_handler))
        .nest("/api", api::api_router())
        .merge(authkestra.axum_router())
        .layer(CookieManagerLayer::new())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);

    tracing::info!("Server starting on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root_redirect_handler() -> Redirect {
    Redirect::to("http://localhost:8080/dashboard")
}
