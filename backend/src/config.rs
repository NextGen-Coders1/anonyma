use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub database_url: String,
    pub host: String,
    pub port: String,
}

impl Config {
    pub fn init() -> Self {
        let client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");
        let client_secret =
            env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set");
        let host = env::var("HOST").expect("HOST must be set");
        let port = env::var("PORT").expect("PORT must be set");
        let redirect_uri = format!("http://{host}:{port}/auth/github/callback");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        Self {
            client_id,
            client_secret,
            redirect_uri,
            database_url,
            host,
            port,
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
