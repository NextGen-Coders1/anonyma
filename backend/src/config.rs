use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub base_url: String,
    pub bind_address: String,
    pub redirect_uri: String,
    pub database_url: String,
    pub frontend_url: String,
}

impl Config {
    pub fn init() -> Self {
        let client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");
        let client_secret =
            env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set");
        let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
        let port = env::var("PORT").expect("PORT must be set");
        let scheme = std::env::var("APP_SCHEME").unwrap_or_else(|_| "http".to_string());
        let base_url = format!("{}://{}:{}", scheme, host, port);
        let bind_address = format!("{}:{}", host, port);
        let redirect_uri = format!("{base_url}/auth/github/callback");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let frontend_url = env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");

        Self {
            client_id,
            client_secret,
            base_url,
            bind_address,
            redirect_uri,
            database_url,
            frontend_url,
        }
    }

    pub fn setup_tracing() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_file(false)
                    .with_line_number(false)
                    .with_thread_ids(false),
            )
            .with(EnvFilter::from_default_env())
            .init();
    }
}
